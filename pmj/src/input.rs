/// Input handling — maps keyboard events to game actions per screen.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::{GameState, Phase, Screen};
use crate::map::Location;
use crate::units::UnitId;

/// Process a key event. Returns true if the app should quit.
pub fn handle_key(game: &mut GameState, key: KeyEvent) -> bool {
    // ── Global keys (available on most game screens) ─────────────
    let on_game_screen = !matches!(
        game.screen,
        Screen::Title | Screen::GameOver { .. } | Screen::HelpScreen
    );
    if on_game_screen {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return true,
            KeyCode::F(1) | KeyCode::Char('?') => {
                game.prev_screen = Some(Box::new(game.screen.clone()));
                game.screen = Screen::HelpScreen;
                return false;
            }
            // Ctrl+S to save from any game screen
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = game.save_game();
                return false;
            }
            _ => {}
        }
    }

    match &game.screen.clone() {
        Screen::Title => {
            match key.code {
                KeyCode::Enter => game.start_administration(),
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    if GameState::save_exists() {
                        let _ = game.load_game();
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => return true,
                _ => {}
            }
        }
        Screen::MctSelect => handle_mct_select(game, key),
        Screen::MctAdjust(idx) => handle_mct_adjust(game, key, *idx),
        Screen::PhaseMenu => handle_phase_menu(game, key),
        Screen::MoveSelectUnit => handle_move_select_unit(game, key),
        Screen::MoveSelectDest(unit_idx) => handle_move_select_dest(game, key, *unit_idx),
        Screen::ContactSelectLocation => handle_contact_select_loc(game, key),
        Screen::ContactSelectTarget { from_loc } => {
            handle_contact_select_target(game, key, *from_loc)
        }
        Screen::ContactSelectAttackers {
            from_loc,
            target_loc,
            available,
            selected,
        } => handle_contact_select_attackers(game, key, *from_loc, *target_loc, available.clone(), selected.clone()),
        Screen::ContactConfirm {
            from_loc,
            target_loc,
            attacker_indices,
        } => handle_contact_confirm(game, key, *from_loc, *target_loc, attacker_indices.clone()),
        Screen::ContactResult { outcome: _, target_loc, attacker_indices } => {
            handle_contact_result(game, key, *target_loc, attacker_indices.clone())
        }
        Screen::AdvanceAfterContact {
            target_loc,
            attacker_indices,
        } => handle_advance(game, key, *target_loc, attacker_indices.clone()),
        Screen::RussianPhaseDisplay => handle_russian_phase(game, key),
        Screen::EndTurnConfirm => handle_end_turn(game, key),
        Screen::ViewLog => handle_view_log(game, key),
        Screen::HelpScreen => handle_help_screen(game, key),
        Screen::UnitDetail(idx) => handle_unit_detail(game, key, *idx),
        Screen::GameOver { .. } => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => return true,
                KeyCode::Char('r') | KeyCode::Char('R') => game.restart(),
                _ => {}
            }
        }
    }

    false
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn cursor_up(game: &mut GameState, max: usize) {
    if game.cursor > 0 {
        game.cursor -= 1;
    } else {
        game.cursor = max.saturating_sub(1);
    }
}

fn cursor_down(game: &mut GameState, max: usize) {
    if game.cursor + 1 < max {
        game.cursor += 1;
    } else {
        game.cursor = 0;
    }
}

// ── Administration Phase: MCT ────────────────────────────────────────────

fn handle_mct_select(game: &mut GameState, key: KeyEvent) {
    let count = 4; // 3 units + "Done"
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Enter => {
            if game.cursor == 3 {
                // Done → start Wagner turn
                game.start_wagner_turn();
            } else {
                game.screen = Screen::MctAdjust(game.cursor);
                game.cursor = 0;
            }
        }
        _ => {}
    }
}

fn handle_mct_adjust(game: &mut GameState, key: KeyEvent, unit_idx: usize) {
    let wagner_ids = [UnitId::Rusich, UnitId::Utkin, UnitId::Serb];
    let count = 3; // Up, Down, No change
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Enter => {
            let id = wagner_ids[unit_idx];
            let mut shifted = false;
            let mut label = String::new();
            let mut direction = "";
            match game.cursor {
                0 => {
                    if let Some(m) = game.mct_for_mut(id) {
                        shifted = m.shift_up();
                        label = m.label();
                        direction = "UP";
                    }
                }
                1 => {
                    if let Some(m) = game.mct_for_mut(id) {
                        shifted = m.shift_down();
                        label = m.label();
                        direction = "DOWN";
                    }
                }
                _ => {} // No change
            }
            if shifted {
                game.record(format!(
                    "{} MCT shifted {} → {}",
                    id.name(), direction, label
                ));
            }
            game.admin_units_adjusted[unit_idx] = true;
            game.screen = Screen::MctSelect;
            game.cursor = unit_idx; // Return cursor to the unit we just adjusted
        }
        KeyCode::Esc => {
            game.screen = Screen::MctSelect;
            game.cursor = unit_idx;
        }
        _ => {}
    }
}

