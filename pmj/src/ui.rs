/// UI module — all ratatui rendering for the game.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::game::{GameState, Phase, Screen};
use crate::map::Location;
use crate::mct::MCT_TRACK;
use crate::units::{Side, UnitId};

// ── Color palette ────────────────────────────────────────────────────────

const WAGNER_COLOR: Color = Color::Rgb(200, 50, 50);     // Red
const RUSSIA_COLOR: Color = Color::Rgb(70, 130, 200);    // Blue
const M4_COLOR: Color = Color::Rgb(80, 180, 255);        // Light blue for M4 routes
const RIVER_COLOR: Color = Color::Rgb(100, 160, 255);    // River crossing color
const ROAD_COLOR: Color = Color::Rgb(140, 140, 100);     // Regular road
const LOC_COLOR: Color = Color::Rgb(220, 200, 140);      // Location box color
const HIGHLIGHT: Color = Color::Yellow;
const DIM: Color = Color::DarkGray;
const GUIDE: Color = Color::Rgb(110, 110, 120);           // Guide text (brighter than DIM)
const MOMENTUM_POS: Color = Color::Rgb(50, 200, 50);     // Green
const MOMENTUM_NEG: Color = Color::Rgb(200, 50, 50);     // Red

/// Main render function — dispatches based on current screen.
pub fn draw(f: &mut Frame, game: &GameState) {
    // Title and Game Over screens are full-width
    if matches!(game.screen, Screen::Title) {
        draw_title(f, f.area(), game.frame_count);
        return;
    }
    if let Screen::GameOver { wagner_wins } = game.screen {
        draw_game_over(f, f.area(), wagner_wins);
        return;
    }

    // Normal game layout: left (map) | right (status + menu)
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(f.area());

    draw_map_panel(f, game, outer[0]);
    draw_right_panel(f, game, outer[1]);
}

// ── Map Panel ────────────────────────────────────────────────────────────

fn draw_map_panel(f: &mut Frame, game: &GameState, area: Rect) {
    let block = Block::default()
        .title(" PRIGOZHIN'S MARCH OF JUSTICE ")
        .title_style(Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Draw edges as lines between locations using Bresenham's algorithm
    for (a, b, props) in game.map.all_edges() {
        let (ax, ay) = a.map_pos();
        let (bx, by) = b.map_pos();
        let style = if props.m4 {
            Style::default().fg(M4_COLOR).add_modifier(Modifier::BOLD)
        } else if props.river {
            Style::default().fg(RIVER_COLOR)
        } else {
            Style::default().fg(ROAD_COLOR)
        };

        // Choose line character based on edge type
        // M4 uses ● (filled circle) — thick and unmistakable as the victory route
        let base_ch = if props.m4 && props.river {
            '●'  // M4 + river: still show M4 prominently
        } else if props.m4 {
            '●'
        } else if props.river {
            '~'
        } else {
            '·'
        };

        // Center of each label box (half the name width + 1 for border)
        let a_name = a.name();
        let a_hw = (a_name.len().min(14) as i32 + 2) / 2;
        let b_name = b.name();
        let b_hw = (b_name.len().min(14) as i32 + 2) / 2;

        // Bresenham line between the two node centers
        let points = bresenham_line(ax as i32 + a_hw, ay as i32 + 1, bx as i32 + b_hw, by as i32 + 1);
        for (px, py) in &points {
            let x = *px as u16;
            let y = *py as u16;
            // Skip points inside or very close to either label box
            // Box is: x in [ax..ax+width], y in [ay..ay+3], plus 1 char margin
            let a_w = (a_name.len().min(14) as i32) + 2;
            let in_a = (x as i32) >= ax as i32 - 1
                && (x as i32) <= ax as i32 + a_w
                && (y as i32) >= ay as i32 - 1
                && (y as i32) <= ay as i32 + 4;
            let b_w = (b_name.len().min(14) as i32) + 2;
            let in_b = (x as i32) >= bx as i32 - 1
                && (x as i32) <= bx as i32 + b_w
                && (y as i32) >= by as i32 - 1
                && (y as i32) <= by as i32 + 4;
            if in_a || in_b {
                continue;
            }
            if x < inner.width && y < inner.height {
                let span = Span::styled(base_ch.to_string(), style);
                let p = Paragraph::new(Line::from(span));
                f.render_widget(p, Rect::new(inner.x + x, inner.y + y, 1, 1));
            }
        }

        // Add river marker '~' at midpoint for river crossings (extra visual cue)
        if props.river {
            let mx = ((ax as i32 + a_hw + bx as i32 + b_hw) / 2) as u16;
            let my = ((ay + by) / 2) as u16;
            if mx < inner.width && my < inner.height {
                let span = Span::styled(
                    "≋",
                    Style::default().fg(RIVER_COLOR).add_modifier(Modifier::BOLD),
                );
                let p = Paragraph::new(Line::from(span));
                f.render_widget(p, Rect::new(inner.x + mx, inner.y + my, 1, 1));
            }
        }
    }

    // Pre-compute movement highlighting if selecting a destination
    let move_info: Option<(Location, Vec<(Location, bool)>)> =
        if let Screen::MoveSelectDest(unit_idx) = &game.screen {
            let unit = &game.units[*unit_idx];
            if let Some(from) = unit.location {
                let neighbors: Vec<(Location, bool)> = game
                    .map
                    .neighbors(from)
                    .iter()
                    .map(|(n, _)| {
                        let cost = game.move_cost(*unit_idx, from, *n);
                        let mp_rem = game.mp_remaining(*unit_idx);
                        let can_afford = cost.map(|c| c <= mp_rem).unwrap_or(false);
                        (*n, can_afford)
                    })
                    .collect();
                Some((from, neighbors))
            } else {
                None
            }
        } else {
            None
        };

    // Draw location nodes
    for loc in Location::all() {
        let (cx, cy) = loc.map_pos();
        if cx + 16 >= inner.width || cy + 4 >= inner.height {
            continue;
        }

        let wagner_here = game.wagner_units_at(*loc);
        let russian_here = game.russian_units_at(*loc);

        // Location label (short name) — override color when selecting move destination
        let label_style = if let Some((from, ref neighbors)) = move_info {
            if *loc == from {
                // Current unit location — highlight yellow
                Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
            } else if let Some((_, can_afford)) = neighbors.iter().find(|(n, _)| n == loc) {
                if *can_afford {
                    // Valid destination — bright green
                    Style::default()
                        .fg(Color::Rgb(50, 220, 50))
                        .add_modifier(Modifier::BOLD)
                } else {
                    // Adjacent but too expensive — dim red
                    Style::default().fg(Color::Rgb(180, 50, 50))
                }
            } else {
                // Not adjacent — dim it out
                Style::default().fg(DIM)
            }
        } else if !wagner_here.is_empty() {
            Style::default().fg(WAGNER_COLOR).add_modifier(Modifier::BOLD)
        } else if !russian_here.is_empty() {
            Style::default().fg(RUSSIA_COLOR).add_modifier(Modifier::BOLD)
        } else if loc.on_m4() {
            Style::default().fg(M4_COLOR)
        } else {
            Style::default().fg(LOC_COLOR)
        };

        // Draw location box with full name
        let name = loc.name();
        let display_name = if name.len() > 14 { &name[..14] } else { name };
        let name_width = display_name.len() as u16 + 2; // +2 for box borders

        if cx + name_width < inner.width && cy + 2 < inner.height {
            // Top border
            let top = format!("┌{}┐", "─".repeat(display_name.len()));
            let p = Paragraph::new(Line::from(Span::styled(&top, label_style)));
            f.render_widget(p, Rect::new(inner.x + cx, inner.y + cy, name_width, 1));

            // Name
            let mid = format!("│{}│", display_name);
            let p = Paragraph::new(Line::from(Span::styled(&mid, label_style)));
            f.render_widget(p, Rect::new(inner.x + cx, inner.y + cy + 1, name_width, 1));

            // Bottom border
            let bot = format!("└{}┘", "─".repeat(display_name.len()));
            let p = Paragraph::new(Line::from(Span::styled(&bot, label_style)));
            f.render_widget(p, Rect::new(inner.x + cx, inner.y + cy + 2, name_width, 1));
        }

        // Draw unit indicators below the location box (halfblock counter symbols)
        let mut unit_line = Vec::new();
        for &idx in &wagner_here {
            let u = &game.units[idx];
            let sp = game.effective_sp(idx);
            let fg = if u.is_reduced { Color::Rgb(160, 40, 40) } else { WAGNER_COLOR };
            unit_line.push(Span::styled("▐", Style::default().fg(fg)));
            unit_line.push(Span::styled(
                u.id.nato_symbol().to_string(),
                Style::default().fg(Color::White).bg(fg),
            ));
            unit_line.push(Span::styled("▌", Style::default().fg(fg)));
            unit_line.push(Span::styled(
                format!("{} ", sp),
                Style::default().fg(fg),
            ));
        }
        for &idx in &russian_here {
            let u = &game.units[idx];
            let sp = game.effective_sp(idx);
            let fg = if u.is_reduced { Color::Rgb(50, 100, 160) } else { RUSSIA_COLOR };
            unit_line.push(Span::styled("▐", Style::default().fg(fg)));
            unit_line.push(Span::styled(
                u.id.nato_symbol().to_string(),
                Style::default().fg(Color::White).bg(fg),
            ));
            unit_line.push(Span::styled("▌", Style::default().fg(fg)));
            unit_line.push(Span::styled(
                format!("{} ", sp),
                Style::default().fg(fg),
            ));
        }

        // Add roadblock indicator — wide and visible
        let has_roadblock = game.roadblocks[0] == Some(*loc) || game.roadblocks[1] == Some(*loc);
        if has_roadblock {
            unit_line.push(Span::styled(
                "▓BLOCK▓",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if !unit_line.is_empty() && cy + 3 < inner.height {
            let width = unit_line.iter().map(|s| s.width() as u16).sum::<u16>();
            let p = Paragraph::new(Line::from(unit_line));
            f.render_widget(
                p,
                Rect::new(inner.x + cx.saturating_sub(1), inner.y + cy + 3, width.min(inner.width - cx), 1),
            );
        }
    }

    // Map legend at bottom
    if inner.height > 2 {
        let legend = Line::from(vec![
            Span::styled(" ●", Style::default().fg(M4_COLOR).add_modifier(Modifier::BOLD)),
            Span::styled(" M4 (victory route) ", Style::default().fg(GUIDE)),
            Span::styled(" ~", Style::default().fg(RIVER_COLOR)),
            Span::styled(" River (+1 MP) ", Style::default().fg(GUIDE)),
            Span::styled(" ·", Style::default().fg(ROAD_COLOR)),
            Span::styled(" Road ", Style::default().fg(GUIDE)),
            Span::styled(" ▓BLOCK▓", Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0))),
            Span::styled(" Roadblock", Style::default().fg(GUIDE)),
        ]);
        let legend_y = inner.y + inner.height - 1;
        f.render_widget(
            Paragraph::new(legend),
            Rect::new(inner.x, legend_y, inner.width, 1),
        );
    }
}

// ── Right Panel ──────────────────────────────────────────────────────────

fn draw_right_panel(f: &mut Frame, game: &GameState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Turn/Phase header
            Constraint::Length(3),   // Momentum bar
            Constraint::Length(8),   // MCT display
            Constraint::Length(12),  // Unit roster (fits all 9 Russian + 3 Wagner)
            Constraint::Min(8),      // Menu / Content area
            Constraint::Length(5),   // Log tail
        ])
        .split(area);

    draw_header(f, game, chunks[0]);
    draw_momentum(f, game, chunks[1]);
    draw_mct(f, game, chunks[2]);
    draw_roster(f, game, chunks[3]);
    draw_content(f, game, chunks[4]);
    draw_log(f, game, chunks[5]);
}

