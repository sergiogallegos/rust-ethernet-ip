use super::EipClient;
use crate::batch::{BatchError, BatchOperation, BatchResult};
use crate::error::{EtherNetIpError, Result};
use crate::monitoring::DiagnosticsSnapshot;
use crate::route::RoutePath;
use crate::types::PlcValue;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};

type BatchReadResults = Vec<(String, std::result::Result<PlcValue, BatchError>)>;
type BatchWriteResults = Vec<(String, std::result::Result<(), BatchError>)>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionEvent {
    Connected,
    Disconnected,
    WorkerStopped,
}

#[derive(Debug, Clone)]
pub struct Client {
    tx: mpsc::Sender<ClientCommand>,
    events: broadcast::Sender<ConnectionEvent>,
}

#[derive(Debug, Clone)]
pub enum Backoff {
    Constant(Duration),
    Exponential {
        initial: Duration,
        max: Duration,
        factor: u32,
    },
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub backoff: Backoff,
    pub retry_writes: bool,
}

impl RetryPolicy {
    pub fn constant(max_attempts: usize, delay: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            backoff: Backoff::Constant(delay),
            retry_writes: false,
        }
    }

    pub fn exponential(max_attempts: usize, initial: Duration, max: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            backoff: Backoff::Exponential {
                initial,
                max,
                factor: 2,
            },
            retry_writes: false,
        }
    }

    pub fn retry_writes(mut self, retry_writes: bool) -> Self {
        self.retry_writes = retry_writes;
        self
    }

    fn delay_for_attempt(&self, attempt_index: usize) -> Duration {
        match self.backoff {
            Backoff::Constant(delay) => delay,
            Backoff::Exponential {
                initial,
                max,
                factor,
            } => {
                let multiplier = factor.saturating_pow(attempt_index as u32);
                initial.saturating_mul(multiplier).min(max)
            }
        }
    }
}

#[derive(Clone)]
pub struct RetryClient {
    client: Client,
    policy: RetryPolicy,
}

enum ClientCommand {
    ReadTag {
        tag_name: String,
        reply: oneshot::Sender<Result<PlcValue>>,
    },
    WriteTag {
        tag_name: String,
        value: PlcValue,
        reply: oneshot::Sender<Result<()>>,
    },
    WriteStringTag {
        tag_name: String,
        value: String,
        reply: oneshot::Sender<Result<()>>,
    },
    WriteUdtMember {
        tag_name: String,
        member_name: String,
        value: PlcValue,
        reply: oneshot::Sender<Result<()>>,
    },
    ExecuteBatch {
        operations: Vec<BatchOperation>,
        reply: oneshot::Sender<Result<Vec<BatchResult>>>,
    },
    ReadTagsBatch {
        tag_names: Vec<String>,
        reply: oneshot::Sender<Result<BatchReadResults>>,
    },
    WriteTagsBatch {
        tag_values: Vec<(String, PlcValue)>,
        reply: oneshot::Sender<Result<BatchWriteResults>>,
    },
    CheckHealth {
        reply: oneshot::Sender<bool>,
    },
    Diagnostics {
        verified: bool,
        reply: oneshot::Sender<Result<DiagnosticsSnapshot>>,
    },
}

impl Client {
    pub async fn connect(addr: &str) -> Result<Self> {
        Self::from_eip_client(EipClient::connect(addr).await?)
    }

    pub async fn with_route_path(addr: &str, route: RoutePath) -> Result<Self> {
        Self::from_eip_client(EipClient::with_route_path(addr, route).await?)
    }

    pub fn from_eip_client(client: EipClient) -> Result<Self> {
        let (tx, rx) = mpsc::channel(128);
        let (events, _) = broadcast::channel(128);
        let actor = Self {
            tx,
            events: events.clone(),
        };

        tokio::spawn(run_client_actor(client, rx, events));
        Ok(actor)
    }

    pub fn events(&self) -> broadcast::Receiver<ConnectionEvent> {
        self.events.subscribe()
    }

    pub fn with_retry(&self, policy: RetryPolicy) -> RetryClient {
        RetryClient {
            client: self.clone(),
            policy,
        }
    }

