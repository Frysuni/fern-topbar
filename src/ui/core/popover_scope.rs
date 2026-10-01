//! Tree-owned registration and dismissal of panel popovers.

use super::{UiScale, motion};
use crate::ui::{Action, Input};
use relm4::gtk::prelude::*;
use relm4::{Sender, gtk};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Identity of one registered menu, independent of its feature or widget address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PopupId(u64);

impl PopupId {
    fn allocate() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        Self(
            NEXT_ID
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("popup identifiers exhausted"),
        )
    }

    /// Allocates a unique identity for tests that exercise lifecycle state without GTK.
    #[cfg(test)]
    pub fn for_test() -> Self {
        Self::allocate()
    }
}

/// A panel-local dismissal scope containing at most one active popover.
///
/// The panel supplies its input sender to receive menu lifecycle events. Default
/// creates an independent standalone scope without a panel event destination.
/// Clones share state and hold only weak widget references.
#[derive(Clone, Default)]
pub struct PopoverScope {
    scale: UiScale,
    active: Rc<RefCell<Option<ActivePopover>>>,
    events: Option<Sender<Input>>,
}

struct ActivePopover {
    id: PopupId,
    popover: gtk::glib::WeakRef<gtk::Popover>,
    trigger: gtk::glib::WeakRef<gtk::Widget>,
    opened: Rc<Cell<bool>>,
}

/// Owns a menu's signal handlers and its membership in the panel's dismissal scope.
///
/// Keep this guard in the component that owns the menu. Dropping it disconnects
/// callbacks and closes/releases that menu without affecting a newer active one.
/// Relm4 components fill their registration field after constructing the widgets.
#[must_use = "retain the registration for the menu's lifetime"]
pub struct PopupRegistration {
    id: PopupId,
    scope: PopoverScope,
    popover: gtk::glib::WeakRef<gtk::Popover>,
    opened: Rc<Cell<bool>>,
    handlers: Vec<gtk::glib::SignalHandlerId>,
    keys: gtk::EventControllerKey,
}

impl PopupRegistration {
    /// Returns the identity assigned when this menu was registered.
    pub fn id(&self) -> PopupId {
        self.id
    }
}

impl Drop for PopupRegistration {
    fn drop(&mut self) {
        let opened = self.opened.get();

        if let Some(popover) = self.popover.upgrade() {
            for handler in self.handlers.drain(..) {
                popover.disconnect(handler);
            }

            popover.remove_controller(&self.keys);
            self.scope.forget(self.id, &self.opened);

            if opened {
                popover.popdown();
            }
        } else {
            self.scope.forget(self.id, &self.opened);
        }
    }
}

impl PopoverScope {
    /// Creates the panel's dismissal scope and lifecycle event destination.
    pub fn new(scale: UiScale, events: Sender<Input>) -> Self {
        Self {
            scale,
            events: Some(events),
            ..Self::default()
        }
    }

    /// Returns the zoom shared by this scope's menu surfaces and feature windows.
    pub fn scale(&self) -> UiScale {
        self.scale
    }

    /// Reports whether this scope currently has a live, visible menu.
    pub fn is_open(&self) -> bool {
        self.active
            .borrow()
            .as_ref()
            .and_then(|active| active.popover.upgrade())
            .is_some_and(|popover| popover.get_visible())
    }

    /// Releases the active menu and asks its surface to close.
    /// State is cleared before GTK signals are emitted to avoid reentrant borrows.
    pub fn close(&self) -> bool {
        let Some(active) = self.active.borrow_mut().take() else {
            return false;
        };
        let visible = active
            .popover
            .upgrade()
            .is_some_and(|popover| popover.get_visible());

        self.close_entry(active);

        visible
    }

    /// Reports whether a widget belongs to the active popover or its trigger.
    pub fn contains(&self, widget: &gtk::Widget) -> bool {
        let active = self.active.borrow();
        let Some(active) = active.as_ref() else {
            return false;
        };
        let Some(popover) = active.popover.upgrade() else {
            return false;
        };
        let trigger = active.trigger.upgrade();
        let mut current = Some(widget.clone());

        while let Some(widget) = current {
            if widget == popover || trigger.as_ref() == Some(&widget) {
                return true;
            }

            current = widget.parent();
        }

        false
    }

    /// Registers the popover already assigned to a menu button.
    ///
    /// Call once before opening it and retain the returned guard. Replacing the
    /// popover requires dropping the old registration and registering the new one.
    pub fn register_button(&self, button: &gtk::MenuButton) -> PopupRegistration {
        self.register(
            &button
                .popover()
                .expect("menu button has a popover before registration"),
            button,
        )
    }

