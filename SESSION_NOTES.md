# PMJ Digital — Session Notes

## Session 6 — 2026-03-13

### Summary
River visual overhaul, movement list stabilization, and cursor-based map highlighting.

### What got done this session
1. **River visuals reworked** — rivers are now hardcoded ≈ arcs attached to location box sides, matching the board map style. Oka River (bottom), Voronezh (left), Bugaevka (below), Rostov-Grozny (vertical between).
2. **Move list stabilization** — fixed flickering where same-cost destinations swapped order each frame (added tiebreaker sort by location).
3. **Cursor-based map highlighting** — when scrolling through the move destination list or contact target list, the selected location lights up on the map (white on blue). Works for unit select, move destinations, contact location select, and contact target select.

### What needs doing next (Session 7)
1. **Playtesting** — run through a full game to verify all changes
2. **River position tweaking** — may need minor adjustments after playtesting
3. **Sound/flash on combat results** — if terminal supports

### Git status
Committed and pushed.
