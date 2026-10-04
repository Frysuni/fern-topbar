//! Typed feature definitions and resolution of the configured groups.

use super::{
    FeatureMountContext, FeatureServices, MountedFeature, audio, availability::Availability,
    battery, brightness, clock, keyboard_layout, microphone, network, notifications, tray,
    workspaces,
};
use crate::config::{FeatureMode, FeatureOptions, Features};
use serde::Deserialize;

/// Feature names accepted by the configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureId {
    Workspaces,
    Clock,
    Audio,
    Microphone,
    KeyboardLayout,
    Brightness,
    Tray,
    Network,
    Battery,
    Notifications,
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
const FEATURE_REGISTRY: &[FeatureRegistration] = &[
    FeatureRegistration {
        id: FeatureId::Workspaces,
        name: "workspaces",
        definition: workspaces::definition,
    },
    FeatureRegistration {
        id: FeatureId::Clock,
        name: "clock",
        definition: clock::definition,
    },
    FeatureRegistration {
        id: FeatureId::Audio,
        name: "audio",
        definition: audio::definition,
    },
    FeatureRegistration {
        id: FeatureId::Microphone,
        name: "microphone",
        definition: microphone::definition,
    },
    FeatureRegistration {
        id: FeatureId::KeyboardLayout,
        name: "keyboard_layout",
        definition: keyboard_layout::definition,
    },
    FeatureRegistration {
        id: FeatureId::Brightness,
        name: "brightness",
        definition: brightness::definition,
    },
    FeatureRegistration {
        id: FeatureId::Tray,
        name: "tray",
        definition: tray::definition,
    },
    FeatureRegistration {
        id: FeatureId::Network,
        name: "network",
        definition: network::definition,
    },
    FeatureRegistration {
        id: FeatureId::Battery,
        name: "battery",
        definition: battery::definition,
    },
    FeatureRegistration {
        id: FeatureId::Notifications,
        name: "notifications",
        definition: notifications::definition,
    },
];

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
    pub show_percent: bool,
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
        show_percent: options.show_percent,
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
                show_percent: false,
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

    #[test]
    fn resolution_preserves_groups_and_order_after_filtering() {
        let config = Features {
            start: vec![
                FeatureOptions {
                    name: FeatureId::KeyboardLayout,
                    mode: FeatureMode::Switch(true),
                    show_percent: false,
                },
                FeatureOptions {
                    name: FeatureId::Network,
                    mode: FeatureMode::Switch(false),
                    show_percent: false,
                },
                FeatureOptions {
                    name: FeatureId::Clock,
                    mode: FeatureMode::Auto(Auto::Auto),
                    show_percent: false,
                },
            ],
            center: vec![FeatureOptions {
                name: FeatureId::Workspaces,
                mode: FeatureMode::Switch(true),
                show_percent: false,
            }],
            end: vec![
                FeatureOptions {
                    name: FeatureId::Audio,
                    mode: FeatureMode::Switch(true),
                    show_percent: false,
                },
                FeatureOptions {
                    name: FeatureId::Microphone,
                    mode: FeatureMode::Switch(false),
                    show_percent: false,
                },
            ],
        };

        let features = resolve(config);
        let names = |group: Vec<EnabledFeature>| {
            group
                .into_iter()
                .map(|feature| feature.name)
                .collect::<Vec<_>>()
        };

        assert_eq!(
            names(features.start),
            [FeatureId::KeyboardLayout, FeatureId::Clock]
        );
        assert_eq!(names(features.center), [FeatureId::Workspaces]);
        assert_eq!(names(features.end), [FeatureId::Audio]);
    }

    #[test]
    fn resolution_retains_unavailable_features_in_both_enabled_modes() {
        let config = Features {
            start: Vec::new(),
            center: Vec::new(),
            end: vec![
                FeatureOptions {
                    name: FeatureId::Battery,
                    mode: FeatureMode::Auto(Auto::Auto),
                    show_percent: false,
                },
                FeatureOptions {
                    name: FeatureId::Notifications,
                    mode: FeatureMode::Switch(true),
                    show_percent: false,
                },
                FeatureOptions {
                    name: FeatureId::Network,
                    mode: FeatureMode::Switch(false),
                    show_percent: false,
                },
            ],
        };

        let group = resolve(config).end;

        assert_eq!(
            group.iter().map(|feature| feature.name).collect::<Vec<_>>(),
            [FeatureId::Battery, FeatureId::Notifications]
        );

        let services = FeatureServices::default();

        for feature in group {
            let missing = Availability::Unavailable(UnavailableReason::ServiceMissing);

            services.availability.publisher(feature.name).set(missing);

            assert_eq!(*(feature.definition.available)(&services).borrow(), missing);
        }
    }
}