    /// Registers a surface and custom trigger, including tray item menus.
    ///
    /// Disables autohide, tracks map/visibility/unmap, and handles Escape. The
    /// panel closes the scope on outside clicks and focus loss. This does not
    /// parent the popover or synchronize the trigger's styling.
    pub fn register(
        &self,
        popover: &gtk::Popover,
        trigger: &impl IsA<gtk::Widget>,
    ) -> PopupRegistration {
        popover.set_autohide(false);
        self.scale.configure_popover(popover);

        if let Some(child) = popover.child() {
            motion::animate_appearance(&child);
        }

        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);

        let mut registration = PopupRegistration {
            id: PopupId::allocate(),
            scope: self.clone(),
            popover: popover.downgrade(),
            opened: Rc::new(Cell::new(false)),
            handlers: Vec::new(),
            keys: keys.clone(),
        };

        let id = registration.id();
        let trigger = trigger.as_ref().downgrade();
        let scope = self.clone();
        let opened = registration.opened.clone();

        registration
            .handlers
            .push(popover.connect_map(move |current| {
                scope.open(id, current, trigger.clone(), opened.clone());
            }));

        let scope = self.clone();
        let opened = registration.opened.clone();

        registration
            .handlers
            .push(popover.connect_visible_notify(move |current| {
                if !current.get_visible() {
                    scope.forget(id, &opened);
                }
            }));

        let scope = self.clone();
        let opened = registration.opened.clone();

        registration
            .handlers
            .push(popover.connect_unmap(move |_| scope.forget(id, &opened)));

        let scope = self.clone();

        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape && scope.close() {
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        popover.add_controller(keys);

        registration
    }

    fn emit(&self, action: Action) {
        if let Some(events) = &self.events {
            let _ = events.send(Input::Action(action));
        }
    }

    // Announce the new owner before closing its predecessor. The application
    // must never observe an empty menu set during a switch between menus.
    fn open(
        &self,
        id: PopupId,
        popover: &gtk::Popover,
        trigger: gtk::glib::WeakRef<gtk::Widget>,
        opened: Rc<Cell<bool>>,
    ) {
        if opened.replace(true) {
            return;
        }

        let previous = self.active.borrow_mut().replace(ActivePopover {
            id,
            popover: popover.downgrade(),
            trigger,
            opened,
        });

        self.emit(Action::PopupOpened(id));

        if let Some(previous) = previous {
            self.close_entry(previous);
        }
    }

    fn close_entry(&self, active: ActivePopover) {
        if active.opened.replace(false) {
            self.emit(Action::PopupClosed(active.id));
        }

        if let Some(popover) = active.popover.upgrade() {
            popover.popdown();
        }
    }

