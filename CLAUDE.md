This is PMJ DIGITAL — a Rust TUI implementation of "Prigozhin's March of Justice," a solitaire board wargame designed by Ray Weiss and published by CSL.

The authoritative rules source is `Prigozhin 1.10-1.pdf` in the project root.
The Python prototype lives in `python_prototype/` — reference only, the Rust version is the real implementation.
The Rust project lives in the `pmj/` subdirectory (standard Cargo layout).

The user is not a programmer — explain technical decisions plainly and avoid jargon.

Always read SESSION_NOTES.md and PENDING_TASKS.md at the start of a new conversation to pick up where we left off.
Always rewrite SESSION_NOTES.md before ending a session with a clear summary of:
- What we were working on
- What got done
- What's next

If context usage reaches ~80%, STOP working immediately, warn the user, and update SESSION_NOTES.md and PENDING_TASKS.md before doing anything else.

## Tech Stack
- Language: Rust
- TUI Framework: ratatui + crossterm
- RNG: rand crate
- Build: cargo build / cargo run

## Architecture (pmj/src/)
- main.rs — App entry point, terminal setup/teardown
- app.rs — App struct, main event loop, screen routing
- game.rs — GameState, turn logic, phase management
- map.rs — Map graph, locations, edges, adjacency
- units.rs — Unit struct, side enum, unit factory
- combat.rs — CRT, force ratio shifts, contact resolution, DRMs
- mct.rs — Maneuver/Combat Track
- ui.rs — All ratatui rendering (map, status panels, menus)
- input.rs — Keyboard input handling per screen/phase
