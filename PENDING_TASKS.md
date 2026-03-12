# PMJ Digital — Pending Tasks

## Phase 1: Core Engine — COMPLETE
- [x] Project setup (Cargo, dependencies, CLAUDE.md, session notes)
- [x] Map module — 13 locations, typed edges (river, M4), adjacency graph
- [x] Units module — Unit struct, sides, all 12 counters from manifest
- [x] MCT module — Maneuver/Combat Track (5 steps, SP/MP modifiers)
- [x] Combat module — CRT lookup, force ratio shifts, DRM calculation, result application
- [x] Game state — Turn counter, momentum, cup, roadblocks, action log
- [x] UI rendering — Map display, status panels, unit lists, menus
- [x] Input handling — Keyboard-driven menus and phase navigation
- [x] Main app loop — Terminal setup, event loop, screen routing

## Phase 2: Game Flow — MOSTLY COMPLETE
- [x] Administration Phase — MCT adjustment per Wagner unit
- [x] Wagner Player Turn — Movement, Contact initiation, Advance After Contact
- [x] Moscow Mobilization Cup — Draw and deploy units, People Are Silent, Roadblock 2
- [x] End Turn Phase — Momentum adjustments (all 8 questions), turn advance
- [x] Victory checking — LOC tracing along M4 from Moscow to Rostov
- [ ] Russian AI movement and attacks (RAPT, section 7.4)
- [ ] Russian momentum expenditure (section 7.2)
- [ ] Flanking DRM in Contact (currently hardcoded to 0)

## Phase 3: Full Rules
- [x] The People Are Silent event marker
- [x] Helicopter river exemption
- [ ] Roadblock mechanics (placement rules, movement cost effects)
- [ ] Stacking enforcement (one side per location)
- [ ] Police units (no offensive capability — OMON, MOSpol cannot attack)
- [ ] Dispersal / GTT return mechanic (units return next turn)
- [ ] Akhmat Tik Tok roll (section 7.4.3 — roll 1d6, only moves on 6)
- [ ] Two-player mode (section 11.0)
- [ ] Russian auto-victory in two-player (occupy Rostov + LOC to Moscow)

## Phase 4: Polish
- [x] Visual map with positioned nodes resembling board game layout
- [x] Color scheme (Wagner=red, Russia=blue, M4=light blue, rivers, highlights)
- [x] Action log with scrollback
- [ ] Better map line drawing between nodes (Bresenham or similar)
- [ ] Unit detail popup / info screen
- [ ] Help screen / rules reference overlay
- [ ] Game over screen with narrative outcome text
- [ ] Sound / visual effects for die rolls and combat
