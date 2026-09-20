//! Concrete owners of mounted feature controllers and their resources.

use super::clock;
use relm4::gtk;

/// Keeps each feature's concrete resources alive until its group is dropped.
pub enum MountedFeature {
    Clock(clock::Mounted),
}

impl MountedFeature {
    pub fn widget(&self) -> &gtk::Widget {
        match self {
            Self::Clock(feature) => feature.widget(),
        }
    }
}
