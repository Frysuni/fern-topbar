pub mod audio;
pub mod availability;
pub mod brightness;
pub mod clock;
pub mod keyboard_layout;
pub mod microphone;
mod mounted;
pub mod network;
mod registry;
mod sound;
pub mod tray;
pub mod workspaces;

use crate::{
    backend::{self, wm::state::WindowManagerState},
    ui::{core::PopoverScope, monitor::MonitorSelection},
};

pub(crate) use sound::AudioService;

pub use mounted::MountedFeature;
pub use registry::{EnabledFeature, EnabledFeatures, FeatureDefinition, FeatureId, resolve};

/// Dependencies passed to features; each service is defined in its owning module.
#[derive(Default)]
pub struct FeatureServices {
    pub availability: availability::FeatureAvailability,
    pub audio: AudioService,
    pub window_manager: WindowManagerState,
    pub wm_commands: Option<tokio::sync::mpsc::UnboundedSender<backend::Command>>,
    pub monitor_selection: MonitorSelection,
}

pub struct FeatureMountContext {
    pub availability: availability::FeatureAvailability,
    pub popovers: PopoverScope,
    pub audio: AudioService,
    pub window_manager: WindowManagerState,
    pub wm_commands: Option<tokio::sync::mpsc::UnboundedSender<backend::Command>>,
    pub monitor_selection: MonitorSelection,
}

impl FeatureMountContext {
    pub fn new(services: &FeatureServices, popovers: PopoverScope) -> Self {
        Self {
            availability: services.availability.clone(),
            popovers,
            audio: services.audio.clone(),
            window_manager: services.window_manager.clone(),
            wm_commands: services.wm_commands.clone(),
            monitor_selection: services.monitor_selection.clone(),
        }
    }
}