fn draw_header(f: &mut Frame, game: &GameState, area: Rect) {
    let phase_name = match game.phase {
        Phase::Administration => "ADMINISTRATION",
        Phase::WagnerTurn => "WAGNER PLAYER TURN",
        Phase::RussianAI => "RUSSIAN AI PHASE",
        Phase::EndTurn => "END TURN PHASE",
    };
    let text = Line::from(vec![
        Span::styled(
            format!(" Turn {}/6 ", game.turn),
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", phase_name),
            Style::default()
                .fg(Color::Black)
                .bg(HIGHLIGHT)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let block = Block::default().borders(Borders::BOTTOM);
    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}

fn draw_momentum(f: &mut Frame, game: &GameState, area: Rect) {
    let mut spans = vec![Span::styled(" Momentum: ", Style::default().fg(Color::White))];

    for val in -3..=3i32 {
        let label = match val {
            -3 => "RUS-3",
            -2 => " -2 ",
            -1 => " -1 ",
            0 => "  0 ",
            1 => " +1 ",
            2 => " +2 ",
            3 => "WAG+3",
            _ => unreachable!(),
        };
        let style = if val == game.momentum {
            let bg = if val > 0 {
                MOMENTUM_POS
            } else if val < 0 {
                MOMENTUM_NEG
            } else {
                Color::Gray
            };
            Style::default()
                .fg(Color::Black)
                .bg(bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };
        spans.push(Span::styled(label, style));
    }

    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}

fn draw_mct(f: &mut Frame, game: &GameState, area: Rect) {
    let block = Block::default()
        .title(" MCT ")
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Header row
    let header = Line::from(vec![
        Span::styled("Step  SP  MP ", Style::default().fg(DIM)),
        Span::styled(
            " Rusich  Utkin   Serb ",
            Style::default().fg(WAGNER_COLOR),
        ),
    ]);
    if inner.height > 0 {
        f.render_widget(
            Paragraph::new(header),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
    }

    let wagner_ids = [UnitId::Rusich, UnitId::Utkin, UnitId::Serb];
    for (step_idx, step) in MCT_TRACK.iter().enumerate() {
        let y = inner.y + 1 + step_idx as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let mut spans = vec![Span::styled(
            format!(" {:>2}   {:>2}  {:>2}  ",
                    step_idx, step.sp_mod, step.mp),
            Style::default().fg(Color::White),
        )];

        for (wi, wid) in wagner_ids.iter().enumerate() {
            let marker = game.mct_for(*wid).unwrap();
            let is_here = marker.step == step_idx;
            // Check if this unit is being adjusted in admin phase
            let is_selecting = matches!(game.screen, Screen::MctAdjust(sel) if sel == wi)
                && marker.step == step_idx;

            if is_here {
                let style = if is_selecting {
                    Style::default()
                        .fg(Color::Black)
                        .bg(HIGHLIGHT)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(WAGNER_COLOR)
                        .add_modifier(Modifier::BOLD)
                };
                spans.push(Span::styled("  ◄██► ", style));
            } else {
                spans.push(Span::styled("   ·   ", Style::default().fg(DIM)));
            }
        }

        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(inner.x, y, inner.width, 1),
        );
    }
}

fn draw_roster(f: &mut Frame, game: &GameState, area: Rect) {
    let block = Block::default()
        .title(" UNIT ROSTER ")
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Separate units into Wagner and Russia columns
    let mut wagner_entries: Vec<(usize, &crate::units::Unit)> = Vec::new();
    let mut russia_entries: Vec<(usize, &crate::units::Unit)> = Vec::new();

    for (idx, unit) in game.units.iter().enumerate() {
        if unit.id.is_wagner() {
            wagner_entries.push((idx, unit));
        } else {
            russia_entries.push((idx, unit));
        }
    }

    // Header line: Wagner side label left, Russia right
    let half_w = inner.width / 2;
    let header = Line::from(vec![
        Span::styled(
            format!(" {:<width$}", "Wagner", width = (half_w as usize).saturating_sub(1)),
            Style::default().fg(WAGNER_COLOR).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Russia",
            Style::default().fg(RUSSIA_COLOR).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(
        Paragraph::new(header),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );

    // Render rows: one Wagner and one Russia unit per row
    let max_rows = (inner.height as usize).saturating_sub(1); // minus header
    let row_count = wagner_entries.len().max(russia_entries.len()).min(max_rows);

    for row in 0..row_count {
        let y = inner.y + 1 + row as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let mut spans: Vec<Span> = Vec::new();

        // Wagner column
        if row < wagner_entries.len() {
            let (idx, unit) = wagner_entries[row];
            let sp = game.effective_sp(idx);
            let fg = if unit.is_reduced { Color::Rgb(160, 40, 40) } else { WAGNER_COLOR };
            let reduced_mark = if unit.is_reduced { "*" } else { "" };
            let loc_str = if let Some(loc) = unit.location {
                loc.short().to_string()
            } else {
                "[off]".to_string()
            };
            spans.push(Span::styled(" \u{2590}", Style::default().fg(fg))); // ▐
            spans.push(Span::styled(
                unit.id.nato_symbol().to_string(),
                Style::default().fg(Color::White).bg(fg),
            ));
            spans.push(Span::styled("\u{258C}", Style::default().fg(fg))); // ▌
            let entry = format!("{}{} {} {}", unit.id.short(), reduced_mark, sp, loc_str);
            let padded = format!("{:<width$}", entry, width = (half_w as usize).saturating_sub(5));
            spans.push(Span::styled(padded, Style::default().fg(fg)));
        } else {
            // Empty Wagner column
            let pad = " ".repeat(half_w as usize);
            spans.push(Span::styled(pad, Style::default()));
        }

        // Russia column
        if row < russia_entries.len() {
            let (idx, unit) = russia_entries[row];
            let sp = game.effective_sp(idx);
            let fg = if unit.is_reduced { Color::Rgb(50, 100, 160) } else { RUSSIA_COLOR };
            let reduced_mark = if unit.is_reduced { "*" } else { "" };
            let loc_str = if let Some(loc) = unit.location {
                loc.short().to_string()
            } else if unit.in_cup {
                "[cup]".to_string()
            } else {
                "[off]".to_string()
            };
            spans.push(Span::styled("\u{2590}", Style::default().fg(fg))); // ▐
            spans.push(Span::styled(
                unit.id.nato_symbol().to_string(),
                Style::default().fg(Color::White).bg(fg),
            ));
            spans.push(Span::styled("\u{258C}", Style::default().fg(fg))); // ▌
            let entry = format!("{}{} {} {}", unit.id.short(), reduced_mark, sp, loc_str);
            spans.push(Span::styled(entry, Style::default().fg(fg)));
        }

        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(inner.x, y, inner.width, 1),
        );
    }
}

fn draw_content(f: &mut Frame, game: &GameState, area: Rect) {
    match &game.screen {
        Screen::Title => draw_title(f, area, game.frame_count),
        Screen::MctSelect => draw_mct_select_menu(f, game, area),
        Screen::MctAdjust(idx) => draw_mct_adjust_menu(f, game, area, *idx),
        Screen::PhaseMenu => draw_phase_menu(f, game, area),
        Screen::MoveSelectUnit => draw_move_unit_menu(f, game, area),
        Screen::MoveSelectDest(unit_idx) => draw_move_dest_menu(f, game, area, *unit_idx),
        Screen::ContactSelectLocation => draw_contact_loc_menu(f, game, area),
        Screen::ContactSelectTarget { from_loc } => {
            draw_contact_target_menu(f, game, area, *from_loc)
        }
        Screen::ContactSelectAttackers {
            from_loc,
            target_loc,
            available,
            selected,
        } => draw_contact_select_attackers(f, game, area, *from_loc, *target_loc, available, selected),
        Screen::ContactConfirm {
            from_loc,
            target_loc,
            attacker_indices,
        } => draw_contact_confirm(f, game, area, *from_loc, *target_loc, attacker_indices),
        Screen::ContactResult {
            outcome,
            target_loc,
            attacker_indices,
        } => draw_contact_result(f, game, area, outcome, *target_loc, attacker_indices),
        Screen::AdvanceAfterContact {
            target_loc,
            attacker_indices,
        } => draw_advance_prompt(f, area, *target_loc, attacker_indices),
        Screen::RussianPhaseDisplay => draw_russian_phase(f, game, area),
        Screen::EndTurnConfirm => draw_end_turn(f, game, area),
        Screen::ViewLog => draw_full_log(f, game, area),
        Screen::HelpScreen => draw_help_screen(f, area),
        Screen::UnitDetail(idx) => draw_unit_detail(f, game, area, *idx),
        Screen::GameOver { wagner_wins } => draw_game_over(f, area, *wagner_wins),
    }
}

// ── Menu helpers ─────────────────────────────────────────────────────────

fn menu_items(labels: &[String], cursor: usize) -> Vec<ListItem<'static>> {
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let style = if i == cursor {
                Style::default()
                    .fg(HIGHLIGHT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == cursor { "► " } else { "  " };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, label),
                style,
            )))
        })
        .collect()
}

fn draw_menu(f: &mut Frame, title: &str, labels: &[String], cursor: usize, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let items = menu_items(labels, cursor);
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

// ── Screen-specific draws ────────────────────────────────────────────────

// ── Title Screen — animated, Russian-themed ─────────────────────────────

// Block letters (5 rows tall) for title text
const L_P: [&str; 5] = ["████ ", "█   █", "████ ", "█    ", "█    "];
const L_R: [&str; 5] = ["████ ", "█   █", "████ ", "█  █ ", "█   █"];
const L_I: [&str; 5] = ["█████", "  █  ", "  █  ", "  █  ", "█████"];
const L_G: [&str; 5] = [" ████", "█    ", "█  ██", "█   █", " ████"];
const L_O: [&str; 5] = [" ███ ", "█   █", "█   █", "█   █", " ███ "];
const L_Z: [&str; 5] = ["█████", "   █ ", "  █  ", " █   ", "█████"];
const L_H: [&str; 5] = ["█   █", "█   █", "█████", "█   █", "█   █"];
const L_N: [&str; 5] = ["█   █", "██  █", "█ █ █", "█  ██", "█   █"];
const L_APOS: [&str; 5] = ["█", "█", " ", " ", " "];
const L_S: [&str; 5] = [" ████", "█    ", " ███ ", "    █", "████ "];
const L_M: [&str; 5] = ["█   █", "██ ██", "█ █ █", "█   █", "█   █"];
const L_A: [&str; 5] = [" ███ ", "█   █", "█████", "█   █", "█   █"];
const L_C: [&str; 5] = [" ████", "█    ", "█    ", "█    ", " ████"];
const L_J: [&str; 5] = ["█████", "   █ ", "   █ ", "█  █ ", " ██  "];
const L_U: [&str; 5] = ["█   █", "█   █", "█   █", "█   █", " ███ "];
const L_T: [&str; 5] = ["█████", "  █  ", "  █  ", "  █  ", "  █  "];
const L_E: [&str; 5] = ["█████", "█    ", "████ ", "█    ", "█████"];
const L_F: [&str; 5] = ["█████", "█    ", "████ ", "█    ", "█    "];
const L_SPC: [&str; 5] = ["   ", "   ", "   ", "   ", "   "]; // word separator

fn compose_word(letters: &[&[&str; 5]], gap: &str) -> Vec<String> {
    (0..5).map(|row| {
        letters.iter().map(|l| l[row]).collect::<Vec<_>>().join(gap)
    }).collect()
}

/// Fade factor: ramps 0.0→1.0 over `duration` frames starting at `start_frame`.
fn fade_factor(frame: u64, start: u64, duration: u64) -> f64 {
    if frame < start { return 0.0; }
    let elapsed = frame - start;
    if elapsed >= duration { 1.0 } else { elapsed as f64 / duration as f64 }
}

/// Center x position for a string within an area (using char count, not byte length).
fn center_x(area: &Rect, text: &str) -> u16 {
    area.left() + (area.width.saturating_sub(text.chars().count() as u16)) / 2
}

/// Dim an RGB color toward black by a factor (0.0 = black, 1.0 = full color).
fn dim_rgb(r: u8, g: u8, b: u8, factor: f64) -> Color {
    Color::Rgb(
        (r as f64 * factor) as u8,
        (g as f64 * factor) as u8,
        (b as f64 * factor) as u8,
    )
}

fn draw_title(f: &mut Frame, area: Rect, frame_count: u64) {
    let buf = f.buffer_mut();

    // ── Background noise (sparse drifting dots) ──────────────────────
    let noise_chars = ['·', '.', ':', '°', '∙'];
    let bg_fade = fade_factor(frame_count, 0, 15);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            // Deterministic pseudo-random hash that drifts slowly
            let seed = (x as u64).wrapping_mul(31)
                ^ (y as u64).wrapping_mul(97)
                ^ (frame_count / 5).wrapping_mul(13);
            let hash = seed.wrapping_mul(2654435761) >> 56;
            if hash < 7 { // ~3% coverage
                let ch = noise_chars[(hash as usize) % noise_chars.len()];
                let cell = &mut buf[(x, y)];
                cell.set_char(ch);
                cell.set_style(Style::default().fg(dim_rgb(30, 30, 40, bg_fade)));
            }
        }
    }

    // ── Russian flag stripe at top ───────────────────────────────────
    let stripe_fade = fade_factor(frame_count, 0, 20);
    let stripe_w = area.width as usize;
    let third = stripe_w / 3;
    if area.height > 2 {
        let y = area.top();
        for x in area.left()..area.right() {
            let rel = (x - area.left()) as usize;
            let (r, g, b) = if rel < third {
                (220, 220, 220) // white
            } else if rel < third * 2 {
                (0, 57, 166)    // blue
            } else {
                (213, 43, 30)   // red
            };
            let cell = &mut buf[(x, y)];
            cell.set_char('▀');
            cell.set_style(Style::default().fg(dim_rgb(r, g, b, stripe_fade)));
        }
    }

    // ── Build block-letter title: PRIGOZHIN'S ────────────────────────
    let prigozhin = compose_word(
        &[&L_P, &L_R, &L_I, &L_G, &L_O, &L_Z, &L_H, &L_I, &L_N, &L_APOS, &L_S],
        " ",
    );
    let title_w = prigozhin[0].chars().count();

    // ── Compute vertical layout ──────────────────────────────────────
    let cy = area.top() + area.height / 2; // vertical center
    let title_start_y = cy.saturating_sub(18);

    // ── Render PRIGOZHIN'S (block letters) ───────────────────────────
    let title_fade = fade_factor(frame_count, 6, 18);
    // Color-cycle: pulse between Wagner red and gold
    let phase = (frame_count as f64 * 0.03).sin();
    let title_r = (200.0 + 55.0 * phase) as u8;
    let title_g = (50.0 + 30.0 * phase) as u8;
    let title_b = 30;
    let title_color = dim_rgb(title_r, title_g, title_b, title_fade);

    for (row, line) in prigozhin.iter().enumerate() {
        let y = title_start_y + 2 + row as u16;
        if y >= area.bottom() { break; }
        let x_start = area.left() + (area.width.saturating_sub(title_w as u16)) / 2;
        for (col, ch) in line.chars().enumerate() {
            let x = x_start + col as u16;
            if x >= area.right() { continue; }
            if ch != ' ' {
                let cell = &mut buf[(x, y)];
                cell.set_char(ch);
                cell.set_style(Style::default().fg(title_color));
            }
        }
    }

    // ── MARCH OF JUSTICE subtitle (block letters, two lines) ────────
    let march_of = compose_word(
        &[&L_M, &L_A, &L_R, &L_C, &L_H, &L_SPC, &L_O, &L_F],
        " ",
    );
    let justice = compose_word(
        &[&L_J, &L_U, &L_S, &L_T, &L_I, &L_C, &L_E],
        " ",
    );
    let sub_fade = fade_factor(frame_count, 18, 15);
    let sub_color = dim_rgb(220, 200, 100, sub_fade);

    // "MARCH OF" line
    let march_w = march_of[0].chars().count();
    for (row, line) in march_of.iter().enumerate() {
        let y = title_start_y + 9 + row as u16;
        if y >= area.bottom() { break; }
        let x_start = area.left() + (area.width.saturating_sub(march_w as u16)) / 2;
        for (col, ch) in line.chars().enumerate() {
            let x = x_start + col as u16;
            if x >= area.right() { continue; }
            if ch != ' ' {
                let cell = &mut buf[(x, y)];
                cell.set_char(ch);
                cell.set_style(Style::default().fg(sub_color));
            }
        }
    }

    // "JUSTICE" line
    let justice_w = justice[0].chars().count();
    for (row, line) in justice.iter().enumerate() {
        let y = title_start_y + 15 + row as u16;
        if y >= area.bottom() { break; }
        let x_start = area.left() + (area.width.saturating_sub(justice_w as u16)) / 2;
        for (col, ch) in line.chars().enumerate() {
            let x = x_start + col as u16;
            if x >= area.right() { continue; }
            if ch != ' ' {
                let cell = &mut buf[(x, y)];
                cell.set_char(ch);
                cell.set_style(Style::default().fg(sub_color));
            }
        }
    }

    // ── Decorative divider ───────────────────────────────────────────
    let div_fade = fade_factor(frame_count, 30, 12);
    let div_y = title_start_y + 21;
    if div_y < area.bottom() {
        let div_str = "░▒▓██████████████████████████▓▒░";
        let x_start = center_x(&area, div_str);
        for (col, ch) in div_str.chars().enumerate() {
            let x = x_start + col as u16;
            if x >= area.right() { continue; }
            let cell = &mut buf[(x, div_y)];
            cell.set_char(ch);
            cell.set_style(Style::default().fg(dim_rgb(180, 50, 50, div_fade)));
        }
    }

    // ── Credits & date ───────────────────────────────────────────────
    let credits_fade = fade_factor(frame_count, 36, 15);
    let credit_lines = [
        ("A solitaire wargame by Ray Weiss", (220, 220, 220)),
        ("Published by Conflict Simulations Ltd.", (160, 160, 170)),
        ("June 23, 2023 \u{2014} The March on Moscow", (180, 160, 100)),
    ];
    for (i, (text, (r, g, b))) in credit_lines.iter().enumerate() {
        let y = title_start_y + 23 + i as u16;
        if y >= area.bottom() || text.is_empty() { continue; }
        let color = dim_rgb(*r, *g, *b, credits_fade);
        let x_start = center_x(&area, text);
        for (col, ch) in text.chars().enumerate() {
            let x = x_start + col as u16;
            if x >= area.right() { continue; }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            cell.set_style(Style::default().fg(color));
        }
    }

    // ── Flavor text ──────────────────────────────────────────────────
    let flavor_fade = fade_factor(frame_count, 46, 15);
    let flavor_lines = [
        ("Wagner forces stand in Rostov-On-Don.", (200, 200, 200)),
        ("The road to Moscow lies open before them.", (200, 200, 200)),
    ];
    for (i, (text, (r, g, b))) in flavor_lines.iter().enumerate() {
        let y = title_start_y + 27 + i as u16;
        if y >= area.bottom() { continue; }
        let color = dim_rgb(*r, *g, *b, flavor_fade);
        let x_start = center_x(&area, text);
        for (col, ch) in text.chars().enumerate() {
            let x = x_start + col as u16;
            if x >= area.right() { continue; }
            let cell = &mut buf[(x, y)];
            cell.set_char(ch);
            cell.set_style(Style::default().fg(color));
        }
    }

    // ── Pushkin quote ────────────────────────────────────────────────
    let quote_fade = fade_factor(frame_count, 56, 15);
    let quote_y = title_start_y + 30;
    if quote_y + 1 < area.bottom() {
        let q1 = "\u{201C}The people are silent.\u{201D}";
        let q2 = "\u{2014} Pushkin, Boris Godunov";
        let q1_color = dim_rgb(255, 255, 255, quote_fade);
        let q2_color = dim_rgb(120, 120, 130, quote_fade);
        for (text, color, dy) in [(q1, q1_color, 0u16), (q2, q2_color, 1)] {
            let y = quote_y + dy;
            if y >= area.bottom() { continue; }
            let x_start = center_x(&area, text);
            for (col, ch) in text.chars().enumerate() {
                let x = x_start + col as u16;
                if x >= area.right() { continue; }
                let cell = &mut buf[(x, y)];
                cell.set_char(ch);
                cell.set_style(Style::default().fg(color).add_modifier(Modifier::ITALIC));
            }
        }
    }

    // ── Menu prompts ─────────────────────────────────────────────────
    let menu_fade = fade_factor(frame_count, 66, 15);
    let menu_y = title_start_y + 33;
    if menu_y + 1 < area.bottom() {
        // Blinking effect on ENTER (toggles every ~30 frames)
        let blink = (frame_count / 30) % 2 == 0;
        let enter_brightness = if blink && menu_fade >= 1.0 { 255 } else { 180 };
        let enter_color = dim_rgb(enter_brightness, enter_brightness, 50, menu_fade);
        let quit_color = dim_rgb(140, 140, 140, menu_fade);

        let enter_text = "Press ENTER to begin";
        let quit_text = "Press Q to quit";
        for (text, color, dy) in [
            (enter_text, enter_color, 0u16),
            (quit_text, quit_color, 1),
        ] {
            let y = menu_y + dy;
            if y >= area.bottom() { continue; }
            let x_start = center_x(&area, text);
            for (col, ch) in text.chars().enumerate() {
                let x = x_start + col as u16;
                if x >= area.right() { continue; }
                let cell = &mut buf[(x, y)];
                cell.set_char(ch);
                cell.set_style(Style::default().fg(color));
            }
        }
    }

    // ── Russian flag stripe at bottom ────────────────────────────────
    if area.height > 2 {
        let y = area.bottom() - 1;
        for x in area.left()..area.right() {
            let rel = (x - area.left()) as usize;
            let (r, g, b) = if rel < third {
                (220, 220, 220)
            } else if rel < third * 2 {
                (0, 57, 166)
            } else {
                (213, 43, 30)
            };
            let cell = &mut buf[(x, y)];
            cell.set_char('▄');
            cell.set_style(Style::default().fg(dim_rgb(r, g, b, stripe_fade)));
        }
    }
}

