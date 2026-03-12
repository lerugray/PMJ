/// Game state module — holds everything that changes during play.

use rand::seq::SliceRandom;

use crate::combat::{self, ContactOutcome, CrtResult};
use crate::map::{GameMap, Location};
use crate::mct::MctMarker;
use crate::units::{self, Side, Unit, UnitId};

/// Which phase of the turn we're in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Administration,  // Set MCT for each Wagner unit
    WagnerTurn,      // Move and initiate Contact
    RussianAI,       // Mobilization, movement, attacks
    EndTurn,         // Momentum adjustments, advance turn
}

/// What the player is currently doing in the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Main menu for the current phase.
    PhaseMenu,
    /// Selecting which Wagner unit's MCT to adjust.
    MctSelect,
    /// Choosing MCT direction for a specific unit.
    MctAdjust(usize), // index into WAGNER_IDS
    /// Selecting a Wagner unit to move.
    MoveSelectUnit,
    /// Selecting destination for a unit.
    MoveSelectDest(usize), // unit index in game.units
    /// Selecting primary attack location.
    ContactSelectLocation,
    /// Selecting target to attack.
    ContactSelectTarget {
        from_loc: Location,
    },
    /// Confirming contact resolution.
    ContactConfirm {
        from_loc: Location,
        target_loc: Location,
        attacker_indices: Vec<usize>,
    },
    /// Showing contact result.
    ContactResult {
        outcome: ContactOutcome,
        target_loc: Location,
        attacker_indices: Vec<usize>,
    },
    /// Advance after contact prompt.
    AdvanceAfterContact {
        target_loc: Location,
        attacker_indices: Vec<usize>,
    },
    /// Russian AI phase display (auto-resolves, player watches).
    RussianPhaseDisplay,
    /// End turn momentum questions.
    EndTurnConfirm,
    /// Showing the action log.
    ViewLog,
    /// Game over screen.
    GameOver { wagner_wins: bool },
}

/// Items that can be in the Moscow Mobilization Cup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CupItem {
    Unit(UnitId),
    PeopleAreSilent,
    Roadblock2, // Second roadblock marker
}

/// The complete game state.
pub struct GameState {
    pub map: GameMap,
    pub units: Vec<Unit>,
    pub mct: [MctMarker; 3], // Rusich, Utkin, Serb
    pub turn: i32,
    pub momentum: i32,
    pub phase: Phase,
    pub screen: Screen,
    pub cup: Vec<CupItem>,
    pub roadblocks: [Option<Location>; 2],
    pub people_are_silent_pulled: bool,
    pub log: Vec<String>,
    pub cursor: usize, // For menu navigation
    pub log_scroll: usize,
    // Turn tracking for momentum questions
    pub russian_reduced_this_turn: bool,
    pub russian_eliminated_this_turn: bool,
    pub wagner_repulsed_this_turn: bool,
    pub wagner_reduced_this_turn: bool,
    pub wagner_eliminated_this_turn: bool,
    pub admin_units_adjusted: [bool; 3], // Track which Wagner units had MCT adjusted
}

const WAGNER_IDS: [UnitId; 3] = [UnitId::Rusich, UnitId::Utkin, UnitId::Serb];

impl GameState {
    pub fn new() -> Self {
        let units = units::create_all_units();

        // Build the Moscow Mobilization Cup
        let mut cup: Vec<CupItem> = Vec::new();
        cup.push(CupItem::PeopleAreSilent);
        cup.push(CupItem::Roadblock2);
        for unit in &units {
            if unit.in_cup {
                cup.push(CupItem::Unit(unit.id));
            }
        }
        let mut rng = rand::thread_rng();
        cup.shuffle(&mut rng);

        GameState {
            map: GameMap::new(),
            units,
            mct: [MctMarker::new(), MctMarker::new(), MctMarker::new()],
            turn: 1,
            momentum: 1,
            phase: Phase::Administration,
            screen: Screen::MctSelect,
            cup,
            roadblocks: [None, None],
            people_are_silent_pulled: false,
            log: vec!["Game started. Wagner forces in Rostov-On-Don.".into()],
            cursor: 0,
            log_scroll: 0,
            russian_reduced_this_turn: false,
            russian_eliminated_this_turn: false,
            wagner_repulsed_this_turn: false,
            wagner_reduced_this_turn: false,
            wagner_eliminated_this_turn: false,
            admin_units_adjusted: [false; 3],
        }
    }

