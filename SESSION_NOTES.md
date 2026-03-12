# PMJ Digital — Session Notes

## Session 1 — 2026-03-12

### What we did
- Reviewed full project: PDF rulebook (12p, v1.10), game map, counter manifest, adjacency list, Python prototype
- Built complete Rust TUI game engine from scratch in `pmj/` using ratatui + crossterm
- Moved Python files to `python_prototype/`
- Initialized git repo, first commit

### Modules built (pmj/src/)
- `map.rs` — 13 locations, edges (river/M4), adjacency, LOC tracing, BFS pathfinding
- `units.rs` — 12 counters, Side enum, reduction/elimination
- `mct.rs` — 5-step MCT (SP+3/MP0 → SP+0/MP4), starting at step 2
- `combat.rs` — Full CRT (8x13), force ratio shifts, all DRMs, contact resolution
- `game.rs` — GameState: turns, momentum, movement, stacking, contact + result application, Moscow mobilization cup, People Are Silent, Russian AI (RAPT movement + attacks + momentum expenditure + roadblock deploy + Akhmat Tik Tok), flanking DRM, end-turn momentum adjustments, LOC victory check
- `ui.rs` — Map panel with positioned nodes, unit counters, momentum bar, MCT grid, context-sensitive menus, contact preview/result, log panel
- `input.rs` — Keyboard nav for all screens
- `app.rs` + `main.rs` — Terminal setup, event loop

### What's working
- Full game loop: Administration → Wagner Turn → Russian AI → End Turn → repeat
- MCT adjustment, unit movement with stacking/river penalties
- Contact with CRT, force ratio, all DRMs including flanking
- Russian AI: mobilization, momentum expenditure, roadblock deploy, RAPT attacks, Akhmat
- Advance After Contact, victory checking
- Colorful TUI with map, panels, scrolling log

### What's next
- Polish map rendering (line drawing between nodes)
- Police units (OMON/MOSpol no offensive capability)
- Dispersal / GTT return mechanic
- Two-player mode
- Help screen / rules reference
- Game over narrative
- Better contact UI (show flanking, let player choose individual attackers)
- Roadblock movement cost effects on Wagner
