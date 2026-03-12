# map_data.py
#
# This file defines the game map.
# The map is stored as a "graph" — a dictionary where each location
# points to a list of neighbors, and each connection has properties.
#
# Edge properties:
#   river = True  →  crossing costs +1 MP (except Helicopters)
#   m4    = True  →  M4 highway route (reserved for future LOC logic)


class MapGraph:
    """
    Stores the game map as an adjacency dictionary.

    self.edges looks like:
    {
        "Rostov-On-Don": [
            ("Grozny Akhmat Base", {"river": True,  "m4": False}),
            ("Bugaevka Border Point", {"river": False, "m4": True}),
        ],
        ...
    }
    """

    def __init__(self):
        self.edges = {}
        self._build_map()

    def _add_edge(self, loc_a, loc_b, river=False, m4=False):
        """
        Add a two-way connection between loc_a and loc_b.
        Both locations will list each other as neighbors.
        """
        props = {"river": river, "m4": m4}

        # Make sure both locations exist in the dictionary
        if loc_a not in self.edges:
            self.edges[loc_a] = []
        if loc_b not in self.edges:
            self.edges[loc_b] = []

        # Add each as the other's neighbor
        self.edges[loc_a].append((loc_b, props))
        self.edges[loc_b].append((loc_a, props))

    def _build_map(self):
        """
        Define every connection on the PMJ map.
        Transcribed directly from the PMJ adjacency list.
        """
        # Rostov-On-Don connections
        self._add_edge("Rostov-On-Don",        "Grozny Akhmat Base",     river=True,  m4=False)
        self._add_edge("Rostov-On-Don",        "Bugaevka Border Point",  river=True, m4=True)

        # Bugaevka Border Point connections
        # Note: Rostov side already added above
        self._add_edge("Bugaevka Border Point", "Voronezh",              river=False,  m4=True)

        # Voronezh connections
        # Note: Bugaevka already added above
        self._add_edge("Voronezh", "Orel",    river=True,  m4=False)
        self._add_edge("Voronezh", "Lipetsk", river=False, m4=True)
        self._add_edge("Voronezh", "Tambov",  river=False, m4=False)

        # Orel connections
        # Note: Voronezh already added above
        self._add_edge("Orel", "Kaluga",  river=False, m4=False)
        self._add_edge("Orel", "Tula",    river=False, m4=False)
        self._add_edge("Orel", "Lipetsk", river=False, m4=False)

        # Lipetsk connections
        # Note: Voronezh and Orel already added above
        self._add_edge("Lipetsk", "Tula",   river=False, m4=True)
        self._add_edge("Lipetsk", "Tambov", river=False, m4=False)

        # Tambov connections
        # Note: Voronezh and Lipetsk already added above
        self._add_edge("Tambov", "Ryazan", river=False, m4=False)

        # Kaluga connections
        # Note: Orel already added above
        self._add_edge("Kaluga", "Tula",      river=False, m4=False)
        self._add_edge("Kaluga", "Oka River", river=False, m4=False)
        self._add_edge("Kaluga", "Moscow",    river=False, m4=False)
        self._add_edge("Kaluga", "Rublevo",   river=False, m4=False)

        # Tula connections
        # Note: Kaluga, Orel, Lipetsk already added above
        self._add_edge("Tula", "Oka River", river=True,  m4=True)
        self._add_edge("Tula", "Ryazan",    river=False, m4=False)

        # Ryazan connections
        # Note: Tambov and Tula already added above
        self._add_edge("Ryazan", "Oka River", river=True, m4=False)

        # Oka River connections
        # Note: Kaluga, Tula, Ryazan already added above
        self._add_edge("Oka River", "Moscow",  river=False, m4=True)
        self._add_edge("Oka River", "Rublevo", river=False, m4=False)

        # Rublevo connections
        # Note: Kaluga and Oka River already added above
        self._add_edge("Rublevo", "Moscow", river=False, m4=False)

        # Moscow connections: all already added above

    def get_edge(self, loc_a, loc_b):
        """
        Return the edge properties dict between loc_a and loc_b.
        Returns None if they are not connected.
        """
        for neighbor, props in self.edges.get(loc_a, []):
            if neighbor == loc_b:
                return props
        return None

    def are_connected(self, loc_a, loc_b):
        """Return True if loc_a and loc_b are directly adjacent."""
        return self.get_edge(loc_a, loc_b) is not None

    def get_neighbors(self, location):
        """
        Return a list of (neighbor_name, props_dict) for a given location.
        Returns empty list if location is not on the map.
        """
        return self.edges.get(location, [])

    def all_locations(self):
        """Return a sorted list of every location on the map."""
        return sorted(self.edges.keys())