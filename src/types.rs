use tokio::time::Instant;

pub use rust_ethernet_ip_types::{PlcValue, TypeError, UdtCodec, UdtData};

/// Connected session information for Class 3 explicit messaging.
#[derive(Debug, Clone)]
pub(crate) struct ConnectedSession {
    pub connection_id: u32,
    pub o_to_t_connection_id: u32,
    pub t_to_o_connection_id: u32,
    pub connection_serial: u16,
    pub originator_vendor_id: u16,
    pub originator_serial: u32,
    pub timeout_multiplier: u8,
    pub rpi: u32,
    pub o_to_t_params: ConnectionParameters,
    pub t_to_o_params: ConnectionParameters,
    #[allow(dead_code)]
    pub established_at: Instant,
    pub is_active: bool,
    pub sequence_count: u16,
}

/// Connection parameters for EtherNet/IP connections.
#[derive(Debug, Clone)]
pub(crate) struct ConnectionParameters {
    pub size: u16,
    pub connection_type: u8,
    pub priority: u8,
    pub variable_size: bool,
}

impl Default for ConnectionParameters {
    fn default() -> Self {
        Self {
            size: 500,
            connection_type: 0x02,
            priority: 0x01,
            variable_size: false,
        }
    }
}

impl ConnectedSession {
    pub fn new(connection_serial: u16) -> Self {
        Self {
            connection_id: 0,
            o_to_t_connection_id: 0,
            t_to_o_connection_id: 0,
            connection_serial,
            originator_vendor_id: 0x1337,
            originator_serial: 0x1234_5678,
            timeout_multiplier: 0x05,
            rpi: 100_000,
            o_to_t_params: ConnectionParameters::default(),
            t_to_o_params: ConnectionParameters::default(),
            established_at: Instant::now(),
            is_active: false,
            sequence_count: 0,
        }
    }

    pub fn with_config(connection_serial: u16, config_id: u8) -> Self {
        let mut session = Self::new(connection_serial);

        match config_id {
            1 => {
                session.timeout_multiplier = 0x07;
                session.rpi = 200_000;
                session.o_to_t_params.size = 504;
                session.t_to_o_params.size = 504;
                session.o_to_t_params.priority = 0x00;
                session.t_to_o_params.priority = 0x00;
            }
            2 => {
                session.timeout_multiplier = 0x03;
                session.rpi = 50000;
                session.o_to_t_params.size = 256;
                session.t_to_o_params.size = 256;
                session.o_to_t_params.priority = 0x02;
                session.t_to_o_params.priority = 0x02;
            }
            3 => {
                session.timeout_multiplier = 0x01;
                session.rpi = 1_000_000;
                session.o_to_t_params.size = 128;
                session.t_to_o_params.size = 128;
                session.o_to_t_params.priority = 0x03;
                session.t_to_o_params.priority = 0x03;
            }
            4 => {
                session.timeout_multiplier = 0x05;
                session.rpi = 100_000;
                session.o_to_t_params.size = 500;
                session.t_to_o_params.size = 500;
                session.o_to_t_params.connection_type = 0x01;
                session.t_to_o_params.connection_type = 0x01;
                session.originator_vendor_id = 0x001D;
            }
            5 => {
                session.timeout_multiplier = 0x0A;
                session.rpi = 500_000;
                session.o_to_t_params.size = 1024;
                session.t_to_o_params.size = 1024;
                session.o_to_t_params.variable_size = true;
                session.t_to_o_params.variable_size = true;
            }
            _ => {}
        }

        session
    }
}
