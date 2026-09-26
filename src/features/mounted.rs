//! Concrete owners of mounted feature controllers and their resources.

use super::{brightness, clock, keyboard_layout, sound, tray, workspaces};
use relm4::gtk;

/// Keeps each feature's concrete resources alive until its group is dropped.
pub enum MountedFeature {
    Workspaces(workspaces::Mounted),
    Clock(clock::Mounted),
    Sound(sound::Mounted),
    KeyboardLayout(keyboard_layout::Mounted),
    Brightness(brightness::Mounted),
    Tray(tray::Mounted),
}

impl MountedFeature {
    pub fn widget(&self) -> &gtk::Widget {
        match self {
            Self::Workspaces(feature) => feature.widget(),
            Self::Clock(feature) => feature.widget(),
            Self::Sound(feature) => feature.widget(),
            Self::KeyboardLayout(feature) => feature.widget(),
            Self::Brightness(feature) => feature.widget(),
            Self::Tray(feature) => feature.widget(),
        }
    }
}
