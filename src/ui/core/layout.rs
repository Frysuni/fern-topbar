//! Declarative core UI widget templates, independent of feature state.

use relm4::gtk::prelude::*;
use relm4::prelude::*;

/// Noninteractive horizontal status indicator with four-pixel spacing.
/// The feature supplies children, status data, visibility and tooltips.
#[relm4::widget_template(pub)]
impl WidgetTemplate for StatusBox {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "topbar-status",
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 4,
        }
    }
}

impl StatusBox {
    /// Borrows the underlying GTK widget without transferring ownership.
    ///
    /// Use this for GTK APIs and imperative updates outside the Relm4 DSL.
    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}

impl AsRef<gtk::Widget> for StatusBox {
    /// Supplies the GTK widget handle required by Relm4 mounting and factories.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

/// Visible vertical divider between panel features, centered with a 1-by-16 request.
/// The stylesheet controls final spacing and minimum size.
#[relm4::widget_template(pub)]
impl WidgetTemplate for FeatureDivider {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Separator {
            add_css_class: "topbar-feature-divider",
            set_orientation: gtk::Orientation::Vertical,
            set_size_request: (1, 16),
            set_valign: gtk::Align::Center,
            set_visible: true,
        }
    }
}

impl FeatureDivider {
    /// Borrows the underlying GTK widget without transferring ownership.
    ///
    /// Use this for GTK APIs and imperative updates outside the Relm4 DSL.
    pub fn widget(&self) -> &gtk::Separator {
        &self.root
    }
}