// ── Wagner Turn: Phase Menu ──────────────────────────────────────────────

fn handle_phase_menu(game: &mut GameState, key: KeyEvent) {
    let count = 4; // Move, Contact, End Turn, Log
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Enter => {
            match game.cursor {
                0 => {
                    // Move
                    game.screen = Screen::MoveSelectUnit;
                    game.cursor = 0;
                }
                1 => {
                    // Contact — auto-skip steps when only one option
                    enter_contact_flow(game);
                }
                2 => {
                    // End Wagner Turn → check victory first
                    if game.check_wagner_victory() {
                        game.screen = Screen::GameOver { wagner_wins: true };
                    } else {
                        game.start_russian_phase();
                    }
                }
                3 => {
                    // View log
                    game.screen = Screen::ViewLog;
                    game.log_scroll = game.log.len().saturating_sub(20);
                }
                _ => {}
            }
        }
        KeyCode::Tab => open_unit_detail(game),
        _ => {}
    }
}

/// Enter the contact flow, auto-skipping screens when there's only one option.
fn enter_contact_flow(game: &mut GameState) {
    let opps = game.contact_opportunities();
    if opps.is_empty() {
        game.flash_message("No contact opportunities.");
        return;
    }

    if opps.len() == 1 {
        let (from_loc, targets) = opps[0].clone();
        if targets.len() == 1 {
            // Single location, single target — skip to attacker selection
            enter_attacker_select(game, from_loc, targets[0]);
            return;
        }
        // Single location, multiple targets
        game.screen = Screen::ContactSelectTarget { from_loc };
        game.cursor = 0;
        return;
    }

    // Multiple locations
    game.screen = Screen::ContactSelectLocation;
    game.cursor = 0;
}

/// Enter attacker selection, auto-skipping if only one eligible attacker.
fn enter_attacker_select(game: &mut GameState, from_loc: Location, target_loc: Location) {
    let available: Vec<usize> = game.wagner_units_at(from_loc)
        .into_iter()
        .filter(|&i| !game.units[i].police)
        .collect();

    if available.len() == 1 {
        // Only one attacker — skip straight to confirm
        game.screen = Screen::ContactConfirm {
            from_loc,
            target_loc,
            attacker_indices: available,
        };
        game.cursor = 0;
    } else {
        let selected = vec![true; available.len()];
        game.screen = Screen::ContactSelectAttackers {
            from_loc,
            target_loc,
            available,
            selected,
        };
        game.cursor = 0;
    }
}

// ── Unit Detail ──────────────────────────────────────────────────────────

fn handle_unit_detail(game: &mut GameState, key: KeyEvent, current_idx: usize) {
    match key.code {
        KeyCode::Tab => {
            // Cycle to next on-map unit
            let total = game.units.len();
            let mut next = current_idx + 1;
            loop {
                if next >= total {
                    next = 0;
                }
                if next == current_idx {
                    break; // wrapped all the way around
                }
                if game.units[next].is_on_map() {
                    game.screen = Screen::UnitDetail(next);
                    return;
                }
                next += 1;
            }
        }
        KeyCode::BackTab => {
            // Cycle to previous on-map unit
            let total = game.units.len();
            let mut prev = if current_idx == 0 { total - 1 } else { current_idx - 1 };
            loop {
                if prev == current_idx {
                    break;
                }
                if game.units[prev].is_on_map() {
                    game.screen = Screen::UnitDetail(prev);
                    return;
                }
                if prev == 0 {
                    prev = total - 1;
                } else {
                    prev -= 1;
                }
            }
        }
        KeyCode::Esc | KeyCode::Enter => {
            // Return to where we came from
            if let Some(prev) = game.prev_screen.take() {
                game.screen = *prev;
            } else {
                game.screen = Screen::PhaseMenu;
                game.cursor = 0;
            }
        }
        _ => {}
    }
}

/// Open unit detail panel from any screen, saving current screen for return.
fn open_unit_detail(game: &mut GameState) {
    let wagner_ids = [UnitId::Rusich, UnitId::Utkin, UnitId::Serb];
    for id in &wagner_ids {
        if let Some(idx) = game.unit_index(*id) {
            if game.units[idx].is_on_map() {
                game.prev_screen = Some(Box::new(game.screen.clone()));
                game.screen = Screen::UnitDetail(idx);
                return;
            }
        }
    }
}

// ── Help Screen ──────────────────────────────────────────────────────────

