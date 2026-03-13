# PMJ Digital — Session Notes

## Session 4 — 2026-03-12

### Summary
UX flow improvements for smoother playtesting, save/load system, and map layout fixes.

### What got done this session
1. **Contact auto-skip** — when there's only one attack location, one target, or one eligible attacker, the game skips straight to the relevant screen
2. **Attack again flow** — after resolving contact, if more attacks are available, Enter goes straight into the next attack. Esc always returns to phase menu.
3. **Movement Esc → PhaseMenu** — Esc from destination picker goes directly back to phase menu
4. **Tab unit detail from more screens** — Tab works from MoveSelectUnit and MoveSelectDest, returns to the screen you came from
5. **Save/Load system** — Ctrl+S saves from any game screen, L loads from title screen. Single save file (pmj_save.json). Status flash in header.
6. **Status flash messages** — transient green badge in header bar for save confirmation, etc.
7. **Map M4 visibility fix** — spread M4 locations to 7-row gaps, tightened Bresenham skip zones so cyan dots are visible between all locations
8. **Dynamic map centering** — map content auto-centers horizontally in the panel regardless of terminal width
9. **Location indicators** — Rublevo ⌂⌂⌂ (suburbs), Moscow ⬤ (capital), Rostov HQ (Wagner base), Grozny ⚑ (Akhmat base)
10. **Right panel centering** — header, momentum bar, and map legend centered in their panels
11. **Bugaevka name** — shortened to "Bugaevka B.P."

### What needs doing next (Session 5)
1. **Playtesting** — run through a full game to verify all the flow improvements feel right
2. **Map fine-tuning** — verify positions/indicators look right after layout changes
3. **Sound/flash on combat results** — if terminal supports

### Git status
All committed and pushed.