    // GTK can report the same closure through several signals. Late callbacks
    // from the previous menu may release only its own identity, never the new one.
    fn forget(&self, id: PopupId, opened: &Cell<bool>) {
        if !opened.replace(false) {
            return;
        }

        {
            let mut active = self.active.borrow_mut();

            if active.as_ref().is_some_and(|active| active.id == id) {
                active.take();
            }
        }

        self.emit(Action::PopupClosed(id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::core::{MenuButtonStyle, MenuPopover, PanelMenuButton, PopoverStyle};
    use futures_util::FutureExt;
    use relm4::WidgetTemplate;

    fn drain_events() {
        let context = gtk::glib::MainContext::default();

        while context.pending() {
            context.iteration(false);
        }
    }

    fn menu_button(label: &str) -> gtk::MenuButton {
        let button_template = PanelMenuButton::init(MenuButtonStyle::Icon);
        let button = button_template.widget().clone();
        button.set_label(label);

        let popover = MenuPopover::init(PopoverStyle::Menu).widget().clone();
        popover.set_child(Some(&gtk::Label::new(Some(label))));
        button.set_popover(Some(&popover));

        button
    }

    fn next_action(events: &relm4::Receiver<Input>) -> Option<Action> {
        events.recv().now_or_never().flatten().map(|input| {
            let Input::Action(action) = input else {
                panic!("unexpected panel input: {input:?}");
            };

            action
        })
    }

    #[gtk::test]
    fn registrations_publish_lifecycle_and_release_only_their_own_menu() {
        let window = gtk::Window::new();
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        window.set_child(Some(&row));

        let (sender, events) = relm4::channel();
        let scope = PopoverScope::new(UiScale::default(), sender);
        let first = menu_button("First");
        let second = menu_button("Second");

        row.append(&first);
        row.append(&second);

        let first_registration = scope.register_button(&first);
        let second_registration = scope.register_button(&second);
        let first_id = first_registration.id();
        let second_id = second_registration.id();

        assert_ne!(first_id, second_id);
        window.present();
        drain_events();

        first.popup();
        assert!(matches!(next_action(&events), Some(Action::PopupOpened(id)) if id == first_id));
        second.popup();
        assert!(matches!(next_action(&events), Some(Action::PopupOpened(id)) if id == second_id));
        assert!(matches!(next_action(&events), Some(Action::PopupClosed(id)) if id == first_id));
        first.popdown();
        drain_events();
        assert!(next_action(&events).is_none());
        assert!(scope.contains(second.upcast_ref()));

        // Removing an inactive item must not close a newer menu, including tray menus.
        drop(first_registration);
        assert!(scope.is_open());
        assert!(second.popover().unwrap().get_visible());
        assert!(next_action(&events).is_none());

        drop(second_registration);
        assert!(matches!(next_action(&events), Some(Action::PopupClosed(id)) if id == second_id));
        drain_events();
        assert!(!scope.is_open());
        assert!(!second.popover().unwrap().get_visible());
        assert!(next_action(&events).is_none());

        // The widget may outlive its component; disconnected callbacks stay inactive.
        second.popup();
        assert!(!scope.is_open());
        assert!(next_action(&events).is_none());
        second.popdown();

        let replacement = scope.register_button(&second);

        assert_ne!(replacement.id(), second_id);
        second.popup();
        assert!(
            matches!(next_action(&events), Some(Action::PopupOpened(id)) if id == replacement.id())
        );

        // Hiding the owning window releases tracking, with no duplicate Drop event.
        window.set_visible(false);
        drain_events();
        assert!(
            matches!(next_action(&events), Some(Action::PopupClosed(id)) if id == replacement.id())
        );
        drop(replacement);
        assert!(next_action(&events).is_none());
        window.destroy();
    }

    #[gtk::test]
    fn only_the_active_popover_is_managed() {
        let window = gtk::Window::new();
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        window.set_child(Some(&row));

        let popovers = PopoverScope::default();

        let first = menu_button("First");
        let first_child = gtk::Label::new(Some("First trigger"));

        first.set_child(Some(&first_child));

        let second = menu_button("Second");
        let other = menu_button("Other panel");

        row.append(&first);
        row.append(&second);
        row.append(&other);

        let _first = popovers.register_button(&first);
        let _second = popovers.register_button(&second);

        let other_popovers = PopoverScope::default();
        let _other = other_popovers.register_button(&other);

        window.present();
        drain_events();

        assert!(!first.popover().unwrap().is_autohide());
        assert!(!popovers.contains(first_child.upcast_ref()));
        assert!(!popovers.close());

        first.popup();
        other.popup();

        assert!(popovers.contains(first_child.upcast_ref()));
        assert!(popovers.contains(first.popover().unwrap().child().unwrap().upcast_ref()));
        assert!(!popovers.contains(second.upcast_ref()));
        assert!(!popovers.contains(other.upcast_ref()));
        assert!(first.popover().unwrap().get_visible());
        assert!(other.popover().unwrap().get_visible());

        second.popup();

        assert!(!first.popover().unwrap().get_visible());
        assert!(second.popover().unwrap().get_visible());
        assert!(other.popover().unwrap().get_visible());
        assert!(!popovers.contains(first_child.upcast_ref()));
        assert!(popovers.contains(second.upcast_ref()));

        let tray_button = gtk::Button::new();
        let tray_popover = gtk::Popover::new();
        tray_popover.set_child(Some(&gtk::Label::new(Some("Tray"))));
        tray_popover.set_parent(&tray_button);

        let _tray = popovers.register(&tray_popover, &tray_button);

        row.append(&tray_button);
        tray_popover.popup();

        assert!(popovers.contains(tray_button.upcast_ref()));
        assert!(!second.popover().unwrap().get_visible());
        assert!(tray_popover.get_visible());

        tray_popover.popdown();

        assert!(!popovers.contains(tray_button.upcast_ref()));
        assert!(!popovers.close());

        tray_popover.popup();

        assert!(popovers.close());
        assert!(!tray_popover.get_visible());
        assert!(other.popover().unwrap().get_visible());
        assert!(!popovers.close());

        tray_popover.popup();
        tray_popover.unparent();
        row.remove(&tray_button);

        assert!(!popovers.contains(tray_button.upcast_ref()));
        assert!(!popovers.close());
        assert!(other_popovers.close());

        window.destroy();
        drain_events();
    }
}
