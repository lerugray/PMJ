# main.py
#
# Entry point. Run this file to start the game.
# This is the CLI loop — it reads player input and calls GameState methods.
#
# To run:
#   python main.py

import sys
from game_state import GameState
from utils import print_header, print_divider, numbered_menu
from combat import (
    resolve_contact_with_flanking,
    location_is_empty_of_enemy,
)


def main():
    """Initialize the game and enter the main menu loop."""

    print_header("PRIGOZHIN'S MARCH ON JUSTICE  —  Phase 1 Engine")
    print("  Loading game state...")
    state = GameState()
    print("  Game ready. Type your choices by number and press Enter.\n")

    while True:
        # Always show the current board state before the menu
        state.display()

        print("  MAIN MENU")
        print_divider()

        choice = numbered_menu([
            "Move a Wagner unit",
            "Adjust MCT marker (Wagner unit)",
            "Adjust Momentum",
            "Initiate Contact (Wagner attacks)",
            "End Turn",
            "View full action log",
            "Quit",
        ])

        if choice == -1:
            print("  Invalid input. Please enter a number from the list.\n")
            continue

        elif choice == 0:
            _menu_move_unit(state)

        elif choice == 1:
            _menu_adjust_mct(state)

        elif choice == 2:
            _menu_adjust_momentum(state)

        elif choice == 3:
            _menu_contact(state)

        elif choice == 4:
            _menu_end_turn(state)

        elif choice == 5:
            _view_log(state)

        elif choice == 6:
            print("\n  Exiting PMJ Engine. Goodbye.\n")
            sys.exit(0)


# ============================================================================
# Sub-menus
# ============================================================================

def _menu_move_unit(state):
    """Let the player choose a Wagner unit and a destination."""

    print_header("MOVE A WAGNER UNIT")

    wagner_units = state.get_wagner_units()

    if not wagner_units:
        print("  No Wagner units are available.\n")
        return

    # Step 1 — choose unit
    unit_labels = []
    for u in wagner_units:
        mct_mp    = state.get_mct_mp(u)
        remaining = state.mp_remaining(u)
        unit_labels.append(
            f"{u.name}  (SP:{u.current_sp} MP:{remaining}/{mct_mp} remaining)"
            f"  @ {u.location}"
        )

    print("  Choose a unit to move:")
    unit_idx = numbered_menu(unit_labels)

    if unit_idx == -1:
        print("  Cancelled.\n")
        return

    unit = wagner_units[unit_idx]

    if state.mp_remaining(unit) == 0:
        print(f"  {unit.name} has no MP remaining this turn.\n")
        return

    # Step 2 — show reachable neighbors and let player choose destination
    neighbors = state.map.get_neighbors(unit.location)

    if not neighbors:
        print(f"  {unit.name} is at {unit.location}, which has no connections.\n")
        return

    print(f"\n  {unit.name} is at {unit.location}.")
    print(f"  Available MP remaining: {state.mp_remaining(unit)}\n")
    print("  Adjacent locations:")

    dest_labels = []
    dest_locs   = []

    for neighbor, props in sorted(neighbors, key=lambda x: x[0]):
        cost = state.move_cost(unit, unit.location, neighbor)
        tags = []
        if props["river"]:
            tags.append("river +1 MP")
        if props["m4"]:
            tags.append("M4 route")
        tag_str  = f"  [{', '.join(tags)}]" if tags else ""
        legal, _ = state.can_move(unit, neighbor)
        avail    = "" if legal else "  ✗ insufficient MP"

        dest_labels.append(f"{neighbor}  (cost: {cost} MP){tag_str}{avail}")
        dest_locs.append(neighbor)

    dest_idx = numbered_menu(dest_labels)

    if dest_idx == -1:
        print("  Cancelled.\n")
        return

    to_loc = dest_locs[dest_idx]
    state.move_unit(unit, to_loc)
    print()


