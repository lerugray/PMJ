# game_state.py
#
# This file is the heart of the engine.
# It holds:
#   - The GameState class (turn, momentum, units, roadblocks)
#   - The MCT (Maneuver/Combat Track) class
#   - The movement system
#   - The display system


from map_data import MapGraph
from units import create_all_units
from utils import print_header, print_divider, format_unit_line


# ============================================================================
# MCT — Maneuver / Combat Track
# ============================================================================

# The MCT is a ladder with 6 steps (0–5).
# Each step defines (SP, MP) values for Wagner units.
# Step 2 is the starting position: SP=2, MP=2.
#
#   Step 0  →  SP:1  MP:1   (weakest)
#   Step 1  →  SP:1  MP:2
#   Step 2  →  SP:2  MP:2   ← all Wagner units start here
#   Step 3  →  SP:2  MP:3
#   Step 4  →  SP:3  MP:3
#   Step 5  →  SP:3  MP:4   (strongest)

MCT_TRACK = [
    {"sp": 4, "mp": 0},
    {"sp": 3, "mp": 1},
    {"sp": 2, "mp": 2},   # index 2 = starting position
    {"sp": 1, "mp": 3},
    {"sp": 0, "mp": 4},
]

MCT_START_STEP = 2   # All Wagner units begin at step 2
MCT_MIN_STEP   = 0
MCT_MAX_STEP   = len(MCT_TRACK) - 1   # = 4


class MCTMarker:
    """
    Tracks the MCT position for one Wagner unit.

    Each Wagner unit has its own marker that slides up or down the track.
    The marker's step determines that unit's SP and MP for the current turn.
    """

    def __init__(self, unit_name):
        self.unit_name = unit_name
        self.step      = MCT_START_STEP   # Start at step 2

    @property
    def sp(self):
        """Current SP from the MCT track."""
        return MCT_TRACK[self.step]["sp"]

    @property
    def mp(self):
        """Current MP from the MCT track."""
        return MCT_TRACK[self.step]["mp"]

    def shift_up(self):
        """
        Move the marker one step up the track (toward stronger values).
        Returns a message describing what happened.
        """
        if self.step >= MCT_MAX_STEP:
            return f"{self.unit_name} MCT is already at maximum (step {self.step})."
        self.step += 1
        return (
            f"{self.unit_name} MCT moved UP to step {self.step} "
            f"→ SP:{self.sp} MP:{self.mp}"
        )

    def shift_down(self):
        """
        Move the marker one step down the track (toward weaker values).
        Returns a message describing what happened.
        """
        if self.step <= MCT_MIN_STEP:
            return f"{self.unit_name} MCT is already at minimum (step {self.step})."
        self.step -= 1
        return (
            f"{self.unit_name} MCT moved DOWN to step {self.step} "
            f"→ SP:{self.sp} MP:{self.mp}"
        )

    def __repr__(self):
        return (
            f"MCT[{self.unit_name}]: step={self.step} "
            f"SP:{self.sp} MP:{self.mp}"
        )


# ============================================================================
# GameState
# ============================================================================

