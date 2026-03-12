# PMJ Digital — Session Notes

## Session 1 — 2026-03-12

### What we built (from scratch in one session)
Complete Rust TUI game engine in `pmj/` — the game is playable end-to-end.

### All modules (pmj/src/)
- `main.rs` + `app.rs` — Entry point, terminal setup, event loop
- `map.rs` — 13 locations, edges (river/M4), adjacency, LOC tracing, BFS pathfinding
- `units.rs` — 12 counters from manifest, Side enum, reduction/elimination
- `mct.rs` — 5-step MCT (SP+3/MP0 → SP+0/MP4), starting at step 2
- `combat.rs` — Full CRT (8x13 const array), force ratio shifts, all DRMs
- `game.rs` — GameState: complete game logic including Russian AI
- `ui.rs` — Full TUI: title screen, map with Bresenham lines, panels, menus
- `input.rs` — Keyboard handling for all screens

### Features implemented
- Title screen with Pushkin quote
- Administration Phase: MCT adjustment per Wagner unit
- Wagner Turn: movement (stacking, river, roadblock costs), Contact (CRT + flanking DRM), Advance After Contact
- Russian AI Phase: Moscow mobilization, momentum expenditure (all 3 tiers), roadblock deployment, RAPT attacks + movement toward Moscow, Akhmat Tik Tok roll
- End Turn: all 8 momentum adjustment questions, turn advance
- Victory: LOC tracing along M4
- Map: Bresenham line drawing, color-coded nodes/units, roadblock markers (⊘)
- Color scheme: Wagner=red, Russia=blue, M4=light blue, rivers, highlights
- Scrollable action log

### Git history
- `fe4c107` Initial commit: full engine
- `4ebce12` Russian AI, flanking, stacking, project reorg
- `edc823b` Map line drawing, roadblocks, polish
- `8af4f30` Title screen

### What's next
- Create GitHub repo (gh CLI not installed, user to create manually)
- Police units (OMON/MOSpol no offensive capability — partially done)
- Dispersal / GTT return mechanic
- Two-player mode (section 11.0)
- Help screen / rules reference (F1/?)
- Game over narrative
- Contact UI: let player choose individual attackers, show flanking count
