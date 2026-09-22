//! Concrete owners of mounted feature controllers and their resources.

use super::{clock, keyboard_layout, workspaces};
use relm4::gtk;

/// Keeps each feature's concrete resources alive until its group is dropped.
pub enum MountedFeature {
    Workspaces(workspaces::Mounted),
    Clock(clock::Mounted),
    KeyboardLayout(keyboard_layout::Mounted),
}

impl MountedFeature {
    pub fn widget(&self) -> &gtk::Widget {
        match self {
            Self::Workspaces(feature) => feature.widget(),
            Self::Clock(feature) => feature.widget(),
            Self::KeyboardLayout(feature) => feature.widget(),
        }
    }
}
