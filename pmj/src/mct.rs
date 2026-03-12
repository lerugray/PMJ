/// Maneuver/Combat Track (MCT) module.
///
/// The MCT is a ladder with 5 steps. Each step sets SP and MP for a Wagner unit.
/// Per rulebook section 3.5: "SP values from the MCT are added to an associated
/// unit's SP rating and units use the unit's current MA chosen on the MCT."
///
/// The track from the rulebook reference card:
///   SP-MP format:  3-0  2-1  1-2  0-3  0-4
/// But the CRT on the back shows the track as:
///   Step 0: SP+3, MP 0  (strongest combat, no movement)
///   Step 1: SP+2, MP 1
///   Step 2: SP+1, MP 2  ← starting position (labeled "2-2" but means SP+2 added... )
///
/// CORRECTED per the actual counter sheet / map reference:
///   3-0  means SP modifier +3, MP = 0
///   2-1  means SP modifier +2, MP = 1
///   1-2  means SP modifier +1, MP = 2   ← start here
///   0-3  means SP modifier +0, MP = 3
///   0-4  means SP modifier +0, MP = 4
///
/// During Administration Phase, each marker may shift one step up or down.

/// One row of the MCT.
#[derive(Debug, Clone, Copy)]
pub struct MctStep {
    pub sp_mod: i32,  // SP modifier added to unit's base SP
    pub mp: i32,      // Movement points available
}

/// The full MCT track (5 steps, index 0 = top/strongest, index 4 = bottom/most mobile).
pub const MCT_TRACK: [MctStep; 5] = [
    MctStep { sp_mod: 3, mp: 0 },
    MctStep { sp_mod: 2, mp: 1 },
    MctStep { sp_mod: 1, mp: 2 },  // Starting position (index 2)
    MctStep { sp_mod: 0, mp: 3 },
    MctStep { sp_mod: 0, mp: 4 },
];

pub const MCT_START: usize = 2;
pub const MCT_MIN: usize = 0;
pub const MCT_MAX: usize = 4;

/// Tracks the MCT position for one Wagner unit.
#[derive(Debug, Clone)]
pub struct MctMarker {
    pub step: usize,
}

impl MctMarker {
    pub fn new() -> Self {
        MctMarker { step: MCT_START }
    }

    pub fn current(&self) -> &MctStep {
        &MCT_TRACK[self.step]
    }

    pub fn sp_mod(&self) -> i32 {
        self.current().sp_mod
    }

    pub fn mp(&self) -> i32 {
        self.current().mp
    }

    /// Shift toward more SP / less MP. Returns true if shifted.
    pub fn shift_up(&mut self) -> bool {
        if self.step > MCT_MIN {
            self.step -= 1;
            true
        } else {
            false
        }
    }

    /// Shift toward less SP / more MP. Returns true if shifted.
    pub fn shift_down(&mut self) -> bool {
        if self.step < MCT_MAX {
            self.step += 1;
            true
        } else {
            false
        }
    }

    /// Label for display: "SP+X / MP Y"
    pub fn label(&self) -> String {
        let s = self.current();
        format!("SP+{} / MP {}", s.sp_mod, s.mp)
    }
}
