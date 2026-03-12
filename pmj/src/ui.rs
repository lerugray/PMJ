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
use crate::units::UnitId;

// ── Color palette ────────────────────────────────────────────────────────

const WAGNER_COLOR: Color = Color::Rgb(200, 50, 50);     // Red
const RUSSIA_COLOR: Color = Color::Rgb(70, 130, 200);    // Blue
const M4_COLOR: Color = Color::Rgb(80, 180, 255);        // Light blue for M4 routes
const RIVER_COLOR: Color = Color::Rgb(100, 160, 255);    // River crossing color
const ROAD_COLOR: Color = Color::Rgb(140, 140, 100);     // Regular road
const LOC_COLOR: Color = Color::Rgb(220, 200, 140);      // Location box color
const HIGHLIGHT: Color = Color::Yellow;
const DIM: Color = Color::DarkGray;
const MOMENTUM_POS: Color = Color::Rgb(50, 200, 50);     // Green
const MOMENTUM_NEG: Color = Color::Rgb(200, 50, 50);     // Red

/// Main render function — dispatches based on current screen.
pub fn draw(f: &mut Frame, game: &GameState) {
    // Overall layout: left (map) | right (status + menu)
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
            Style::default().fg(M4_COLOR)
        } else if props.river {
            Style::default().fg(RIVER_COLOR)
        } else {
            Style::default().fg(ROAD_COLOR)
        };

        // Choose line character based on edge type
        let base_ch = if props.m4 && props.river {
            '≈'
        } else if props.m4 {
            '·'
        } else if props.river {
            '~'
        } else {
            '·'
        };

        // Bresenham line between the two node centers (+2 to center on [XXX])
        let points = bresenham_line(ax as i32 + 2, ay as i32, bx as i32 + 2, by as i32);
        for (px, py) in &points {
            let x = *px as u16;
            let y = *py as u16;
            // Skip points too close to either node (within 3 chars of label)
            let da = ((x as i32 - ax as i32 - 2).abs()).max((y as i32 - ay as i32).abs());
            let db = ((x as i32 - bx as i32 - 2).abs()).max((y as i32 - by as i32).abs());
            if da < 3 || db < 3 {
                continue;
            }
            if x < inner.width && y < inner.height {
                // Pick directional char based on slope at this point
                let ch = if props.m4 || props.river {
                    base_ch
                } else {
                    base_ch
                };
                let span = Span::styled(ch.to_string(), style);
                let p = Paragraph::new(Line::from(span));
                f.render_widget(p, Rect::new(inner.x + x, inner.y + y, 1, 1));
            }
        }

        // Add river marker '~' at midpoint for river crossings (extra visual cue)
        if props.river {
            let mx = ((ax + bx) / 2) as u16 + 1;
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

    // Draw location nodes
    for loc in Location::all() {
        let (cx, cy) = loc.map_pos();
        if cx + 4 >= inner.width || cy >= inner.height {
            continue;
        }

        let wagner_here = game.wagner_units_at(*loc);
        let russian_here = game.russian_units_at(*loc);

        // Location label (short name)
        let label_style = if !wagner_here.is_empty() {
            Style::default().fg(WAGNER_COLOR).add_modifier(Modifier::BOLD)
        } else if !russian_here.is_empty() {
            Style::default().fg(RUSSIA_COLOR).add_modifier(Modifier::BOLD)
        } else if loc.on_m4() {
            Style::default().fg(M4_COLOR)
        } else {
            Style::default().fg(LOC_COLOR)
        };

        // Draw location box: [SHORT]
        let label = format!("[{}]", loc.short());
        let p = Paragraph::new(Line::from(Span::styled(label, label_style)));
        f.render_widget(
            p,
            Rect::new(inner.x + cx, inner.y + cy, 5, 1),
        );

        // Draw unit indicators below the location
        let mut unit_line = Vec::new();
        for &idx in &wagner_here {
            let u = &game.units[idx];
            let sp = game.effective_sp(idx);
            let reduced = if u.is_reduced { "r" } else { "" };
            unit_line.push(Span::styled(
                format!("{}{}{} ", u.id.short(), sp, reduced),
                Style::default().fg(WAGNER_COLOR),
            ));
        }
        for &idx in &russian_here {
            let u = &game.units[idx];
            let sp = game.effective_sp(idx);
            let reduced = if u.is_reduced { "r" } else { "" };
            unit_line.push(Span::styled(
                format!("{}{}{} ", u.id.short(), sp, reduced),
                Style::default().fg(RUSSIA_COLOR),
            ));
        }

        // Add roadblock indicator
        let has_roadblock = game.roadblocks[0] == Some(*loc) || game.roadblocks[1] == Some(*loc);
        if has_roadblock {
            unit_line.push(Span::styled(
                "⊘ ",
                Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
            ));
        }

        if !unit_line.is_empty() && cy + 1 < inner.height {
            let width = unit_line.iter().map(|s| s.width() as u16).sum::<u16>();
            let p = Paragraph::new(Line::from(unit_line));
            f.render_widget(
                p,
                Rect::new(inner.x + cx.saturating_sub(1), inner.y + cy + 1, width.min(inner.width - cx), 1),
            );
        }
    }
}

// ── Right Panel ──────────────────────────────────────────────────────────

