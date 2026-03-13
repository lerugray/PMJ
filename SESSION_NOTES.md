# PMJ Digital — Session Notes

## Session 4 — 2026-03-12

### Summary
UX flow improvements for smoother playtesting + save/load system.

### What got done this session
1. **Contact auto-skip** — when there's only one attack location, one target, or one eligible attacker, the game skips straight to the relevant screen instead of making you click through menus with only one option
2. **Attack again flow** — after resolving contact, if more attacks are available, Enter goes straight into the next attack (with auto-skip). Esc always returns to phase menu. The contact result screen shows which option is which.
3. **Movement Esc → PhaseMenu** — pressing Esc from the destination picker now goes directly back to the phase menu instead of the unit selector (you already committed to that unit)
4. **Tab unit detail from more screens** — Tab now opens unit detail from MoveSelectUnit and MoveSelectDest, not just PhaseMenu. Unit detail returns to the screen you came from (not always PhaseMenu).
5. **Save/Load system** — Ctrl+S saves from any game screen, L loads from title screen. Single save file (pmj_save.json). Uses serde for serialization. Status flash ("Game saved!") shows in header for ~2 seconds.
6. **Status flash messages** — transient messages (save confirmation, "no contact opportunities") display as green badges in the header bar, auto-clearing after ~2 seconds.

### What needs doing next (Session 5)
1. **Playtesting** — run through a full game to verify all the flow improvements feel right
2. **Map position fine-tuning** — verify layout looks good after all changes
3. **More Tab access** — could add Tab to contact screens too
4. **Sound/flash on combat results** — if terminal supports
5. **Save slot on game over** — auto-delete save on game end?

### Git status
All changes uncommitted.
