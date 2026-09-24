mod view;

use super::{FeatureDefinition, FeatureId};

/// Describes availability and mounting without starting the feature.
pub fn definition() -> FeatureDefinition {
    FeatureDefinition {
        available: |services| services.availability.subscribe(FeatureId::Microphone),
        mount: view::mount,
    }
}
