# units.py
#
# This file defines:
#   - The Unit class (one object per counter)
#   - A factory function that creates every counter in the game
#
# Each Unit stores all the data printed on a physical game counter.


class Unit:
    """
    Represents one physical game counter.

    Attributes
    ----------
    name        : str   — The counter's label (e.g. "Rusich")
    side        : str   — "Wagner" or "Russia"
    base_sp     : int   — Strength Points printed on the counter
    base_mp     : int   — Movement Points printed (0 = Wagner, uses MCT instead)
    reduced     : bool  — Does this counter have a reduced (flip) side?
    is_reduced  : bool  — Is it currently flipped to the reduced side?
    switchable  : bool  — (Z) Can this unit switch sides? (future feature)
    police      : bool  — (P) Cannot initiate combat (future feature)
    in_cup      : bool  — (C) Deploys via mobilization cup (future feature)
    location    : str   — Current map location, or None if off-map
    mp_spent    : int   — MP used so far this turn (reset each turn)
    """

    def __init__(
        self,
        name,
        side,
        base_sp,
        base_mp=0,
        reduced=False,
        switchable=False,
        police=False,
        in_cup=False,
        location=None,
    ):
        self.name       = name
        self.side       = side
        self.base_sp    = base_sp
        self.base_mp    = base_mp
        self.reduced    = reduced       # HAS a reduced side
        self.is_reduced = False         # Currently on reduced side?
        self.switchable = switchable
        self.police     = police
        self.in_cup     = in_cup
        self.location   = location
        self.mp_spent   = 0            # Tracks movement this turn

    @property
    def current_sp(self):
        """
        Current Strength Points.
        If the unit is flipped to its reduced side, SP drops by 1 (minimum 1).
        """
        if self.is_reduced and self.reduced:
            return max(1, self.base_sp - 1)
        return self.base_sp

    def is_wagner(self):
        """Convenience check — Wagner units get MP from MCT, not base_mp."""
        return self.side == "Wagner"

    def is_helicopter(self):
        """Helicopters ignore river crossing penalties."""
        return self.name == "Helicopters"

    def reset_mp(self):
        """Call at the start of each turn to reset movement spending."""
        self.mp_spent = 0

    def __repr__(self):
        """Human-readable one-line summary of this unit."""
        loc_str     = self.location if self.location else "Off Map"
        reduced_str = " [REDUCED]" if self.is_reduced else ""
        return (
            f"[{self.side}] {self.name}"
            f" | SP:{self.current_sp} MP:{self.base_mp}"
            f" | {loc_str}{reduced_str}"
        )


# ---------------------------------------------------------------------------
# Factory — builds every counter described in the PMJ Counter Manifest
# ---------------------------------------------------------------------------

def create_all_units():
    """
    Create and return a list of every Unit in the game.

    Starting locations:
      - Wagner units begin at Rostov-On-Don (the march's starting point).
      - Russian units begin off-map (location=None) until placed.
    """
    units = []

    # ── Wagner ──────────────────────────────────────────────────────────────
    # Wagner units have base_mp=0 because their MP is set by the MCT.
    units.append(Unit(
        name="Rusich", side="Wagner", base_sp=1,
        location="Rostov-On-Don"
    ))
    units.append(Unit(
        name="Utkin", side="Wagner", base_sp=2,
        location="Rostov-On-Don"
    ))
    units.append(Unit(
        name="Serb", side="Wagner", base_sp=1,
        location="Rostov-On-Don"
    ))

    # ── Russia ───────────────────────────────────────────────────────────────
    units.append(Unit(
        name="Mechanized Regiment", side="Russia",
        base_sp=3, base_mp=4, switchable=True, in_cup=True,
    ))
    units.append(Unit(
        name="Motorized Infantry", side="Russia",
        base_sp=1, base_mp=3, switchable=True, in_cup=True
    ))
    units.append(Unit(
        name="Armored Regiment", side="Russia",
        base_sp=3, base_mp=3, switchable=True, in_cup=True
    ))
    units.append(Unit(
        name="OMON", side="Russia",
        base_sp=2, base_mp=2, reduced=True, police=True,
        location="Moscow"
    ))
    units.append(Unit(
        name="Akhmat", side="Russia",
        base_sp=2, base_mp=2, reduced=True, in_cup=True
    ))
    units.append(Unit(
        name="FSB", side="Russia",
        base_sp=2, base_mp=2, reduced=True,
        location="Moscow"
    ))
    units.append(Unit(
        name="Helicopters", side="Russia",
        base_sp=3, base_mp=4, reduced=True, in_cup=True
    ))
    units.append(Unit(
        name="SOBR", side="Russia",
        base_sp=2, base_mp=2, reduced=True,
        location="Moscow"
    ))
    units.append(Unit(
        name="MOSpol", side="Russia",
        base_sp=1, base_mp=2, police=True, in_cup=True
    ))

    return units