fn draw_mct_select_menu(f: &mut Frame, game: &GameState, area: Rect) {
    let wagner_ids = [UnitId::Rusich, UnitId::Utkin, UnitId::Serb];
    let mut labels: Vec<String> = wagner_ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let marker = game.mct_for(*id).unwrap();
            let adjusted = if game.admin_units_adjusted[i] {
                " ✓"
            } else {
                ""
            };
            format!("{} — {}{}", id.name(), marker.label(), adjusted)
        })
        .collect();
    labels.push("Done → Start Wagner Turn".into());

    let mut items = menu_items(&labels, game.cursor);

    // Guide text
    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "─── MANEUVER/COMBAT TRACK (MCT) ───",
        Style::default().fg(M4_COLOR),
    ))));
    for line in [
        "Each Wagner unit has a position on the MCT that",
        "sets its Strike Power (SP) and Movement (MP).",
        "",
        "  Shift UP   = stronger attacks, slower movement",
        "  Shift DOWN = weaker attacks, faster movement",
        "",
        "All units start at 2 SP / 2 MP. Adjust each unit",
        "once per turn to match your plan for this turn.",
    ] {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(GUIDE),
        ))));
    }

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "  ↑↓ Navigate  Enter Select  ? Help  Q Quit",
        Style::default().fg(GUIDE),
    ))));

    let block = Block::default()
        .title(" ADMINISTRATION: Adjust MCT ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_mct_adjust_menu(f: &mut Frame, game: &GameState, area: Rect, _unit_idx: usize) {
    let labels = vec![
        "Shift UP (more SP, less MP)".into(),
        "Shift DOWN (less SP, more MP)".into(),
        "No change".into(),
    ];
    draw_menu(f, "Adjust MCT Direction", &labels, game.cursor, area);
}

