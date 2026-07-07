use super::EipClient;
use crate::error::EtherNetIpError;

const STRING_COMPAT_REASON: &str = "this exploratory STRING write path never had a valid, atomic wire contract; use write_tag(..., PlcValue::String(...)) or write_string_tag instead; removal is planned for 2.0";
const STRING_CONNECTED_REASON: &str = "the retired Class 3 connected STRING subsystem parsed Forward Open and connected replies at the wrong layer; use write_tag(..., PlcValue::String(...)) or write_string_tag instead; removal is planned for 2.0";

fn unsupported(api: &'static str, reason: &'static str) -> crate::error::Result<()> {
    Err(EtherNetIpError::Unsupported { api, reason })
}

impl EipClient {
    /// Retired non-atomic AB STRING component writer.
    #[deprecated(
        since = "1.2.0",
        note = "never provided a safe atomic STRING write contract; use write_tag(..., PlcValue::String(...)) or write_string_tag; removal planned for 2.0"
    )]
    pub async fn write_ab_string_components(
        &mut self,
        _tag_name: &str,
        _value: &str,
    ) -> crate::error::Result<()> {
        unsupported("write_ab_string_components", STRING_COMPAT_REASON)
    }

    /// Retired malformed AB STRING-as-UDT writer.
    #[deprecated(
        since = "1.2.0",
        note = "never parsed the PLC reply correctly and could report false success; use write_tag(..., PlcValue::String(...)) or write_string_tag; removal planned for 2.0"
    )]
    pub async fn write_ab_string_udt(
        &mut self,
        _tag_name: &str,
        _value: &str,
    ) -> crate::error::Result<()> {
        unsupported(
            "write_ab_string_udt",
            "this exploratory STRING-as-UDT path parsed the raw CPF envelope as a CIP reply and could report false success; use write_tag(..., PlcValue::String(...)) or write_string_tag instead; removal is planned for 2.0",
        )
    }

    /// Retired connected explicit-messaging STRING writer.
    #[deprecated(
        since = "1.2.0",
        note = "connected STRING messaging never established a valid Forward Open path; use write_tag(..., PlcValue::String(...)) or write_string_tag; removal planned for 2.0"
    )]
    pub async fn write_string_connected(
        &mut self,
        _tag_name: &str,
        _value: &str,
    ) -> crate::error::Result<()> {
        unsupported("write_string_connected", STRING_CONNECTED_REASON)
    }

    /// Retired reverse-engineered unconnected STRING writer.
    #[deprecated(
        since = "1.2.0",
        note = "malformed exploratory STRING payload; use write_tag(..., PlcValue::String(...)) or write_string_tag; removal planned for 2.0"
    )]
    pub async fn write_string_unconnected(
        &mut self,
        _tag_name: &str,
        _value: &str,
    ) -> crate::error::Result<()> {
        unsupported(
            "write_string_unconnected",
            "this exploratory unconnected STRING payload does not match the Logix Write Tag structure format; use write_tag(..., PlcValue::String(...)) or write_string_tag instead; removal is planned for 2.0",
        )
    }

    /// Retired legacy STRING writer.
    ///
    /// Use [`EipClient::write_tag`] with [`crate::PlcValue::String`] or
    /// [`EipClient::write_string_tag`]. Those paths use the hardware-validated
    /// standard Logix STRING structure encoding.
    #[deprecated(
        since = "1.2.0",
        note = "legacy malformed STRING writer; use write_tag(..., PlcValue::String(...)) or write_string_tag; removal planned for 2.0"
    )]
    pub async fn write_string(
        &mut self,
        _tag_name: &str,
        _value: &str,
    ) -> crate::error::Result<()> {
        unsupported(
            "write_string",
            "the legacy writer omitted the request path-size byte and read the service-reply byte as status; use write_tag(..., PlcValue::String(...)) or write_string_tag instead; removal is planned for 2.0",
        )
    }
}