fn handle_help_screen(game: &mut GameState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?') => {
            if let Some(prev) = game.prev_screen.take() {
                game.screen = *prev;
            } else {
                game.screen = Screen::PhaseMenu;
                game.cursor = 0;
            }
        }
        _ => {}
    }
}

// ── Move Unit ────────────────────────────────────────────────────────────

fn moveable_unit_indices(game: &GameState) -> Vec<usize> {
    let mut indices = Vec::new();
    // Core Wagner units first
    for id in UnitId::wagner_units() {
        if let Some(idx) = game.unit_index(*id) {
            if game.units[idx].is_on_map() && game.units[idx].is_wagner() {
                indices.push(idx);
            }
        }
    }
    // Switched units
    for (idx, unit) in game.units.iter().enumerate() {
        if unit.is_wagner() && unit.is_on_map() && !unit.id.is_wagner() {
            indices.push(idx);
        }
    }
    indices
}

fn handle_move_select_unit(game: &mut GameState, key: KeyEvent) {
    let unit_indices = moveable_unit_indices(game);
    let count = unit_indices.len() + 1; // +1 for "Back"

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Enter => {
            if game.cursor >= unit_indices.len() {
                // Back
                game.screen = Screen::PhaseMenu;
                game.cursor = 0;
            } else {
                let idx = unit_indices[game.cursor];
                game.screen = Screen::MoveSelectDest(idx);
                game.cursor = 0;
            }
        }
        KeyCode::Esc => {
            game.screen = Screen::PhaseMenu;
            game.cursor = 0;
        }
        KeyCode::Tab => open_unit_detail(game),
        _ => {}
    }
}

fn handle_move_select_dest(game: &mut GameState, key: KeyEvent, unit_idx: usize) {
    let from = game.units[unit_idx].location.unwrap();
    let neighbors: Vec<Location> = game
        .map
        .neighbors(from)
        .iter()
        .map(|(n, _)| *n)
        .collect();
    let count = neighbors.len() + 1; // +1 for Back

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Enter => {
            if game.cursor >= neighbors.len() {
                // Back → straight to phase menu (already picked this unit)
                game.screen = Screen::PhaseMenu;
                game.cursor = 0;
            } else {
                let dest = neighbors[game.cursor];
                // Only attempt move if valid (enough MP, no enemy)
                if game.can_move(unit_idx, dest).is_ok() {
                    game.move_unit(unit_idx, dest);
                    // If unit still has MP, stay in move mode at new location
                    if game.mp_remaining(unit_idx) > 0 {
                        game.screen = Screen::MoveSelectDest(unit_idx);
                        game.cursor = 0;
                    } else {
                        game.screen = Screen::PhaseMenu;
                        game.cursor = 0;
                    }
                }
                // Invalid moves are silently ignored (menu shows ✗ marker)
            }
        }
        KeyCode::Esc => {
            // Esc from destination picker goes directly to phase menu
            game.screen = Screen::PhaseMenu;
            game.cursor = 0;
        }
        KeyCode::Tab => open_unit_detail(game),
        _ => {}
    }
}

// ── Contact ──────────────────────────────────────────────────────────────

fn handle_contact_select_loc(game: &mut GameState, key: KeyEvent) {
    let opportunities = game.contact_opportunities();
    let count = opportunities.len() + 1;

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Enter => {
            if game.cursor >= opportunities.len() {
                game.screen = Screen::PhaseMenu;
                game.cursor = 1;
            } else {
                let (from_loc, targets) = opportunities[game.cursor].clone();
                if targets.len() == 1 {
                    // Auto-skip target selection
                    enter_attacker_select(game, from_loc, targets[0]);
                } else {
                    game.screen = Screen::ContactSelectTarget { from_loc };
                    game.cursor = 0;
                }
            }
        }
        KeyCode::Esc => {
            game.screen = Screen::PhaseMenu;
            game.cursor = 1;
        }
        _ => {}
    }
}

fn handle_contact_select_target(game: &mut GameState, key: KeyEvent, from_loc: Location) {
    let opportunities = game.contact_opportunities();
    let targets = opportunities
        .iter()
        .find(|(loc, _)| *loc == from_loc)
        .map(|(_, t)| t.clone())
        .unwrap_or_default();
    let count = targets.len() + 1;

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Enter => {
            if game.cursor >= targets.len() {
                game.screen = Screen::ContactSelectLocation;
                game.cursor = 0;
            } else {
                let target_loc = targets[game.cursor];
                enter_attacker_select(game, from_loc, target_loc);
            }
        }
        KeyCode::Esc => {
            game.screen = Screen::ContactSelectLocation;
            game.cursor = 0;
        }
        _ => {}
    }
}

