use super::Event;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

pub mod niri;
pub mod state;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    pub id: u64,
    pub index: u8,
    pub name: Option<String>,
    pub output: Option<String>,
    pub active: bool,
    pub urgent: bool,
}

pub enum Command {
    FocusWorkspace(u64),
    CloseOverview,
}

/// Selects the window manager available in the current session.
/// Returns no command channel when the session has no supported WM.
pub fn start(
    events: UnboundedSender<Event>,
    availability: crate::features::availability::FeatureAvailability,
) -> Option<UnboundedSender<Command>> {
    let Some(socket_path) = std::env::var_os(niri_ipc::socket::SOCKET_PATH_ENV) else {
        use crate::features::{
            FeatureId,
            availability::{Availability, UnavailableReason},
        };

        for feature in [FeatureId::Workspaces, FeatureId::KeyboardLayout] {
            availability
                .publisher(feature)
                .set(Availability::Unavailable(
                    UnavailableReason::UnsupportedSession,
                ));
        }

        return None;
    };

    Some(niri::start(
        PathBuf::from(socket_path),
        events,
        availability,
    ))
}
