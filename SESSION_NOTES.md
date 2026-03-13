# PMJ Digital — Session Notes

## Session 6 — 2026-03-13

### Summary
River visual overhaul, cursor-based map highlighting, and code review bug fixes.

### What got done this session
1. **River visuals reworked** — rivers are now hardcoded ≈ arcs attached to location box sides, matching the board map style. Oka River (bottom), Voronezh (left), Bugaevka (below), Rostov-Grozny (vertical between).
2. **Move list stabilization** — fixed flickering where same-cost destinations swapped order each frame (added tiebreaker sort by location).
3. **Cursor-based map highlighting** — when scrolling through unit select, move destinations, or contact targets, the selected location lights up on the map (white on blue).
4. **Bug: Roadblock deployment** — second roadblock slot never got filled due to `is_some()`/`is_none()` inversion. Now correctly deploys to empty slot or repositions if both filled.
5. **Bug: Flanking counted police units** — OMON/MOSpol were being counted as flanking combat units even though they can't attack. Fixed to only count non-police Wagner units.
6. **Bug: Division by zero in combat** — if attack_sp somehow reached 0, force_ratio_shift would produce NaN. Added guard returning -3L shift.
7. **UX: Tab unit detail feedback** — Tab now flashes "No units on map" instead of silently failing.
8. **UX: Post-combat cursor** — after contact resolves, cursor returns to Contact menu option instead of Move.
9. **UX: [NO MP] indicator** — units with 0 MP remaining show [NO MP] tag in the move unit list.

### What needs doing next (Session 7)
1. **Playtesting** — run through a full game to verify all fixes
2. **Sound/flash on combat results** — if terminal supports

### Git status
Committed and pushed.
