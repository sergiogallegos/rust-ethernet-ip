use crate::client::{Client, ConnectionEvent};
use crate::error::Result;
use crate::route::RoutePath;
use std::collections::HashMap;
use std::hash::Hash;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

/// Fleet-level connection event annotated with the PLC identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetEvent<PlcId> {
    /// Application-defined controller identifier.
    pub plc_id: PlcId,
    /// Connection lifecycle event from that controller's client.
    pub event: ConnectionEvent,
}

/// Multi-PLC pool built from actor-backed [`Client`] handles.
#[derive(Debug)]
pub struct Fleet<PlcId> {
    clients: HashMap<PlcId, Client>,
    forwarders: HashMap<PlcId, JoinHandle<()>>,
    events: broadcast::Sender<FleetEvent<PlcId>>,
}

impl<PlcId> Default for Fleet<PlcId>
where
    PlcId: Clone + Eq + Hash + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<PlcId> Fleet<PlcId>
where
    PlcId: Clone + Eq + Hash + Send + Sync + 'static,
{
    /// Creates an empty fleet.
    #[must_use]
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(128);
        Self {
            clients: HashMap::new(),
            forwarders: HashMap::new(),
            events,
        }
    }

    /// Connects and adds one PLC by address.
    pub async fn connect(&mut self, plc_id: PlcId, addr: &str) -> Result<Client> {
        let client = Client::connect(addr).await?;
        self.insert_client(plc_id, client.clone());
        Ok(client)
    }

    /// Connects and adds one routed PLC by address and route path.
    pub async fn connect_with_route(
        &mut self,
        plc_id: PlcId,
        addr: &str,
        route: RoutePath,
    ) -> Result<Client> {
        let client = Client::with_route_path(addr, route).await?;
        self.insert_client(plc_id, client.clone());
        Ok(client)
    }

    /// Inserts an existing actor client into the fleet.
    pub fn insert_client(&mut self, plc_id: PlcId, client: Client) -> Option<Client> {
        if let Some(forwarder) = self.forwarders.remove(&plc_id) {
            forwarder.abort();
        }

        let previous = self.clients.insert(plc_id.clone(), client.clone());
        let _ = self.events.send(FleetEvent {
            plc_id: plc_id.clone(),
            event: ConnectionEvent::Connected,
        });
        let forwarder = self.forward_events(plc_id.clone(), client);
        self.forwarders.insert(plc_id, forwarder);
        previous
    }

    /// Returns a cloneable client handle for one PLC.
    #[must_use]
    pub fn client(&self, plc_id: &PlcId) -> Option<Client> {
        self.clients.get(plc_id).cloned()
    }

    /// Subscribes to fleet-level connection events.
    pub fn events(&self) -> broadcast::Receiver<FleetEvent<PlcId>> {
        self.events.subscribe()
    }

    /// Performs a health check against every PLC currently in the fleet.
    pub async fn check_health(&self) -> HashMap<PlcId, Result<bool>> {
        let mut health = HashMap::with_capacity(self.clients.len());
        for (plc_id, client) in &self.clients {
            health.insert(plc_id.clone(), client.check_health().await);
        }
        health
    }

    /// Returns the number of PLCs in the fleet.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Returns true when the fleet has no PLC clients.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    fn forward_events(&self, plc_id: PlcId, client: Client) -> JoinHandle<()> {
        let events = self.events.clone();
        tokio::spawn(async move {
            forward_events_loop(plc_id, client.events(), events).await;
        })
    }
}

impl<PlcId> Drop for Fleet<PlcId> {
    fn drop(&mut self) {
        for (_, forwarder) in self.forwarders.drain() {
            forwarder.abort();
        }
    }
}

async fn forward_events_loop<PlcId>(
    plc_id: PlcId,
    mut client_events: broadcast::Receiver<ConnectionEvent>,
    events: broadcast::Sender<FleetEvent<PlcId>>,
) where
    PlcId: Clone,
{
    loop {
        match client_events.recv().await {
            Ok(ConnectionEvent::Connected) => continue,
            Ok(event) => {
                let _ = events.send(FleetEvent {
                    plc_id: plc_id.clone(),
                    event,
                });
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn forward_events_loop_continues_after_lagged_source() {
        let (source_tx, source_rx) = broadcast::channel(1);
        let (fleet_tx, mut fleet_rx) = broadcast::channel(8);
        let task = tokio::spawn(forward_events_loop("plc-a", source_rx, fleet_tx));

        let _ = source_tx.send(ConnectionEvent::Connected);
        let _ = source_tx.send(ConnectionEvent::Disconnected);
        let _ = source_tx.send(ConnectionEvent::WorkerStopped);

        let forwarded = fleet_rx.recv().await.expect("event after lag");
        assert_eq!(forwarded.plc_id, "plc-a");
        assert_eq!(forwarded.event, ConnectionEvent::WorkerStopped);

        drop(source_tx);
        task.await.expect("forwarder exits after source closes");
    }
}
