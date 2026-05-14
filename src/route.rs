/// Ordered route hop for PLC communication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteHop {
    /// Backplane/chassis hop. Rockwell ControlLogix backplanes normally use port 1.
    Backplane { port: u8, slot: u8 },
    /// Ethernet hop using an IPv4 link address. Rockwell Ethernet ports commonly use port 2.
    Ethernet { port: u8, address: String },
}

/// Route path for PLC communication.
///
/// The `slots`, `ports`, and `addresses` fields are retained for compatibility with
/// the original public API and wrappers. New code should prefer [`RoutePath::hops`]
/// or the explicit hop builders because CIP routing is ordered. If `hops` is empty,
/// encoding falls back to the legacy grouped-field order for compatibility.
#[derive(Debug, Clone)]
pub struct RoutePath {
    pub slots: Vec<u8>,
    pub ports: Vec<u8>,
    pub addresses: Vec<String>,
    pub hops: Vec<RouteHop>,
}

impl RoutePath {
    const DEFAULT_BACKPLANE_PORT: u8 = 1;
    const DEFAULT_ETHERNET_PORT: u8 = 2;

    /// Creates a new route path
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            ports: Vec::new(),
            addresses: Vec::new(),
            hops: Vec::new(),
        }
    }

    /// Adds a backplane slot to the route
    #[must_use]
    pub fn add_slot(mut self, slot: u8) -> Self {
        self.slots.push(slot);
        self.hops.push(RouteHop::Backplane {
            port: Self::DEFAULT_BACKPLANE_PORT,
            slot,
        });
        self
    }

    /// Adds a network port to the route
    #[must_use]
    pub fn add_port(mut self, port: u8) -> Self {
        let port_index = self.ports.len();
        self.ports.push(port);
        self.update_ethernet_hop_port(port_index, port);
        self
    }

    /// Adds a network address to the route
    #[must_use]
    pub fn add_address(mut self, address: String) -> Self {
        let port = self
            .ports
            .get(self.addresses.len())
            .copied()
            .unwrap_or(Self::DEFAULT_ETHERNET_PORT);
        self.addresses.push(address.clone());
        self.hops.push(RouteHop::Ethernet { port, address });
        self
    }

    /// Adds a backplane hop with an explicit port number.
    #[must_use]
    pub fn add_backplane(mut self, port: u8, slot: u8) -> Self {
        self.slots.push(slot);
        self.hops.push(RouteHop::Backplane { port, slot });
        self
    }

    /// Adds an Ethernet hop using the common Rockwell Ethernet port number, 2.
    #[must_use]
    pub fn add_ethernet(self, address: impl Into<String>) -> Self {
        self.add_ethernet_with_port(Self::DEFAULT_ETHERNET_PORT, address)
    }

    /// Adds an Ethernet hop with an explicit port number.
    #[must_use]
    pub fn add_ethernet_with_port(mut self, port: u8, address: impl Into<String>) -> Self {
        let address = address.into();
        self.ports.push(port);
        self.addresses.push(address.clone());
        self.hops.push(RouteHop::Ethernet { port, address });
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
        if self.hops.is_empty() {
            return self.legacy_grouped_fields_to_cip_bytes();
        }

        let mut path = Vec::new();

        for hop in &self.hops {
            Self::append_hop(&mut path, hop);
        }

        path
    }

    fn append_hop(path: &mut Vec<u8>, hop: &RouteHop) {
        match hop {
            RouteHop::Backplane { port, slot } => {
                path.push(*port);
                path.push(*slot);
            }
            RouteHop::Ethernet { port, address } => {
                Self::append_extended_link_address_segment(path, *port, address);
            }
        }
    }

    fn append_extended_link_address_segment(path: &mut Vec<u8>, port: u8, address: &str) {
        path.push(0x10 | (port & 0x0F));
        path.push(address.len().saturating_add(1) as u8);
        path.extend_from_slice(address.as_bytes());
        path.push(0x00);
        if !(address.len() + 1).is_multiple_of(2) {
            path.push(0x00);
        }
    }

    fn legacy_grouped_fields_to_cip_bytes(&self) -> Vec<u8> {
        let mut path = Vec::new();

        for &slot in &self.slots {
            Self::append_hop(
                &mut path,
                &RouteHop::Backplane {
                    port: Self::DEFAULT_BACKPLANE_PORT,
                    slot,
                },
            );
        }

        for (i, address) in self.addresses.iter().enumerate() {
            let port = self
                .ports
                .get(i)
                .copied()
                .unwrap_or(Self::DEFAULT_ETHERNET_PORT);
            Self::append_hop(
                &mut path,
                &RouteHop::Ethernet {
                    port,
                    address: address.clone(),
                },
            );
        }

        path
    }

    fn update_ethernet_hop_port(&mut self, port_index: usize, port: u8) -> bool {
        if let Some(RouteHop::Ethernet { port: hop_port, .. }) = self
            .hops
            .iter_mut()
            .filter(|hop| matches!(hop, RouteHop::Ethernet { .. }))
            .nth(port_index)
        {
            *hop_port = port;
            true
        } else {
            false
        }
    }
}

impl Default for RoutePath {
    fn default() -> Self {
        Self::new()
    }
}
