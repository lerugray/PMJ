# PMJ Digital — Session Notes

## Session 2 — 2026-03-12

### Summary
Implemented three features from the Session 1 priority list: halfblock unit symbols, police unit restrictions, and dispersal/GTT return mechanic.

### Changes made
1. **Halfblock unit symbols on map** — Wagner units show as `▐R▌` `▐U▌` `▐S▌`, Russian units show NATO-style symbols: infantry `▐╳▌`, armor `▐◇▌`, helicopters `▐H▌`, security `▐█▌`. SP number displayed after each counter. Reduced units shown in dimmed colors. Added `nato_symbol()` method to UnitId.
2. **Police units enforcement** — OMON and MOSpol (police=true) cannot initiate attacks. Filtered out of `contact_opportunities()` in game.rs and `attacker_indices` in input.rs. Russian AI already had this filter.
3. **Dispersal / GTT return** — Added `dispersed: bool` field to Unit struct. When a unit has no retreat path, it's marked dispersed (not eliminated). At the start of each turn (`start_administration`), dispersed units return to their home location (Rostov for Wagner, Moscow for Russia).

### Files modified
- `pmj/src/units.rs` — Added `nato_symbol()`, added `dispersed` field
- `pmj/src/ui.rs` — Replaced text unit labels with halfblock counter rendering
- `pmj/src/game.rs` — Police filter in contact_opportunities, dispersal return in start_administration, dispersed flag in retreat_unit
- `pmj/src/input.rs` — Police filter in attacker_indices

### What's next (Session 3 — priority order)
1. Contact UI improvements (choose individual attackers, show flanking count)
2. Help screen / rules reference (F1/?)
3. Game over narrative screen
4. Two-player mode (section 11.0)
5. General UI polish
