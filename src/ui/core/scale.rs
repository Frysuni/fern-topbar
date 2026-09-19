//! Declarative core UI widget templates, independent of feature state.

use relm4::gtk::prelude::*;
use relm4::prelude::*;

/// Horizontal percentage slider from the initialization minimum to 100.
/// Uses a 148-pixel width request, increments of 1 and 10 and hides the value.
/// Initialize with 0 for sound or 1 for brightness; the minimum must be below 100.
/// Features own signal handling, backend commands and value synchronization.
#[relm4::widget_template(pub)]
impl WidgetTemplate for PercentScale {
    type Init = u8;

    view! {
        #[name = "root"]
        gtk::Scale {
            set_orientation: gtk::Orientation::Horizontal,
            set_size_request: (148, -1),
            set_draw_value: false,
            set_range: ({
                assert!(init < 100, "percentage scale requires a nonempty range");

                f64::from(init)
            }, 100.0),
            set_increments: (1.0, 10.0),
        }
    }
}

impl PercentScale {
    /// Borrows the underlying GTK widget without transferring ownership.
    ///
    /// Use this for GTK APIs and imperative updates outside the Relm4 DSL.
    pub fn widget(&self) -> &gtk::Scale {
        &self.root
    }
}

/// Empty right-aligned four-character percentage readout.
/// The feature formats its text; there is no automatic scale binding.
#[relm4::widget_template(pub)]
impl WidgetTemplate for PercentValue {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Label {
            add_css_class: "topbar-volume-value",
            set_width_chars: 4,
            set_max_width_chars: 4,
            set_xalign: 1.0,
        }
    }
}