def _menu_adjust_mct(state):
    """Slide a Wagner unit's MCT marker up or down one step."""

    print_header("ADJUST MCT MARKER")
    print("  The MCT controls Wagner unit SP and MP.\n")

    # Choose unit
    print("  Choose a Wagner unit:")
    unit_choice = numbered_menu(state.WAGNER_UNIT_NAMES)

    if unit_choice == -1:
        print("  Cancelled.\n")
        return

    unit_name = state.WAGNER_UNIT_NAMES[unit_choice]
    marker    = state.mct[unit_name]

    print(f"\n  {marker}")
    print("  Choose direction:")
    dir_choice = numbered_menu(["Shift UP (weaker/more MP)", "Shift DOWN (stronger/less MP)"])

    if dir_choice == -1:
        print("  Cancelled.\n")
        return

    if dir_choice == 0:
        msg = marker.shift_up()
    else:
        msg = marker.shift_down()

    state.record(f"MCT adjusted — {msg}")
    print()


def _menu_adjust_momentum(state):
    """Manually shift momentum by +1 or -1."""

    print_header("ADJUST MOMENTUM")
    print(f"  Current momentum: {state.momentum:+d}\n")
    print("  Choose direction:")

    choice = numbered_menu([
        "Shift +1 toward Wagner",
        "Shift -1 toward Russia",
    ])

    if choice == -1:
        print("  Cancelled.\n")
        return

    delta = 1 if choice == 0 else -1
    state.adjust_momentum(delta)
    print()


def _menu_contact(state):
    """
    Wagner-initiated Contact sub-menu.

    Flow:
      1. Choose a primary attacking location (must have Wagner units).
      2. Choose the target (adjacent Russian-occupied location).
      3. Choose which Wagner units at the primary location attack.
      4. Optionally add flanking units from other adjacent Wagner locations.
      5. Show the full odds breakdown and confirm.
      6. Roll, resolve, report.
      7. Offer Advance After Contact if the target is now empty.
    """

    print_header("INITIATE CONTACT")
    print("  Only Wagner units may initiate Contact.\n")

    # ── Step 1: find Wagner-occupied locations that have adjacent Russian units ──
    wagner_locs_with_targets = _find_contact_opportunities(state)

    if not wagner_locs_with_targets:
        print("  No Wagner units are adjacent to any Russian-occupied location.\n")
        return

    # ── Step 2: choose the primary attacking location ─────────────────────────
    loc_labels = []
    for wloc, targets in wagner_locs_with_targets:
        target_str = ", ".join(targets)
        units_here = [u for u in state.units_at(wloc) if u.side == "Wagner"]
        names_str  = ", ".join(u.name for u in units_here)
        loc_labels.append(
            f"{wloc}  [{names_str}]  →  can attack: {target_str}"
        )

    print("  Choose primary attacking location:")
    loc_idx = numbered_menu(loc_labels)

    if loc_idx == -1:
        print("  Cancelled.\n")
        return

    primary_loc, target_locs = wagner_locs_with_targets[loc_idx]
    primary_wagner = [u for u in state.units_at(primary_loc) if u.side == "Wagner"]

    # ── Step 3: choose the target location ───────────────────────────────────
    if len(target_locs) == 1:
        target_loc = target_locs[0]
        print(f"\n  Only one valid target: {target_loc}")
    else:
        print(f"\n  Choose target location to attack:")
        tgt_idx = numbered_menu(target_locs)
        if tgt_idx == -1:
            print("  Cancelled.\n")
            return
        target_loc = target_locs[tgt_idx]

    defending_units = [u for u in state.units_at(target_loc) if u.side == "Russia"]

    # ── Step 4: choose which Wagner units at primary location attack ──────────
    print(f"\n  Wagner units at {primary_loc}:")
    unit_labels = []
    for u in primary_wagner:
        eff_sp = state.get_effective_sp(u)
        unit_labels.append(
            f"{u.name}  (effective SP:{eff_sp})"
        )
    unit_labels.append("All of the above")

    print("  Choose attacking unit(s):")
    unit_choice = numbered_menu(unit_labels)

    if unit_choice == -1:
        print("  Cancelled.\n")
        return

    if unit_choice == len(primary_wagner):
        # "All" selected
        chosen_attackers = list(primary_wagner)
    else:
        chosen_attackers = [primary_wagner[unit_choice]]

    # ── Step 5: optional flanking units ──────────────────────────────────────
    flanking_groups = _collect_flanking_units(
        state, primary_loc, target_loc, chosen_attackers
    )

    # ── Step 6: show breakdown and confirm ───────────────────────────────────
    _display_contact_preview(
        state,
        chosen_attackers,
        primary_loc,
        flanking_groups,
        defending_units,
        target_loc,
    )

    confirm = input("\n  Resolve Contact? (y/n): ").strip().lower()
    if confirm != "y":
        print("  Contact cancelled.\n")
        return

    # ── Step 7: resolve ───────────────────────────────────────────────────────
    # Combine all attackers for flanking resolution
    all_attacking  = list(chosen_attackers)
    for grp, _ in flanking_groups:
        all_attacking.extend(grp)

    flanking_drm = min(len(flanking_groups), 2)

    outcome = resolve_contact_with_flanking(
        state,
        chosen_attackers,
        primary_loc,
        flanking_groups,
        defending_units,
        target_loc,
    )

    # ── Step 8: print resolution summary ─────────────────────────────────────
    _print_contact_outcome(outcome, target_loc)

    # ── Step 9: Advance After Contact (rule 6.2.4) ───────────────────────────
    if location_is_empty_of_enemy(target_loc, "Wagner", state):
        advance = input(
            f"\n  Target location {target_loc} is now clear. "
            f"Advance attacking units? (y/n): "
        ).strip().lower()
        if advance == "y":
            for unit in all_attacking:
                if unit.location is not None:   # Still on map
                    unit.location = target_loc
                    state.record(
                        f"{unit.name} advanced into {target_loc} after Contact"
                    )
            outcome["advance_taken"] = True

    print()

