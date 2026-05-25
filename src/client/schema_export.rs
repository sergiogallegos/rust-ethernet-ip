use super::EipClient;
use crate::schema::{self, SchemaExport};

impl EipClient {
    /// Exports a stable schema view built from the current discovery APIs.
    pub async fn export_schema(&mut self) -> crate::error::Result<SchemaExport> {
        let (discovered_tags, discovery_warnings) =
            self.discover_tags_detailed_internal(true).await?;
        let route_path = self.route_path_snapshot();
        let mut schema = SchemaExport::new(route_path.as_ref());
        schema.warnings.extend(discovery_warnings);

        let mut tags = discovered_tags;
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        schema.tags = tags.iter().map(schema::SchemaTag::from).collect();

        let mut udt_sources: std::collections::HashMap<u32, (&str, u32)> =
            std::collections::HashMap::new();
        for tag in &tags {
            if tag.data_type == 0x00A0
                && let Some(template_instance_id) = tag.template_instance_id
            {
                udt_sources
                    .entry(template_instance_id)
                    .or_insert((tag.name.as_str(), tag.size));
            }
        }

        let mut template_ids: Vec<u32> = udt_sources.keys().copied().collect();
        template_ids.sort_unstable();

        for template_id in template_ids {
            let Some((tag_name, size_bytes)) = udt_sources.get(&template_id).copied() else {
                continue;
            };

            match self
                .get_udt_definition_by_template_id(template_id, tag_name)
                .await
            {
                Ok((definition, structure_size_bytes)) => {
                    schema.udts.push(schema::SchemaUdt::from_definition(
                        &definition,
                        Some(template_id),
                        if size_bytes == 0 {
                            structure_size_bytes
                        } else {
                            size_bytes
                        },
                    ));
                }
                Err(err) => schema.warnings.push(format!(
                    "Failed to resolve UDT definition for tag '{}' (template {}): {}",
                    tag_name, template_id, err
                )),
            }
        }

        schema.udts.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(schema)
    }

    /// Exports the current schema view as pretty-printed JSON.
    pub async fn export_schema_json(&mut self) -> crate::error::Result<String> {
        let schema = self.export_schema().await?;
        serde_json::to_string_pretty(&schema).map_err(|err| {
            crate::error::EtherNetIpError::Protocol(format!(
                "Failed to serialize schema export to JSON: {}",
                err
            ))
        })
    }
}
