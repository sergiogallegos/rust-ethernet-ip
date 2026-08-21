use crate::PlcValue;
use crate::error::{EtherNetIpError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::{LazyLock, Mutex as StdMutex};
use tokio::sync::{Mutex, mpsc};

use futures::{Stream, stream};

/// Configuration options for tag subscriptions
#[derive(Debug, Clone)]
pub struct SubscriptionOptions {
    /// Update rate in milliseconds
    pub update_rate: u32,
    /// Absolute change threshold (deadband) applied to floating-point values: a
    /// REAL/LREAL update notifies only when `|new - old| >= change_threshold`.
    /// This is an absolute delta, not a percentage. Non-float types notify on
    /// any change regardless of this value.
    pub change_threshold: f32,
    /// Timeout in milliseconds
    pub timeout: u32,
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self {
            update_rate: 100,        // 100ms default update rate
            change_threshold: 0.001, // absolute deadband for REAL/LREAL
            timeout: 5000,           // 5 second timeout
        }
    }
}

/// Represents a subscription to a PLC tag
#[derive(Debug, Clone)]
pub struct TagSubscription {
    /// The path of the subscribed tag
    pub tag_path: String,
    /// Subscription configuration
    pub options: SubscriptionOptions,
    /// Last received value
    pub last_value: Arc<Mutex<Option<PlcValue>>>,
    /// Channel sender for value updates
    pub sender: Arc<Mutex<mpsc::Sender<PlcValue>>>,
    /// Channel receiver for value updates
    pub receiver: Arc<Mutex<mpsc::Receiver<PlcValue>>>,
    /// Whether the subscription is active
    pub is_active: Arc<AtomicBool>,
}

/// Event emitted by a tag subscription poll loop.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TagSubscriptionEvent {
    /// A value update passed the subscription's deadband and was published.
    Value(PlcValue),
    /// A polling error occurred. `terminal` means the poll loop stopped.
    Error {
        /// Human-readable polling failure.
        message: String,
        /// Whether the subscription stopped after this error.
        terminal: bool,
    },
}

#[derive(Clone)]
struct TagSubscriptionEventChannels {
    sender: Arc<Mutex<mpsc::Sender<TagSubscriptionEvent>>>,
    receiver: Arc<Mutex<mpsc::Receiver<TagSubscriptionEvent>>>,
}

static TAG_SUBSCRIPTION_EVENTS: LazyLock<StdMutex<HashMap<usize, TagSubscriptionEventChannels>>> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

impl TagSubscription {
    /// Creates a new tag subscription
    pub fn new(tag_name: String, options: SubscriptionOptions) -> Self {
        let (sender, receiver) = mpsc::channel(100); // Buffer size of 100
        let subscription = Self {
            tag_path: tag_name,
            options,
            last_value: Arc::new(Mutex::new(None)),
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
            is_active: Arc::new(AtomicBool::new(true)),
        };
        subscription.event_channels();
        subscription
    }

