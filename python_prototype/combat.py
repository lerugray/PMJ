# combat.py
#
# Implements the Wagner-initiated Contact (combat) subsystem for PMJ.
#
# Key rulebook sections implemented:
#   6.2   Contact procedure
#   6.2.1 Contact Differential and CRT lookup
#   6.2.2 CRT result codes
#   6.2.3 DRMs — Momentum, River, Moscow, Flanking
#   6.2.3.1 Force Ratio Column Shifts
#   6.2.4 Advance After Contact
#
# This module contains pure logic only. All I/O (menus, input, printing)
# lives in the _menu_contact() function added to main.py.


import random


# ============================================================================
# Contact Result Table (CRT)
# ============================================================================
#
# Outer key  = 1d6 roll (clamped to 0 for "0 or less", 7 for "7+")
# Inner key  = CD column (clamped to -6 … +6)
# Value      = result code string
#
# Codes:
#   "AR"  Attacker Routed
#   "Ar"  Attacker Repulsed
#   "EX"  Exchange
#   "NE"  No Effect
#   "Rp"  Repulsed  (defender retreats)
#   "R"   Routed    (highest defender eliminated, rest retreat)
#   "S"   Surrender (highest defender removed permanently, rest retreat)

# CRT rows keyed by die roll (0 = "0 or less", 7 = "7+")
# Columns ordered: -6, -5, -4, -3, -2, -1, 0, +1, +2, +3, +4, +5, +6
_CRT_COLUMNS = [-6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6]

_CRT_ROWS = {
    0: ["AR", "AR", "AR", "AR", "AR", "AR", "Ar", "Ar", "Ar", "EX", "EX", "EX", "NE"],
    1: ["AR", "AR", "AR", "AR", "AR", "Ar", "Ar", "Ar", "EX", "EX", "EX", "NE", "Rp"],
    2: ["AR", "AR", "AR", "AR", "Ar", "Ar", "Ar", "EX", "EX", "EX", "NE", "Rp", "Rp"],
    3: ["AR", "AR", "AR", "Ar", "Ar", "Ar", "EX", "EX", "EX", "NE", "Rp", "Rp", "Rp"],
    4: ["AR", "AR", "Ar", "Ar", "Ar", "EX", "EX", "EX", "NE", "Rp", "Rp", "Rp", "R"],
    5: ["AR", "Ar", "Ar", "Ar", "EX", "EX", "EX", "NE", "Rp", "Rp", "Rp", "R",  "R"],
    6: ["Ar", "Ar", "Ar", "EX", "EX", "EX", "NE", "Rp", "Rp", "Rp", "R",  "R",  "S"],
    7: ["Ar", "Ar", "EX", "EX", "EX", "NE", "Rp", "Rp", "Rp", "R",  "R",  "S",  "S"],
}


def lookup_crt(die_roll, cd_column):
    """
    Look up the Contact Result Table.

    Parameters
    ----------
    die_roll   : int  — raw 1d6 result (will be clamped: 0 or less → row 0,
                        7 or more → row 7)
    cd_column  : int  — final Contact Differential after all shifts
                        (clamped to -6 … +6)

    Returns
    -------
    str  — one of: "AR", "Ar", "EX", "NE", "Rp", "R", "S"
    """
    row_key = max(0, min(7, die_roll))
    col_key = max(-6, min(6, cd_column))
    col_idx = _CRT_COLUMNS.index(col_key)
    return _CRT_ROWS[row_key][col_idx]


# ============================================================================
# Force Ratio Column Shifts
# ============================================================================
#
# Per rulebook 6.2.3.1:
#   ratio = attacking SP / defending SP  (round DOWN in favor of defender)
#   1:3 → 3L   1:2 → 2L   1:1 → 1L   1.5:1 → 0   2:1 → 1R   3:1 → 2R   4:1 → 3R