class GameState:
    """
    The complete state of one game session.

    Holds everything that changes as the game is played:
      - Current turn (1–6)
      - Momentum (-3 to +3, starts at +1 for Wagner advantage)
      - All unit objects
      - MCT markers for the 3 Wagner units
      - Roadblock placement
      - The map graph
      - An action log
    """

    WAGNER_UNIT_NAMES = ["Rusich", "Utkin", "Serb"]

    def __init__(self):
        # ── Core state ───────────────────────────────────────────────────────
        self.turn     = 1
        self.momentum = 1          # +1 = Wagner advantage at game start
        
        # __ Cup ______________________________________________________________
        self.russian_cup = []
        self.russian_cup.append("PEOPLE_ARE_SILENT")

        # ── Map ──────────────────────────────────────────────────────────────
        self.map = MapGraph()

        # ── Units ────────────────────────────────────────────────────────────
        self.units = create_all_units()
        for unit in self.units:
            if unit.side == "Russia" and getattr(unit, "in_cup", False):
                self.russian_cup.append(unit)
        import random
        random.shuffle(self.russian_cup)

        # ── MCT markers — one per Wagner unit ────────────────────────────────
        self.mct = {
            name: MCTMarker(name)
            for name in self.WAGNER_UNIT_NAMES
        }

        # ── Roadblock markers ────────────────────────────────────────────────
        # Each roadblock is either None (not placed) or a location string.
        self.roadblocks = {
            "Roadblock 1": None,
            "Roadblock 2": None,
        }

        # ── Action log ───────────────────────────────────────────────────────
        # Every significant game action is recorded here as a string.
        self.log = []

    # ────────────────────────────────────────────────────────────────────────
    # Convenience lookups
    # ────────────────────────────────────────────────────────────────────────

    def get_unit(self, name):
        """Return the Unit object with this name, or None."""
        for unit in self.units:
            if unit.name == name:
                return unit
        return None

    def get_wagner_units(self):
        """Return a list of all Wagner Unit objects."""
        return [u for u in self.units if u.side == "Wagner"]

    def get_russia_units(self):
        """Return a list of all Russian Unit objects (on map only)."""
        return [u for u in self.units if u.side == "Russia" and u.location]

    def units_at(self, location):
        """Return all units currently at a given location."""
        return [u for u in self.units if u.location == location]

    def get_mct_mp(self, unit):
        """
        Return the effective MP for a unit.
        Wagner units use their MCT marker. Russian units use base_mp.
        """
        if unit.is_wagner():
            return self.mct[unit.name].mp
        return unit.base_mp    
        
    def get_effective_sp(self, unit):
        if unit.is_wagner():
            return unit.current_sp + self.mct[unit.name].sp
        return unit.current_sp
    
    def draw_from_russian_cup(self):
        if not self.russian_cup:
            return None
        return self.russian_cup.pop(0)
        
    # ────────────────────────────────────────────────────────────────────────
    # Logging
    # ────────────────────────────────────────────────────────────────────────

    def record(self, message):
        """Add a message to the action log and print it immediately."""
        entry = f"[Turn {self.turn}] {message}"
        self.log.append(entry)
        print(f"  ✦ {entry}")

    # ────────────────────────────────────────────────────────────────────────
    # Movement System
    # ────────────────────────────────────────────────────────────────────────

    def move_cost(self, unit, from_loc, to_loc):
        """
        Calculate the MP cost to move unit from from_loc to to_loc.

        Rules:
          - Base cost = 1 MP per hex (location).
          - River crossing = +1 MP extra.
          - Helicopters are exempt from the river penalty.
          - Returns None if the move is illegal (not adjacent).
        """
        edge = self.map.get_edge(from_loc, to_loc)

        if edge is None:
            return None   # Locations are not connected — illegal move

        cost = 1   # Base movement cost

        if edge["river"] and not unit.is_helicopter():
            cost += 1   # River penalty

        return cost

    def mp_remaining(self, unit):
        """How many MP does this unit still have available this turn?"""
        total_mp = self.get_mct_mp(unit)
        return total_mp - unit.mp_spent

    def can_move(self, unit, to_loc):
        """
        Check whether a unit is legally able to move to to_loc.

        Returns (True, cost) if legal, or (False, reason_string) if not.
        """
        if unit.location is None:
            return False, "Unit is off the map and cannot move."

        if unit.location == to_loc:
            return False, "Unit is already at that location."

        cost = self.move_cost(unit, unit.location, to_loc)

        if cost is None:
            return False, f"{unit.location} and {to_loc} are not adjacent."

        remaining = self.mp_remaining(unit)

        if cost > remaining:
            return False, (
                f"Not enough MP. Needs {cost}, has {remaining} remaining "
                f"(spent {unit.mp_spent} of {self.get_mct_mp(unit)})."
            )

        return True, cost

    def move_unit(self, unit, to_loc):
        """
        Attempt to move a unit to to_loc.

        On success: updates location, deducts MP, logs the action.
        On failure: prints the reason and does nothing.

        Returns True on success, False on failure.
        """
        legal, result = self.can_move(unit, to_loc)

        if not legal:
            print(f"  ✗ Cannot move {unit.name}: {result}")
            return False

        cost      = result
        from_loc  = unit.location

        unit.location  = to_loc
        unit.mp_spent += cost

        river_note = ""
        edge = self.map.get_edge(from_loc, to_loc)
        if edge and edge["river"] and not unit.is_helicopter():
            river_note = " [river crossing +1]"

        self.record(
            f"{unit.name} moved {from_loc} → {to_loc} "
            f"(cost {cost} MP{river_note}, "
            f"{self.mp_remaining(unit)} MP remaining)"
        )
        return True

    # ────────────────────────────────────────────────────────────────────────
    # Turn Management
    # ────────────────────────────────────────────────────────────────────────

    def end_turn(self):
        """
        Advance to the next turn.
        Resets MP spending for all units.
        """
        for unit in self.units:
            unit.reset_mp()

        self.turn += 1
        self.record(f"Turn advanced to {self.turn}.")

    def adjust_momentum(self, delta):
        """
        Shift momentum by delta (positive = toward Wagner, negative = toward Russia).
        Momentum is clamped between -3 and +3.
        """
        old = self.momentum
        self.momentum = max(-3, min(3, self.momentum + delta))
        self.record(
            f"Momentum shifted {delta:+d}: {old:+d} → {self.momentum:+d}"
        )

    # ────────────────────────────────────────────────────────────────────────
    # Display
    # ────────────────────────────────────────────────────────────────────────

    def display(self):
        """Print a full status screen to the terminal."""

        print_header(f"PMJ ENGINE  |  Turn {self.turn} of 6")

        # ── Momentum bar ─────────────────────────────────────────────────────
        self._display_momentum()

        # ── MCT grid ─────────────────────────────────────────────────────────
        self._display_mct()

        # ── Units on the map ─────────────────────────────────────────────────
        self._display_units_by_location()

        # ── Roadblocks ───────────────────────────────────────────────────────
        self._display_roadblocks()

        # ── Recent log entries ───────────────────────────────────────────────
        self._display_log_tail()

        print_divider()

    def _display_momentum(self):
        """Show momentum as a visual bar."""
        print()
        steps  = ["RUS -3", "-2", "-1", " 0", "+1", "+2", "WAG +3"]
        values = [-3, -2, -1, 0, 1, 2, 3]

        bar_parts = []
        for val, label in zip(values, steps):
            if val == self.momentum:
                bar_parts.append(f"[{label}]")
            else:
                bar_parts.append(f" {label} ")

        print("  MOMENTUM: " + "  ".join(bar_parts))
        print()

    def _display_mct(self):
        """Show the MCT grid for all three Wagner units."""
        print("  MANEUVER / COMBAT TRACK")
        print(f"  {'Step':<6}  {'SP':<4}  {'MP':<4}  " +
              "  ".join(f"{n:<8}" for n in self.WAGNER_UNIT_NAMES))
        print_divider()

        for step, row in enumerate(MCT_TRACK):
            # Mark which step each Wagner unit is on
            markers = []
            for name in self.WAGNER_UNIT_NAMES:
                if self.mct[name].step == step:
                    markers.append("  ◄ HERE")
                else:
                    markers.append("        ")

            print(
                f"  {step:<6}  {row['sp']:<4}  {row['mp']:<4}  "
                + "".join(markers)
            )

        print()

    def _display_units_by_location(self):
        """Group all on-map units by location and display them."""
        print("  UNITS ON MAP")
        print_divider()

        # Build a dict: location → [units]
        loc_map = {}
        for unit in self.units:
            if unit.location:
                loc_map.setdefault(unit.location, []).append(unit)

        if not loc_map:
            print("  (no units on map)")
        else:
            for location in sorted(loc_map.keys()):
                print(f"\n  ▸ {location}")
                for unit in loc_map[location]:
                    mct_mp = self.get_mct_mp(unit) if unit.is_wagner() else None
                    sp_override = self.get_effective_sp(unit)
                    print(format_unit_line(unit, mct_mp=mct_mp, sp_override=sp_override))
                    if unit.is_wagner():
                        spent = unit.mp_spent
                        total = self.get_mct_mp(unit)
                        print(f"    {'':26} MP used: {spent}/{total}")

        print()

    def _display_roadblocks(self):
        """Show roadblock marker positions."""
        print("  ROADBLOCKS")
        print_divider()
        for name, loc in self.roadblocks.items():
            placed = loc if loc else "(not placed)"
            print(f"  {name:<16} → {placed}")
        print()

    def _display_log_tail(self, lines=5):
        """Show the last few log entries."""
        print("  RECENT ACTIONS")
        print_divider()
        tail = self.log[-lines:] if len(self.log) > lines else self.log
        if not tail:
            print("  (none yet)")
        for entry in tail:
            print(f"  {entry}")
        print()