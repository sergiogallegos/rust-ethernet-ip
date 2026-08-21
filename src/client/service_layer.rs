use super::EipClient;
use crate::error::{EtherNetIpError, Result};
use crate::types::{PlcValue, UdtData};
use crate::udt::UserDefinedType;
use std::future::Future;
use std::pin::Pin;

type WriteFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

trait MemberWriteStrategy {
    fn write_direct_member<'a>(
        &'a mut self,
        member_path: &'a str,
        value: PlcValue,
    ) -> WriteFuture<'a>;

    fn write_member_via_full_value<'a>(
        &'a mut self,
        udt_tag_name: &'a str,
        member_name: &'a str,
        value: PlcValue,
    ) -> WriteFuture<'a>;
}

impl MemberWriteStrategy for EipClient {
    fn write_direct_member<'a>(
        &'a mut self,
        member_path: &'a str,
        value: PlcValue,
    ) -> WriteFuture<'a> {
        Box::pin(async move { self.write_tag(member_path, value).await })
    }

    fn write_member_via_full_value<'a>(
        &'a mut self,
        udt_tag_name: &'a str,
        member_name: &'a str,
        value: PlcValue,
    ) -> WriteFuture<'a> {
        Box::pin(async move {
            self.write_udt_member_via_full_value(udt_tag_name, member_name, value)
                .await
        })
    }
}

impl EipClient {
    /// Writes a built-in or custom Logix string using its discovered structure handle.
    pub async fn write_string_tag(&mut self, tag_name: &str, value: &str) -> Result<()> {
        self.write_tag(tag_name, PlcValue::String(value.to_string()))
            .await
    }

    /// Reads a Logix STRING tag as text, whether it is the built-in `STRING` type or a custom
    /// string type (own name/length). `read_tag` decodes only the built-in handle (0x0FCE) to
    /// `PlcValue::String` and returns any other structure as `PlcValue::Udt`; here the caller has
    /// asserted the tag is a string, so a structure payload is decoded as `[handle][LEN][DATA]`.
    pub async fn read_string_tag(&mut self, tag_name: &str) -> Result<String> {
        match self.read_tag(tag_name).await? {
            PlcValue::String(value) => Ok(value),
            PlcValue::Udt(udt) if udt.data.len() >= 6 => {
                let len = u32::from_le_bytes([udt.data[2], udt.data[3], udt.data[4], udt.data[5]])
                    as usize;
                let end = (6 + len).min(udt.data.len());
                Ok(String::from_utf8_lossy(&udt.data[6..end]).to_string())
            }
            other => Err(EtherNetIpError::DataTypeMismatch {
                expected: "STRING".to_string(),
                actual: format!("{other:?}"),
            }),
        }
    }