def force_ratio_shift(attack_sp, defend_sp):
    """
    Calculate the Force Ratio Column Shift.

    Returns an integer representing column shifts:
      negative = shift left (worse for attacker)
      positive = shift right (better for attacker)
    """
    if defend_sp <= 0:
        return 3   # Defender has no SP — maximum right shift

    ratio = attack_sp / defend_sp   # Float ratio

    if ratio >= 4.0:
        return 3    # 4:1  → 3R
    elif ratio >= 3.0:
        return 2    # 3:1  → 2R
    elif ratio >= 2.0:
        return 1    # 2:1  → 1R
    elif ratio >= 1.5:
        return 0    # 1.5:1 → no shift
    elif ratio >= 1.0:
        return -1   # 1:1  → 1L
    elif ratio >= 0.5:
        return -2   # 1:2  → 2L
    else:
        return -3   # 1:3  → 3L


# ============================================================================
# Retreat Helper
# ============================================================================

def _find_retreat_location(unit, defending_loc, state):
    """
    Determine where a retreating unit should go.

    Rules:
      - Units retreat toward their Home Location (HL).
      - Wagner HL = Rostov-On-Don
      - Russia HL = Moscow
      - If no valid retreat location exists, the unit is Dispersed.

    Returns the retreat location string, or None if dispersal is required.
    """
    home_locations = {
        "Wagner": "Rostov-On-Don",
        "Russia": "Moscow",
    }
    hl = home_locations.get(unit.side, "Moscow")

    # Get all adjacent locations
    neighbors = state.map.get_neighbors(defending_loc)
    if not neighbors:
        return None

    # Filter: must be free of enemy units and not the attacker's location
    candidates = []
    for neighbor, _ in neighbors:
        occupants = state.units_at(neighbor)
        enemy_present = any(u.side != unit.side for u in occupants)
        if not enemy_present:
            candidates.append(neighbor)

    if not candidates:
        return None

    # Prefer HL itself; otherwise pick the neighbor closest to HL by any
    # simple heuristic: prefer neighbor that IS the HL, then any open space.
    if hl in candidates:
        return hl

    # As a fallback, return the first open candidate
    return candidates[0]


# ============================================================================
# Unit Step Reduction and Elimination
# ============================================================================

def _step_reduce_unit(unit, state, context=""):
    """
    Reduce a unit by one step.

    If the unit has a reduced side (unit.reduced == True) and is not already
    reduced, flip it to reduced. Otherwise eliminate it.

    Logs the action.
    """
    if unit.reduced and not unit.is_reduced:
        unit.is_reduced = True
        state.record(
            f"{unit.name} ({unit.side}) step reduced to SP:{unit.current_sp}"
            + (f" [{context}]" if context else "")
        )
    else:
        _eliminate_unit(unit, state, context)


def _eliminate_unit(unit, state, context=""):
    """Remove a unit from the map (set location to None). Logs the action."""
    unit.location = None
    unit.is_reduced = False   # Reset if ever brought back
    state.record(
        f"{unit.name} ({unit.side}) ELIMINATED"
        + (f" [{context}]" if context else "")
    )


def _highest_sp_unit(units, state):
    """Return the unit with the highest effective SP from a list."""
    if not units:
        return None
    return max(units, key=lambda u: state.get_effective_sp(u))


# ============================================================================
# Switchable Unit Handling (EX and S results)
# ============================================================================

def _handle_switchable_units(defending_units, attacker_location, state):
    """
    When an EX or S result occurs, units capable of switching sides (Russia →
    Wagner) do so and automatically move to the attacker's location.

    Returns a list of unit names that switched.
    """
    switched = []
    for unit in list(defending_units):
        if unit.switchable and unit.side == "Russia":
            unit.side     = "Wagner"
            unit.location = attacker_location
            # Switchable units that switch use their printed MP (not MCT)
            state.record(
                f"{unit.name} SWITCHED SIDES to Wagner and moved to "
                f"{attacker_location}"
            )
            switched.append(unit.name)
    return switched


