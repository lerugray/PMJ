# PMJ Digital — Pending Tasks

## Phase 1: Core Engine — COMPLETE
- [x] Map, Units, MCT, Combat, Game State, UI, Input, App loop
- [x] All 12 counters, 13 locations, full CRT, force ratio shifts

## Phase 2: Game Flow — COMPLETE
- [x] Administration Phase (MCT adjustment)
- [x] Wagner Turn (movement, contact, advance after contact)
- [x] Russian AI Phase (mobilization, momentum expenditure, roadblock deploy, RAPT attacks, Akhmat)
- [x] End Turn Phase (all 8 momentum questions, turn advance)
- [x] Victory checking (LOC tracing along M4)
- [x] Flanking DRM in Contact
- [x] Stacking enforcement (one side per location)

## Phase 3: Remaining Rules — COMPLETE
- [x] Police units — OMON/MOSpol cannot initiate attacks
- [x] Roadblock movement cost — Wagner pays +1 MP to enter a roadblocked location
- [x] Dispersal / GTT return — dispersed units return to home location next turn

## Phase 4: Polish — COMPLETE
- [x] Halfblock unit symbols on map (▐R▌ ▐╳▌ etc., NATO-style, color-coded)
- [x] Contact UI improvements (attacker toggle, flanking count display)
- [x] Help screen / rules reference overlay (? key)
- [x] Game over screen with narrative outcome text + restart (R)
- [x] Unit detail popup (Tab to open, Tab/Shift+Tab to cycle)
- [x] Unit roster panel in right sidebar
- [x] Movement destination highlights (green/red/dim on map)
- [x] Key input fix (Press only, no Release/Repeat on Windows)
- [x] Full-screen title and game over screens
- [x] Map rework — full-name boxed labels, spread-out layout matching board
- [x] Animated title screen — block-letter art, Russian flags, fade-in, noise
- [x] Continuous movement — stay in move mode until MP exhausted or Esc
- [x] Global ? help and Q quit from any screen
- [x] Contextual phase guides — rules explanations on every major screen
- [x] M4 route prominent display (● bold cyan)
- [x] Roadblock visible (▓BLOCK▓ on orange)
- [x] Map legend (M4/river/road/roadblock)
- [x] Bresenham line centering + bounding box skip
- [x] Roster expanded to show all units

## Phase 5: UX Flow Improvements — COMPLETE
- [x] Auto-skip contact screens when only one option (1 location, 1 target, 1 attacker)
- [x] Tab unit detail available from more screens (move select, destination picker)
- [x] After contact, auto-attack-again if more opportunities (Enter = next attack, Esc = menu)
- [x] Movement Esc goes to PhaseMenu directly (already moved, no need to re-select)
- [x] Unit detail returns to correct screen (not always PhaseMenu)
- [x] Status flash messages for feedback (save, no contact, etc.)

## Phase 6: Features & Map Polish — COMPLETE
- [x] Save/load game state (Ctrl+S save, L on title to load, single slot)
- [x] M4 road visibility fix (spread positions, tighten skip zones)
- [x] Dynamic map centering (computes actual content bounds, works at any terminal width)
- [x] Location indicators (Rublevo ⌂⌂⌂ suburbs, Moscow ⬤ capital, Rostov HQ, Grozny ⚑ base)
- [x] Bugaevka name shortened to "Bugaevka B.P."
- [x] Right panel centering (header, momentum, legend)
- [x] Minimum terminal size check (100x40, friendly resize message)

## Phase 7: Movement & Map Visuals — COMPLETE
- [x] Multi-hop movement — destination picker shows all reachable locations within MP (Dijkstra pathfinding)
- [x] River edge drawing — ≈ chars along full edge length, not just midpoint marker

## Phase 8: Remaining — TODO
- [ ] Playtest a full game to verify flow and map rendering
- [ ] Sound or flash on combat results (if terminal supports it)
