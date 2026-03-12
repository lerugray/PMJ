# PMJ Digital — Session Notes

## Session 1 — 2026-03-12

### Summary
Built complete Rust TUI game engine from scratch. Game is playable end-to-end.

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

### Git commits (master)
1. `fe4c107` Initial commit: full engine
2. `4ebce12` Russian AI, flanking, stacking, project reorg
3. `edc823b` Map line drawing, roadblocks, polish
4. `8af4f30` Title screen
5. `ab1b114` Session notes update

### Design note from Ray
Use halfblock NATO-style unit symbols on the map: `▐╳▌` for infantry, etc. (from another project)

### What's next (Session 2)
- Apply ▐╳▌ halfblock unit symbols to map display
- Police units enforcement (OMON/MOSpol cannot attack)
- Dispersal / GTT return mechanic (units return next turn)
- Two-player mode (section 11.0)
- Help screen / rules reference (F1/?)
- Game over narrative screen
- Contact UI: let player choose individual attackers, show flanking count
- General UI polish