fn draw_right_panel(f: &mut Frame, game: &GameState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Turn/Phase header
            Constraint::Length(3),  // Momentum bar
            Constraint::Length(8),  // MCT display
            Constraint::Min(10),   // Menu / Content area
            Constraint::Length(8), // Log tail
        ])
        .split(area);

    draw_header(f, game, chunks[0]);
    draw_momentum(f, game, chunks[1]);
    draw_mct(f, game, chunks[2]);
    draw_content(f, game, chunks[3]);
    draw_log(f, game, chunks[4]);
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

fn draw_content(f: &mut Frame, game: &GameState, area: Rect) {
    match &game.screen {
        Screen::Title => draw_title(f, area),
        Screen::MctSelect => draw_mct_select_menu(f, game, area),
        Screen::MctAdjust(idx) => draw_mct_adjust_menu(f, game, area, *idx),
        Screen::PhaseMenu => draw_phase_menu(f, game, area),
        Screen::MoveSelectUnit => draw_move_unit_menu(f, game, area),
        Screen::MoveSelectDest(unit_idx) => draw_move_dest_menu(f, game, area, *unit_idx),
        Screen::ContactSelectLocation => draw_contact_loc_menu(f, game, area),
        Screen::ContactSelectTarget { from_loc } => {
            draw_contact_target_menu(f, game, area, *from_loc)
        }
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

fn draw_title(f: &mut Frame, area: Rect) {
    let title_art = vec![
        "",
        "  ╔═══════════════════════════════════════════════╗",
        "  ║                                               ║",
        "  ║    P R I G O Z H I N ' S                      ║",
        "  ║        M A R C H   O F   J U S T I C E        ║",
        "  ║                                               ║",
        "  ║    A solitaire wargame by Ray Weiss           ║",
        "  ║    Published by Conflict Simulations Ltd.     ║",
        "  ║                                               ║",
        "  ║    June 23, 2023 — The March on Moscow        ║",
        "  ║                                               ║",
        "  ╚═══════════════════════════════════════════════╝",
        "",
        "         Wagner forces stand in Rostov-On-Don.",
        "      The road to Moscow lies open before them.",
        "",
        "          \"The people are silent.\"",
        "                     — Pushkin, Boris Godunov",
        "",
        "",
        "              Press ENTER to begin",
        "                Press Q to quit",
    ];

    let lines: Vec<Line> = title_art
        .iter()
        .enumerate()
        .map(|(i, &text)| {
            let style = if i >= 2 && i <= 11 {
                // Title box
                if text.contains("P R I G") || text.contains("M A R C H") {
                    Style::default().fg(WAGNER_COLOR).add_modifier(Modifier::BOLD)
                } else if text.contains("Ray Weiss") || text.contains("Conflict") {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(HIGHLIGHT)
                }
            } else if text.contains("Pushkin") {
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC)
            } else if text.contains("people are silent") {
                Style::default().fg(Color::White).add_modifier(Modifier::ITALIC)
            } else if text.contains("Press") {
                Style::default().fg(HIGHLIGHT)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(text.to_string(), style))
        })
        .collect();

    let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM));
    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
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
    // Add hint at bottom
    let hint = ListItem::new(Line::from(Span::styled(
        "  ↑↓ Navigate  Enter Select  Esc Back",
        Style::default().fg(DIM),
    )));

    let block = Block::default()
        .title(" ADMINISTRATION: Adjust MCT ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM));
    items.push(ListItem::new(Line::from("")));
    items.push(hint);
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
    draw_menu(f, "WAGNER ACTIONS", &labels, game.cursor, area);
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
    draw_menu(f, "SELECT UNIT TO MOVE", &labels, game.cursor, area);
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
    draw_menu(f, &format!("MOVE {} TO", unit.id.name()), &labels, game.cursor, area);
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
    draw_menu(f, "CONTACT: Select Location", &labels, game.cursor, area);
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
    game: &GameState,
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
    let recent: Vec<&String> = game.log.iter().rev().take(6).collect();
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

    let block = Block::default()
        .title(" RUSSIAN AI PHASE ")
        .title_style(Style::default().fg(RUSSIA_COLOR))
        .borders(Borders::ALL);
    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_end_turn(f: &mut Frame, game: &GameState, area: Rect) {
    let lines = vec![
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

    let block = Block::default()
        .title(" END TURN ")
        .title_style(Style::default().fg(HIGHLIGHT))
        .borders(Borders::ALL);
    let p = Paragraph::new(lines).block(block);
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

fn draw_game_over(f: &mut Frame, area: Rect, wagner_wins: bool) {
    let (msg, color) = if wagner_wins {
        (
            "WAGNER VICTORY! LOC traced from Moscow to Rostov along M4.",
            MOMENTUM_POS,
        )
    } else {
        (
            "Turn 6 complete. Lukashenko brokers 'amnesty' in Belarus. Wagner does not achieve victory.",
            RUSSIA_COLOR,
        )
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "GAME OVER",
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(msg, Style::default().fg(color))),
        Line::from(""),
        Line::from(Span::styled(
            "Q → Quit",
            Style::default().fg(HIGHLIGHT),
        )),
    ];

    let block = Block::default()
        .title(" GAME OVER ")
        .borders(Borders::ALL);
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
