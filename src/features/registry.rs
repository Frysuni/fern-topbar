//! Typed feature definitions and resolution of the configured groups.

use super::{
    FeatureMountContext, FeatureServices, MountedFeature, availability::Availability, clock,
};
use crate::config::{FeatureMode, FeatureOptions, Features};
use serde::Deserialize;

/// Feature names accepted by the configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureId {
    Clock,
}

impl FeatureId {
    pub fn all() -> impl Iterator<Item = Self> {
        FEATURE_REGISTRY.iter().map(|feature| feature.id)
    }

    pub fn as_str(self) -> &'static str {
        self.registration().name
    }

    fn registration(self) -> &'static FeatureRegistration {
        FEATURE_REGISTRY
            .iter()
            .find(|feature| feature.id == self)
            .expect("every FeatureId must be registered")
    }
}

struct FeatureRegistration {
    id: FeatureId,
    name: &'static str,
    definition: fn() -> FeatureDefinition,
}

/// Shared registration list for configuration resolution, names and readiness channels.
/// Register each new FeatureId here; availability derives its keys from this list.
const FEATURE_REGISTRY: &[FeatureRegistration] = &[FeatureRegistration {
    id: FeatureId::Clock,
    name: "clock",
    definition: clock::definition,
}];

/// Creates the UI owner; backend failures are reported through `Availability`.
pub type Mount = fn(FeatureMountContext) -> MountedFeature;

/// A feature's readiness subscription and UI constructor.
///
/// Mounting is infallible: the UI and its backend owner exist while services are
/// missing, so readiness transitions can reveal the same component after recovery.
pub struct FeatureDefinition {
    pub available: fn(&FeatureServices) -> tokio::sync::watch::Receiver<Availability>,
    pub mount: Mount,
}

/// Configured features remain registered while dependencies are unavailable.
pub struct EnabledFeature {
    pub name: FeatureId,
    pub mode: FeatureMode,
    pub definition: FeatureDefinition,
}

#[derive(Default)]
pub struct EnabledFeatures {
    pub start: Vec<EnabledFeature>,
    pub center: Vec<EnabledFeature>,
    pub end: Vec<EnabledFeature>,
}

pub fn resolve(config: Features) -> EnabledFeatures {
    EnabledFeatures {
        start: resolve_group(config.start),
        center: resolve_group(config.center),
        end: resolve_group(config.end),
    }
}

fn resolve_group(options: Vec<FeatureOptions>) -> Vec<EnabledFeature> {
    options.into_iter().filter_map(resolve_feature).collect()
}

fn resolve_feature(options: FeatureOptions) -> Option<EnabledFeature> {
    if options.mode == FeatureMode::Switch(false) {
        return None;
    }

    let definition = (options.name.registration().definition)();

    Some(EnabledFeature {
        name: options.name,
        mode: options.mode,
        definition,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Auto, features::availability::UnavailableReason};

    #[test]
    fn registered_definitions_observe_their_own_readiness() {
        let services = FeatureServices::default();

        for name in FeatureId::all() {
            let feature = resolve_feature(FeatureOptions {
                name,
                mode: FeatureMode::Switch(true),
            })
            .unwrap();

            let updates = (feature.definition.available)(&services);

            assert_eq!(*updates.borrow(), Availability::Checking, "{name:?}");

            let publisher = services.availability.publisher(name);
            publisher.set(Availability::Available);

            assert_eq!(*updates.borrow(), Availability::Available, "{name:?}");

            publisher.set(Availability::Checking);
        }
    }
}