    // ── Logging ──────────────────────────────────────────────────────────

    pub fn record(&mut self, msg: impl Into<String>) {
        let entry = format!("[Turn {}] {}", self.turn, msg.into());
        self.log.push(entry);
    }

    // ── Unit lookups ─────────────────────────────────────────────────────

    pub fn find_unit(&self, id: UnitId) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }

    pub fn find_unit_mut(&mut self, id: UnitId) -> Option<&mut Unit> {
        self.units.iter_mut().find(|u| u.id == id)
    }

    pub fn unit_index(&self, id: UnitId) -> Option<usize> {
        self.units.iter().position(|u| u.id == id)
    }

    pub fn units_at(&self, loc: Location) -> Vec<usize> {
        self.units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.location == Some(loc))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn wagner_units_at(&self, loc: Location) -> Vec<usize> {
        self.units_at(loc)
            .into_iter()
            .filter(|&i| self.units[i].is_wagner())
            .collect()
    }

    pub fn russian_units_at(&self, loc: Location) -> Vec<usize> {
        self.units_at(loc)
            .into_iter()
            .filter(|&i| self.units[i].side == Side::Russia)
            .collect()
    }

    /// All locations where Wagner has units.
    pub fn wagner_locations(&self) -> Vec<Location> {
        let mut locs: Vec<Location> = self
            .units
            .iter()
            .filter(|u| u.is_wagner() && u.is_on_map())
            .filter_map(|u| u.location)
            .collect();
        locs.sort_by_key(|l| *l as u8);
        locs.dedup();
        locs
    }

    /// All locations where Russia has units.
    pub fn russian_locations(&self) -> Vec<Location> {
        let mut locs: Vec<Location> = self
            .units
            .iter()
            .filter(|u| u.side == Side::Russia && u.is_on_map())
            .filter_map(|u| u.location)
            .collect();
        locs.sort_by_key(|l| *l as u8);
        locs.dedup();
        locs
    }

    // ── MCT / effective stats ────────────────────────────────────────────

    fn wagner_mct_index(id: UnitId) -> Option<usize> {
        WAGNER_IDS.iter().position(|&wid| wid == id)
    }

    pub fn mct_for(&self, id: UnitId) -> Option<&MctMarker> {
        Self::wagner_mct_index(id).map(|i| &self.mct[i])
    }

    pub fn mct_for_mut(&mut self, id: UnitId) -> Option<&mut MctMarker> {
        Self::wagner_mct_index(id).map(|i| &mut self.mct[i])
    }

    /// Effective SP for a unit (base + MCT modifier for Wagner).
    pub fn effective_sp(&self, idx: usize) -> i32 {
        let unit = &self.units[idx];
        if unit.is_wagner() {
            let mct_sp = self.mct_for(unit.id).map(|m| m.sp_mod()).unwrap_or(0);
            unit.current_sp() + mct_sp
        } else {
            unit.current_sp()
        }
    }

    /// Effective MP for a unit (MCT MP for Wagner, base_mp for Russia).
    pub fn effective_mp(&self, idx: usize) -> i32 {
        let unit = &self.units[idx];
        if unit.is_wagner() {
            self.mct_for(unit.id).map(|m| m.mp()).unwrap_or(0)
        } else {
            unit.base_mp
        }
    }

    pub fn mp_remaining(&self, idx: usize) -> i32 {
        self.effective_mp(idx) - self.units[idx].mp_spent
    }

    // ── Movement ─────────────────────────────────────────────────────────

    pub fn move_cost(&self, idx: usize, from: Location, to: Location) -> Option<i32> {
        let edge = self.map.edge(from, to)?;
        let mut cost = 1;
        if edge.river && !self.units[idx].is_helicopter() {
            cost += 1;
        }
        Some(cost)
    }

    pub fn can_move(&self, idx: usize, to: Location) -> Result<i32, String> {
        let unit = &self.units[idx];
        let from = unit.location.ok_or("Unit is off the map.")?;
        if from == to {
            return Err("Already there.".into());
        }
        let cost = self
            .move_cost(idx, from, to)
            .ok_or("Locations not connected.")?;
        let remaining = self.mp_remaining(idx);
        if cost > remaining {
            return Err(format!("Need {} MP, have {}.", cost, remaining));
        }
        Ok(cost)
    }

    pub fn move_unit(&mut self, idx: usize, to: Location) -> bool {
        let cost = match self.can_move(idx, to) {
            Ok(c) => c,
            Err(msg) => {
                self.record(format!("Cannot move: {}", msg));
                return false;
            }
        };
        let from = self.units[idx].location.unwrap();
        let name = self.units[idx].id.name().to_string();
        self.units[idx].location = Some(to);
        self.units[idx].mp_spent += cost;
        let remaining = self.mp_remaining(idx);

        let river_note = if let Some(edge) = self.map.edge(from, to) {
            if edge.river && !self.units[idx].is_helicopter() {
                " [river +1]"
            } else {
                ""
            }
        } else {
            ""
        };

        self.record(format!(
            "{} moved {} → {} (cost {}{}MP, {} remaining)",
            name,
            from.name(),
            to.name(),
            cost,
            river_note,
            remaining
        ));
        true
    }

    // ── Contact ──────────────────────────────────────────────────────────

    /// Find Wagner locations that have adjacent Russian-occupied locations.
    pub fn contact_opportunities(&self) -> Vec<(Location, Vec<Location>)> {
        let mut results = Vec::new();
        for wloc in self.wagner_locations() {
            let mut targets = Vec::new();
            for &(neighbor, _) in self.map.neighbors(wloc) {
                if !self.russian_units_at(neighbor).is_empty() {
                    targets.push(neighbor);
                }
            }
            if !targets.is_empty() {
                results.push((wloc, targets));
            }
        }
        results
    }

    /// Resolve contact and apply results.
    pub fn resolve_contact(
        &mut self,
        attacker_indices: &[usize],
        from_loc: Location,
        target_loc: Location,
    ) -> ContactOutcome {
        let attack_sp: i32 = attacker_indices.iter().map(|&i| self.effective_sp(i)).sum();
        let defender_indices = self.russian_units_at(target_loc);
        let defend_sp: i32 = defender_indices.iter().map(|&i| self.effective_sp(i)).sum();

        let river = self
            .map
            .edge(from_loc, target_loc)
            .map(|e| e.river)
            .unwrap_or(false);
        let all_heli = attacker_indices
            .iter()
            .all(|&i| self.units[i].is_helicopter());
        let attacking_moscow = target_loc == Location::Moscow;

        // TODO: flanking from other locations
        let flanking = 0;

        let outcome = combat::resolve_contact(
            attack_sp,
            defend_sp,
            self.momentum,
            river,
            attacking_moscow,
            flanking,
            all_heli,
        );

        // Log the contact
        let atk_names: Vec<&str> = attacker_indices
            .iter()
            .map(|&i| self.units[i].id.name())
            .collect();
        self.record(format!(
            "Contact: {} @ {} → {} | ATK:{} DEF:{} | CD:{:+} | Roll:{} DRM:{:+} Final:{} | {}",
            atk_names.join(", "),
            from_loc.name(),
            target_loc.name(),
            attack_sp,
            defend_sp,
            outcome.cd_adjusted,
            outcome.die_roll,
            outcome.drm_total,
            outcome.final_die,
            outcome.result.code()
        ));

        // Apply the result
        self.apply_contact_result(&outcome, attacker_indices, from_loc, &defender_indices, target_loc);

        outcome
    }

    fn apply_contact_result(
        &mut self,
        outcome: &ContactOutcome,
        attackers: &[usize],
        atk_loc: Location,
        defenders: &[usize],
        def_loc: Location,
    ) {
        match outcome.result {
            CrtResult::AR => {
                // Highest SP attacker eliminated, rest retreat
                if let Some(&idx) = self.highest_sp_unit(attackers) {
                    let name = self.units[idx].id.name().to_string();
                    self.units[idx].eliminate();
                    self.record(format!("{} ELIMINATED [AR]", name));
                    self.wagner_eliminated_this_turn = true;
                }
                for &idx in attackers {
                    if self.units[idx].is_on_map() {
                        self.retreat_unit(idx, atk_loc);
                    }
                }
                self.wagner_repulsed_this_turn = true;
            }
            CrtResult::Ar => {
                // All attackers retreat
                for &idx in attackers {
                    self.retreat_unit(idx, atk_loc);
                }
                self.wagner_repulsed_this_turn = true;
            }
            CrtResult::EX => {
                // Check for switchable defenders first
                let switched = self.handle_switchable(defenders, atk_loc);
                if switched.is_empty() {
                    if let Some(&idx) = self.highest_sp_unit(defenders) {
                        self.step_reduce_unit(idx, "EX");
                    }
                }
                if let Some(&idx) = self.highest_sp_unit(attackers) {
                    self.step_reduce_unit(idx, "EX");
                }
            }
            CrtResult::NE => {
                // Nothing happens
            }
            CrtResult::Rp => {
                if def_loc == Location::Moscow {
                    if let Some(&idx) = self.highest_sp_unit(defenders) {
                        self.step_reduce_unit(idx, "Rp-Moscow");
                    }
                } else {
                    for &idx in defenders {
                        self.retreat_unit(idx, def_loc);
                    }
                }
            }
            CrtResult::R => {
                if let Some(&idx) = self.highest_sp_unit(defenders) {
                    let name = self.units[idx].id.name().to_string();
                    self.units[idx].eliminate();
                    self.record(format!("{} ELIMINATED [R]", name));
                    self.russian_eliminated_this_turn = true;
                }
                let remaining: Vec<usize> = defenders
                    .iter()
                    .copied()
                    .filter(|&i| self.units[i].is_on_map())
                    .collect();
                if def_loc == Location::Moscow {
                    for &idx in &remaining {
                        self.step_reduce_unit(idx, "R-Moscow");
                    }
                } else {
                    for &idx in &remaining {
                        self.retreat_unit(idx, def_loc);
                    }
                }
            }
            CrtResult::S => {
                let switched = self.handle_switchable(defenders, atk_loc);
                let remaining: Vec<usize> = defenders
                    .iter()
                    .copied()
                    .filter(|&i| !switched.contains(&self.units[i].id) && self.units[i].is_on_map())
                    .collect();
                if let Some(&idx) = self.highest_sp_unit(&remaining) {
                    let name = self.units[idx].id.name().to_string();
                    self.units[idx].eliminate();
                    self.record(format!("{} SURRENDERED — permanently removed [S]", name));
                    self.russian_eliminated_this_turn = true;
                }
                let rest: Vec<usize> = remaining
                    .iter()
                    .copied()
                    .filter(|&i| self.units[i].is_on_map())
                    .collect();
                if def_loc == Location::Moscow {
                    for &idx in &rest {
                        self.step_reduce_unit(idx, "S-Moscow");
                    }
                } else {
                    for &idx in &rest {
                        self.retreat_unit(idx, def_loc);
                    }
                }
            }
        }
    }

    fn highest_sp_unit<'a>(&self, indices: &'a [usize]) -> Option<&'a usize> {
        indices
            .iter()
            .filter(|&&i| self.units[i].is_on_map())
            .max_by_key(|&&i| self.effective_sp(i))
    }

    fn step_reduce_unit(&mut self, idx: usize, context: &str) {
        let name = self.units[idx].id.name().to_string();
        let is_wagner = self.units[idx].is_wagner();
        if self.units[idx].step_reduce() {
            self.record(format!(
                "{} step-reduced to SP:{} [{}]",
                name,
                self.units[idx].current_sp(),
                context
            ));
            if is_wagner {
                self.wagner_reduced_this_turn = true;
            } else {
                self.russian_reduced_this_turn = true;
            }
        } else {
            self.record(format!("{} ELIMINATED [{}]", name, context));
            if is_wagner {
                self.wagner_eliminated_this_turn = true;
            } else {
                self.russian_eliminated_this_turn = true;
            }
        }
    }

    fn retreat_unit(&mut self, idx: usize, from: Location) {
        let unit = &self.units[idx];
        let home = if unit.is_wagner() {
            Location::RostovOnDon
        } else {
            Location::Moscow
        };
        let name = unit.id.name().to_string();
        let side = unit.side;

        // Find candidate retreat locations (adjacent, no enemy)
        let neighbors = self.map.neighbors(from);
        let mut candidates: Vec<Location> = Vec::new();
        for &(neighbor, _) in neighbors {
            let enemy_here = self
                .units_at(neighbor)
                .iter()
                .any(|&i| self.units[i].side != side);
            if !enemy_here {
                candidates.push(neighbor);
            }
        }

        if candidates.contains(&home) {
            self.units[idx].location = Some(home);
            self.record(format!("{} retreated to {} (HL)", name, home.name()));
        } else if let Some(&dest) = candidates.first() {
            self.units[idx].location = Some(dest);
            self.record(format!("{} retreated to {}", name, dest.name()));
        } else {
            self.units[idx].location = None;
            self.record(format!("{} DISPERSED (no retreat path)", name));
        }
    }

    fn handle_switchable(&mut self, defenders: &[usize], wagner_loc: Location) -> Vec<UnitId> {
        let mut switched = Vec::new();
        for &idx in defenders {
            if self.units[idx].switchable && self.units[idx].side == Side::Russia {
                let name = self.units[idx].id.name().to_string();
                let id = self.units[idx].id;
                self.units[idx].side = Side::Wagner;
                self.units[idx].location = Some(wagner_loc);
                self.record(format!(
                    "{} SWITCHED SIDES to Wagner → {}",
                    name,
                    wagner_loc.name()
                ));
                switched.push(id);
            }
        }
        switched
    }

    // ── Advance After Contact ────────────────────────────────────────────

    pub fn target_empty_of_russians(&self, loc: Location) -> bool {
        self.russian_units_at(loc).is_empty()
    }

    pub fn advance_units(&mut self, unit_indices: &[usize], to: Location) {
        for &idx in unit_indices {
            if self.units[idx].is_on_map() {
                let name = self.units[idx].id.name().to_string();
                self.units[idx].location = Some(to);
                self.record(format!("{} advanced into {}", name, to.name()));
            }
        }
    }

    // ── Turn Management ──────────────────────────────────────────────────

    pub fn start_administration(&mut self) {
        self.phase = Phase::Administration;
        self.screen = Screen::MctSelect;
        self.cursor = 0;
        self.admin_units_adjusted = [false; 3];
    }

    pub fn start_wagner_turn(&mut self) {
        self.phase = Phase::WagnerTurn;
        self.screen = Screen::PhaseMenu;
        self.cursor = 0;
        // Reset MP for all units
        for unit in &mut self.units {
            unit.reset_mp();
        }
        self.record("Wagner Player Turn begins.");
    }

    pub fn start_russian_phase(&mut self) {
        self.phase = Phase::RussianAI;
        self.screen = Screen::RussianPhaseDisplay;
        self.cursor = 0;

        // Moscow Mobilization - draw from cup
        self.run_russian_mobilization();
    }

    fn run_russian_mobilization(&mut self) {
        if self.cup.is_empty() {
            self.record("Moscow Mobilization Cup is empty.");
            return;
        }

        let item = self.cup.remove(0);

        match item {
            CupItem::PeopleAreSilent => {
                self.people_are_silent_pulled = true;
                self.record("'The People Are Silent' drawn from cup!");

                if self.momentum > 0 {
                    // Reduce Russian units based on momentum
                    let reducible: Vec<usize> = self
                        .units
                        .iter()
                        .enumerate()
                        .filter(|(_, u)| {
                            u.side == Side::Russia
                                && u.is_on_map()
                                && !u.is_reduced
                                && u.has_reduced_side
                        })
                        .map(|(i, _)| i)
                        .collect();

                    let count = (self.momentum as usize).min(reducible.len());
                    for &idx in &reducible[..count] {
                        let name = self.units[idx].id.name().to_string();
                        self.units[idx].is_reduced = true;
                        self.record(format!(
                            "{} flipped to REDUCED (People Are Silent, momentum {:+})",
                            name, self.momentum
                        ));
                        self.russian_reduced_this_turn = true;
                    }
                } else {
                    self.record("No reduction effect (Momentum 0 or less).");
                }
            }
            CupItem::Roadblock2 => {
                self.roadblocks[1] = Some(Location::OkaRiver); // Default placement
                self.record("Second Roadblock marker drawn. Placed at Oka River.");
            }
            CupItem::Unit(unit_id) => {
                // Determine deployment location
                let deploy_loc = match unit_id {
                    UnitId::Akhmat => Location::GroznyAkhmatBase,
                    UnitId::MechanizedRegiment | UnitId::ArmoredRegiment => Location::Kaluga,
                    _ => Location::Moscow,
                };

                // Check if enemy occupies the deployment location
                let final_loc = if !self.wagner_units_at(deploy_loc).is_empty() {
                    // Find adjacent non-enemy location
                    let neighbors = self.map.neighbors(deploy_loc);
                    let alt = neighbors
                        .iter()
                        .find(|(n, _)| self.wagner_units_at(*n).is_empty())
                        .map(|(n, _)| *n);
                    match alt {
                        Some(loc) => loc,
                        None => {
                            // Place on next turn (dispersed) — simplified: just place at Moscow
                            Location::Moscow
                        }
                    }
                } else {
                    deploy_loc
                };

                if let Some(unit) = self.find_unit_mut(unit_id) {
                    unit.location = Some(final_loc);
                }
                self.record(format!(
                    "{} deployed to {} from Moscow Mobilization Cup.",
                    unit_id.name(),
                    final_loc.name()
                ));
            }
        }
    }

    pub fn start_end_turn_phase(&mut self) {
        self.phase = Phase::EndTurn;
        self.screen = Screen::EndTurnConfirm;
        self.cursor = 0;
    }

    /// Apply momentum adjustments per rulebook 8.1, then advance the turn.
    pub fn end_turn(&mut self) {
        let mut delta = 0i32;

        // Check momentum questions
        let wagner_in_rublevo = self
            .units
            .iter()
            .any(|u| u.is_wagner() && u.location == Some(Location::Rublevo));
        let wagner_in_moscow = self
            .units
            .iter()
            .any(|u| u.is_wagner() && u.location == Some(Location::Moscow));
        let russia_in_rostov = self
            .units
            .iter()
            .any(|u| u.side == Side::Russia && u.location == Some(Location::RostovOnDon));

        if wagner_in_rublevo {
            delta += 1;
            self.record("Momentum +1: Wagner occupies Rublevo.");
        }
        if self.russian_reduced_this_turn {
            delta += 1;
            self.record("Momentum +1: Russian unit reduced this turn.");
        }
        if wagner_in_moscow {
            delta += 2;
            self.record("Momentum +2: Wagner occupies Moscow.");
        }
        if self.russian_eliminated_this_turn {
            delta += 2;
            self.record("Momentum +2: Russian unit eliminated this turn.");
        }
        if russia_in_rostov {
            delta -= 1;
            self.record("Momentum -1: Russia occupies Rostov.");
        }
        if self.wagner_repulsed_this_turn {
            delta -= 1;
            self.record("Momentum -1: Wagner unit repulsed this turn.");
        }
        if self.wagner_reduced_this_turn {
            delta -= 2;
            self.record("Momentum -2: Wagner unit reduced this turn.");
        }
        if self.wagner_eliminated_this_turn {
            delta -= 3;
            self.record("Momentum -3: Wagner unit eliminated this turn.");
        }

        let old = self.momentum;
        self.momentum = (self.momentum + delta).clamp(-3, 3);
        if delta != 0 {
            self.record(format!(
                "Momentum: {:+} → {:+} (delta {:+})",
                old, self.momentum, delta
            ));
        }

        // Reset turn tracking flags
        self.russian_reduced_this_turn = false;
        self.russian_eliminated_this_turn = false;
        self.wagner_repulsed_this_turn = false;
        self.wagner_reduced_this_turn = false;
        self.wagner_eliminated_this_turn = false;

        // Advance turn
        self.turn += 1;
        self.record(format!("Turn advanced to {}.", self.turn));

        // Check game over
        if self.turn > 6 {
            self.screen = Screen::GameOver {
                wagner_wins: false,
            };
        } else {
            self.start_administration();
        }
    }

    /// Check for Wagner automatic victory (LOC along M4).
    pub fn check_wagner_victory(&self) -> bool {
        let wlocs = self.wagner_locations();
        let rlocs = self.russian_locations();
        self.map.can_trace_loc(&wlocs, &rlocs)
    }
}
