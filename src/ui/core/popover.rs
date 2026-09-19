//! Declarative core UI widget templates, independent of feature state.

use relm4::gtk::prelude::*;
use relm4::prelude::*;

/// Styling for a panel menu surface.
#[derive(Clone, Copy, Debug)]
pub enum PopoverStyle {
    /// Standard menu styling.
    Menu,
    /// Mixer styling including sound sliders.
    Mixer,
}

/// Arrowless menu below its trigger with a 12-pixel offset.
/// Initialize with menu or mixer styling, then assign content and parent.
/// Register once with [`super::PopoverScope`] before the first opening.
#[relm4::widget_template(pub)]
impl WidgetTemplate for MenuPopover {
    type Init = PopoverStyle;

    view! {
        #[name = "root"]
        gtk::Popover {
            add_css_class: match init {
                PopoverStyle::Menu => "topbar-menu",
                PopoverStyle::Mixer => "topbar-mixer",
            },
            set_has_arrow: false,
            set_position: gtk::PositionType::Bottom,
            set_offset: (0, 12),
        }
    }
}

impl MenuPopover {
    /// Borrows the underlying GTK widget without transferring ownership.
    ///
    /// Use this for GTK APIs and imperative updates outside the Relm4 DSL.
    pub fn widget(&self) -> &gtk::Popover {
        &self.root
    }
}
