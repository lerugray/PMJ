/// Map module — 13 point-to-point locations with typed edges.
/// Transcribed directly from the PMJ rulebook adjacency list and map.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Every location on the game map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Location {
    RostovOnDon,
    GroznyAkhmatBase,
    BugaevkaBorderPoint,
    Voronezh,
    Orel,
    Lipetsk,
    Tambov,
    Kaluga,
    Tula,
    Ryazan,
    OkaRiver,
    Rublevo,
    Moscow,
}

impl Location {
    pub fn name(&self) -> &'static str {
        match self {
            Location::RostovOnDon => "Rostov-On-Don",
            Location::GroznyAkhmatBase => "Grozny",
            Location::BugaevkaBorderPoint => "Bugaevka B.P.",
            Location::Voronezh => "Voronezh",
            Location::Orel => "Orel",
            Location::Lipetsk => "Lipetsk",
            Location::Tambov => "Tambov",
            Location::Kaluga => "Kaluga",
            Location::Tula => "Tula",
            Location::Ryazan => "Ryazan",
            Location::OkaRiver => "Oka River",
            Location::Rublevo => "Rublevo",
            Location::Moscow => "Moscow",
        }
    }

    /// Short 3-letter label for the map display.
    pub fn short(&self) -> &'static str {
        match self {
            Location::RostovOnDon => "ROS",
            Location::GroznyAkhmatBase => "GRO",
            Location::BugaevkaBorderPoint => "BUG",
            Location::Voronezh => "VOR",
            Location::Orel => "ORL",
            Location::Lipetsk => "LIP",
            Location::Tambov => "TAM",
            Location::Kaluga => "KAL",
            Location::Tula => "TUL",
            Location::Ryazan => "RYA",
            Location::OkaRiver => "OKA",
            Location::Rublevo => "RUB",
            Location::Moscow => "MOS",
        }
    }

    /// All locations in order (south to north, roughly).
    pub fn all() -> &'static [Location] {
        &[
            Location::GroznyAkhmatBase,
            Location::RostovOnDon,
            Location::BugaevkaBorderPoint,
            Location::Voronezh,
            Location::Tambov,
            Location::Lipetsk,
            Location::Orel,
            Location::Tula,
            Location::Ryazan,
            Location::Kaluga,
            Location::OkaRiver,
            Location::Rublevo,
            Location::Moscow,
        ]
    }

    /// Is this location on the M4 route (used for LOC tracing)?
    pub fn on_m4(&self) -> bool {
        matches!(
            self,
            Location::RostovOnDon
                | Location::BugaevkaBorderPoint
                | Location::Voronezh
                | Location::Lipetsk
                | Location::Tula
                | Location::OkaRiver
                | Location::Moscow
        )
    }

    /// Map display coordinates (col, row) for the TUI map.
    /// These approximate the physical map layout.
    /// Positions are spread so adjacent M4 locations have ≥7 rows between them,
    /// ensuring Bresenham road lines are visible between label boxes.
    pub fn map_pos(&self) -> (u16, u16) {
        match self {
            // (col, row) — origin top-left, centered in ~64-wide map panel
            // M4 route runs roughly down the center: Moscow→Oka→Tula→Lipetsk→Voronezh→Bugaevka→Rostov
            Location::Moscow =>              (39, 0),
            Location::Rublevo =>             (19, 3),
            Location::OkaRiver =>            (43, 7),
            Location::Kaluga =>              (9, 11),
            Location::Tula =>                (33, 13),
            Location::Ryazan =>              (54, 11),
            Location::Orel =>                (9, 20),
            Location::Lipetsk =>             (33, 20),
            Location::Tambov =>              (54, 20),
            Location::Voronezh =>            (33, 27),
            Location::BugaevkaBorderPoint => (33, 34),
            Location::RostovOnDon =>         (25, 41),
            Location::GroznyAkhmatBase =>    (52, 43),
        }
    }
}

/// Properties of a connection between two locations.
#[derive(Debug, Clone, Copy)]
pub struct EdgeProps {
    pub river: bool,
    pub m4: bool,
}

/// The game map as an adjacency graph.
pub struct GameMap {
    edges: HashMap<Location, Vec<(Location, EdgeProps)>>,
}

