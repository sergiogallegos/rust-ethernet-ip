use super::EipClient;
use crate::error::{EtherNetIpError, Result};
use crate::subscription::{SubscriptionOptions, TagSubscription};
use crate::tag_group::{
    TagGroupConfig, TagGroupEvent, TagGroupEventKind, TagGroupFailureDiagnostic, TagGroupSnapshot,
    TagGroupSubscription, TagGroupValueResult,
};
use crate::types::PlcValue;

impl EipClient {
    /// Subscribes to a tag for real-time updates.
    ///
    /// The returned [`TagSubscription`] can be used to:
    /// - [`wait_for_update()`](TagSubscription::wait_for_update) for the next value
    /// - [`get_last_value()`](TagSubscription::get_last_value) for the latest cached value
    /// - [`into_stream()`](TagSubscription::into_stream) for an async `Stream` of updates
    ///
    /// This API validates the tag with an initial read before returning so invalid or
    /// inaccessible tags fail fast instead of surfacing only through background polling logs.
    ///
    /// The background poll loop uses [`SubscriptionOptions::update_rate`] (milliseconds) between reads.
    pub async fn subscribe_to_tag(
        &self,
        tag_path: &str,
        options: SubscriptionOptions,
    ) -> Result<TagSubscription> {
        let subscription = TagSubscription::new(tag_path.to_string(), options.clone());
        let mut validation_client = self.clone();
        let initial_value = validation_client.read_tag(tag_path).await?;
        subscription.update_value(&initial_value).await?;

        let mut subscriptions = self.subscriptions.lock().await;
        let update_rate_ms = options.update_rate;
        subscriptions.push(subscription.clone());
        drop(subscriptions);

        let tag_path = tag_path.to_string();
        let mut client = self.clone();
        tokio::spawn(async move {
            let interval = std::time::Duration::from_millis(update_rate_ms as u64);
            loop {
                match client.read_tag(&tag_path).await {
                    Ok(value) => {
                        if let Err(e) = client.update_subscription(&tag_path, &value).await {
                            tracing::error!("Error updating subscription: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error reading tag {}: {}", tag_path, e);
                        break;
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });
        Ok(subscription)
    }

    /// Subscribes to multiple tags. Returns one [`TagSubscription`] per tag in order.
    pub async fn subscribe_to_tags(
        &self,
        tags: &[(&str, SubscriptionOptions)],
    ) -> Result<Vec<TagSubscription>> {
        let mut subs = Vec::with_capacity(tags.len());
        for (tag_name, options) in tags {
            subs.push(self.subscribe_to_tag(tag_name, options.clone()).await?);
        }
        Ok(subs)
    }

    /// Registers or replaces a named tag group for grouped polling.
    ///
    /// Tag groups are useful for HMI/SCADA-style scan classes where multiple tags
    /// share a polling interval and should be read together.
    pub async fn upsert_tag_group(
        &self,
        group_name: &str,
        tags: &[&str],
        update_rate_ms: u32,
    ) -> Result<()> {
        if group_name.trim().is_empty() {
            return Err(EtherNetIpError::Protocol(
                "Tag group name cannot be empty".to_string(),
            ));
        }
        if tags.is_empty() {
            return Err(EtherNetIpError::Protocol(
                "Tag group must contain at least one tag".to_string(),
            ));
        }
        if update_rate_ms == 0 {
            return Err(EtherNetIpError::Protocol(
                "Tag group update rate must be greater than 0 ms".to_string(),
            ));
        }

        let config = TagGroupConfig {
            name: group_name.to_string(),
            tags: tags.iter().map(|s| (*s).to_string()).collect(),
            update_rate_ms,
        };

        let mut groups = self.tag_groups.lock().await;
        groups.insert(group_name.to_string(), config);
        Ok(())
    }

    /// Removes a named tag group.
    pub async fn remove_tag_group(&self, group_name: &str) -> bool {
        let mut groups = self.tag_groups.lock().await;
        groups.remove(group_name).is_some()
    }

    /// Lists all currently registered tag groups.
    pub async fn list_tag_groups(&self) -> Vec<TagGroupConfig> {
        let groups = self.tag_groups.lock().await;
        groups.values().cloned().collect()
    }

    /// Reads all tags in a group once and returns a snapshot.
    pub async fn read_tag_group_once(&self, group_name: &str) -> Result<TagGroupSnapshot> {
        let config = {
            let groups = self.tag_groups.lock().await;
            groups.get(group_name).cloned().ok_or_else(|| {
                EtherNetIpError::Protocol(format!("Tag group '{}' is not registered", group_name))
            })?
        };

        let mut client = self.clone();
        let tag_refs: Vec<&str> = config.tags.iter().map(String::as_str).collect();
        let values = client.read_tags_batch(&tag_refs).await?;

        let mapped = values
            .into_iter()
            .map(|(tag_name, result)| match result {
                Ok(value) => TagGroupValueResult {
                    tag_name,
                    value: Some(value),
                    error: None,
                },
                Err(e) => TagGroupValueResult {
                    tag_name,
                    value: None,
                    error: Some(e.to_string()),
                },
            })
            .collect();

        Ok(TagGroupSnapshot {
            group_name: config.name,
            sampled_at: std::time::SystemTime::now(),
            values: mapped,
        })
    }

    /// Starts background polling for a registered tag group.
    ///
    /// Use the returned subscription to await snapshots and to stop polling.
    pub async fn subscribe_tag_group(&self, group_name: &str) -> Result<TagGroupSubscription> {
        let config = {
            let groups = self.tag_groups.lock().await;
            groups.get(group_name).cloned().ok_or_else(|| {
                EtherNetIpError::Protocol(format!("Tag group '{}' is not registered", group_name))
            })?
        };

        let subscription = TagGroupSubscription::new(config.name.clone(), config.update_rate_ms);
        let subscription_task = subscription.clone();
        let mut client = self.clone();
        let tags = config.tags.clone();
        let interval = std::time::Duration::from_millis(config.update_rate_ms as u64);
        let group_name_owned = config.name.clone();

        tokio::spawn(async move {
            while subscription_task.is_active() {
                let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
                match client.read_tags_batch(&tag_refs).await {
                    Ok(values) => {
                        let has_errors = values.iter().any(|(_, result)| result.is_err());
                        let snapshot = TagGroupSnapshot {
                            group_name: group_name_owned.clone(),
                            sampled_at: std::time::SystemTime::now(),
                            values: values
                                .into_iter()
                                .map(|(tag_name, result)| match result {
                                    Ok(value) => TagGroupValueResult {
                                        tag_name,
                                        value: Some(value),
                                        error: None,
                                    },
                                    Err(e) => TagGroupValueResult {
                                        tag_name,
                                        value: None,
                                        error: Some(e.to_string()),
                                    },
                                })
                                .collect(),
                        };

                        let event = TagGroupEvent {
                            kind: if has_errors {
                                TagGroupEventKind::PartialError
                            } else {
                                TagGroupEventKind::Data
                            },
                            snapshot,
                            error: None,
                            failure: None,
                        };

                        if let Err(e) = subscription_task.publish_event(event).await {
                            tracing::error!(
                                "Tag group '{}' publish failed: {}",
                                group_name_owned,
                                e
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Tag group '{}' polling read failed: {}",
                            group_name_owned,
                            e
                        );
                        let failure_event = TagGroupEvent {
                            kind: TagGroupEventKind::ReadFailure,
                            snapshot: TagGroupSnapshot {
                                group_name: group_name_owned.clone(),
                                sampled_at: std::time::SystemTime::now(),
                                values: Vec::new(),
                            },
                            error: Some(e.to_string()),
                            failure: Some(TagGroupFailureDiagnostic::from_error(&e)),
                        };
                        if let Err(publish_error) =
                            subscription_task.publish_event(failure_event).await
                        {
                            tracing::error!(
                                "Tag group '{}' failure-event publish failed: {}",
                                group_name_owned,
                                publish_error
                            );
                            break;
                        }
                    }
                }
                tokio::time::sleep(interval).await;
            }
        });

        Ok(subscription)
    }

    async fn update_subscription(&self, tag_name: &str, value: &PlcValue) -> Result<()> {
        let subscriptions = {
            let subscriptions = self.subscriptions.lock().await;
            subscriptions.clone()
        };
        for subscription in &subscriptions {
            if subscription.tag_path == tag_name && subscription.is_active() {
                subscription.update_value(value).await?;
            }
        }
        Ok(())
    }
}
