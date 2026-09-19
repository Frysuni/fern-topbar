//! Panel monitor selection shared with UI components following the panel.

use relm4::gtk;
use relm4::gtk::prelude::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

/// Selected GTK monitor, independent of window-manager data.
#[derive(Clone, Default)]
pub struct MonitorSelection {
    inner: Rc<RefCell<MonitorSubscriptions>>,
}

#[derive(Default)]
struct MonitorSubscriptions {
    current: Option<gtk::gdk::Monitor>,
    // Components use different input enums; callbacks preserve their typed senders.
    subscribers: HashMap<usize, Box<dyn Fn(gtk::gdk::Monitor)>>,
    next_id: usize,
}

/// Removes a component's monitor subscription when its mounted owner is dropped.
#[must_use]
pub struct MonitorSubscription {
    selection: Weak<RefCell<MonitorSubscriptions>>,
    id: usize,
}

impl Drop for MonitorSubscription {
    fn drop(&mut self) {
        if let Some(selection) = self.selection.upgrade() {
            selection.borrow_mut().subscribers.remove(&self.id);
        }
    }
}

impl MonitorSelection {
    pub fn subscribe<T: 'static>(
        &self,
        sender: relm4::Sender<T>,
        map: fn(gtk::gdk::Monitor) -> T,
    ) -> MonitorSubscription {
        let mut state = self.inner.borrow_mut();

        if let Some(monitor) = state.current.clone() {
            let _ = sender.send(map(monitor));
        }

        let id = state.next_id;

        state.next_id += 1;
        state.subscribers.insert(
            id,
            Box::new(move |monitor| {
                let _ = sender.send(map(monitor));
            }),
        );

        MonitorSubscription {
            selection: Rc::downgrade(&self.inner),
            id,
        }
    }

    pub fn set(&self, monitor: gtk::gdk::Monitor) {
        let mut state = self.inner.borrow_mut();
        state.current = Some(monitor.clone());

        for subscriber in state.subscribers.values() {
            subscriber(monitor.clone());
        }
    }
}

/// Resolves a connector name, returning `None` without a display or matching monitor.
pub fn monitor_for_output(output: &str) -> Option<gtk::gdk::Monitor> {
    let monitors = gtk::gdk::Display::default()?.monitors();

    (0..monitors.n_items())
        .filter_map(|index| monitors.item(index).and_downcast::<gtk::gdk::Monitor>())
        .find(|monitor| monitor.connector().as_deref() == Some(output))
}

/// Lists connector names for diagnostics, returning an empty list without a display.
pub fn available_monitor_names() -> Vec<String> {
    let Some(display) = gtk::gdk::Display::default() else {
        return Vec::new();
    };
    let monitors = display.monitors();

    (0..monitors.n_items())
        .filter_map(|index| monitors.item(index).and_downcast::<gtk::gdk::Monitor>())
        .filter_map(|monitor| monitor.connector().map(|name| name.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    #[gtk::test]
    fn subscriptions_keep_current_monitor_and_release_independently() {
        let monitor = gtk::gdk::Display::default()
            .unwrap()
            .monitors()
            .item(0)
            .and_downcast::<gtk::gdk::Monitor>()
            .unwrap();

        let selection = MonitorSelection::default();
        selection.set(monitor.clone());

        let (sender, receiver) = relm4::channel();
        let first = selection.subscribe(sender.clone(), |monitor| monitor);
        let second = selection.subscribe(sender, |monitor| monitor);

        for _ in 0..2 {
            assert_eq!(receiver.recv().now_or_never(), Some(Some(monitor.clone())));
        }

        drop(first);
        selection.set(monitor.clone());
        assert_eq!(receiver.recv().now_or_never(), Some(Some(monitor.clone())));
        assert!(receiver.recv().now_or_never().is_none());

        drop(second);
        selection.set(monitor);
        assert_eq!(receiver.recv().now_or_never(), Some(None));
    }
}