def _russian_phase(state):
    print_divider("RUSSIAN PHASE", 60)

    unit = state.draw_from_russian_cup()
    if unit == "PEOPLE_ARE_SILENT":
        print(" The People Are Silent marker pulled.")
        state.record("People Are Silent — no Russian unit deployed this turn.")
        
        momentum = state.momentum

        if momentum > 0:
            # Get Russian units currently on the map and not already reduced
            russian_units = [
                u for u in state.units
                if u.side == "Russia" and u.location and not u.is_reduced and u.reduced
            ]

            # Determine how many to reduce
            reduce_count = min(momentum, len(russian_units))

            for u in russian_units[:reduce_count]:
                u.is_reduced = True
                state.record(f"{u.name} flipped to REDUCED due to People Are Silent.")

        else:
            state.record("No reduction effect (Momentum 0 or less).")

        # Skip mobilization draw this turn
        print_divider()
        return

    if not unit:
        print("  No Russian units left in cup.")
        return

    # Default deployment
    deploy_loc = "Moscow"

    # Rule exceptions
    if unit.name == "Akhmat":
        deploy_loc = "Grozny Akhmat Base"
    elif unit.name in ("Mechanized Regiment", "Armored Regiment"):
        deploy_loc = "Kaluga"

    unit.location = deploy_loc
    state.record(f"{unit.name} deployed to {deploy_loc} from cup")
    print(f"  {unit.name} deployed to {deploy_loc}.")

# ============================================================================
# Contact helper functions
# ============================================================================

def _find_contact_opportunities(state):
    """
    Return a list of (wagner_location, [target_locations]) for each
    Wagner-occupied location that is adjacent to at least one Russian-occupied
    location.
    """
    results = []
    checked = set()

    for unit in state.get_wagner_units():
        loc = unit.location
        if not loc or loc in checked:
            continue
        checked.add(loc)

        neighbors = state.map.get_neighbors(loc)
        targets   = []
        for neighbor, _ in neighbors:
            occupants = state.units_at(neighbor)
            if any(u.side == "Russia" for u in occupants):
                targets.append(neighbor)

        if targets:
            results.append((loc, targets))

    return results


