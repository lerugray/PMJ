/// Combat module — Contact Result Table, Force Ratio shifts, DRM calculation.
///
/// Implements rulebook sections 6.2 through 6.2.4.

use rand::Rng;

/// CRT result codes per rulebook 6.2.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrtResult {
    AR, // Attacker Routed
    Ar, // Attacker Repulsed
    EX, // Exchange
    NE, // No Effect
    Rp, // Repulsed (defender retreats)
    R,  // Routed (highest defender eliminated, rest retreat)
    S,  // Surrender (highest defender permanently removed)
}

impl CrtResult {
    pub fn name(&self) -> &'static str {
        match self {
            CrtResult::AR => "ATTACKER ROUTED",
            CrtResult::Ar => "Attacker Repulsed",
            CrtResult::EX => "Exchange",
            CrtResult::NE => "No Effect",
            CrtResult::Rp => "Repulsed",
            CrtResult::R => "ROUTED",
            CrtResult::S => "SURRENDER",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            CrtResult::AR => "AR",
            CrtResult::Ar => "Ar",
            CrtResult::EX => "EX",
            CrtResult::NE => "NE",
            CrtResult::Rp => "Rp",
            CrtResult::R => "R",
            CrtResult::S => "S",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CrtResult::AR => "Highest SP attacker eliminated, rest retreat.",
            CrtResult::Ar => "All attacking units retreat toward HL.",
            CrtResult::EX => "Both sides step-reduce highest SP unit. Switchable units switch sides.",
            CrtResult::NE => "No changes.",
            CrtResult::Rp => "Defenders retreat (or step-reduce in Moscow).",
            CrtResult::R => "Highest SP defender eliminated, rest retreat.",
            CrtResult::S => "Highest SP defender permanently removed, rest retreat. Switchable units switch.",
        }
    }
}

/// Contact Result Table.
/// Rows: die roll (0 = "0 or less", 7 = "7+")
/// Columns: CD from -6 to +6 (index 0 = -6, index 12 = +6)
const CRT: [[CrtResult; 13]; 8] = {
    use CrtResult::*;
    [
        // Row 0 (die 0-)
        [AR, AR, AR, AR, AR, AR, Ar, Ar, Ar, EX, EX, EX, NE],
        // Row 1
        [AR, AR, AR, AR, AR, Ar, Ar, Ar, EX, EX, EX, NE, Rp],
        // Row 2
        [AR, AR, AR, AR, Ar, Ar, Ar, EX, EX, EX, NE, Rp, Rp],
        // Row 3
        [AR, AR, AR, Ar, Ar, Ar, EX, EX, EX, NE, Rp, Rp, Rp],
        // Row 4
        [AR, AR, Ar, Ar, Ar, EX, EX, EX, NE, Rp, Rp, Rp, R],
        // Row 5
        [AR, Ar, Ar, Ar, EX, EX, EX, NE, Rp, Rp, Rp, R, R],
        // Row 6
        [Ar, Ar, Ar, EX, EX, EX, NE, Rp, Rp, Rp, R, R, S],
        // Row 7 (die 7+)
        [Ar, Ar, EX, EX, EX, NE, Rp, Rp, Rp, R, R, S, S],
    ]
};

/// Look up the CRT given a final die roll and Contact Differential column.
pub fn lookup_crt(die_roll: i32, cd_column: i32) -> CrtResult {
    let row = die_roll.clamp(0, 7) as usize;
    let col = (cd_column.clamp(-6, 6) + 6) as usize; // -6 -> index 0, +6 -> index 12
    CRT[row][col]
}

/// Force Ratio Column Shift (rulebook 6.2.3.1).
/// Returns column shifts: negative = left (bad for attacker), positive = right (good).
pub fn force_ratio_shift(attack_sp: i32, defend_sp: i32) -> i32 {
    if defend_sp <= 0 {
        return 3; // Max right shift
    }

    let ratio = attack_sp as f64 / defend_sp as f64;

    if ratio >= 4.0 {
        3  // 4:1 -> 3R
    } else if ratio >= 3.0 {
        2  // 3:1 -> 2R
    } else if ratio >= 2.0 {
        1  // 2:1 -> 1R
    } else if ratio >= 1.5 {
        0  // 1.5:1 -> no shift
    } else if ratio >= 1.0 {
        -1 // 1:1 -> 1L
    } else if ratio >= 0.5 {
        -2 // 1:2 -> 2L
    } else {
        -3 // 1:3 -> 3L
    }
}

/// A DRM entry for display purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct DrmEntry {
    pub label: String,
    pub value: i32,
}

/// Full breakdown of a contact resolution.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactOutcome {
    pub attack_sp: i32,
    pub defend_sp: i32,
    pub cd_raw: i32,
    pub fr_shift: i32,
    pub cd_adjusted: i32,
    pub drms: Vec<DrmEntry>,
    pub drm_total: i32,
    pub die_roll: i32,
    pub final_die: i32,
    pub result: CrtResult,
}

/// Calculate contact resolution.
/// Does NOT apply results to units — the caller handles that based on the outcome.
pub fn resolve_contact(
    attack_sp: i32,
    defend_sp: i32,
    momentum: i32,
    river_crossing: bool,
    attacking_moscow: bool,
    flanking_count: i32, // 0, 1, or 2
    has_helicopter_only: bool, // true if ALL attackers are helicopters
) -> ContactOutcome {
    // Contact Differential
    let cd_raw = attack_sp - defend_sp;
    let fr_shift = force_ratio_shift(attack_sp, defend_sp);
    let cd_adjusted = cd_raw + fr_shift;

    // Build DRM list
    let mut drms = Vec::new();
    let mut drm_total = 0;

    // Momentum
    drms.push(DrmEntry {
        label: "Momentum".into(),
        value: momentum,
    });
    drm_total += momentum;

    // River crossing (-1, unless all attackers are helicopters)
    if river_crossing && !has_helicopter_only {
        drms.push(DrmEntry {
            label: "River crossing".into(),
            value: -1,
        });
        drm_total -= 1;
    }

    // Moscow defense (-2)
    if attacking_moscow {
        drms.push(DrmEntry {
            label: "Moscow defense".into(),
            value: -2,
        });
        drm_total -= 2;
    }

    // Flanking (+1 per extra location, max +2)
    let flank_drm = flanking_count.min(2);
    if flank_drm > 0 {
        drms.push(DrmEntry {
            label: "Flanking".into(),
            value: flank_drm,
        });
        drm_total += flank_drm;
    }

    // Roll
    let die_roll = rand::thread_rng().gen_range(1..=6);
    let final_die = die_roll + drm_total;

    // CRT lookup
    let result = lookup_crt(final_die, cd_adjusted);

    ContactOutcome {
        attack_sp,
        defend_sp,
        cd_raw,
        fr_shift,
        cd_adjusted,
        drms,
        drm_total,
        die_roll,
        final_die,
        result,
    }
}
