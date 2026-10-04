//! Latest WM data, retained independently of mounted feature components.

use super::{KeyboardLayouts, Workspace};
use tokio::sync::watch;

/// Shares current workspaces and keyboard layout with existing and late subscribers.
/// Monitor selection belongs to the panel and does not change these WM streams.
#[derive(Clone)]
pub struct WindowManagerState {
    workspaces: watch::Sender<Vec<Workspace>>,
    keyboard_layout: watch::Sender<Option<KeyboardLayouts>>,
}

impl Default for WindowManagerState {
    fn default() -> Self {
        Self {
            workspaces: watch::channel(Vec::new()).0,
            keyboard_layout: watch::channel(None).0,
        }
    }
}

impl WindowManagerState {
    pub fn subscribe_workspaces(&self) -> watch::Receiver<Vec<Workspace>> {
        self.workspaces.subscribe()
    }

    pub fn subscribe_keyboard_layout(&self) -> watch::Receiver<Option<KeyboardLayouts>> {
        self.keyboard_layout.subscribe()
    }

    pub fn set_workspaces(&self, workspaces: Vec<Workspace>) {
        self.workspaces.send_if_modified(|current| {
            if *current == workspaces {
                return false;
            }

            *current = workspaces;

            true
        });
    }

    pub fn set_keyboard_layout(&self, layout: Option<KeyboardLayouts>) {
        self.keyboard_layout.send_if_modified(|current| {
            if *current == layout {
                return false;
            }

            *current = layout;

            true
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribers_receive_current_and_future_state_across_clones() {
        let state = WindowManagerState::default();
        let shared = state.clone();
        let workspaces = vec![Workspace {
            id: 7,
            index: 2,
            name: Some("Code".into()),
            output: Some("DP-1".into()),
            active: true,
            urgent: false,
        }];

        // Initialization can finish before any feature is mounted.
        state.set_workspaces(workspaces.clone());
        state.set_keyboard_layout(Some(KeyboardLayouts {
            names: vec!["English (US)".into()],
            current_idx: 0,
        }));

        let mut first = state.subscribe_workspaces();
        let mut second = shared.subscribe_workspaces();
        let mut keyboard = shared.subscribe_keyboard_layout();

        assert_eq!(*first.borrow_and_update(), workspaces);
        assert_eq!(*second.borrow_and_update(), workspaces);
        assert_eq!(
            keyboard
                .borrow_and_update()
                .as_ref()
                .and_then(KeyboardLayouts::current_name),
            Some("English (US)")
        );

        shared.set_workspaces(workspaces);
        shared.set_keyboard_layout(Some(KeyboardLayouts {
            names: vec!["English (US)".into()],
            current_idx: 0,
        }));

        assert!(!first.has_changed().unwrap());
        assert!(!second.has_changed().unwrap());
        assert!(!keyboard.has_changed().unwrap());

        shared.set_workspaces(Vec::new());

        assert!(first.has_changed().unwrap());
        assert!(second.has_changed().unwrap());
        assert!(first.borrow_and_update().is_empty());
        assert!(second.borrow_and_update().is_empty());
        assert!(!keyboard.has_changed().unwrap());

        state.set_keyboard_layout(None);

        assert!(keyboard.has_changed().unwrap());
        assert_eq!(*keyboard.borrow_and_update(), None);
        assert!(!first.has_changed().unwrap());

        drop(first);
        drop(second);
        shared.set_workspaces(vec![Workspace {
            id: 8,
            index: 1,
            name: None,
            output: Some("HDMI-A-1".into()),
            active: true,
            urgent: false,
        }]);

        assert_eq!(state.subscribe_workspaces().borrow()[0].id, 8);
    }
}
