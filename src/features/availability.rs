//! Current feature readiness, independent of configured placement and UI lifetime.

use super::registry::FeatureId;
use snafu::Snafu;
use std::{collections::HashMap, future::Future, time::Duration};
use tokio::sync::watch;

pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Availability {
    #[default]
    Checking,
    Available,
    Unavailable(UnavailableReason),
    Failed(ProbeError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Snafu)]
pub enum UnavailableReason {
    #[snafu(display("unsupported session"))]
    UnsupportedSession,
    #[snafu(display("service missing"))]
    ServiceMissing,
    #[snafu(display("device missing"))]
    DeviceMissing,
    #[snafu(display("name occupied"))]
    NameOccupied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Snafu)]
pub enum ProbeError {
    #[snafu(display("connection failed"))]
    Connect,
    #[snafu(display("read failed"))]
    Read,
    #[snafu(display("invalid response"))]
    Protocol,
    #[snafu(display("permission denied"))]
    PermissionDenied,
    #[snafu(display("probe timed out"))]
    Timeout,
}

impl Availability {
    pub fn is_available(self) -> bool {
        self == Self::Available
    }
}

/// Bounds initialization and snapshot reads, without bounding long-lived subscriptions.
pub async fn probe<T>(
    future: impl Future<Output = Result<T, Availability>>,
) -> Result<T, Availability> {
    match tokio::time::timeout(PROBE_TIMEOUT, future).await {
        Ok(result) => result,
        Err(_) => Err(Availability::Failed(ProbeError::Timeout)),
    }
}

/// Publishes readiness transitions for one feature.
#[derive(Clone)]
pub struct AvailabilityPublisher(watch::Sender<Availability>);

impl Default for AvailabilityPublisher {
    fn default() -> Self {
        Self(watch::channel(Availability::Checking).0)
    }
}

impl AvailabilityPublisher {
    pub fn set(&self, next: Availability) {
        self.0.send_if_modified(|current| {
            if *current == next {
                return false;
            }

            *current = next;

            true
        });
    }

    pub fn current(&self) -> Availability {
        *self.0.borrow()
    }

    pub fn subscribe(&self) -> watch::Receiver<Availability> {
        self.0.subscribe()
    }
}

/// Retains the latest readiness of each feature for existing and new subscribers.
/// All channels exist before cloning, so clones share each feature's publisher.
#[derive(Clone)]
pub struct FeatureAvailability {
    states: HashMap<FeatureId, AvailabilityPublisher>,
}

impl Default for FeatureAvailability {
    fn default() -> Self {
        let states = FeatureId::all()
            .map(|feature| (feature, AvailabilityPublisher::default()))
            .collect();

        Self { states }
    }
}

impl FeatureAvailability {
    pub fn publisher(&self, feature: FeatureId) -> AvailabilityPublisher {
        self.states
            .get(&feature)
            .expect("every feature has an availability publisher")
            .clone()
    }

    pub fn subscribe(&self, feature: FeatureId) -> watch::Receiver<Availability> {
        self.publisher(feature).subscribe()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub async fn wait_for(updates: &mut watch::Receiver<Availability>, expected: Availability) {
        tokio::time::timeout(Duration::from_secs(4), async {
            loop {
                if *updates.borrow_and_update() == expected {
                    return;
                }

                updates.changed().await.unwrap();
            }
        })
        .await
        .unwrap_or_else(|_| panic!("expected {expected:?}, got {:?}", updates.borrow()));
    }

    #[tokio::test]
    async fn a_stalled_probe_expires() {
        let result = probe(std::future::pending::<Result<(), Availability>>()).await;

        assert_eq!(result, Err(Availability::Failed(ProbeError::Timeout)));
    }

    #[tokio::test]
    async fn subscribers_keep_latest_readiness_without_duplicate_updates() {
        let availability = FeatureAvailability::default();
        let shared = availability.clone();
        let mut updates = shared.subscribe(FeatureId::Battery);
        let publisher = availability.publisher(FeatureId::Battery);
        let missing = Availability::Unavailable(UnavailableReason::DeviceMissing);

        publisher.set(missing);
        updates.changed().await.unwrap();
        assert_eq!(*updates.borrow_and_update(), missing);
        publisher.set(missing);
        assert!(!updates.has_changed().unwrap());
        publisher.set(Availability::Available);
        updates.changed().await.unwrap();
        assert_eq!(*updates.borrow_and_update(), Availability::Available);
        assert_eq!(
            *availability.subscribe(FeatureId::Battery).borrow(),
            Availability::Available
        );
        assert_eq!(
            *availability.subscribe(FeatureId::Network).borrow(),
            Availability::Checking
        );
    }
}
