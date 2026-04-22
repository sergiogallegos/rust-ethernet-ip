use crate::PlcValue;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tokio::sync::{Mutex, mpsc};

/// Defines a named tag group with a polling interval.
#[derive(Debug, Clone)]
pub struct TagGroupConfig {
    pub name: String,
    pub tags: Vec<String>,
    pub update_rate_ms: u32,
}

/// Per-tag result in a group snapshot.
#[derive(Debug, Clone)]
pub struct TagGroupValueResult {
    pub tag_name: String,
    pub value: Option<PlcValue>,
    pub error: Option<String>,
}

/// Snapshot of one polling cycle for a group.
#[derive(Debug, Clone)]
pub struct TagGroupSnapshot {
    pub group_name: String,
    pub sampled_at: SystemTime,
    pub values: Vec<TagGroupValueResult>,
}

/// High-level classification for tag-group polling events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagGroupEventKind {
    Data,
    PartialError,
    ReadFailure,
}

/// Structured category for tag-group polling failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagGroupFailureCategory {
    Network,
    Timeout,
    PlcStatus,
    Protocol,
    Permission,
    Tag,
    Data,
    Other,
}

/// Structured diagnostics for read failures during tag-group polling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagGroupFailureDiagnostic {
    pub category: TagGroupFailureCategory,
    pub retriable: bool,
    pub status_code: Option<u8>,
}

impl TagGroupFailureDiagnostic {
    pub fn from_error(error: &crate::EtherNetIpError) -> Self {
        use crate::EtherNetIpError;

        let (category, status_code) = match error {
            EtherNetIpError::Timeout(_) => (TagGroupFailureCategory::Timeout, None),
            EtherNetIpError::Io(_)
            | EtherNetIpError::Connection(_)
            | EtherNetIpError::ConnectionLost(_) => (TagGroupFailureCategory::Network, None),
            EtherNetIpError::CipError { code, .. } => {
                (TagGroupFailureCategory::PlcStatus, Some(*code))
            }
            EtherNetIpError::ReadError { status, .. }
            | EtherNetIpError::WriteError { status, .. }
            | EtherNetIpError::StringReadError { status, .. }
            | EtherNetIpError::StringWriteError { status, .. } => {
                (TagGroupFailureCategory::PlcStatus, Some(*status))
            }
            EtherNetIpError::Permission(_) => (TagGroupFailureCategory::Permission, None),
            EtherNetIpError::TagNotFound(_) | EtherNetIpError::Tag(_) => {
                (TagGroupFailureCategory::Tag, None)
            }
            EtherNetIpError::DataTypeMismatch { .. }
            | EtherNetIpError::Udt(_)
            | EtherNetIpError::StringTooLong { .. }
            | EtherNetIpError::InvalidString { .. } => (TagGroupFailureCategory::Data, None),
            EtherNetIpError::Protocol(_)
            | EtherNetIpError::InvalidResponse { .. }
            | EtherNetIpError::InvalidStringResponse { .. }
            | EtherNetIpError::Subscription(_)
            | EtherNetIpError::Utf8(_) => (TagGroupFailureCategory::Protocol, None),
            EtherNetIpError::Other(_) => (TagGroupFailureCategory::Other, None),
        };

        Self {
            category,
            retriable: error.is_retriable(),
            status_code,
        }
    }
}

/// Event emitted by background tag-group polling.
#[derive(Debug, Clone)]
pub struct TagGroupEvent {
    pub kind: TagGroupEventKind,
    pub snapshot: TagGroupSnapshot,
    pub error: Option<String>,
    pub failure: Option<TagGroupFailureDiagnostic>,
}

/// Live subscription to a tag group polling stream.
#[derive(Debug, Clone)]
pub struct TagGroupSubscription {
    pub group_name: String,
    pub update_rate_ms: u32,
    is_active: Arc<AtomicBool>,
    sender: Arc<Mutex<mpsc::Sender<TagGroupEvent>>>,
    receiver: Arc<Mutex<mpsc::Receiver<TagGroupEvent>>>,
}

