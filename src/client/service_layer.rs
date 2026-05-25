use super::EipClient;
use crate::error::{EtherNetIpError, Result};
use crate::types::{PlcValue, UdtData};
use crate::udt::UserDefinedType;

impl EipClient {
    pub async fn write_string_tag(&mut self, tag_name: &str, value: &str) -> Result<()> {
        self.write_tag(tag_name, PlcValue::String(value.to_string()))
            .await
    }

    pub async fn write_udt_member(
        &mut self,
        udt_tag_name: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<()> {
        self.write_udt_member_via_full_value(udt_tag_name, member_name, value)
            .await
    }

    pub async fn write_udt_array_member(
        &mut self,
        udt_array_element_path: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<()> {
        self.write_udt_member_via_full_value(udt_array_element_path, member_name, value)
            .await
    }

    async fn write_udt_member_via_full_value(
        &mut self,
        udt_tag_name: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<()> {
        let current = self.read_tag(udt_tag_name).await?;
        let PlcValue::Udt(udt_data) = current else {
            return Err(EtherNetIpError::DataTypeMismatch {
                expected: "UDT".to_string(),
                actual: format!("{current:?}"),
            });
        };

        let definition = self.get_udt_definition(udt_tag_name).await?;
        let mut user_def = UserDefinedType::new(definition.name.clone());
        for member in &definition.members {
            user_def.add_member(member.clone());
        }

        let mut members = udt_data.parse(&user_def)?;
        if !members.contains_key(member_name) {
            return Err(EtherNetIpError::TagNotFound(format!(
                "{udt_tag_name}.{member_name}"
            )));
        }

        members.insert(member_name.to_string(), value);
        let modified = UdtData::from_hash_map(&members, &user_def, udt_data.symbol_id)?;
        self.write_tag(udt_tag_name, PlcValue::Udt(modified)).await
    }
}
