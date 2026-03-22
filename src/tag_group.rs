use crate::PlcValue;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, Mutex};

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

/// Event emitted by background tag-group polling.
#[derive(Debug, Clone)]
pub struct TagGroupEvent {
    pub kind: TagGroupEventKind,
    pub snapshot: TagGroupSnapshot,
    pub error: Option<String>,
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
        };
        self.publish_event(event).await
    }

    pub async fn publish_event(&self, event: TagGroupEvent) -> Result<(), String> {
        let sender = self.sender.lock().await;
        sender.send(event).await.map_err(|e| e.to_string())
    }

    pub async fn wait_for_update(&self) -> Option<TagGroupEvent> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }
}