impl TagGroupSubscription {
    pub fn new(group_name: String, update_rate_ms: u32) -> Self {
        let (sender, receiver) = mpsc::channel(64);
        Self {
            group_name,
            update_rate_ms,
            is_active: Arc::new(AtomicBool::new(true)),
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.is_active.store(false, Ordering::Relaxed);
    }

    pub async fn publish(&self, snapshot: TagGroupSnapshot) -> Result<(), String> {
        let event = TagGroupEvent {
            kind: if snapshot.values.iter().any(|v| v.error.is_some()) {
                TagGroupEventKind::PartialError
            } else {
                TagGroupEventKind::Data
            },
            snapshot,
            error: None,
            failure: None,
        };
        self.publish_event(event).await
    }

    pub async fn publish_event(&self, event: TagGroupEvent) -> Result<(), String> {
        let sender = {
            let sender = self.sender.lock().await;
            sender.clone()
        };
        let send_result = sender.send(event).await;
        send_result.map_err(|e| e.to_string())
    }

    pub async fn wait_for_update(&self) -> Option<TagGroupEvent> {
        let mut receiver = self.receiver.lock().await;
        let next_event = receiver.recv().await;
        drop(receiver);
        next_event
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EtherNetIpError;

    #[test]
    fn maps_cip_status_failure_diagnostic() {
        let diagnostic = TagGroupFailureDiagnostic::from_error(&EtherNetIpError::CipError {
            code: 0x05,
            message: "Path destination unknown".to_string(),
        });
        assert_eq!(diagnostic.category, TagGroupFailureCategory::PlcStatus);
        assert_eq!(diagnostic.status_code, Some(0x05));
        assert!(!diagnostic.retriable);
    }

    #[test]
    fn maps_timeout_failure_diagnostic_as_retriable() {
        let diagnostic = TagGroupFailureDiagnostic::from_error(&EtherNetIpError::Timeout(
            std::time::Duration::from_secs(2),
        ));
        assert_eq!(diagnostic.category, TagGroupFailureCategory::Timeout);
        assert_eq!(diagnostic.status_code, None);
        assert!(diagnostic.retriable);
    }

    #[tokio::test]
    async fn publish_assigns_partial_error_kind() {
        let sub = TagGroupSubscription::new("group".to_string(), 100);
        let snapshot = TagGroupSnapshot {
            group_name: "group".to_string(),
            sampled_at: SystemTime::now(),
            values: vec![TagGroupValueResult {
                tag_name: "Tag1".to_string(),
                value: None,
                error: Some("Read failed".to_string()),
            }],
        };

        sub.publish(snapshot).await.expect("publish should succeed");
        let event = sub.wait_for_update().await.expect("event should exist");

        assert_eq!(event.kind, TagGroupEventKind::PartialError);
        assert!(event.error.is_none());
        assert!(event.failure.is_none());
    }

    #[tokio::test]
    async fn publish_assigns_data_kind_when_all_values_are_ok() {
        let sub = TagGroupSubscription::new("group".to_string(), 100);
        let snapshot = TagGroupSnapshot {
            group_name: "group".to_string(),
            sampled_at: SystemTime::now(),
            values: vec![TagGroupValueResult {
                tag_name: "Tag1".to_string(),
                value: Some(crate::PlcValue::Dint(42)),
                error: None,
            }],
        };

        sub.publish(snapshot).await.expect("publish should succeed");
        let event = sub.wait_for_update().await.expect("event should exist");

        assert_eq!(event.kind, TagGroupEventKind::Data);
        assert!(event.error.is_none());
        assert!(event.failure.is_none());
    }

    #[tokio::test]
    async fn publish_event_preserves_read_failure_diagnostics() {
        let sub = TagGroupSubscription::new("group".to_string(), 100);
        let event = TagGroupEvent {
            kind: TagGroupEventKind::ReadFailure,
            snapshot: TagGroupSnapshot {
                group_name: "group".to_string(),
                sampled_at: SystemTime::now(),
                values: Vec::new(),
            },
            error: Some("timeout while reading tag group".to_string()),
            failure: Some(TagGroupFailureDiagnostic {
                category: TagGroupFailureCategory::Timeout,
                retriable: true,
                status_code: None,
            }),
        };

        sub.publish_event(event.clone())
            .await
            .expect("publish_event should succeed");
        let received = sub.wait_for_update().await.expect("event should exist");

        assert_eq!(received.kind, TagGroupEventKind::ReadFailure);
        assert_eq!(received.error, event.error);
        assert_eq!(received.failure, event.failure);
    }

    #[test]
    fn stop_marks_subscription_inactive() {
        let sub = TagGroupSubscription::new("group".to_string(), 100);
        assert!(sub.is_active());
        sub.stop();
        assert!(!sub.is_active());
    }
}
