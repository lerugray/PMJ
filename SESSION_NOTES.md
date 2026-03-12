# PMJ Digital — Session Notes

## Session 3 — 2026-03-12

### Summary
Flashy animated title screen, continuous movement, global help/quit, and contextual phase guides for new players.

### What got done this session
1. **Animated title screen** — Veridian Contraption-style with:
   - Block-letter ASCII art for PRIGOZHIN'S / MARCH OF / JUSTICE
   - Russian flag stripes (white/blue/red) at top and bottom
   - Staggered fade-in animation for each section
   - Background noise texture (drifting dots)
   - Color-cycling title (red/gold pulse)
   - Blinking "Press ENTER" prompt
   - Frame counter added to GameState and app loop
2. **Continuous movement** — after moving a unit, if it still has MP it stays in the destination picker so you can keep moving without re-selecting. Title bar shows remaining MP.
3. **Global `?` help** — works from any game screen, not just phase menu. Returns to exact previous screen.
4. **Global `Q` quit** — works from any game screen mid-game.
5. **Contextual phase guides** — every major screen now has built-in rules explanations:
   - MCT Select: explains what MCT does, shift up/down tradeoffs
   - Wagner Turn: explains options, victory conditions
   - Move Select: explains MP costs (river, roadblock), how to stop
   - Move Destination: shows remaining MP in title bar
   - Contact Select: explains CD, DRMs, result range
   - Russian AI: explains mobilization cup, roadblocks, AI behavior
   - End Turn: full momentum adjustment table (all 8 conditions)

### Second playtest feedback (from Ray)
- Title was slightly off-center (fixed: UTF-8 byte vs char count)
- "MARCH OF" ran together as one word (fixed: added word spacer)
- Wanted MARCH OF JUSTICE in block letters too (done: split into two lines)
- Movement one-area-at-a-time was tedious (fixed: continuous movement)
- `?` help didn't work outside phase menu (fixed: global handler)
- No way to quit mid-game (fixed: global Q handler)
- Wanted in-game rules guidance for new players (done: contextual guides)

### What's next (Session 4)
1. **Map position fine-tuning** — verify layout looks right during play
2. **Sound/flash on combat results** — if terminal supports it
3. **Save/load game** — Ray noted this will be needed eventually
4. **General playtesting polish** — anything that feels rough

### Git status
All committed and pushed to origin/master.