    /// Checks if the subscription is active
    pub fn is_active(&self) -> bool {
        self.is_active.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Stops the subscription
    pub fn stop(&self) {
        self.is_active
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Updates the subscription value.
    ///
    /// Update delivery is nonblocking. If a consumer falls behind and the
    /// bounded channel is full, the oldest queued item is dropped where possible
    /// so the background poll task can keep running.
    pub async fn update_value(&self, value: &PlcValue) -> Result<()> {
        let mut last_value = self.last_value.lock().await;

        // Check if value has changed enough to notify
        if let Some(old) = last_value.as_ref()
            && !Self::value_changed(old, value, self.options.change_threshold)
        {
            return Ok(());
        }

        // Update value and send notification
        *last_value = Some(value.clone());
        drop(last_value);
        try_send_drop_oldest(&self.sender, &self.receiver, value.clone())
            .await
            .map_err(|e| EtherNetIpError::Subscription(format!("Failed to send update: {e}")))?;
        try_send_drop_oldest(
            &self.event_channels().sender,
            &self.event_channels().receiver,
            TagSubscriptionEvent::Value(value.clone()),
        )
        .await
        .map_err(|e| EtherNetIpError::Subscription(format!("Failed to send event: {e}")))?;

        Ok(())
    }

    /// Publishes a polling error without blocking the poll task.
    ///
    /// If the event buffer is full, the oldest queued event is dropped where
    /// possible. If `terminal` is true, the subscription is marked inactive.
    pub async fn publish_error(&self, error: &EtherNetIpError, terminal: bool) -> Result<()> {
        if terminal {
            self.stop();
        }

        try_send_drop_oldest(
            &self.event_channels().sender,
            &self.event_channels().receiver,
            TagSubscriptionEvent::Error {
                message: error.to_string(),
                terminal,
            },
        )
        .await
        .map_err(|e| EtherNetIpError::Subscription(format!("Failed to send event: {e}")))
    }

    /// Checks whether a value has changed enough to warrant a notification.
    /// For floating-point types, uses the change_threshold as a deadband.
    /// For all other types, triggers on any change.
    fn value_changed(old: &PlcValue, new: &PlcValue, threshold: f32) -> bool {
        match (old, new) {
            (PlcValue::Real(o), PlcValue::Real(n)) => (*n - *o).abs() >= threshold,
            (PlcValue::Lreal(o), PlcValue::Lreal(n)) => (*n - *o).abs() >= threshold as f64,
            (PlcValue::Bool(o), PlcValue::Bool(n)) => o != n,
            (PlcValue::Sint(o), PlcValue::Sint(n)) => o != n,
            (PlcValue::Int(o), PlcValue::Int(n)) => o != n,
            (PlcValue::Dint(o), PlcValue::Dint(n)) => o != n,
            (PlcValue::Lint(o), PlcValue::Lint(n)) => o != n,
            (PlcValue::Usint(o), PlcValue::Usint(n)) => o != n,
            (PlcValue::Uint(o), PlcValue::Uint(n)) => o != n,
            (PlcValue::Udint(o), PlcValue::Udint(n)) => o != n,
            (PlcValue::Ulint(o), PlcValue::Ulint(n)) => o != n,
            (PlcValue::String(o), PlcValue::String(n)) => o != n,
            // Different types or UDTs — always notify
            _ => true,
        }
    }

    /// Waits for the next value update
    pub async fn wait_for_update(&self) -> Result<PlcValue> {
        let mut receiver = self.receiver.lock().await;
        let next_value = receiver.recv().await;
        drop(receiver);
        next_value.ok_or_else(|| EtherNetIpError::Subscription("Channel closed".to_string()))
    }

    /// Waits for the next value/error event.
    pub async fn wait_for_event(&self) -> Result<TagSubscriptionEvent> {
        let channels = self.event_channels();
        let mut receiver = channels.receiver.lock().await;
        let next_event = receiver.recv().await;
        drop(receiver);
        next_event.ok_or_else(|| EtherNetIpError::Subscription("Channel closed".to_string()))
    }

    /// Gets the last value received
    pub async fn get_last_value(&self) -> Option<PlcValue> {
        self.last_value.lock().await.clone()
    }

    async fn recv_next_value(&self) -> Option<PlcValue> {
        let mut receiver = self.receiver.lock().await;
        let next_value = receiver.recv().await;
        drop(receiver);
        next_value
    }

    async fn recv_next_event(&self) -> Option<TagSubscriptionEvent> {
        let channels = self.event_channels();
        let mut receiver = channels.receiver.lock().await;
        let next_event = receiver.recv().await;
        drop(receiver);
        next_event
    }

    /// Returns an async stream of value updates for this subscription.
    ///
    /// The stream yields each value as it is received from the background poll loop.
    /// Use with `StreamExt` (e.g. `.next().await`) or `select!` for composition.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use futures_util::StreamExt;
    ///
    /// let subscription = client.subscribe_to_tag("MyTag", SubscriptionOptions::default()).await?;
    /// let mut stream = subscription.into_stream();
    /// while let Some(value) = stream.next().await {
    ///     println!("Update: {:?}", value);
    /// }
    /// ```
    pub fn into_stream(self: Arc<Self>) -> impl Stream<Item = PlcValue> + Send {
        stream::unfold(self, |subscription| async move {
            let next_value = subscription.recv_next_value().await;
            next_value.map(|plc_value| (plc_value, subscription))
        })
    }

    /// Returns an async stream of value/error events for this subscription.
    pub fn into_event_stream(self: Arc<Self>) -> impl Stream<Item = TagSubscriptionEvent> + Send {
        stream::unfold(self, |subscription| async move {
            let next_event = subscription.recv_next_event().await;
            next_event.map(|event| (event, subscription))
        })
    }

    fn event_channels(&self) -> TagSubscriptionEventChannels {
        let key = self.event_key();
        let mut channels = TAG_SUBSCRIPTION_EVENTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        channels
            .entry(key)
            .or_insert_with(|| {
                let (sender, receiver) = mpsc::channel(100);
                TagSubscriptionEventChannels {
                    sender: Arc::new(Mutex::new(sender)),
                    receiver: Arc::new(Mutex::new(receiver)),
                }
            })
            .clone()
    }

    fn event_key(&self) -> usize {
        Arc::as_ptr(&self.is_active) as usize
    }
}

impl Drop for TagSubscription {
    fn drop(&mut self) {
        if Arc::strong_count(&self.is_active) == 1
            && let Ok(mut channels) = TAG_SUBSCRIPTION_EVENTS.lock()
        {
            channels.remove(&self.event_key());
        }
    }
}

pub(crate) async fn try_send_drop_oldest<T>(
    sender: &Arc<Mutex<mpsc::Sender<T>>>,
    receiver: &Arc<Mutex<mpsc::Receiver<T>>>,
    value: T,
) -> std::result::Result<(), String> {
    let sender = {
        let sender = sender.lock().await;
        sender.clone()
    };

    match sender.try_send(value) {
        Ok(()) => Ok(()),
        Err(mpsc::error::TrySendError::Closed(_)) => Err("channel closed".to_string()),
        Err(mpsc::error::TrySendError::Full(value)) => {
            if let Ok(mut receiver) = receiver.try_lock() {
                let _ = receiver.try_recv();
                match sender.try_send(value) {
                    Ok(()) => Ok(()),
                    Err(mpsc::error::TrySendError::Closed(_)) => Err("channel closed".to_string()),
                    Err(mpsc::error::TrySendError::Full(_)) => Ok(()),
                }
            } else {
                Ok(())
            }
        }
    }
}

/// Manages multiple tag subscriptions
#[derive(Debug, Clone)]
#[deprecated(
    since = "1.2.0",
    note = "SubscriptionManager is not used by EipClient; use EipClient subscription methods or Client tag groups instead. The type will be removed in 2.0."
)]
pub struct SubscriptionManager {
    subscriptions: Arc<Mutex<Vec<TagSubscription>>>,
}

