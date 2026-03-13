# PMJ Digital — Session Notes

## Session 3 — 2026-03-12

### Summary
Flashy animated title screen, continuous movement, global help/quit, contextual phase guides, and UI/UX audit with map fixes.

### What got done this session
1. **Animated title screen** — Veridian Contraption-style: block-letter ASCII art (PRIGOZHIN'S / MARCH OF / JUSTICE), Russian flag stripes, staggered fade-in, background noise, color-cycling, blinking prompt
2. **Continuous movement** — units stay in destination picker until MP exhausted or Esc. Title bar shows remaining MP.
3. **Global `?` help** — works from any game screen, returns to exact previous screen (uses prev_screen field)
4. **Global `Q` quit** — works from any game screen
5. **Contextual phase guides** — rules explanations on every major screen (MCT, Wagner Turn, Movement, Contact, Russian AI, End Turn with full momentum table)
6. **UI/UX audit fixes:**
   - Bresenham lines now connect to actual center of variable-width label boxes
   - Edge skip uses proper bounding-box collision instead of fixed offset
   - "Grozny Akhmat Base" shortened to "Grozny" in map.rs name()
   - Roster expanded from 9 to 12 rows (fits all units)
   - New GUIDE color (RGB 110,110,120) for guide text — brighter than DIM
   - Map legend at bottom showing M4/river/road/roadblock symbols
   - M4 route uses `●` (bold) instead of `·` — much more visible
   - Roadblock changed from tiny `⊘` to `▓BLOCK▓` on orange background
   - Unit halfblock counters kept as-is (Ray liked them)

### What needs doing next (Session 4)
1. **Gameplay flow audit** — was about to review for more friction points like the movement fix. Specific things to check:
   - Contact flow: currently 4 screens deep (location → target → select attackers → confirm). Could any steps be skipped when there's only one option?
   - MCT adjustment: 2 screens per unit. Could streamline?
   - After contact result, returning to phase menu loses context — could offer "attack again?"
   - Tab for unit detail only works from phase menu, not from other screens
   - No "undo last move" option
   - Movement Esc goes to MoveSelectUnit, could go directly to PhaseMenu since you already moved
2. **Map position fine-tuning** — verify layout after Bresenham fixes
3. **Save/load game** — Ray noted this will be needed
4. **Sound/flash on combat results** — if terminal supports

### Uncommitted changes
- map.rs: Grozny name shortened
- ui.rs: M4 ● bold, roadblock ▓BLOCK▓, Bresenham center/skip fix, roster height, GUIDE color, map legend, guide text color
- All the session 3 stuff was committed as 41fc79b but these audit fixes are NOT yet committed

### Git status
Session 3 main work committed and pushed as 41fc79b. Audit fixes are uncommitted.