def _collect_flanking_units(state, primary_loc, target_loc, already_attacking):
    """
    Interactively ask the player if they want to add flanking attackers.

    Flanking units must be:
      - Wagner
      - In a location adjacent to the target (but NOT the primary location)
      - Not already part of the attack

    Returns list of (list of Unit, location) tuples.
    """
    flanking_groups = []

    # Find Wagner locations adjacent to the target (excluding primary)
    neighbors_of_target = state.map.get_neighbors(target_loc)
    flank_locs          = []

    for neighbor, _ in neighbors_of_target:
        if neighbor == primary_loc:
            continue
        wagner_here = [
            u for u in state.units_at(neighbor)
            if u.side == "Wagner" and u not in already_attacking
        ]
        if wagner_here:
            flank_locs.append((neighbor, wagner_here))

    if not flank_locs:
        return flanking_groups

    print(f"\n  Flanking opportunity: Wagner units adjacent to {target_loc}:")
    for floc, funits in flank_locs:
        names = ", ".join(u.name for u in funits)
        print(f"    {floc}: {names}")

    add_flanking = input(
        "  Add flanking units? (y/n): "
    ).strip().lower()

    if add_flanking != "y":
        return flanking_groups

    for floc, funits in flank_locs:
        if len(flanking_groups) >= 2:
            # Rulebook caps flanking DRM at +2
            print("  Flanking cap (+2) reached.\n")
            break

        print(f"\n  Units at {floc}:")
        flank_labels = [
            f"{u.name}  (effective SP:{state.get_effective_sp(u)})"
            for u in funits
        ]
        flank_labels.append("None from this location")

        flank_choice = numbered_menu(flank_labels)

        if flank_choice == -1 or flank_choice == len(funits):
            continue   # Skip this location

        flanking_groups.append(([funits[flank_choice]], floc))

    return flanking_groups


def _display_contact_preview(
    state,
    primary_attackers,
    primary_loc,
    flanking_groups,
    defending_units,
    target_loc,
):
    """Print a pre-resolution breakdown so the player can review the odds."""

    from combat import force_ratio_shift

    print_divider()
    print("  CONTACT PREVIEW")
    print_divider()

    # Attacking SP
    all_atk = list(primary_attackers)
    for grp, _ in flanking_groups:
        all_atk.extend(grp)

    attack_sp = sum(state.get_effective_sp(u) for u in all_atk)
    defend_sp = sum(state.get_effective_sp(u) for u in defending_units)
    cd_raw    = attack_sp - defend_sp
    fr_shift  = force_ratio_shift(attack_sp, defend_sp)
    cd_adj    = cd_raw + fr_shift

    print(f"  Attackers:  {', '.join(u.name for u in all_atk)}")
    print(f"  Attack SP:  {attack_sp}")
    print(f"  Defenders:  {', '.join(u.name for u in defending_units)}")
    print(f"  Defend SP:  {defend_sp}")
    print(f"  CD (raw):   {cd_raw:+d}")
    fr_dir = f"{'R' if fr_shift > 0 else 'L'}{abs(fr_shift)}" if fr_shift != 0 else "none"
    print(f"  FR Shift:   {fr_dir}  → adjusted CD: {cd_adj:+d}")

    # DRMs
    drms = []
    drms.append(("Momentum", state.momentum))

    edge = state.map.get_edge(primary_loc, target_loc)
    if edge and edge["river"]:
        non_heli = [u for u in primary_attackers if not u.is_helicopter()]
        if non_heli:
            drms.append(("River crossing", -1))

    if target_loc == "Moscow":
        drms.append(("Moscow defense", -2))

    flanking_count = min(len(flanking_groups), 2)
    if flanking_count:
        drms.append(("Flanking", flanking_count))

    total_drm = sum(v for _, v in drms)
    print(f"  DRMs:       " + "  ".join(f"{lbl} {v:+d}" for lbl, v in drms))
    print(f"  Total DRM:  {total_drm:+d}")
    print(f"  CRT col:    {max(-6, min(6, cd_adj))}")
    print_divider()