fn draw_phase_menu(f: &mut Frame, game: &GameState, area: Rect) {
    let labels = match game.phase {
        Phase::WagnerTurn => vec![
            "Move a Wagner unit".into(),
            "Initiate Contact (attack)".into(),
            "End Wagner Turn → Russian AI".into(),
            "View full action log".into(),
        ],
        _ => vec!["Continue".into()],
    };
    let block = Block::default()
        .title(format!(" WAGNER ACTIONS — Turn {} ", game.turn))
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let mut items = menu_items(&labels, game.cursor);

    // Guide text
    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "─── WAGNER PLAYER TURN ───",
        Style::default().fg(M4_COLOR),
    ))));
    for line in [
        "Move your units along roads toward Moscow.",
        "Initiate Contact to attack adjacent enemies.",
        "Units can move and attack in the same turn.",
        "",
        "VICTORY: Trace a Line of Communication along",
        "the M4 (blue road) from Rostov to Moscow with",
        "no enemy units blocking and Moscow occupied.",
    ] {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(GUIDE),
        ))));
    }

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "  ? Help   Tab Unit Info   Q Quit",
        Style::default().fg(GUIDE),
    ))));
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_move_unit_menu(f: &mut Frame, game: &GameState, area: Rect) {
    let mut labels: Vec<String> = Vec::new();
    let wagner_ids = [UnitId::Rusich, UnitId::Utkin, UnitId::Serb];
    for id in &wagner_ids {
        if let Some(idx) = game.unit_index(*id) {
            let unit = &game.units[idx];
            if unit.is_on_map() {
                let mp_rem = game.mp_remaining(idx);
                let sp = game.effective_sp(idx);
                labels.push(format!(
                    "{} SP:{} MP:{}/{} @ {}",
                    id.name(),
                    sp,
                    mp_rem,
                    game.effective_mp(idx),
                    unit.location.unwrap().name()
                ));
            }
        }
    }
    // Also include any switched Russian units now on Wagner side
    for unit in &game.units {
        if unit.is_wagner()
            && unit.is_on_map()
            && !unit.id.is_wagner() // Originally Russian
        {
            if let Some(idx) = game.unit_index(unit.id) {
                let mp_rem = game.mp_remaining(idx);
                labels.push(format!(
                    "{} SP:{} MP:{}/{} @ {}",
                    unit.id.name(),
                    game.effective_sp(idx),
                    mp_rem,
                    game.effective_mp(idx),
                    unit.location.unwrap().name()
                ));
            }
        }
    }
    labels.push("Back".into());

    let block = Block::default()
        .title(" SELECT UNIT TO MOVE ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let mut items = menu_items(&labels, game.cursor);

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "─── MOVEMENT ───",
        Style::default().fg(M4_COLOR),
    ))));
    for line in [
        "Pick a unit, then choose destinations one at a",
        "time until it runs out of Movement Points (MP).",
        "",
        "  Moving between locations: 1 MP",
        "  River crossing (blue edge): +1 MP extra",
        "  Roadblocked location: +1 MP extra",
        "",
        "Units cannot enter enemy-occupied locations.",
        "Press Esc at any time to stop a unit's movement.",
    ] {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(GUIDE),
        ))));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_move_dest_menu(f: &mut Frame, game: &GameState, area: Rect, unit_idx: usize) {
    let unit = &game.units[unit_idx];
    let from = unit.location.unwrap();
    let neighbors = game.map.neighbors(from);

    let mut labels: Vec<String> = Vec::new();
    for &(neighbor, props) in neighbors {
        let cost = game.move_cost(unit_idx, from, neighbor);
        let mp_rem = game.mp_remaining(unit_idx);
        let mut tags = Vec::new();
        if props.river {
            tags.push("river +1");
        }
        if props.m4 {
            tags.push("M4");
        }
        let tag_str = if tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", tags.join(", "))
        };
        let can = cost.map(|c| c <= mp_rem).unwrap_or(false);
        let status = if !can { " ✗" } else { "" };
        labels.push(format!(
            "{} (cost: {} MP){}{}", neighbor.name(),
            cost.unwrap_or(0), tag_str, status
        ));
    }
    labels.push("Back".into());

    let mp_rem = game.mp_remaining(unit_idx);
    let mp_max = game.effective_mp(unit_idx);
    let title = format!(" MOVE {} — MP: {}/{} ", unit.id.name(), mp_rem, mp_max);
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let mut items = menu_items(&labels, game.cursor);

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "  ✗ = not enough MP   Esc/Back = done moving",
        Style::default().fg(GUIDE),
    ))));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_contact_loc_menu(f: &mut Frame, game: &GameState, area: Rect) {
    let opportunities = game.contact_opportunities();
    let mut labels: Vec<String> = opportunities
        .iter()
        .map(|(loc, targets)| {
            let units: Vec<&str> = game
                .wagner_units_at(*loc)
                .iter()
                .map(|&i| game.units[i].id.name())
                .collect();
            let target_names: Vec<&str> = targets.iter().map(|t| t.name()).collect();
            format!(
                "{} [{}] → can attack: {}",
                loc.name(),
                units.join(", "),
                target_names.join(", ")
            )
        })
        .collect();
    labels.push("Back".into());

    let block = Block::default()
        .title(" CONTACT: Select Location ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let mut items = menu_items(&labels, game.cursor);

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "─── CONTACT (COMBAT) ───",
        Style::default().fg(M4_COLOR),
    ))));
    for line in [
        "Attack enemy units in an adjacent location.",
        "Pick which of your units at this location join.",
        "",
        "  CD = your SP - their SP (higher is better)",
        "  DRMs: Momentum, Flanking (+1 per extra loc),",
        "    River crossing (-1), Moscow (-2)",
        "",
        "Results range from Surrender (best) to Attacker",
        "Routed (worst). See ? Help for the full CRT.",
    ] {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(GUIDE),
        ))));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_contact_target_menu(
    f: &mut Frame,
    game: &GameState,
    area: Rect,
    from_loc: Location,
) {
    let opportunities = game.contact_opportunities();
    let targets = opportunities
        .iter()
        .find(|(loc, _)| *loc == from_loc)
        .map(|(_, t)| t.as_slice())
        .unwrap_or(&[]);

    let mut labels: Vec<String> = targets
        .iter()
        .map(|t| {
            let defenders: Vec<String> = game
                .russian_units_at(*t)
                .iter()
                .map(|&i| {
                    format!("{} SP:{}", game.units[i].id.name(), game.effective_sp(i))
                })
                .collect();
            format!("{} [{}]", t.name(), defenders.join(", "))
        })
        .collect();
    labels.push("Back".into());
    draw_menu(f, "CONTACT: Select Target", &labels, game.cursor, area);
}