    pub async fn read_tag(&self, tag_name: &str) -> Result<PlcValue> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::ReadTag {
            tag_name: tag_name.to_string(),
            reply,
        })
        .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    pub async fn write_tag(&self, tag_name: &str, value: PlcValue) -> Result<()> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::WriteTag {
            tag_name: tag_name.to_string(),
            value,
            reply,
        })
        .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    pub async fn write_string_tag(&self, tag_name: &str, value: &str) -> Result<()> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::WriteStringTag {
            tag_name: tag_name.to_string(),
            value: value.to_string(),
            reply,
        })
        .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    pub async fn write_udt_member(
        &self,
        udt_tag_name: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<()> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::WriteUdtMember {
            tag_name: udt_tag_name.to_string(),
            member_name: member_name.to_string(),
            value,
            reply,
        })
        .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    pub async fn write_udt_array_member(
        &self,
        udt_array_element_path: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<()> {
        self.write_udt_member(udt_array_element_path, member_name, value)
            .await
    }

    pub async fn execute_batch(&self, operations: &[BatchOperation]) -> Result<Vec<BatchResult>> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::ExecuteBatch {
            operations: operations.to_vec(),
            reply,
        })
        .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    pub async fn read_tags_batch(&self, tag_names: &[&str]) -> Result<BatchReadResults> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::ReadTagsBatch {
            tag_names: tag_names.iter().map(|name| (*name).to_string()).collect(),
            reply,
        })
        .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    pub async fn write_tags_batch(
        &self,
        tag_values: &[(&str, PlcValue)],
    ) -> Result<BatchWriteResults> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::WriteTagsBatch {
            tag_values: tag_values
                .iter()
                .map(|(name, value)| ((*name).to_string(), value.clone()))
                .collect(),
            reply,
        })
        .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    pub async fn check_health(&self) -> Result<bool> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::CheckHealth { reply }).await?;
        rx.await.map_err(|_| actor_stopped_error())
    }

    pub async fn get_diagnostics_snapshot(&self) -> Result<DiagnosticsSnapshot> {
        self.diagnostics(false).await
    }

    pub async fn get_diagnostics_snapshot_detailed(&self) -> Result<DiagnosticsSnapshot> {
        self.diagnostics(true).await
    }

    async fn diagnostics(&self, verified: bool) -> Result<DiagnosticsSnapshot> {
        let (reply, rx) = oneshot::channel();
        self.send(ClientCommand::Diagnostics { verified, reply })
            .await?;
        rx.await.unwrap_or_else(|_| actor_stopped())
    }

    async fn send(&self, command: ClientCommand) -> Result<()> {
        self.tx
            .send(command)
            .await
            .map_err(|_| actor_stopped_error())
    }
}

impl RetryClient {
    pub async fn read_tag(&self, tag_name: &str) -> Result<PlcValue> {
        self.retry(|| async { self.client.read_tag(tag_name).await })
            .await
    }

    pub async fn write_tag(&self, tag_name: &str, value: PlcValue) -> Result<()> {
        if !self.policy.retry_writes {
            return self.client.write_tag(tag_name, value).await;
        }

        self.retry(|| {
            let value = value.clone();
            async move { self.client.write_tag(tag_name, value).await }
        })
        .await
    }

    pub async fn write_string_tag(&self, tag_name: &str, value: &str) -> Result<()> {
        if !self.policy.retry_writes {
            return self.client.write_string_tag(tag_name, value).await;
        }

        self.retry(|| async { self.client.write_string_tag(tag_name, value).await })
            .await
    }

    async fn retry<T, Fut, Op>(&self, mut op: Op) -> Result<T>
    where
        Fut: std::future::Future<Output = Result<T>>,
        Op: FnMut() -> Fut,
    {
        let mut attempt = 0;
        loop {
            match op().await {
                Ok(value) => return Ok(value),
                Err(err) if err.is_retriable() && attempt + 1 < self.policy.max_attempts => {
                    let delay = self.policy.delay_for_attempt(attempt);
                    attempt += 1;
                    tokio::time::sleep(delay).await;
                }
                Err(err) => return Err(err),
            }
        }
    }
}

async fn run_client_actor(
    mut client: EipClient,
    mut rx: mpsc::Receiver<ClientCommand>,
    events: broadcast::Sender<ConnectionEvent>,
) {
    let _ = events.send(ConnectionEvent::Connected);

    while let Some(command) = rx.recv().await {
        match command {
            ClientCommand::ReadTag { tag_name, reply } => {
                let _ = reply.send(client.read_tag(&tag_name).await);
            }
            ClientCommand::WriteTag {
                tag_name,
                value,
                reply,
            } => {
                let _ = reply.send(client.write_tag(&tag_name, value).await);
            }
            ClientCommand::WriteStringTag {
                tag_name,
                value,
                reply,
            } => {
                let _ = reply.send(client.write_string_tag(&tag_name, &value).await);
            }
            ClientCommand::WriteUdtMember {
                tag_name,
                member_name,
                value,
                reply,
            } => {
                let _ = reply.send(
                    client
                        .write_udt_member(&tag_name, &member_name, value)
                        .await,
                );
            }
            ClientCommand::ExecuteBatch { operations, reply } => {
                let _ = reply.send(client.execute_batch(&operations).await);
            }
            ClientCommand::ReadTagsBatch { tag_names, reply } => {
                let refs: Vec<&str> = tag_names.iter().map(String::as_str).collect();
                let _ = reply.send(client.read_tags_batch(&refs).await);
            }
            ClientCommand::WriteTagsBatch { tag_values, reply } => {
                let refs: Vec<(&str, PlcValue)> = tag_values
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.clone()))
                    .collect();
                let _ = reply.send(client.write_tags_batch(&refs).await);
            }
            ClientCommand::CheckHealth { reply } => {
                let _ = reply.send(client.check_health().await);
            }
            ClientCommand::Diagnostics { verified, reply } => {
                let result = if verified {
                    client.get_diagnostics_snapshot_detailed().await
                } else {
                    Ok(client.get_diagnostics_snapshot().await)
                };
                let _ = reply.send(result);
            }
        }
    }

    let _ = client.unregister_session().await;
    let _ = events.send(ConnectionEvent::Disconnected);
    let _ = events.send(ConnectionEvent::WorkerStopped);
}

fn actor_stopped<T>() -> Result<T> {
    Err(actor_stopped_error())
}

fn actor_stopped_error() -> EtherNetIpError {
    EtherNetIpError::ConnectionLost("client actor stopped".to_string())
}