def _resolve_with_flanking_drm(
    state,
    primary_attackers,
    primary_loc,
    flanking_groups,
    defending_units,
    target_loc,
    flanking_drm,
):
    """
    Resolve Contact, correctly including the flanking DRM in the die roll
    BEFORE the CRT is looked up.

    This is a targeted wrapper around the raw contact math so that flanking
    is properly baked in rather than appended after the fact.
    """
    import random
    from combat import (
        force_ratio_shift,
        lookup_crt,
        _highest_sp_unit,
        _step_reduce_unit,
        _eliminate_unit,
        _find_retreat_location,
        _handle_switchable_units,
    )

    all_attackers = list(primary_attackers)
    for grp, _ in flanking_groups:
        all_attackers.extend(grp)

    attack_sp = sum(state.get_effective_sp(u) for u in all_attackers)
    defend_sp = sum(state.get_effective_sp(u) for u in defending_units)

    cd_raw   = attack_sp - defend_sp
    fr_shift = force_ratio_shift(attack_sp, defend_sp)
    cd_adj   = cd_raw + fr_shift

    drm_breakdown = []
    momentum_drm  = state.momentum
    drm_breakdown.append(("Momentum", momentum_drm))

    river_drm = 0
    edge = state.map.get_edge(primary_loc, target_loc)
    if edge and edge["river"]:
        non_heli = [u for u in primary_attackers if not u.is_helicopter()]
        if non_heli:
            river_drm = -1
            drm_breakdown.append(("River crossing", river_drm))

    moscow_drm = 0
    if target_loc == "Moscow":
        moscow_drm = -2
        drm_breakdown.append(("Moscow defense", moscow_drm))

    if flanking_drm > 0:
        drm_breakdown.append(("Flanking", flanking_drm))

    drm_total = momentum_drm + river_drm + moscow_drm + flanking_drm
    die_roll  = random.randint(1, 6)
    final_die = die_roll + drm_total
    result    = lookup_crt(final_die, cd_adj)

    # ── Apply result ─────────────────────────────────────────────────────────
    state.record(
        f"Contact: {', '.join(u.name for u in all_attackers)} "
        f"@ {primary_loc} → {target_loc}  |  "
        f"ATK SP:{attack_sp} DEF SP:{defend_sp}  |  "
        f"CD:{cd_adj:+d}  |  "
        f"Roll:{die_roll} DRM:{drm_total:+d} Final:{final_die}  |  "
        f"Result: {result}"
    )

    advance_taken = False

    if result == "AR":
        highest_atk = _highest_sp_unit(all_attackers, state)
        if highest_atk:
            _eliminate_unit(highest_atk, state, "AR — Attacker Routed")
        for unit in all_attackers:
            if unit.location is not None:
                retreat_to = _find_retreat_location(unit, unit.location, state)
                if retreat_to:
                    unit.location = retreat_to
                    state.record(
                        f"{unit.name} retreated → {retreat_to} [AR]"
                    )
                else:
                    unit.location = None
                    state.record(f"{unit.name} DISPERSED [AR]")

    elif result == "Ar":
        for unit in all_attackers:
            retreat_to = _find_retreat_location(unit, unit.location, state)
            if retreat_to:
                unit.location = retreat_to
                state.record(
                    f"{unit.name} retreated → {retreat_to} [Ar]"
                )
            else:
                unit.location = None
                state.record(f"{unit.name} DISPERSED [Ar]")

    elif result == "EX":
        switched = _handle_switchable_units(
            defending_units, primary_loc, state
        )
        if not switched:
            highest_def = _highest_sp_unit(defending_units, state)
            if highest_def:
                _step_reduce_unit(highest_def, state, "EX")
        highest_atk = _highest_sp_unit(all_attackers, state)
        if highest_atk:
            _step_reduce_unit(highest_atk, state, "EX")

    elif result == "NE":
        state.record("Contact result: No Effect (NE)")

    elif result == "Rp":
        if target_loc == "Moscow":
            highest_def = _highest_sp_unit(defending_units, state)
            if highest_def:
                _step_reduce_unit(highest_def, state, "Rp — Moscow")
        else:
            for unit in defending_units:
                retreat_to = _find_retreat_location(unit, target_loc, state)
                if retreat_to:
                    unit.location = retreat_to
                    state.record(
                        f"{unit.name} retreated {target_loc} → "
                        f"{retreat_to} [Rp]"
                    )
                else:
                    unit.location = None
                    state.record(f"{unit.name} DISPERSED [Rp]")

    elif result == "R":
        highest_def = _highest_sp_unit(defending_units, state)
        if highest_def:
            _eliminate_unit(highest_def, state, "R — Routed")
        remaining_def = [u for u in defending_units if u.location is not None]
        for unit in remaining_def:
            if target_loc == "Moscow":
                _step_reduce_unit(unit, state, "R — Moscow")
            else:
                retreat_to = _find_retreat_location(unit, target_loc, state)
                if retreat_to:
                    unit.location = retreat_to
                    state.record(
                        f"{unit.name} retreated {target_loc} → "
                        f"{retreat_to} [R]"
                    )
                else:
                    unit.location = None
                    state.record(f"{unit.name} DISPERSED [R]")

    elif result == "S":
        switched = _handle_switchable_units(
            defending_units, primary_loc, state
        )
        remaining_to_process = [
            u for u in defending_units
            if u.name not in switched and u.location is not None
        ]
        highest_def = _highest_sp_unit(remaining_to_process, state)
        if highest_def:
            highest_def.location  = None
            highest_def.is_reduced = False
            state.record(
                f"{highest_def.name} SURRENDERED — permanently removed [S]"
            )
        remaining_def = [
            u for u in remaining_to_process
            if u.location is not None and u is not highest_def
        ]
        for unit in remaining_def:
            if target_loc == "Moscow":
                _step_reduce_unit(unit, state, "S — Moscow")
            else:
                retreat_to = _find_retreat_location(unit, target_loc, state)
                if retreat_to:
                    unit.location = retreat_to
                    state.record(
                        f"{unit.name} retreated {target_loc} → "
                        f"{retreat_to} [S]"
                    )
                else:
                    unit.location = None
                    state.record(f"{unit.name} DISPERSED [S]")

    return {
        "die_roll":      die_roll,
        "cd_raw":        cd_raw,
        "fr_shift":      fr_shift,
        "cd_adjusted":   cd_adj,
        "drm_total":     drm_total,
        "final_die":     final_die,
        "cd_final":      max(-6, min(6, cd_adj)),
        "result":        result,
        "attack_sp":     attack_sp,
        "defend_sp":     defend_sp,
        "drm_breakdown": drm_breakdown,
        "advance_taken": advance_taken,
        "flanking_drm":  flanking_drm,
    }


