use crate::PlcValue;
use crate::error::{EtherNetIpError, Result};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{Mutex, mpsc};

use futures::{Stream, stream};

/// Configuration options for tag subscriptions
#[derive(Debug, Clone)]
pub struct SubscriptionOptions {
    /// Update rate in milliseconds
    pub update_rate: u32,
    /// Change threshold for numeric values
    pub change_threshold: f32,
    /// Timeout in milliseconds
    pub timeout: u32,
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self {
            update_rate: 100,        // 100ms default update rate
            change_threshold: 0.001, // 0.1% change threshold
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

impl TagSubscription {
    /// Creates a new tag subscription
    pub fn new(tag_name: String, options: SubscriptionOptions) -> Self {
        let (sender, receiver) = mpsc::channel(100); // Buffer size of 100
        Self {
            tag_path: tag_name,
            options,
            last_value: Arc::new(Mutex::new(None)),
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
            is_active: Arc::new(AtomicBool::new(true)),
        }
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

    /// Updates the subscription value
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
        let sender = {
            let sender = self.sender.lock().await;
            sender.clone()
        };
        sender
            .send(value.clone())
            .await
            .map_err(|e| EtherNetIpError::Subscription(format!("Failed to send update: {e}")))?;

        Ok(())
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
}

/// Manages multiple tag subscriptions
#[derive(Debug, Clone)]
pub struct SubscriptionManager {
    subscriptions: Arc<Mutex<Vec<TagSubscription>>>,
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

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