fn draw_contact_select_attackers(
    f: &mut Frame,
    game: &GameState,
    area: Rect,
    _from_loc: Location,
    _target_loc: Location,
    available: &[usize],
    selected: &[bool],
) {
    let mut items: Vec<ListItem> = Vec::new();
    let selected_count: usize = selected.iter().filter(|&&s| s).count();

    for (i, &unit_idx) in available.iter().enumerate() {
        let unit = &game.units[unit_idx];
        let sp = game.effective_sp(unit_idx);
        let mp_rem = game.mp_remaining(unit_idx);
        let mp_max = game.effective_mp(unit_idx);
        let check = if selected[i] { "[X]" } else { "[ ]" };

        let is_cursor = i == game.cursor;
        let style = if is_cursor {
            Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
        } else if selected[i] {
            Style::default().fg(WAGNER_COLOR)
        } else {
            Style::default().fg(DIM)
        };
        let prefix = if is_cursor { "► " } else { "  " };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{}{} {} SP:{} MP:{}/{}", prefix, check, unit.id.name(), sp, mp_rem, mp_max),
            style,
        ))));
    }

    // Confirm row
    let confirm_cursor = game.cursor == available.len();
    let confirm_style = if confirm_cursor {
        if selected_count > 0 {
            Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        }
    } else if selected_count > 0 {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(DIM)
    };
    let prefix = if confirm_cursor { "► " } else { "  " };
    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        format!("{}Confirm Attack", prefix),
        confirm_style,
    ))));

    // Hint line
    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "  Space/Enter: toggle   C: confirm   Esc: back",
        Style::default().fg(DIM),
    ))));

    let block = Block::default()
        .title(" SELECT ATTACKERS ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_contact_confirm(
    f: &mut Frame,
    game: &GameState,
    area: Rect,
    from_loc: Location,
    target_loc: Location,
    attacker_indices: &[usize],
) {
    let atk_sp: i32 = attacker_indices.iter().map(|&i| game.effective_sp(i)).sum();
    let def_indices = game.russian_units_at(target_loc);
    let def_sp: i32 = def_indices.iter().map(|&i| game.effective_sp(i)).sum();
    let cd = atk_sp - def_sp;

    let atk_names: Vec<&str> = attacker_indices
        .iter()
        .map(|&i| game.units[i].id.name())
        .collect();
    let def_names: Vec<&str> = def_indices
        .iter()
        .map(|&i| game.units[i].id.name())
        .collect();

    let lines = vec![
        Line::from(Span::styled(
            format!("Attackers: {} (SP: {})", atk_names.join(", "), atk_sp),
            Style::default().fg(WAGNER_COLOR),
        )),
        Line::from(Span::styled(
            format!("Defenders: {} (SP: {})", def_names.join(", "), def_sp),
            Style::default().fg(RUSSIA_COLOR),
        )),
        Line::from(Span::styled(
            format!("Contact Differential: {:+}", cd),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!("Momentum DRM: {:+}", game.momentum),
            Style::default().fg(if game.momentum > 0 { MOMENTUM_POS } else { MOMENTUM_NEG }),
        )),
        {
            let flanking = game.count_flanking(from_loc, target_loc);
            Line::from(Span::styled(
                format!("Flanking locations: {} (DRM: {:+})", flanking, flanking),
                Style::default().fg(if flanking > 0 { MOMENTUM_POS } else { DIM }),
            ))
        },
        Line::from(""),
        Line::from(Span::styled(
            "Enter → Resolve Contact    Esc → Cancel",
            Style::default().fg(HIGHLIGHT),
        )),
    ];

    let block = Block::default()
        .title(" CONFIRM CONTACT ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL);
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_contact_result(
    f: &mut Frame,
    _game: &GameState,
    area: Rect,
    outcome: &crate::combat::ContactOutcome,
    _target_loc: Location,
    _attacker_indices: &[usize],
) {
    let result_color = match outcome.result {
        crate::combat::CrtResult::AR | crate::combat::CrtResult::Ar => WAGNER_COLOR,
        crate::combat::CrtResult::NE | crate::combat::CrtResult::EX => Color::Yellow,
        _ => MOMENTUM_POS,
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("ATK SP: {}  DEF SP: {}", outcome.attack_sp, outcome.defend_sp),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(
                "CD: {:+}  FR Shift: {:+}  → Column: {:+}",
                outcome.cd_raw, outcome.fr_shift, outcome.cd_adjusted
            ),
            Style::default().fg(Color::White),
        )),
    ];

    for drm in &outcome.drms {
        lines.push(Line::from(Span::styled(
            format!("  {} {:+}", drm.label, drm.value),
            Style::default().fg(DIM),
        )));
    }

    lines.push(Line::from(Span::styled(
        format!(
            "Die: {}  DRM: {:+}  Final: {}",
            outcome.die_roll, outcome.drm_total, outcome.final_die
        ),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("══► {} ◄══", outcome.result.name()),
        Style::default()
            .fg(result_color)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        outcome.result.description().to_string(),
        Style::default().fg(result_color),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter → Continue",
        Style::default().fg(HIGHLIGHT),
    )));

    let block = Block::default()
        .title(" CONTACT RESOLVED ")
        .title_style(Style::default().fg(result_color))
        .borders(Borders::ALL);
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_advance_prompt(
    f: &mut Frame,
    area: Rect,
    target_loc: Location,
    _attacker_indices: &[usize],
) {
    let lines = vec![
        Line::from(Span::styled(
            format!("{} is now clear of enemy units!", target_loc.name()),
            Style::default().fg(MOMENTUM_POS),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Advance attacking units into the location?",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Y → Advance    N → Stay",
            Style::default().fg(HIGHLIGHT),
        )),
    ];

    let block = Block::default()
        .title(" ADVANCE AFTER CONTACT ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL);
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn draw_russian_phase(f: &mut Frame, game: &GameState, area: Rect) {
    let mut lines = vec![
        Line::from(Span::styled(
            "Moscow Mobilization and Russian AI actions complete.",
            Style::default().fg(RUSSIA_COLOR),
        )),
        Line::from(""),
    ];

    // Show last few log entries related to Russian phase
    let recent: Vec<&String> = game.log.iter().rev().take(8).collect();
    for entry in recent.iter().rev() {
        lines.push(Line::from(Span::styled(
            entry.to_string(),
            Style::default().fg(DIM),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter → Continue to End Turn Phase",
        Style::default().fg(HIGHLIGHT),
    )));

    // Guide text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─── RUSSIAN AI PHASE ───",
        Style::default().fg(M4_COLOR),
    )));
    for text in [
        "  Each turn, Russia pulls 1 unit from the Moscow",
        "  Mobilization Cup and places it on the map.",
        "  Russia then deploys roadblocks, moves units,",
        "  and attacks Wagner if the odds are favorable.",
        "  \"The People Are Silent\" event can weaken Russia",
        "  if Wagner has high Momentum.",
    ] {
        lines.push(Line::from(Span::styled(
            text, Style::default().fg(GUIDE),
        )));
    }

    let block = Block::default()
        .title(" RUSSIAN AI PHASE ")
        .title_style(Style::default().fg(RUSSIA_COLOR))
        .borders(Borders::ALL);
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_end_turn(f: &mut Frame, game: &GameState, area: Rect) {
    let mut lines = vec![
        Line::from(Span::styled(
            format!("End of Turn {}.", game.turn),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Momentum adjustments will be calculated automatically.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter → Apply & Advance Turn    Esc → Back",
            Style::default().fg(HIGHLIGHT),
        )),
    ];

    // Guide text
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─── MOMENTUM ADJUSTMENTS ───",
        Style::default().fg(M4_COLOR),
    )));
    for text in [
        "  Wagner in Rubelo?           +1",
        "  Russian unit reduced?       +1",
        "  Wagner in Moscow?           +2",
        "  Russian unit eliminated?    +2",
        "  Russia in Rostov?           -1",
        "  Wagner unit repulsed?       -1",
        "  Wagner unit reduced?        -2",
        "  Wagner unit eliminated?     -3",
        "",
        "  Positive = Wagner advantage (DRM in combat)",
        "  Negative = Russian advantage",
    ] {
        lines.push(Line::from(Span::styled(
            text, Style::default().fg(GUIDE),
        )));
    }

    let block = Block::default()
        .title(" END TURN ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL);
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_full_log(f: &mut Frame, game: &GameState, area: Rect) {
    let block = Block::default()
        .title(" ACTION LOG (Esc to close) ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_visible = inner.height as usize;
    let total = game.log.len();
    let scroll = game.log_scroll.min(total.saturating_sub(max_visible));
    let visible = &game.log[scroll..total.min(scroll + max_visible)];

    let items: Vec<ListItem> = visible
        .iter()
        .map(|entry| {
            let style = if entry.contains("ELIMINATED") || entry.contains("ROUTED") {
                Style::default().fg(WAGNER_COLOR)
            } else if entry.contains("SURRENDER") || entry.contains("SWITCHED") {
                Style::default().fg(MOMENTUM_POS)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(entry.clone(), style)))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

// ── Unit Detail Panel ────────────────────────────────────────────────────

fn draw_unit_detail(f: &mut Frame, game: &GameState, area: Rect, unit_idx: usize) {
    let unit = &game.units[unit_idx];
    let is_wagner = unit.side == Side::Wagner;
    let side_color = if is_wagner { WAGNER_COLOR } else { RUSSIA_COLOR };
    let side_label = if is_wagner { "Wagner" } else { "Russia" };

    let block = Block::default()
        .title(" UNIT DETAIL ")
        .title_style(Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(side_color));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Header: symbol + name + side
    let symbol = format!("\u{2590}{}\u{258C}", unit.id.nato_symbol());
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", symbol),
            Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", unit.id.name()),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", side_label),
            Style::default().fg(side_color),
        ),
    ]));

    // Separator
    let sep_width = inner.width.saturating_sub(2) as usize;
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(sep_width),
        Style::default().fg(DIM),
    )));

    // Strike Power
    let effective_sp = game.effective_sp(unit_idx);
    let base_sp = unit.current_sp();
    let sp_detail = if is_wagner {
        let mct_mod = game.mct_for(unit.id).map(|m| m.sp_mod()).unwrap_or(0);
        format!("{} (base: {}, MCT: +{})", effective_sp, base_sp, mct_mod)
    } else {
        if unit.is_reduced {
            format!("{} (reduced)", effective_sp)
        } else {
            format!("{}", effective_sp)
        }
    };
    lines.push(detail_line("Strike Power:    ", &sp_detail));

    // Movement Points
    let effective_mp = game.effective_mp(unit_idx);
    let mp_remaining = game.mp_remaining(unit_idx);
    let mp_spent = unit.mp_spent;
    let mp_detail = if is_wagner {
        format!("{}/{} (spent: {})", mp_remaining, effective_mp, mp_spent)
    } else {
        format!("{}/{} (spent: {})", mp_remaining, effective_mp, mp_spent)
    };
    lines.push(detail_line("Movement Points: ", &mp_detail));

    // MCT Step (Wagner only)
    if is_wagner {
        if let Some(mct) = game.mct_for(unit.id) {
            let step_label = mct.label();
            lines.push(detail_line(
                "MCT Step:        ",
                &format!("{} ({})", mct.step, step_label),
            ));
        }
    }

    // Location
    let loc_str = match unit.location {
        Some(loc) => loc.name().to_string(),
        None => "Off map".to_string(),
    };
    lines.push(detail_line("Location:        ", &loc_str));

    // Status
    let mut statuses: Vec<&str> = Vec::new();
    if unit.is_reduced {
        statuses.push("Reduced");
    }
    if unit.dispersed {
        statuses.push("Dispersed");
    }
    if statuses.is_empty() {
        statuses.push("Full strength");
    }
    lines.push(detail_line("Status:          ", &statuses.join(", ")));

    lines.push(Line::from(""));

    // Special traits (Russian units)
    if !is_wagner {
        let mut traits: Vec<&str> = Vec::new();
        if unit.police {
            traits.push("Police (P) — no offensive capability");
        }
        if unit.switchable {
            traits.push("Switchable (Z) — can defect to Wagner");
        }
        if unit.in_cup {
            traits.push("Cup (C) — starts in Moscow Mobilization Cup");
        }
        if unit.id == UnitId::Helicopters {
            traits.push("Air unit — ignores river crossing costs");
        }
        if unit.has_reduced_side {
            traits.push("Has reduced side");
        }
        if traits.is_empty() {
            lines.push(detail_line("Special:         ", "None"));
        } else {
            lines.push(detail_line("Special:         ", traits[0]));
            for t in &traits[1..] {
                lines.push(detail_line("                 ", t));
            }
        }
    } else {
        // Wagner special info
        lines.push(detail_line("Special:         ", "Wagner PMC — uses MCT for SP/MP"));
    }

    lines.push(Line::from(""));

    // Footer: navigation hints
    lines.push(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(HIGHLIGHT)),
        Span::styled(" \u{2192} Next unit    ", Style::default().fg(DIM)),
        Span::styled("Shift+Tab", Style::default().fg(HIGHLIGHT)),
        Span::styled(" \u{2192} Prev    ", Style::default().fg(DIM)),
        Span::styled("Esc", Style::default().fg(HIGHLIGHT)),
        Span::styled(" \u{2192} Back", Style::default().fg(DIM)),
    ]));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