def _print_contact_outcome(outcome, target_loc):
    """Print a formatted summary of the contact resolution."""
    print_divider()
    print("  CONTACT RESOLVED")
    print_divider()
    print(f"  Attack SP:    {outcome['attack_sp']}")
    print(f"  Defend SP:    {outcome['defend_sp']}")
    print(f"  CD (raw):     {outcome['cd_raw']:+d}")
    fr_shift = outcome['fr_shift']
    fr_dir   = (f"{'R' if fr_shift > 0 else 'L'}{abs(fr_shift)}"
                if fr_shift != 0 else "none")
    print(f"  FR Shift:     {fr_dir}  → CD col: {outcome['cd_final']:+d}")
    drm_str  = "  ".join(
        f"{lbl} {v:+d}" for lbl, v in outcome['drm_breakdown']
    )
    print(f"  DRMs:         {drm_str if drm_str else 'none'}")
    print(f"  Total DRM:    {outcome['drm_total']:+d}")
    print(f"  Die roll:     {outcome['die_roll']}")
    print(f"  Final roll:   {outcome['final_die']}  (clamped for CRT lookup)")
    print_divider()
    print(f"  ══► RESULT:  {outcome['result']}  ◄══")
    print_divider()

    result_descriptions = {
        "AR": "ATTACKER ROUTED — highest SP attacker eliminated, rest retreat.",
        "Ar": "ATTACKER REPULSED — all attacking units retreat.",
        "EX": "EXCHANGE — both sides step-reduce their highest SP unit.",
        "NE": "NO EFFECT — no changes.",
        "Rp": "REPULSED — defending units retreat (or step-reduced in Moscow).",
        "R":  "ROUTED — highest SP defender eliminated, rest retreat.",
        "S":  "SURRENDER — highest SP defender permanently removed, rest retreat.",
    }
    desc = result_descriptions.get(outcome['result'], "")
    if desc:
        print(f"  {desc}")
    print_divider()


def _menu_end_turn(state):
    """Confirm and advance the turn counter."""

    print_header("END TURN")

    if state.turn >= 6:
        print("  The game is already on Turn 6 (the final turn).")
        print("  (Full end-game logic not yet implemented.)\n")
        return

    print(f"  End Turn {state.turn} and advance to Turn {state.turn + 1}?")
    confirm = input("  Type YES to confirm: ").strip().upper()

    if confirm == "YES":
        state.end_turn()
        _russian_phase(state)
        print(f"  Turn {state.turn} has begun.\n")
    else:
        print("  Turn not ended.\n")


def _view_log(state):
    """Print the entire action log."""

    print_header("FULL ACTION LOG")

    if not state.log:
        print("  (no actions recorded yet)\n")
        return

    for i, entry in enumerate(state.log, 1):
        print(f"  {i:>3}. {entry}")

    print()


# ============================================================================
# Run
# ============================================================================

if __name__ == "__main__":
    main()
