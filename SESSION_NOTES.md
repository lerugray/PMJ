# PMJ Digital — Session Notes

## Session 1 — 2026-03-12

### What we did
- Reviewed the full project: PDF rulebook (12 pages, v1.10), game map (JPG), counter manifest, adjacency list, and all 6 Python source files
- Assessed the Python prototype: solid but incomplete (missing Russian AI priority table, roadblock deployment, LOC tracing, momentum expenditure by AI, end-game scoring)
- Decided to rewrite in Rust as a TUI using ratatui, fitting Ray's roguelike project style
- Set up project infrastructure: CLAUDE.md, SESSION_NOTES.md, PENDING_TASKS.md, memory files
- Initialized Rust project (`pmj/`) with cargo, dependencies: ratatui 0.29, crossterm 0.28, rand 0.8
- Built ALL core modules — project compiles and runs:
  - `map.rs` — 13 locations, typed edges (river/M4), adjacency graph, LOC tracing
  - `units.rs` — Unit struct, all 12 counters from manifest, Side enum
  - `mct.rs` — 5-step MCT track (SP+3/MP0 through SP+0/MP4), markers
  - `combat.rs` — Full CRT (8x13 const array), force ratio shifts, DRM calculation, contact resolution
  - `game.rs` — GameState with turn/momentum/cup/roadblocks, movement system, contact application, advance after contact, Moscow mobilization, People Are Silent event, end-turn momentum adjustments, victory check
  - `ui.rs` — Full ratatui rendering: map panel with positioned location nodes and unit counters, momentum bar, MCT grid with markers, context-sensitive menus, contact preview/result screens, log panel
  - `input.rs` — Keyboard handling for all screens: MCT adjustment, movement, contact, advance, Russian phase, end turn, log viewer
  - `app.rs` — Terminal setup/teardown, main event loop
  - `main.rs` — Module declarations, entry point

### What's working
- TUI launches and renders correctly
- Administration Phase: MCT adjustment for each Wagner unit
- Wagner Turn: Unit movement with MP cost/river penalties, Contact initiation with CRT resolution
- Moscow Mobilization Cup draws (including People Are Silent and Roadblock 2)
- End Turn momentum adjustments (all 8 questions from rulebook 8.1)
- Victory check (LOC tracing along M4)
- Action log with scrolling

### Key design decisions
- MCT track: SP+3/MP0 at top (strongest combat), SP+0/MP4 at bottom (most mobile) — matches rulebook reference card "3-0, 2-1, 1-2, 0-3, 0-4"
- CRT is const 2D array, DRMs applied to die roll before lookup
- Map positions hand-tuned to approximate the physical game map layout
- Color coding: Wagner=red, Russia=blue, M4=light blue, rivers=blue, highlights=yellow

### What's next (Phase 2+)
- Russian AI movement and attacks (RAPT from section 7.4)
- Russian momentum expenditure (section 7.2)
- Roadblock deployment and mechanics
- Flanking DRM in contact (currently hardcoded to 0)
- Stacking enforcement (one side per location)
- Akhmat Tik Tok roll
- Polish map rendering (better line drawing between nodes)
- Two-player mode
- Game over narrative screen