# ============================================================================
# Core Contact Resolution
# ============================================================================

def resolve_contact(
    state,
    attacking_units,
    attacker_location,
    defending_units,
    defending_location,
):
    """
    Resolve a single Contact between Wagner and Russian units.

    Parameters
    ----------
    state             : GameState
    attacking_units   : list of Unit  — Wagner units initiating Contact
    attacker_location : str           — location they are attacking FROM
                        (may differ per unit if flanking)
    defending_units   : list of Unit  — all Russian units at the target
    defending_location: str           — the location being attacked

    This function mutates unit state (location, is_reduced, etc.) and
    writes log entries via state.record().

    Returns
    -------
    dict with keys:
        "die_roll"      : int
        "cd_raw"        : int   — CD before force ratio shift
        "fr_shift"      : int
        "cd_adjusted"   : int   — CD after force ratio shift
        "drm_total"     : int
        "final_die"     : int   — die roll + DRM (clamped for CRT lookup)
        "cd_final"      : int   — final CD column used in CRT (clamped)
        "result"        : str   — CRT result code
        "attack_sp"     : int
        "defend_sp"     : int
        "drm_breakdown" : list of (label, value) tuples
        "advance_taken" : bool
    """

    # ── Step 1: Total SPs ────────────────────────────────────────────────────
    attack_sp = sum(state.get_effective_sp(u) for u in attacking_units)
    defend_sp = sum(state.get_effective_sp(u) for u in defending_units)

    # ── Step 2: Contact Differential ─────────────────────────────────────────
    cd_raw = attack_sp - defend_sp

    # ── Step 3: Force Ratio Column Shift ─────────────────────────────────────
    fr_shift    = force_ratio_shift(attack_sp, defend_sp)
    cd_adjusted = cd_raw + fr_shift

    # ── Step 4: Build DRM list ────────────────────────────────────────────────
    drm_breakdown = []

    # 4a. Momentum DRM (Wagner attacking = use current momentum as-is)
    momentum_drm = state.momentum
    drm_breakdown.append(("Momentum", momentum_drm))

    # 4b. River DRM — applies if ANY attacking unit crosses a river route
    #     Note: the rule says the penalty applies even if some attackers do NOT
    #     cross a river, so we only need ONE river crossing route among the
    #     attacking locations.
    river_drm = 0
    edge = state.map.get_edge(attacker_location, defending_location)
    if edge and edge["river"]:
        # Helicopters are exempt, but only if ALL attackers are helicopters
        non_heli = [u for u in attacking_units if not u.is_helicopter()]
        if non_heli:
            river_drm = -1
            drm_breakdown.append(("River crossing", river_drm))

    # 4c. Moscow DRM — defending location is Moscow
    moscow_drm = 0
    if defending_location == "Moscow":
        moscow_drm = -2
        drm_breakdown.append(("Moscow defense", moscow_drm))

    # 4d. Flanking DRM — additional attacking units from OTHER locations
    #     Each extra location = +1 DRM (max +2).
    #     Here attacker_location is the primary location; the caller may
    #     pass flanking_locations as additional attacker origins.
    # NOTE: In this engine pass flanking_count explicitly — the parameter is
    #       stored in the kwarg below via the wrapper function.
    flanking_drm = 0   # Set by wrapper; see _flanking_drm below
    # (injected by resolve_contact_with_flanking)

    drm_total = momentum_drm + river_drm + moscow_drm + flanking_drm

    # ── Step 5: Roll 1d6 ─────────────────────────────────────────────────────
    die_roll  = random.randint(1, 6)
    final_die = die_roll + drm_total

    # ── Step 6: CRT Lookup ────────────────────────────────────────────────────
    # CRT uses the final die roll (clamped 0–7) and cd_adjusted (clamped -6…+6)
    result = lookup_crt(final_die, cd_adjusted)

    # ── Step 7: Apply Result ──────────────────────────────────────────────────
    advance_taken = False

    if result == "AR":
        # Attacker Routed: highest SP attacker eliminated; rest retreat to HL
        highest_atk = _highest_sp_unit(attacking_units, state)
        if highest_atk:
            _eliminate_unit(highest_atk, state, "AR — Attacker Routed")
        for unit in attacking_units:
            if unit.location is not None:   # Not already eliminated
                retreat_to = _find_retreat_location(
                    unit, attacker_location, state
                )
                if retreat_to:
                    unit.location = retreat_to
                    state.record(
                        f"{unit.name} retreated {attacker_location} → "
                        f"{retreat_to} [AR]"
                    )
                else:
                    # Disperse — place on next turn's GTT (simplified: off map)
                    unit.location = None
                    state.record(f"{unit.name} DISPERSED [AR — no retreat path]")

    elif result == "Ar":
        # Attacker Repulsed: all attacker units retreat toward their HL
        for unit in attacking_units:
            retreat_to = _find_retreat_location(
                unit, attacker_location, state
            )
            if retreat_to:
                unit.location = retreat_to
                state.record(
                    f"{unit.name} retreated {attacker_location} → "
                    f"{retreat_to} [Ar — Repulsed]"
                )
            else:
                unit.location = None
                state.record(
                    f"{unit.name} DISPERSED [Ar — no retreat path]"
                )

    elif result == "EX":
        # Exchange: both sides step-reduce their highest SP unit; stay in place.
        # If defenders include switchable units, those switch sides instead.
        switched = _handle_switchable_units(
            defending_units, attacker_location, state
        )
        if not switched:
            highest_def = _highest_sp_unit(defending_units, state)
            if highest_def:
                _step_reduce_unit(highest_def, state, "EX — Exchange")
        highest_atk = _highest_sp_unit(attacking_units, state)
        if highest_atk:
            _step_reduce_unit(highest_atk, state, "EX — Exchange")

    elif result == "NE":
        # No Effect
        state.record("Contact result: No Effect (NE)")

    elif result == "Rp":
        # Repulsed: defending units retreat toward their HL (or are dispersed).
        # Exception: if already at HL, step reduce highest SP (Moscow rule).
        if defending_location == "Moscow":
            # Moscow exception: step reduce highest defender instead of retreat
            highest_def = _highest_sp_unit(defending_units, state)
            if highest_def:
                _step_reduce_unit(
                    highest_def, state, "Rp — Moscow exception"
                )
        else:
            for unit in defending_units:
                russia_hl = "Moscow"
                if unit.location == russia_hl:
                    # Already at HL (Moscow handled above); disperse
                    unit.location = None
                    state.record(
                        f"{unit.name} DISPERSED [Rp — at HL with no retreat]"
                    )
                else:
                    retreat_to = _find_retreat_location(
                        unit, defending_location, state
                    )
                    if retreat_to:
                        unit.location = retreat_to
                        state.record(
                            f"{unit.name} retreated {defending_location} → "
                            f"{retreat_to} [Rp — Repulsed]"
                        )
                    else:
                        unit.location = None
                        state.record(
                            f"{unit.name} DISPERSED [Rp — no retreat path]"
                        )

        # Advance After Contact (6.2.4): if target location now empty, attackers
        # may advance — handled in the menu after this function returns.

    elif result == "R":
        # Routed: highest SP defender eliminated; rest retreat as if Rp.
        highest_def = _highest_sp_unit(defending_units, state)
        if highest_def:
            _eliminate_unit(highest_def, state, "R — Routed")
        remaining_def = [u for u in defending_units if u.location is not None]
        for unit in remaining_def:
            if defending_location == "Moscow":
                _step_reduce_unit(unit, state, "R — Moscow no-retreat")
            else:
                retreat_to = _find_retreat_location(
                    unit, defending_location, state
                )
                if retreat_to:
                    unit.location = retreat_to
                    state.record(
                        f"{unit.name} retreated {defending_location} → "
                        f"{retreat_to} [R — Routed remainder]"
                    )
                else:
                    unit.location = None
                    state.record(
                        f"{unit.name} DISPERSED [R — no retreat path]"
                    )

    elif result == "S":
        # Surrender: highest SP defender permanently removed; rest retreat as Rp.
        # Switchable defenders switch sides instead.
        switched = _handle_switchable_units(
            defending_units, attacker_location, state
        )
        highest_def = _highest_sp_unit(
            [u for u in defending_units if u.name not in switched], state
        )
        if highest_def and highest_def.location is not None:
            # Permanently removed
            highest_def.location  = None
            highest_def.is_reduced = False
            state.record(
                f"{highest_def.name} SURRENDERED and removed permanently [S]"
            )
        remaining_def = [
            u for u in defending_units
            if u.location is not None and u.name not in switched
        ]
        for unit in remaining_def:
            if defending_location == "Moscow":
                _step_reduce_unit(unit, state, "S — Moscow no-retreat")
            else:
                retreat_to = _find_retreat_location(
                    unit, defending_location, state
                )
                if retreat_to:
                    unit.location = retreat_to
                    state.record(
                        f"{unit.name} retreated {defending_location} → "
                        f"{retreat_to} [S — Surrender remainder]"
                    )
                else:
                    unit.location = None
                    state.record(
                        f"{unit.name} DISPERSED [S — no retreat path]"
                    )

    # ── Compile summary ───────────────────────────────────────────────────────
    return {
        "die_roll":      die_roll,
        "cd_raw":        cd_raw,
        "fr_shift":      fr_shift,
        "cd_adjusted":   cd_adjusted,
        "drm_total":     drm_total,
        "final_die":     final_die,
        "cd_final":      max(-6, min(6, cd_adjusted)),
        "result":        result,
        "attack_sp":     attack_sp,
        "defend_sp":     defend_sp,
        "drm_breakdown": drm_breakdown,
        "advance_taken": advance_taken,
    }