fn handle_contact_select_attackers(
    game: &mut GameState,
    key: KeyEvent,
    from_loc: Location,
    target_loc: Location,
    available: Vec<usize>,
    mut selected: Vec<bool>,
) {
    let count = available.len() + 1; // +1 for "Confirm" row
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => cursor_up(game, count),
        KeyCode::Down | KeyCode::Char('j') => cursor_down(game, count),
        KeyCode::Char(' ') | KeyCode::Enter => {
            if game.cursor < available.len() {
                // Toggle this unit
                selected[game.cursor] = !selected[game.cursor];
                game.screen = Screen::ContactSelectAttackers {
                    from_loc,
                    target_loc,
                    available,
                    selected,
                };
            } else {
                // Confirm row — proceed if at least one selected
                let attacker_indices: Vec<usize> = available
                    .iter()
                    .zip(selected.iter())
                    .filter(|(_, &sel)| sel)
                    .map(|(&idx, _)| idx)
                    .collect();
                if !attacker_indices.is_empty() {
                    game.screen = Screen::ContactConfirm {
                        from_loc,
                        target_loc,
                        attacker_indices,
                    };
                    game.cursor = 0;
                }
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            // Quick confirm shortcut
            let attacker_indices: Vec<usize> = available
                .iter()
                .zip(selected.iter())
                .filter(|(_, &sel)| sel)
                .map(|(&idx, _)| idx)
                .collect();
            if !attacker_indices.is_empty() {
                game.screen = Screen::ContactConfirm {
                    from_loc,
                    target_loc,
                    attacker_indices,
                };
                game.cursor = 0;
            }
        }
        KeyCode::Esc => {
            game.screen = Screen::ContactSelectTarget { from_loc };
            game.cursor = 0;
        }
        _ => {}
    }
}

fn handle_contact_confirm(
    game: &mut GameState,
    key: KeyEvent,
    from_loc: Location,
    target_loc: Location,
    attacker_indices: Vec<usize>,
) {
    match key.code {
        KeyCode::Enter => {
            let outcome = game.resolve_contact(&attacker_indices, from_loc, target_loc);
            game.screen = Screen::ContactResult {
                outcome,
                target_loc,
                attacker_indices,
            };
        }
        KeyCode::Esc => {
            game.screen = Screen::PhaseMenu;
            game.cursor = 1;
        }
        _ => {}
    }
}

fn handle_contact_result(
    game: &mut GameState,
    key: KeyEvent,
    target_loc: Location,
    attacker_indices: Vec<usize>,
) {
    match key.code {
        KeyCode::Enter => {
            // Check if target is now empty for advance
            if game.target_empty_of_russians(target_loc) {
                let alive: Vec<usize> = attacker_indices
                    .iter()
                    .copied()
                    .filter(|&i| game.units[i].is_on_map())
                    .collect();
                if !alive.is_empty() {
                    game.screen = Screen::AdvanceAfterContact {
                        target_loc,
                        attacker_indices: alive,
                    };
                    return;
                }
            }
            // After contact, check if more attacks are available
            return_or_attack_again(game);
        }
        KeyCode::Esc => {
            game.screen = Screen::PhaseMenu;
            game.cursor = 0;
        }
        _ => {}
    }
}

fn handle_advance(
    game: &mut GameState,
    key: KeyEvent,
    target_loc: Location,
    attacker_indices: Vec<usize>,
) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            game.advance_units(&attacker_indices, target_loc);
            return_or_attack_again(game);
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            return_or_attack_again(game);
        }
        _ => {}
    }
}

/// After contact resolves, check for more attack opportunities.
/// If there are more, jump straight into the contact flow; otherwise back to menu.
fn return_or_attack_again(game: &mut GameState) {
    let opps = game.contact_opportunities();
    if !opps.is_empty() {
        enter_contact_flow(game);
    } else {
        game.screen = Screen::PhaseMenu;
        game.cursor = 0;
    }
}

// ── Russian Phase ────────────────────────────────────────────────────────

fn handle_russian_phase(game: &mut GameState, key: KeyEvent) {
    if matches!(key.code, KeyCode::Enter) {
        game.start_end_turn_phase();
    }
}

// ── End Turn ─────────────────────────────────────────────────────────────

fn handle_end_turn(game: &mut GameState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            game.end_turn();
        }
        KeyCode::Esc => {
            // Go back to Wagner turn (if they didn't mean to end)
            game.phase = Phase::WagnerTurn;
            game.screen = Screen::PhaseMenu;
            game.cursor = 0;
        }
        _ => {}
    }
}

// ── Log Viewer ───────────────────────────────────────────────────────────

fn handle_view_log(game: &mut GameState, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            game.log_scroll = game.log_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            game.log_scroll = game.log_scroll.saturating_add(1);
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            game.screen = Screen::PhaseMenu;
            game.cursor = 3; // Return to "View log" option
        }
        _ => {}
    }
}
