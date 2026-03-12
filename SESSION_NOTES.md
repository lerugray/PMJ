# PMJ Digital — Session Notes

## Session 1 — 2026-03-12

### Summary
Built complete Rust TUI game engine from scratch. Game is playable end-to-end.
Repo pushed to https://github.com/lerugray/PMJ

### All modules (pmj/src/, ~2800 lines total)
- `main.rs` + `app.rs` — Entry point, terminal setup, event loop
- `map.rs` — 13 locations, edges (river/M4), adjacency, LOC tracing, BFS pathfinding
- `units.rs` — 12 counters from manifest, Side enum, reduction/elimination
- `mct.rs` — 5-step MCT (SP+3/MP0 → SP+0/MP4), starting at step 2
- `combat.rs` — Full CRT (8x13 const array), force ratio shifts, all DRMs
- `game.rs` — GameState: complete game logic including full Russian AI
- `ui.rs` — Full TUI: title screen, map with Bresenham lines, panels, menus
- `input.rs` — Keyboard handling for all screens

### Features implemented
- Title screen with Pushkin quote
- Administration Phase: MCT adjustment per Wagner unit
- Wagner Turn: movement (stacking, river, roadblock costs), Contact (CRT + flanking DRM), Advance After Contact
- Russian AI Phase: Moscow mobilization, momentum expenditure (3 tiers), roadblock deployment, RAPT attacks + movement toward Moscow, Akhmat Tik Tok roll
- End Turn: all 8 momentum adjustment questions from rulebook 8.1
- Victory: LOC tracing along M4
- Map: Bresenham line drawing, color-coded nodes/units, roadblock markers (⊘)
- Scrollable action log

### What's next (Session 2 — priority order)
1. **Halfblock unit symbols on map** — Wagner: `▐R▌` `▐U▌` `▐S▌` (letter initials), Russia: `▐╳▌` (NATO infantry), color-coded red/blue. See memory for details.
2. Police units enforcement (OMON/MOSpol cannot initiate attacks)
3. Dispersal / GTT return mechanic (units return to map next turn)
4. Contact UI improvements (choose individual attackers, show flanking count)
5. Help screen / rules reference (F1/?)
6. Game over narrative screen
7. Two-player mode (section 11.0)
8. General UI polish