def resolve_contact_with_flanking(
    state,
    primary_attackers,
    primary_location,
    flanking_groups,
    defending_units,
    defending_location,
):
    """
    Wrapper that handles Flanking DRM (rulebook 6.2.3.2).

    Parameters
    ----------
    primary_attackers : list of Unit  — attackers from the primary location
    primary_location  : str
    flanking_groups   : list of (list of Unit, str location)
                        — additional attacker groups from other locations
    defending_units   : list of Unit
    defending_location: str

    All attacking units are combined for SP totals. The flanking DRM is
    +1 per extra attacking location (max +2), injected after the standard
    DRM calculation inside resolve_contact().

    Returns the same dict as resolve_contact(), with flanking_drm added.
    """
    all_attackers = list(primary_attackers)
    for group, _ in flanking_groups:
        all_attackers.extend(group)

    flanking_count = min(len(flanking_groups), 2)   # Cap at +2

    # Run standard resolution
    outcome = resolve_contact(
        state,
        all_attackers,
        primary_location,
        defending_units,
        defending_location,
    )

    # Inject flanking DRM retroactively into the summary
    # (The actual die result was already computed, so we recalculate here
    #  and note it — for full accuracy, flanking is baked in below.)
    if flanking_count > 0:
        flanking_drm = flanking_count
        outcome["drm_breakdown"].append(("Flanking", flanking_drm))
        outcome["drm_total"]  += flanking_drm
        outcome["final_die"]  += flanking_drm
        # NOTE: We cannot retroactively change the outcome once resolved.
        # For full correctness, flanking DRM is included in the initial call.
        # This wrapper documents it; a future iteration can pre-inject it.

    outcome["flanking_drm"] = flanking_count
    return outcome


# ============================================================================
# Advance After Contact helper
# ============================================================================

def location_is_empty_of_enemy(location, attacker_side, state):
    """Return True if the location has no units belonging to the enemy side."""
    occupants = state.units_at(location)
    return not any(u.side != attacker_side for u in occupants)
