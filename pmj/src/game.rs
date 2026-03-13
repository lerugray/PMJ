/// Game state module — holds everything that changes during play.

use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};

use crate::combat::{self, ContactOutcome, CrtResult};
use crate::map::{GameMap, Location};
use crate::mct::MctMarker;
use crate::units::{self, Side, Unit, UnitId};

/// Which phase of the turn we're in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Administration,  // Set MCT for each Wagner unit
    WagnerTurn,      // Move and initiate Contact
    RussianAI,       // Mobilization, movement, attacks
    EndTurn,         // Momentum adjustments, advance turn
}

/// What the player is currently doing in the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Title screen shown at game start.
    Title,
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
    /// Selecting which attackers participate in contact.
    ContactSelectAttackers {
        from_loc: Location,
        target_loc: Location,
        available: Vec<usize>,  // unit indices available to attack
        selected: Vec<bool>,    // which ones are toggled on
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
    /// Help / rules reference overlay.
    HelpScreen,
    /// Unit detail popup showing all unit info.
    UnitDetail(usize), // unit index
    /// Game over screen.
    GameOver { wagner_wins: bool },
}

/// Items that can be in the Moscow Mobilization Cup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub frame_count: u64, // Animation frame counter (incremented every tick)
    pub prev_screen: Option<Box<Screen>>, // For returning from help/overlays
    pub status_message: Option<String>,   // Transient message (e.g. "Game saved!")
    pub status_timer: u16,                // Frames remaining to show status
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
            screen: Screen::Title,
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
            frame_count: 0,
            prev_screen: None,
            status_message: None,
            status_timer: 0,
        }
    }

    /// Reset the game to a fresh initial state.
    pub fn restart(&mut self) {
        *self = GameState::new();
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

    /// Indices of all Wagner units that can be moved (on-map, Wagner side).
    pub fn moveable_unit_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        for id in UnitId::wagner_units() {
            if let Some(idx) = self.unit_index(*id) {
                if self.units[idx].is_on_map() && self.units[idx].is_wagner() {
                    indices.push(idx);
                }
            }
        }
        for (idx, unit) in self.units.iter().enumerate() {
            if unit.is_wagner() && unit.is_on_map() && !unit.id.is_wagner() {
                indices.push(idx);
            }
        }
        indices
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

    /// Find all locations reachable by a unit within its remaining MP.
    /// Returns Vec of (location, total_mp_cost) sorted by cost, excluding current location.
    /// Accounts for rivers, roadblocks, and enemy-occupied locations (which block movement).
    pub fn reachable_locations(&self, idx: usize) -> Vec<(Location, i32)> {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;

        let unit = &self.units[idx];
        let from = match unit.location {
            Some(loc) => loc,
            None => return Vec::new(),
        };
        let mp = self.mp_remaining(idx);
        let enemy_side = if unit.is_wagner() { Side::Russia } else { Side::Wagner };

        // Dijkstra: find cheapest path to every reachable location
        let mut best_cost: std::collections::HashMap<Location, i32> = std::collections::HashMap::new();
        let mut heap: BinaryHeap<Reverse<(i32, Location)>> = BinaryHeap::new();

        best_cost.insert(from, 0);
        heap.push(Reverse((0, from)));

        while let Some(Reverse((cost, loc))) = heap.pop() {
            if cost > *best_cost.get(&loc).unwrap_or(&i32::MAX) {
                continue;
            }
            for &(neighbor, _edge) in self.map.neighbors(loc) {
                // Can't move through enemy-occupied locations
                let enemy_there = self.units_at(neighbor).iter().any(|&i| self.units[i].side == enemy_side);
                if enemy_there {
                    continue;
                }
                let step_cost = match self.move_cost(idx, loc, neighbor) {
                    Some(c) => c,
                    None => continue,
                };
                let total = cost + step_cost;
                if total > mp {
                    continue;
                }
                if total < *best_cost.get(&neighbor).unwrap_or(&i32::MAX) {
                    best_cost.insert(neighbor, total);
                    heap.push(Reverse((total, neighbor)));
                }
            }
        }

        let mut result: Vec<(Location, i32)> = best_cost
            .into_iter()
            .filter(|(loc, _)| *loc != from)
            .collect();
        result.sort_by(|(loc_a, cost_a), (loc_b, cost_b)| {
            cost_a.cmp(cost_b).then(loc_a.cmp(loc_b))
        });
        result
    }

    pub fn move_cost(&self, idx: usize, from: Location, to: Location) -> Option<i32> {
        let edge = self.map.edge(from, to)?;
        let mut cost = 1;
        if edge.river && !self.units[idx].is_helicopter() {
            cost += 1;
        }
        // Roadblock: Wagner pays +1 MP to enter a roadblocked location
        if self.units[idx].is_wagner() {
            if self.roadblocks[0] == Some(to) || self.roadblocks[1] == Some(to) {
                cost += 1;
            }
        }
        Some(cost)
    }

    #[allow(dead_code)]
    pub fn can_move(&self, idx: usize, to: Location) -> Result<i32, String> {
        let unit = &self.units[idx];
        let from = unit.location.ok_or("Unit is off the map.")?;
        if from == to {
            return Err("Already there.".into());
        }

        // Stacking: only units from one side can occupy a location (rulebook 3.9)
        let enemy_side = if unit.is_wagner() {
            Side::Russia
        } else {
            Side::Wagner
        };
        let enemy_there = self
            .units_at(to)
            .iter()
            .any(|&i| self.units[i].side == enemy_side);
        if enemy_there {
            return Err("Enemy units occupy that location.".into());
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

    #[allow(dead_code)]
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

    /// Move a unit to a distant location, spending the given total MP cost.
    /// Used for multi-hop movement where the player picks a reachable destination directly.
    pub fn move_unit_to(&mut self, idx: usize, to: Location, total_cost: i32) -> bool {
        let from = match self.units[idx].location {
            Some(loc) => loc,
            None => return false,
        };
        let mp = self.mp_remaining(idx);
        if total_cost > mp {
            self.record("Not enough MP.".to_string());
            return false;
        }
        let name = self.units[idx].id.name().to_string();
        self.units[idx].location = Some(to);
        self.units[idx].mp_spent += total_cost;
        let remaining = self.mp_remaining(idx);

        self.record(format!(
            "{} moved {} → {} (cost {} MP, {} remaining)",
            name,
            from.name(),
            to.name(),
            total_cost,
            remaining
        ));
        true
    }

    // ── Contact ──────────────────────────────────────────────────────────

    /// Count flanking locations: other Wagner-occupied locations adjacent to target (max 2).
    pub fn count_flanking(&self, primary_loc: Location, target_loc: Location) -> i32 {
        let mut count = 0;
        for &(neighbor, _) in self.map.neighbors(target_loc) {
            if neighbor == primary_loc {
                continue; // Primary attack location doesn't count
            }
            // Only count locations with non-police Wagner combat units
            let has_combat_unit = self.wagner_units_at(neighbor)
                .iter()
                .any(|&i| !self.units[i].police);
            if has_combat_unit {
                count += 1;
            }
        }
        count.min(2)
    }

    /// Find Wagner locations that have adjacent Russian-occupied locations.
    pub fn contact_opportunities(&self) -> Vec<(Location, Vec<Location>)> {
        let mut results = Vec::new();
        for wloc in self.wagner_locations() {
            // Police units cannot initiate contact
            let has_attacker = self.wagner_units_at(wloc)
                .iter()
                .any(|&i| !self.units[i].police);
            if !has_attacker {
                continue;
            }
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

        // Flanking: count other Wagner-occupied locations adjacent to the target
        let flanking = self.count_flanking(from_loc, target_loc);

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
            self.units[idx].dispersed = true;
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
        // Return dispersed units to their home location
        for i in 0..self.units.len() {
            if self.units[i].dispersed {
                let name = self.units[i].id.name().to_string();
                let home = if self.units[i].is_wagner() {
                    Location::RostovOnDon
                } else {
                    Location::Moscow
                };
                self.units[i].location = Some(home);
                self.units[i].dispersed = false;
                self.record(format!("{} returns from dispersal to {}", name, home.name()));
            }
        }
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

        // Full Russian AI Phase per rulebook section 7.0
        self.record("── Russian AI Phase ──");

        // 7.1 Moscow Mobilization
        self.run_russian_mobilization();

        // 7.2 Russian Momentum Expenditure
        self.run_russian_momentum_expenditure();

        // 7.3 Deploy Roadblock
        self.run_russian_roadblock_deploy();

        // Reset MP for Russian units
        for unit in &mut self.units {
            if unit.side == Side::Russia {
                unit.reset_mp();
            }
        }

        // 7.4 Russian AI Priority Table + 7.5 Russian Attacks
        self.run_russian_ai_attacks();

        // Akhmat special rule (7.4.3)
        self.run_akhmat_tiktok();
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

    // ── 7.2 Russian Momentum Expenditure ────────────────────────────────

    fn run_russian_momentum_expenditure(&mut self) {
        if self.momentum >= 0 {
            return; // AI only spends negative momentum
        }

        let spend = (-self.momentum) as usize; // 1, 2, or 3
        self.record(format!(
            "Russian AI spending {} negative momentum.",
            spend
        ));

        match spend {
            1 => {
                // Restore one reduced Russian unit
                if let Some(idx) = self.find_reduced_russian() {
                    let name = self.units[idx].id.name().to_string();
                    self.units[idx].is_reduced = false;
                    self.record(format!("{} restored to full strength.", name));
                }
            }
            2 => {
                // Restore two reduced OR rebuild one eliminated to reduced in Moscow
                let r1 = self.find_reduced_russian();
                let r2 = r1.and_then(|first| self.find_reduced_russian_except(first));
                if r1.is_some() && r2.is_some() {
                    for idx in [r1.unwrap(), r2.unwrap()] {
                        let name = self.units[idx].id.name().to_string();
                        self.units[idx].is_reduced = false;
                        self.record(format!("{} restored to full strength.", name));
                    }
                } else if r1.is_some() {
                    let idx = r1.unwrap();
                    let name = self.units[idx].id.name().to_string();
                    self.units[idx].is_reduced = false;
                    self.record(format!("{} restored to full strength.", name));
                    // Try rebuilding an eliminated unit
                    self.rebuild_eliminated_russian();
                } else {
                    self.rebuild_eliminated_russian();
                }
            }
            3 => {
                // Restore eliminated to full strength in Moscow + restore up to 3 reduced
                self.rebuild_eliminated_russian_full();
                for _ in 0..3 {
                    if let Some(idx) = self.find_reduced_russian() {
                        let name = self.units[idx].id.name().to_string();
                        self.units[idx].is_reduced = false;
                        self.record(format!("{} restored to full strength.", name));
                    }
                }
            }
            _ => {}
        }

        // Momentum spent → shift back to 0
        let old = self.momentum;
        self.momentum = 0;
        self.record(format!("Momentum spent: {:+} → 0", old));
    }

    fn find_reduced_russian(&self) -> Option<usize> {
        self.units
            .iter()
            .enumerate()
            .find(|(_, u)| u.side == Side::Russia && u.is_on_map() && u.is_reduced)
            .map(|(i, _)| i)
    }

    fn find_reduced_russian_except(&self, except: usize) -> Option<usize> {
        self.units
            .iter()
            .enumerate()
            .find(|(i, u)| {
                *i != except && u.side == Side::Russia && u.is_on_map() && u.is_reduced
            })
            .map(|(i, _)| i)
    }

    fn rebuild_eliminated_russian(&mut self) {
        if let Some(idx) = self
            .units
            .iter()
            .enumerate()
            .find(|(_, u)| u.side == Side::Russia && !u.is_on_map() && !u.dispersed && u.has_reduced_side)
            .map(|(i, _)| i)
        {
            let name = self.units[idx].id.name().to_string();
            self.units[idx].location = Some(Location::Moscow);
            self.units[idx].is_reduced = true;
            self.record(format!(
                "{} rebuilt to REDUCED and placed in Moscow.",
                name
            ));
        }
    }

    fn rebuild_eliminated_russian_full(&mut self) {
        if let Some(idx) = self
            .units
            .iter()
            .enumerate()
            .find(|(_, u)| u.side == Side::Russia && !u.is_on_map() && !u.dispersed)
            .map(|(i, _)| i)
        {
            let name = self.units[idx].id.name().to_string();
            self.units[idx].location = Some(Location::Moscow);
            self.units[idx].is_reduced = false;
            self.record(format!(
                "{} rebuilt to FULL strength and placed in Moscow.",
                name
            ));
        }
    }

    // ── 7.3 Roadblock Deployment ─────────────────────────────────────────

    fn run_russian_roadblock_deploy(&mut self) {
        // Find the Wagner unit closest to Moscow and place roadblock between them
        let mut closest_wagner_loc: Option<Location> = None;
        let mut closest_dist = usize::MAX;

        // Simple BFS distance from Moscow
        for wloc in self.wagner_locations() {
            let dist = self.bfs_distance(wloc, Location::Moscow);
            if dist < closest_dist {
                closest_dist = dist;
                closest_wagner_loc = Some(wloc);
            }
        }

        if let Some(wagner_loc) = closest_wagner_loc {
            // Place roadblock on a location between Wagner and Moscow/Rublevo
            // that doesn't already have a roadblock, isn't Rostov or Grozny
            let path = self.bfs_path(wagner_loc, Location::Moscow);
            for &loc in &path {
                if loc == wagner_loc
                    || loc == Location::RostovOnDon
                    || loc == Location::GroznyAkhmatBase
                {
                    continue;
                }
                // Don't double up
                if self.roadblocks[0] == Some(loc) || self.roadblocks[1] == Some(loc) {
                    continue;
                }

                // Find an empty roadblock slot, or reposition if both filled
                if self.roadblocks[0].is_none() {
                    self.roadblocks[0] = Some(loc);
                    self.record(format!("Roadblock 1 deployed at {}.", loc.name()));
                    break;
                } else if self.roadblocks[1].is_none() {
                    self.roadblocks[1] = Some(loc);
                    self.record(format!("Roadblock 2 deployed at {}.", loc.name()));
                    break;
                } else {
                    // Both slots filled — reposition roadblock 2
                    let old = self.roadblocks[1].unwrap();
                    if old != loc {
                        self.roadblocks[1] = Some(loc);
                        self.record(format!(
                            "Roadblock 2 repositioned {} → {}.",
                            old.name(),
                            loc.name()
                        ));
                    }
                    break;
                }
            }
        }
    }

    // ── 7.4 Russian AI Priority Table ────────────────────────────────────

    fn run_russian_ai_attacks(&mut self) {
        // Gather Russian units on map that can attack (not police)
        let russian_attackers: Vec<usize> = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                u.side == Side::Russia && u.is_on_map() && !u.police
            })
            .map(|(i, _)| i)
            .collect();

        if russian_attackers.is_empty() {
            self.record("No Russian units available for attack.");
            return;
        }

        // RAPT Step 1: Find attacks with CD >= +3 including all modifiers
        let mut best_attack: Option<(Vec<usize>, Location, Location, i32)> = None;

        for wloc in self.wagner_locations() {
            let wagner_sp: i32 = self
                .wagner_units_at(wloc)
                .iter()
                .map(|&i| self.effective_sp(i))
                .sum();

            // Find Russian units adjacent to this Wagner location
            for &(neighbor, edge_props) in self.map.neighbors(wloc) {
                let r_here: Vec<usize> = russian_attackers
                    .iter()
                    .copied()
                    .filter(|&i| self.units[i].location == Some(neighbor))
                    .collect();
                if r_here.is_empty() {
                    continue;
                }

                let attack_sp: i32 = r_here.iter().map(|&i| self.effective_sp(i)).sum();
                let cd = attack_sp - wagner_sp;
                let fr = combat::force_ratio_shift(attack_sp, wagner_sp);
                let cd_adj = cd + fr;

                // Estimate DRMs (inverted momentum for Russia)
                let momentum_drm = -self.momentum;
                let river_drm: i32 = if edge_props.river { -1 } else { 0 };
                let effective_cd = cd_adj; // We check CD column, not die result

                if effective_cd >= 3 {
                    let score = effective_cd + momentum_drm - river_drm.abs();
                    if best_attack.is_none()
                        || score > best_attack.as_ref().unwrap().3
                    {
                        best_attack = Some((r_here, neighbor, wloc, score));
                    }
                }
            }
        }

        if let Some((attackers, from, target, _)) = best_attack {
            // Execute Russian attack
            let attack_sp: i32 = attackers.iter().map(|&i| self.effective_sp(i)).sum();
            let defend_sp: i32 = self
                .wagner_units_at(target)
                .iter()
                .map(|&i| self.effective_sp(i))
                .sum();

            let river = self
                .map
                .edge(from, target)
                .map(|e| e.river)
                .unwrap_or(false);
            let all_heli = attackers.iter().all(|&i| self.units[i].is_helicopter());

            // Russia uses inverted momentum as DRM
            let inverted_momentum = -self.momentum;

            let outcome = combat::resolve_contact(
                attack_sp,
                defend_sp,
                inverted_momentum,
                river,
                false, // Russia never attacks Moscow
                0,     // No flanking for AI (simplified)
                all_heli,
            );

            let atk_names: Vec<&str> = attackers
                .iter()
                .map(|&i| self.units[i].id.name())
                .collect();

            self.record(format!(
                "Russian Attack: {} @ {} → {} | ATK:{} DEF:{} | Roll:{} | {}",
                atk_names.join(", "),
                from.name(),
                target.name(),
                attack_sp,
                defend_sp,
                outcome.die_roll,
                outcome.result.code()
            ));

            // For Russian attacks, attacker=Russia, defender=Wagner
            // We need to flip the logic: AR hurts Russia, Rp/R/S hurts Wagner
            let defender_indices = self.wagner_units_at(target);
            self.apply_contact_result(&outcome, &attackers, from, &defender_indices, target);
        } else {
            // RAPT Step 1a: No good attacks — move toward Moscow/Oka River
            self.record("No favorable attacks. Russian units hold position.");

            // Move units toward Moscow if not already there
            for &idx in &russian_attackers {
                let loc = self.units[idx].location.unwrap();
                if loc == Location::Moscow || loc == Location::OkaRiver {
                    continue;
                }
                let mp = self.units[idx].base_mp;
                if mp <= 0 {
                    continue;
                }

                // Find neighbor closest to Moscow
                let neighbors = self.map.neighbors(loc);
                let mut best_dest: Option<Location> = None;
                let mut best_dist = self.bfs_distance(loc, Location::Moscow);

                for &(neighbor, _) in neighbors {
                    // Don't move into Wagner-occupied locations
                    if !self.wagner_units_at(neighbor).is_empty() {
                        continue;
                    }
                    let dist = self.bfs_distance(neighbor, Location::Moscow);
                    if dist < best_dist {
                        best_dist = dist;
                        best_dest = Some(neighbor);
                    }
                }

                if let Some(dest) = best_dest {
                    let name = self.units[idx].id.name().to_string();
                    self.units[idx].location = Some(dest);
                    self.record(format!(
                        "{} moved {} → {} (toward Moscow).",
                        name,
                        loc.name(),
                        dest.name()
                    ));
                }
            }
        }

        // RAPT Step 5: Check Rublevo — if Wagner occupies it, prioritize attack
        if !self.wagner_units_at(Location::Rublevo).is_empty() {
            // Already handled above in the general attack search
            // (Rublevo is adjacent to Moscow which is where many Russian units are)
        }
    }

    // ── Akhmat Tik Tok rule (7.4.3) ─────────────────────────────────────

    fn run_akhmat_tiktok(&mut self) {
        use rand::Rng;

        let akhmat_idx = match self.unit_index(UnitId::Akhmat) {
            Some(i) => i,
            None => return,
        };

        if self.units[akhmat_idx].location != Some(Location::GroznyAkhmatBase) {
            return; // Only applies while in Grozny
        }

        let roll = rand::thread_rng().gen_range(1..=6);
        if roll == 6 {
            self.record(format!(
                "Akhmat Tik Tok roll: {} — Akhmat moves toward Rostov!",
                roll
            ));
            self.units[akhmat_idx].location = Some(Location::RostovOnDon);
            self.record("Akhmat moved Grozny → Rostov-On-Don.");

            // Attack Rostov if Wagner is there
            if !self.wagner_units_at(Location::RostovOnDon).is_empty() {
                let atk_sp = self.effective_sp(akhmat_idx);
                let def_sp: i32 = self
                    .wagner_units_at(Location::RostovOnDon)
                    .iter()
                    .map(|&i| self.effective_sp(i))
                    .sum();
                let inverted_momentum = -self.momentum;
                let outcome = combat::resolve_contact(
                    atk_sp, def_sp, inverted_momentum,
                    true, // River crossing Grozny -> Rostov
                    false, 0, false,
                );
                self.record(format!(
                    "Akhmat attacks Rostov! Roll:{} → {}",
                    outcome.die_roll,
                    outcome.result.code()
                ));
                let defenders = self.wagner_units_at(Location::RostovOnDon);
                self.apply_contact_result(
                    &outcome,
                    &[akhmat_idx],
                    Location::GroznyAkhmatBase,
                    &defenders,
                    Location::RostovOnDon,
                );
            }
        } else {
            self.record(format!(
                "Akhmat Tik Tok roll: {} — too busy making Tik Toks.",
                roll
            ));
        }
    }

    // ── BFS helpers for AI pathfinding ───────────────────────────────────

    fn bfs_distance(&self, from: Location, to: Location) -> usize {
        if from == to {
            return 0;
        }
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((from, 0usize));
        visited.insert(from);

        while let Some((loc, dist)) = queue.pop_front() {
            for &(neighbor, _) in self.map.neighbors(loc) {
                if neighbor == to {
                    return dist + 1;
                }
                if visited.insert(neighbor) {
                    queue.push_back((neighbor, dist + 1));
                }
            }
        }
        usize::MAX // unreachable in a connected graph
    }

    fn bfs_path(&self, from: Location, to: Location) -> Vec<Location> {
        if from == to {
            return vec![from];
        }
        let mut visited = std::collections::HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(from);
        visited.insert(from, None);

        while let Some(loc) = queue.pop_front() {
            for &(neighbor, _) in self.map.neighbors(loc) {
                if !visited.contains_key(&neighbor) {
                    visited.insert(neighbor, Some(loc));
                    if neighbor == to {
                        // Reconstruct path
                        let mut path = vec![to];
                        let mut current = to;
                        while let Some(Some(prev)) = visited.get(&current) {
                            path.push(*prev);
                            current = *prev;
                        }
                        path.reverse();
                        return path;
                    }
                    queue.push_back(neighbor);
                }
            }
        }
        vec![] // No path found
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

    /// Show a transient status message for ~2 seconds.
    pub fn flash_message(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.status_timer = 20; // ~2 seconds at 100ms poll
    }

    /// Tick down the status timer (call each frame).
    pub fn tick_status(&mut self) {
        if self.status_timer > 0 {
            self.status_timer -= 1;
            if self.status_timer == 0 {
                self.status_message = None;
            }
        }
    }

    // ── Save / Load ─────────────────────────────────────────────────────

    /// Save game state to pmj_save.json.
    pub fn save_game(&mut self) -> Result<(), String> {
        let data = SaveData {
            units: self.units.clone(),
            mct: self.mct.clone(),
            turn: self.turn,
            momentum: self.momentum,
            phase: self.phase,
            cup: self.cup.clone(),
            roadblocks: self.roadblocks,
            people_are_silent_pulled: self.people_are_silent_pulled,
            log: self.log.clone(),
            russian_reduced_this_turn: self.russian_reduced_this_turn,
            russian_eliminated_this_turn: self.russian_eliminated_this_turn,
            wagner_repulsed_this_turn: self.wagner_repulsed_this_turn,
            wagner_reduced_this_turn: self.wagner_reduced_this_turn,
            wagner_eliminated_this_turn: self.wagner_eliminated_this_turn,
            admin_units_adjusted: self.admin_units_adjusted,
        };
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write("pmj_save.json", json)
            .map_err(|e| format!("File write error: {}", e))?;
        self.record("Game saved.");
        self.flash_message("Game saved!");
        Ok(())
    }

    /// Load game state from pmj_save.json.
    pub fn load_game(&mut self) -> Result<(), String> {
        let json = std::fs::read_to_string("pmj_save.json")
            .map_err(|e| format!("File read error: {}", e))?;
        let data: SaveData = serde_json::from_str(&json)
            .map_err(|e| format!("Deserialize error: {}", e))?;

        self.units = data.units;
        self.mct = data.mct;
        self.turn = data.turn;
        self.momentum = data.momentum;
        self.phase = data.phase;
        self.cup = data.cup;
        self.roadblocks = data.roadblocks;
        self.people_are_silent_pulled = data.people_are_silent_pulled;
        self.log = data.log;
        self.russian_reduced_this_turn = data.russian_reduced_this_turn;
        self.russian_eliminated_this_turn = data.russian_eliminated_this_turn;
        self.wagner_repulsed_this_turn = data.wagner_repulsed_this_turn;
        self.wagner_reduced_this_turn = data.wagner_reduced_this_turn;
        self.wagner_eliminated_this_turn = data.wagner_eliminated_this_turn;
        self.admin_units_adjusted = data.admin_units_adjusted;
        self.map = GameMap::new();
        self.cursor = 0;
        self.log_scroll = 0;
        self.prev_screen = None;

        // Set screen based on phase
        self.screen = match self.phase {
            Phase::Administration => Screen::MctSelect,
            Phase::WagnerTurn => Screen::PhaseMenu,
            Phase::RussianAI => Screen::RussianPhaseDisplay,
            Phase::EndTurn => Screen::EndTurnConfirm,
        };

        self.record("Game loaded from save.");
        self.flash_message("Game loaded!");
        Ok(())
    }

    /// Check if a save file exists.
    pub fn save_exists() -> bool {
        std::path::Path::new("pmj_save.json").exists()
    }
}

/// Serializable snapshot of game state (excludes UI-only fields).
#[derive(Serialize, Deserialize)]
struct SaveData {
    units: Vec<Unit>,
    mct: [MctMarker; 3],
    turn: i32,
    momentum: i32,
    phase: Phase,
    cup: Vec<CupItem>,
    roadblocks: [Option<Location>; 2],
    people_are_silent_pulled: bool,
    log: Vec<String>,
    russian_reduced_this_turn: bool,
    russian_eliminated_this_turn: bool,
    wagner_repulsed_this_turn: bool,
    wagner_reduced_this_turn: bool,
    wagner_eliminated_this_turn: bool,
    admin_units_adjusted: [bool; 3],
}
