//! Declarative core UI widget templates, independent of feature state.

use relm4::gtk::prelude::*;
use relm4::prelude::*;

/// Unstyled button for feature-specific content, geometry and click handlers.
/// Pointer clicks preserve focus; keyboard navigation and activation stay enabled.
#[relm4::widget_template(pub)]
impl WidgetTemplate for Button {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Button {
            set_focus_on_click: false,
        }
    }
}

impl Button {
    /// Borrows the button's GTK root without transferring ownership.
    pub fn widget(&self) -> &gtk::Button {
        &self.root
    }
}

impl AsRef<gtk::Widget> for Button {
    /// Supplies the widget handle used for composition and mounting.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

/// Compact surface-backed action button shared with notification history.
/// Features provide its label or child, alignment and click handler.
/// Pointer clicks preserve keyboard focus; Tab navigation remains available.
#[relm4::widget_template(pub)]
impl WidgetTemplate for ActionButton {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Button {
            add_css_class: "topbar-action-button",
            set_has_frame: true,
            set_focus_on_click: false,
            set_halign: gtk::Align::Start,
        }
    }
}

impl ActionButton {
    /// Borrows the action button's GTK root without transferring ownership.
    pub fn widget(&self) -> &gtk::Button {
        &self.root
    }
}

impl AsRef<gtk::Widget> for ActionButton {
    /// Supplies the widget handle used for composition and mounting.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

/// Toggle counterpart of [`ActionButton`] with identical padding and styling.
/// Features provide its content and react to GTK's active state.
/// Pointer clicks preserve keyboard focus; Tab navigation remains available.
#[relm4::widget_template(pub)]
impl WidgetTemplate for ActionToggleButton {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::ToggleButton {
            add_css_class: "topbar-action-button",
            set_has_frame: true,
            set_focus_on_click: false,
            set_halign: gtk::Align::Start,
        }
    }
}

impl ActionToggleButton {
    /// Borrows the toggle button's GTK root without transferring ownership.
    pub fn widget(&self) -> &gtk::ToggleButton {
        &self.root
    }
}

impl AsRef<gtk::Widget> for ActionToggleButton {
    /// Supplies the widget handle used for composition and mounting.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

/// Padding for a panel menu trigger.
#[derive(Clone, Copy, Debug)]
pub enum MenuButtonStyle {
    /// Compact icon-only padding.
    Icon,
    /// Extra horizontal padding for text content.
    Labeled,
}

/// Frameless panel menu trigger whose active state is managed by GTK.
/// Initialize with icon or labeled padding; the feature supplies content and signals.
/// Configure its popover with the panel's shared scope after assigning it.
/// Pointer clicks do not focus the trigger; keyboard navigation stays enabled.
#[relm4::widget_template(pub)]
impl WidgetTemplate for PanelMenuButton {
    type Init = MenuButtonStyle;

    view! {
        #[name = "root"]
        gtk::MenuButton {
            add_css_class: "topbar-feature-button",
            set_has_frame: false,
            set_focus_on_click: false,
            set_class_active: ("topbar-labeled-button", matches!(init, MenuButtonStyle::Labeled)),
        }
    }
}

impl PanelMenuButton {
    /// Borrows the underlying GTK widget without transferring ownership.
    ///
    /// Use this for GTK APIs and imperative updates outside the Relm4 DSL.
    pub fn widget(&self) -> &gtk::MenuButton {
        &self.root
    }
}

impl AsRef<gtk::Widget> for PanelMenuButton {
    /// Supplies the GTK widget handle required by Relm4 mounting and factories.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

/// Clickable panel icon preserving primary-click application activation.
/// The feature supplies content and synchronizes any popover's checked styling.
/// Pointer clicks preserve focus; keyboard navigation and activation stay enabled.
#[relm4::widget_template(pub)]
impl WidgetTemplate for PanelIconButton {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Button {
            add_css_class: "topbar-icon-button",
            set_focus_on_click: false,
        }
    }
}

impl PanelIconButton {
    /// Borrows the underlying GTK widget without transferring ownership.
    ///
    /// Use this for GTK APIs and imperative updates outside the Relm4 DSL.
    pub fn widget(&self) -> &gtk::Button {
        &self.root
    }
}

impl AsRef<gtk::Widget> for PanelIconButton {
    /// Supplies the GTK widget handle required by Relm4 mounting and factories.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}
