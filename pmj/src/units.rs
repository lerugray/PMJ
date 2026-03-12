/// Units module — all game counters from the PMJ Counter Manifest.

use crate::map::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Wagner,
    Russia,
}

/// Unique identifier for each unit type in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitId {
    // Wagner
    Rusich,
    Utkin,
    Serb,
    // Russia
    MechanizedRegiment,
    MotorizedInfantry,
    ArmoredRegiment,
    Omon,
    Akhmat,
    Fsb,
    Helicopters,
    Sobr,
    Mospol,
}

impl UnitId {
    pub fn name(&self) -> &'static str {
        match self {
            UnitId::Rusich => "Rusich",
            UnitId::Utkin => "Utkin",
            UnitId::Serb => "Serb",
            UnitId::MechanizedRegiment => "Mech Regiment",
            UnitId::MotorizedInfantry => "Motor Infantry",
            UnitId::ArmoredRegiment => "Armor Regiment",
            UnitId::Omon => "OMON",
            UnitId::Akhmat => "Akhmat",
            UnitId::Fsb => "FSB",
            UnitId::Helicopters => "Helicopters",
            UnitId::Sobr => "SOBR",
            UnitId::Mospol => "MOSpol",
        }
    }

    /// Short label for compact display (max 4 chars).
    pub fn short(&self) -> &'static str {
        match self {
            UnitId::Rusich => "RSCH",
            UnitId::Utkin => "UTKN",
            UnitId::Serb => "SERB",
            UnitId::MechanizedRegiment => "MECH",
            UnitId::MotorizedInfantry => "MOTR",
            UnitId::ArmoredRegiment => "ARMR",
            UnitId::Omon => "OMON",
            UnitId::Akhmat => "AKHM",
            UnitId::Fsb => "FSB ",
            UnitId::Helicopters => "HELI",
            UnitId::Sobr => "SOBR",
            UnitId::Mospol => "MPOL",
        }
    }

    /// NATO-style center symbol for halfblock counter display.
    pub fn nato_symbol(&self) -> char {
        match self {
            // Wagner: use starting letter for visual distinction
            UnitId::Rusich => 'R',
            UnitId::Utkin => 'U',
            UnitId::Serb => 'S',
            // Russian military: NATO infantry symbol
            UnitId::MechanizedRegiment => '╳',
            UnitId::MotorizedInfantry => '╳',
            UnitId::ArmoredRegiment => '◇',
            UnitId::Omon => '╳',
            UnitId::Akhmat => '╳',
            UnitId::Fsb => '█',
            UnitId::Helicopters => 'H',
            UnitId::Sobr => '█',
            UnitId::Mospol => '╳',
        }
    }

    pub fn is_wagner(&self) -> bool {
        matches!(self, UnitId::Rusich | UnitId::Utkin | UnitId::Serb)
    }

    /// All Wagner unit IDs.
    pub fn wagner_units() -> &'static [UnitId] {
        &[UnitId::Rusich, UnitId::Utkin, UnitId::Serb]
    }
}

/// A single game counter.
#[derive(Debug, Clone)]
pub struct Unit {
    pub id: UnitId,
    pub side: Side,
    pub base_sp: i32,
    pub base_mp: i32,
    pub has_reduced_side: bool,
    pub is_reduced: bool,
    pub switchable: bool,  // (Z) can switch sides Russia -> Wagner
    pub police: bool,      // (P) no offensive capability
    pub in_cup: bool,      // (C) starts in Moscow Mobilization Cup
    pub location: Option<Location>,
    pub mp_spent: i32,
    pub dispersed: bool,
}

impl Unit {
    /// Current SP accounting for reduction (reduced = base_sp - 1, min 1).
    pub fn current_sp(&self) -> i32 {
        if self.is_reduced && self.has_reduced_side {
            (self.base_sp - 1).max(1)
        } else {
            self.base_sp
        }
    }

    pub fn is_wagner(&self) -> bool {
        self.side == Side::Wagner
    }

    pub fn is_helicopter(&self) -> bool {
        self.id == UnitId::Helicopters
    }