impl GameMap {
    pub fn new() -> Self {
        let mut map = GameMap {
            edges: HashMap::new(),
        };
        map.build();
        map
    }

    fn add_edge(&mut self, a: Location, b: Location, river: bool, m4: bool) {
        let props = EdgeProps { river, m4 };
        self.edges.entry(a).or_default().push((b, props));
        self.edges.entry(b).or_default().push((a, props));
    }

    fn build(&mut self) {
        use Location::*;

        // Rostov-On-Don
        self.add_edge(RostovOnDon, GroznyAkhmatBase, true, false);
        self.add_edge(RostovOnDon, BugaevkaBorderPoint, true, true);

        // Bugaevka Border Point
        self.add_edge(BugaevkaBorderPoint, Voronezh, false, true);

        // Voronezh
        self.add_edge(Voronezh, Orel, true, false);
        self.add_edge(Voronezh, Lipetsk, false, true);
        self.add_edge(Voronezh, Tambov, false, false);

        // Orel
        self.add_edge(Orel, Kaluga, false, false);
        self.add_edge(Orel, Tula, false, false);
        self.add_edge(Orel, Lipetsk, false, false);

        // Lipetsk
        self.add_edge(Lipetsk, Tula, false, true);
        self.add_edge(Lipetsk, Tambov, false, false);

        // Tambov
        self.add_edge(Tambov, Ryazan, false, false);

        // Kaluga
        self.add_edge(Kaluga, Tula, false, false);
        self.add_edge(Kaluga, OkaRiver, false, false);
        self.add_edge(Kaluga, Moscow, false, false);
        self.add_edge(Kaluga, Rublevo, false, false);

        // Tula
        self.add_edge(Tula, OkaRiver, true, true);
        self.add_edge(Tula, Ryazan, false, false);

        // Ryazan
        self.add_edge(Ryazan, OkaRiver, true, false);

        // Oka River
        self.add_edge(OkaRiver, Moscow, false, true);
        self.add_edge(OkaRiver, Rublevo, false, false);

        // Rublevo
        self.add_edge(Rublevo, Moscow, false, false);
    }

    /// Get all neighbors of a location with their edge properties.
    pub fn neighbors(&self, loc: Location) -> &[(Location, EdgeProps)] {
        self.edges.get(&loc).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get edge properties between two locations, if connected.
    pub fn edge(&self, a: Location, b: Location) -> Option<EdgeProps> {
        self.neighbors(a)
            .iter()
            .find(|(n, _)| *n == b)
            .map(|(_, p)| *p)
    }

    /// Are two locations directly connected?
    pub fn connected(&self, a: Location, b: Location) -> bool {
        self.edge(a, b).is_some()
    }

    /// Check if Wagner can trace a Line of Communications along M4 from Moscow to Rostov.
    /// All M4 locations must be free of enemy (Russian) units and Moscow must have a friendly unit.
    /// `friendly_locs` = set of locations occupied by Wagner, `enemy_locs` = locations with Russian units.
    pub fn can_trace_loc(
        &self,
        wagner_locations: &[Location],
        russian_locations: &[Location],
    ) -> bool {
        use Location::*;
        // M4 path in order: Rostov -> Bugaevka -> Voronezh -> Lipetsk -> Tula -> OkaRiver -> Moscow
        let m4_path = [
            RostovOnDon,
            BugaevkaBorderPoint,
            Voronezh,
            Lipetsk,
            Tula,
            OkaRiver,
            Moscow,
        ];

        // Moscow must be Wagner-occupied
        if !wagner_locations.contains(&Moscow) {
            return false;
        }

        // Every M4 location must be free of Russian units
        for loc in &m4_path {
            if russian_locations.contains(loc) {
                return false;
            }
        }

        true
    }

    /// Get all edges for drawing the map. Returns (from, to, props) without duplicates.
    pub fn all_edges(&self) -> Vec<(Location, Location, EdgeProps)> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (&loc, neighbors) in &self.edges {
            for &(neighbor, props) in neighbors {
                let key = if loc as u8 <= neighbor as u8 {
                    (loc, neighbor)
                } else {
                    (neighbor, loc)
                };
                if seen.insert(key) {
                    result.push((loc, neighbor, props));
                }
            }
        }
        result
    }
}
