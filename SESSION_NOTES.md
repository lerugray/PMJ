# PMJ Digital — Session Notes

## Session 4 — 2026-03-12

### Summary
UX flow improvements for smoother playtesting, save/load system, and map layout fixes.

### What got done this session
1. **Contact auto-skip** — when there's only one attack location, one target, or one eligible attacker, the game skips straight to the relevant screen instead of making you click through menus with only one option
2. **Attack again flow** — after resolving contact, if more attacks are available, Enter goes straight into the next attack (with auto-skip). Esc always returns to phase menu. The contact result screen shows which option is which.
3. **Movement Esc → PhaseMenu** — pressing Esc from the destination picker now goes directly back to the phase menu instead of the unit selector (you already committed to that unit)
4. **Tab unit detail from more screens** — Tab now opens unit detail from MoveSelectUnit and MoveSelectDest, not just PhaseMenu. Unit detail returns to the screen you came from (not always PhaseMenu).
5. **Save/Load system** — Ctrl+S saves from any game screen, L loads from title screen. Single save file (pmj_save.json). Uses serde for serialization. Status flash ("Game saved!") shows in header for ~2 seconds.
6. **Status flash messages** — transient messages (save confirmation, "no contact opportunities") display as green badges in the header bar, auto-clearing after ~2 seconds.
7. **Map M4 visibility fix** — spread M4 locations to 7-row vertical gaps, tightened Bresenham skip zones from 6 rows to 4 (exact box + unit row, no margins). M4 cyan dots now visible between all locations.
8. **Map centered** — shifted all positions right ~4 columns to center in panel
9. **Location indicators** — Rublevo shows ⌂⌂⌂ (Moscow suburbs), Moscow shows ⬤ (capital), Rostov shows HQ (Wagner base), Grozny shows ⚑ (Akhmat base)
10. **Bugaevka name** — shortened from "Bugaevka Border Pt" to "Bugaevka B.P." to avoid ugly truncation

### What needs doing next (Session 5)
1. **Playtesting** — run through a full game to verify all the flow improvements feel right
2. **Map fine-tuning** — verify positions/indicators look right after all layout changes
3. **Sound/flash on combat results** — if terminal supports

### Git status
All committed and pushed.