    pub fn is_on_map(&self) -> bool {
        self.location.is_some()
    }

    pub fn reset_mp(&mut self) {
        self.mp_spent = 0;
    }

    /// Step-reduce this unit. Returns true if reduced, false if eliminated.
    pub fn step_reduce(&mut self) -> bool {
        if self.has_reduced_side && !self.is_reduced {
            self.is_reduced = true;
            true
        } else {
            self.eliminate();
            false
        }
    }

    /// Remove unit from the map.
    pub fn eliminate(&mut self) {
        self.location = None;
        self.is_reduced = false;
        self.dispersed = false;
    }
}

/// Create all units per the PMJ Counter Manifest.
pub fn create_all_units() -> Vec<Unit> {
    vec![
        // === Wagner ===
        // Wagner units have base_mp=0 because MP comes from MCT
        Unit {
            id: UnitId::Rusich,
            side: Side::Wagner,
            base_sp: 1,
            base_mp: 0,
            has_reduced_side: false,
            is_reduced: false,
            switchable: false,
            police: false,
            in_cup: false,
            location: Some(Location::RostovOnDon),
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Utkin,
            side: Side::Wagner,
            base_sp: 2,
            base_mp: 0,
            has_reduced_side: false,
            is_reduced: false,
            switchable: false,
            police: false,
            in_cup: false,
            location: Some(Location::RostovOnDon),
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Serb,
            side: Side::Wagner,
            base_sp: 1,
            base_mp: 0,
            has_reduced_side: false,
            is_reduced: false,
            switchable: false,
            police: false,
            in_cup: false,
            location: Some(Location::RostovOnDon),
            mp_spent: 0,
            dispersed: false,
        },
        // === Russia ===
        Unit {
            id: UnitId::MechanizedRegiment,
            side: Side::Russia,
            base_sp: 3,
            base_mp: 4,
            has_reduced_side: false,
            is_reduced: false,
            switchable: true,
            police: false,
            in_cup: true,
            location: None,
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::MotorizedInfantry,
            side: Side::Russia,
            base_sp: 1,
            base_mp: 3,
            has_reduced_side: false,
            is_reduced: false,
            switchable: true,
            police: false,
            in_cup: true,
            location: None,
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::ArmoredRegiment,
            side: Side::Russia,
            base_sp: 3,
            base_mp: 3,
            has_reduced_side: false,
            is_reduced: false,
            switchable: true,
            police: false,
            in_cup: true,
            location: None,
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Omon,
            side: Side::Russia,
            base_sp: 2,
            base_mp: 2,
            has_reduced_side: true,
            is_reduced: false,
            switchable: false,
            police: true,
            in_cup: false,
            location: Some(Location::Moscow),
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Akhmat,
            side: Side::Russia,
            base_sp: 2,
            base_mp: 2,
            has_reduced_side: true,
            is_reduced: false,
            switchable: false,
            police: false,
            in_cup: true,
            location: None,
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Fsb,
            side: Side::Russia,
            base_sp: 2,
            base_mp: 2,
            has_reduced_side: true,
            is_reduced: false,
            switchable: false,
            police: false,
            in_cup: false,
            location: Some(Location::Moscow),
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Helicopters,
            side: Side::Russia,
            base_sp: 3,
            base_mp: 4,
            has_reduced_side: true,
            is_reduced: false,
            switchable: false,
            police: false,
            in_cup: true,
            location: None,
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Sobr,
            side: Side::Russia,
            base_sp: 2,
            base_mp: 2,
            has_reduced_side: true,
            is_reduced: false,
            switchable: false,
            police: false,
            in_cup: false,
            location: Some(Location::Moscow),
            mp_spent: 0,
            dispersed: false,
        },
        Unit {
            id: UnitId::Mospol,
            side: Side::Russia,
            base_sp: 1,
            base_mp: 2,
            has_reduced_side: false,
            is_reduced: false,
            switchable: false,
            police: true,
            in_cup: true,
            location: None,
            mp_spent: 0,
            dispersed: false,
        },
    ]
}
