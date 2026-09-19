// The template macro emits a struct outside its impl, so the expectation must
// cover this module. Tests read the named fields directly; production uses the
// same widgets through initialization-time signal closures.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Relm4 retains named signal widgets in the template struct"
    )
)]

use super::ActionToggleButton;
use relm4::{gtk::prelude::*, prelude::*};

/// Compact text disclosure with a chevron and a collapsible content area.
///
/// Initialize with a label. Features populate `content` and may update `title`.
/// The control uses the notification-history action button, including its padding.
/// Expanding and collapsing the content slides over 180 milliseconds.
#[relm4::widget_template(pub)]
impl WidgetTemplate for Disclosure {
    type Init = &'static str;

    view! {
        #[name = "root"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "toggle"]
            #[template]
            ActionToggleButton {
                connect_toggled[revealer, chevron] => move |button| {
                    let expanded = button.is_active();

                    revealer.set_reveal_child(expanded);
                    chevron.set_icon_name(Some(if expanded {
                        "pan-down-symbolic"
                    } else {
                        "pan-end-symbolic"
                    }));
                },

                gtk::Box {
                    set_spacing: 4,

                    #[name = "title"]
                    gtk::Label {
                        set_label: init,
                        set_xalign: 0.0,
                    },

                    #[name = "chevron"]
                    gtk::Image {
                        set_icon_name: Some("pan-end-symbolic"),
                    },
                },
            },

            #[name = "revealer"]
            gtk::Revealer {
                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                set_transition_duration: 180,

                #[name = "content"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_top: 8,
                },
            },
        }
    }
}

impl Disclosure {
    /// Borrows the disclosure section's GTK root.
    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}

impl AsRef<gtk::Widget> for Disclosure {
    /// Supplies the GTK widget handle used for composition.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gtk::test]
    fn disclosure_buttons_toggle_content_and_chevrons_together() {
        let section = Disclosure::init("Connection details");

        assert_eq!(section.toggle.widget().halign(), gtk::Align::Start);
        assert!(!section.toggle.widget().gets_focus_on_click());
        assert!(section.toggle.widget().is_focusable());
        assert!(
            section
                .toggle
                .widget()
                .has_css_class("topbar-action-button")
        );
        assert!(!section.revealer.reveals_child());
        assert_eq!(section.revealer.transition_duration(), 180);

        section.toggle.widget().set_active(true);

        assert!(section.revealer.reveals_child());
        assert_eq!(
            section.chevron.icon_name().as_deref(),
            Some("pan-down-symbolic")
        );

        section.toggle.widget().set_active(false);

        assert!(!section.revealer.reveals_child());
        assert_eq!(
            section.chevron.icon_name().as_deref(),
            Some("pan-end-symbolic")
        );
    }
}