/// Helper: one line with a dim label and a white value.
fn detail_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {}", label), Style::default().fg(DIM)),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}

fn draw_help_screen(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "PRIGOZHIN'S MARCH OF JUSTICE — Quick Reference",
            Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "June 23, 2023: Wagner PMC marches on Moscow along the M4 highway.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "─── KEY BINDINGS ───",
            Style::default().fg(M4_COLOR).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  Arrows / j,k ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Navigate menus", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Enter        ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Select / Confirm", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc          ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Go back", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Space        ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Toggle attacker selection", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Y / N        ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Answer prompts", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  F1 / ?       ", Style::default().fg(HIGHLIGHT)),
            Span::styled("This help screen", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Q            ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Quit game", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "─── TURN SEQUENCE ───",
            Style::default().fg(M4_COLOR).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  1. ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Administration", Style::default().fg(Color::White)),
            Span::styled(" — adjust each Wagner unit's MCT position", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("  2. ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Wagner Turn", Style::default().fg(WAGNER_COLOR)),
            Span::styled(" — move units, initiate contact (attacks)", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("  3. ", Style::default().fg(HIGHLIGHT)),
            Span::styled("Russian AI", Style::default().fg(RUSSIA_COLOR)),
            Span::styled(" — mobilization, movement, roadblocks, attacks", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("  4. ", Style::default().fg(HIGHLIGHT)),
            Span::styled("End Turn", Style::default().fg(Color::White)),
            Span::styled(" — momentum adjustments, advance turn counter", Style::default().fg(DIM)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "─── CONTACT RESOLUTION ───",
            Style::default().fg(M4_COLOR).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  CD = Attacker SP - Defender SP (Contact Differential)",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  DRMs: Momentum, Flanking (+1 per extra adjacent loc), River (-1)",
            Style::default().fg(DIM),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "─── MANEUVER/COMBAT TRACK (MCT) ───",
            Style::default().fg(M4_COLOR).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Higher position = more SP (combat power), fewer MP (movement)",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  Lower position  = fewer SP, more MP (faster movement)",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "─── VICTORY ───",
            Style::default().fg(M4_COLOR).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Wagner wins by tracing a Line of Communication (LOC) along",
            Style::default().fg(WAGNER_COLOR),
        )),
        Line::from(Span::styled(
            "  the M4 highway from Moscow to Rostov-On-Don, with no",
            Style::default().fg(WAGNER_COLOR),
        )),
        Line::from(Span::styled(
            "  uncontested Russian units blocking the route.",
            Style::default().fg(WAGNER_COLOR),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or ? to return",
            Style::default().fg(HIGHLIGHT),
        )),
    ];

    let block = Block::default()
        .title(" HELP — Rules Reference (F1) ")
        .title_style(Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(M4_COLOR));
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_game_over(f: &mut Frame, area: Rect, wagner_wins: bool) {
    let lines = if wagner_wins {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "WAGNER VICTORY",
                Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Wagner forces have secured a continuous Line of Communication",
                Style::default().fg(WAGNER_COLOR),
            )),
            Line::from(Span::styled(
                "along the M4 highway from Moscow to Rostov-On-Don.",
                Style::default().fg(WAGNER_COLOR),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Prigozhin's march on Moscow succeeds — the Russian military",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "establishment is thrown into chaos. The world watches in",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "disbelief as a private military company brings a nuclear",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "superpower to its knees.",
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "History has been rewritten.",
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Q to quit  |  Press R to restart",
                Style::default().fg(HIGHLIGHT),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "THE MARCH ENDS",
                Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Turn 6 complete. Wagner forces have failed to secure a",
                Style::default().fg(RUSSIA_COLOR),
            )),
            Line::from(Span::styled(
                "continuous Line of Communication along the M4.",
                Style::default().fg(RUSSIA_COLOR),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Lukashenko brokers an \"amnesty\" — Prigozhin agrees to stand",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "down and relocate to Belarus. The March of Justice ends not",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "with a bang, but with a phone call.",
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "As in history, the rebellion fades. But for 24 hours,",
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
            )),
            Line::from(Span::styled(
                "the world held its breath.",
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Q to quit  |  Press R to restart",
                Style::default().fg(HIGHLIGHT),
            )),
        ]
    };

    let title_color = if wagner_wins { WAGNER_COLOR } else { RUSSIA_COLOR };
    let block = Block::default()
        .title(" GAME OVER ")
        .title_style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_color));
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_log(f: &mut Frame, game: &GameState, area: Rect) {
    let block = Block::default()
        .title(" Recent Actions ")
        .title_style(Style::default().fg(DIM))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_lines = inner.height as usize;
    let start = game.log.len().saturating_sub(max_lines);
    let tail = &game.log[start..];

    let items: Vec<ListItem> = tail
        .iter()
        .map(|e| ListItem::new(Line::from(Span::styled(e.clone(), Style::default().fg(DIM)))))
        .collect();
    let list = List::new(items);
    f.render_widget(list, inner);
}

// ── Bresenham line drawing ───────────────────────────────────────────────

fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        points.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
}