    /// Writes one named member of a UDT tag.
    pub async fn write_udt_member(
        &mut self,
        udt_tag_name: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<()> {
        write_udt_member_direct_first(self, udt_tag_name, member_name, value).await
    }

    /// Writes one named member of a UDT array element.
    pub async fn write_udt_array_member(
        &mut self,
        udt_array_element_path: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<()> {
        write_udt_member_direct_first(self, udt_array_element_path, member_name, value).await
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

async fn write_udt_member_direct_first<T: MemberWriteStrategy>(
    strategy: &mut T,
    udt_tag_name: &str,
    member_name: &str,
    value: PlcValue,
) -> Result<()> {
    if matches!(value, PlcValue::String(_)) {
        return strategy
            .write_member_via_full_value(udt_tag_name, member_name, value)
            .await;
    }

    let member_path = format!("{udt_tag_name}.{member_name}");
    match strategy
        .write_direct_member(&member_path, value.clone())
        .await
    {
        Ok(()) => Ok(()),
        Err(error) if is_2107_type_mismatch(&error) => {
            strategy
                .write_member_via_full_value(udt_tag_name, member_name, value)
                .await
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn is_2107_type_mismatch(error: &EtherNetIpError) -> bool {
    error.to_string().to_ascii_lowercase().contains("0x2107")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future;

    #[derive(Default)]
    struct FakeMemberWriteStrategy {
        direct_error: Option<EtherNetIpError>,
        direct_paths: Vec<String>,
        rmw_calls: Vec<(String, String, PlcValue)>,
    }

    impl MemberWriteStrategy for FakeMemberWriteStrategy {
        fn write_direct_member<'a>(
            &'a mut self,
            member_path: &'a str,
            _value: PlcValue,
        ) -> WriteFuture<'a> {
            self.direct_paths.push(member_path.to_string());
            Box::pin(future::ready(match self.direct_error.take() {
                Some(error) => Err(error),
                None => Ok(()),
            }))
        }

        fn write_member_via_full_value<'a>(
            &'a mut self,
            udt_tag_name: &'a str,
            member_name: &'a str,
            value: PlcValue,
        ) -> WriteFuture<'a> {
            self.rmw_calls
                .push((udt_tag_name.to_string(), member_name.to_string(), value));
            Box::pin(future::ready(Ok(())))
        }
    }

    #[tokio::test]
    async fn scalar_member_write_falls_back_to_rmw_on_2107() {
        let mut strategy = FakeMemberWriteStrategy {
            direct_error: Some(EtherNetIpError::Protocol(
                "CIP Extended Error: Read/Write Tag data-type mismatch extended error: 0x2107"
                    .to_string(),
            )),
            ..Default::default()
        };

        write_udt_member_direct_first(
            &mut strategy,
            "UDT_ARRAY[3]",
            "DINT_MEMBER",
            PlcValue::Dint(77),
        )
        .await
        .expect("fallback should succeed");

        assert_eq!(strategy.direct_paths, vec!["UDT_ARRAY[3].DINT_MEMBER"]);
        assert_eq!(strategy.rmw_calls.len(), 1);
        assert_eq!(strategy.rmw_calls[0].0, "UDT_ARRAY[3]");
        assert_eq!(strategy.rmw_calls[0].1, "DINT_MEMBER");
        assert_eq!(strategy.rmw_calls[0].2, PlcValue::Dint(77));
    }

    #[tokio::test]
    async fn string_member_write_uses_rmw_without_direct_attempt() {
        let mut strategy = FakeMemberWriteStrategy {
            direct_error: Some(EtherNetIpError::Protocol(
                "direct write should not be called".to_string(),
            )),
            ..Default::default()
        };

        write_udt_member_direct_first(
            &mut strategy,
            "gTestUDT",
            "Member5_String",
            PlcValue::String("updated".to_string()),
        )
        .await
        .expect("string RMW should succeed");

        assert!(strategy.direct_paths.is_empty());
        assert_eq!(strategy.rmw_calls.len(), 1);
        assert_eq!(strategy.rmw_calls[0].0, "gTestUDT");
        assert_eq!(strategy.rmw_calls[0].1, "Member5_String");
        assert_eq!(
            strategy.rmw_calls[0].2,
            PlcValue::String("updated".to_string())
        );
    }

    #[tokio::test]
    async fn non_2107_direct_error_does_not_fallback() {
        let mut strategy = FakeMemberWriteStrategy {
            direct_error: Some(EtherNetIpError::Protocol(
                "CIP Error 0x04: Path segment error".to_string(),
            )),
            ..Default::default()
        };

        let error = write_udt_member_direct_first(
            &mut strategy,
            "UDT_ARRAY[3]",
            "DINT_MEMBER",
            PlcValue::Dint(77),
        )
        .await
        .expect_err("non-2107 error should be returned");

        assert!(error.to_string().contains("0x04"));
        assert_eq!(strategy.direct_paths, vec!["UDT_ARRAY[3].DINT_MEMBER"]);
        assert!(strategy.rmw_calls.is_empty());
    }
}