#[expect(
    deprecated,
    reason = "CODEX-AQ keeps SubscriptionManager compatibility until 2.0 removal"
)]
impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[expect(
    deprecated,
    reason = "CODEX-AQ keeps SubscriptionManager compatibility until 2.0 removal"
)]
impl SubscriptionManager {
    /// Creates a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a new subscription
    pub async fn add_subscription(&self, subscription: TagSubscription) {
        let mut subscriptions = self.subscriptions.lock().await;
        subscriptions.push(subscription);
    }

    /// Removes a subscription
    pub async fn remove_subscription(&self, tag_name: &str) {
        let mut subscriptions = self.subscriptions.lock().await;
        subscriptions.retain(|sub| sub.tag_path != tag_name);
    }

    /// Updates a value for all matching subscriptions
    pub async fn update_value(&self, tag_name: &str, value: &PlcValue) -> Result<()> {
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

    /// Gets all active subscriptions
    pub async fn get_subscriptions(&self) -> Vec<TagSubscription> {
        let subscriptions = self.subscriptions.lock().await;
        subscriptions.clone()
    }

    /// Gets a specific subscription by tag name
    pub async fn get_subscription(&self, tag_name: &str) -> Option<TagSubscription> {
        let subscriptions = self.subscriptions.lock().await;
        subscriptions
            .iter()
            .find(|sub| sub.tag_path == tag_name)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_deadband_is_absolute_not_relative() {
        // change_threshold is an absolute deadband: a delta below it is
        // suppressed, a delta at/above it notifies, regardless of magnitude.
        let threshold = 0.001_f32;
        // Below the deadband -> no notification.
        assert!(!TagSubscription::value_changed(
            &PlcValue::Real(1000.0),
            &PlcValue::Real(1000.0005),
            threshold
        ));
        // At/above the deadband -> notify.
        assert!(TagSubscription::value_changed(
            &PlcValue::Real(1000.0),
            &PlcValue::Real(1000.002),
            threshold
        ));
        // Same absolute delta near zero behaves identically (proves absolute,
        // not relative/percentage, semantics).
        assert!(TagSubscription::value_changed(
            &PlcValue::Real(0.0),
            &PlcValue::Real(0.002),
            threshold
        ));
    }

    #[test]
    fn non_float_types_notify_on_any_change() {
        assert!(TagSubscription::value_changed(
            &PlcValue::Dint(1),
            &PlcValue::Dint(2),
            0.001
        ));
        assert!(!TagSubscription::value_changed(
            &PlcValue::Dint(5),
            &PlcValue::Dint(5),
            0.001
        ));
    }
}
