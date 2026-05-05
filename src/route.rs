/// Route path for PLC communication
#[derive(Debug, Clone)]
pub struct RoutePath {
    pub slots: Vec<u8>,
    pub ports: Vec<u8>,
    pub addresses: Vec<String>,
}

impl RoutePath {
    /// Creates a new route path
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            ports: Vec::new(),
            addresses: Vec::new(),
        }
    }

    /// Adds a backplane slot to the route
    #[must_use]
    pub fn add_slot(mut self, slot: u8) -> Self {
        self.slots.push(slot);
        self
    }

    /// Adds a network port to the route
    #[must_use]
    pub fn add_port(mut self, port: u8) -> Self {
        self.ports.push(port);
        self
    }

    /// Adds a network address to the route
    #[must_use]
    pub fn add_address(mut self, address: String) -> Self {
        self.addresses.push(address);
        self
    }

    /// Builds CIP route path bytes
    ///
    /// Reference: EtherNetIP_Connection_Paths_and_Routing.md, Port Segment Encoding
    /// According to the examples: Port 1 (backplane), Slot X = [0x01, X]
    /// The 0x01 byte encodes both "Port Segment (8-bit link)" AND "Port 1 (backplane)"
    /// Examples from documentation:
    ///   - Slot 0: `01 00`
    ///   - Slot 1: `01 01`
    ///   - Slot 2: `01 02`
    #[must_use]
    pub fn to_cip_bytes(&self) -> Vec<u8> {
        let mut path = Vec::new();

        // Add backplane slots
        // Reference: EtherNetIP_Connection_Paths_and_Routing.md, Backplane Port Segment Examples
        // Format: [0x01, slot] where:
        //   - 0x01 = Port Segment (8-bit link) for Port 1 (backplane)
        //   - slot = Slot number (0-255)
        // Examples: Slot 0 = [0x01, 0x00], Slot 1 = [0x01, 0x01], etc.
        for &slot in &self.slots {
            path.push(0x01); // Port Segment (8-bit link) for Port 1 (backplane)
            path.push(slot); // Slot number
        }

        // Add network hops
        for (i, address) in self.addresses.iter().enumerate() {
            if i < self.ports.len() {
                path.push(self.ports[i]); // Port number
            } else {
                path.push(0x01); // Default port
            }

            // Parse IP address and add to path
            if let Ok(ip) = address.parse::<std::net::Ipv4Addr>() {
                let octets = ip.octets();
                path.extend_from_slice(&octets);
            }
        }

        path
    }
}

impl Default for RoutePath {
    fn default() -> Self {
        Self::new()
    }
}
