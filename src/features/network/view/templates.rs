use relm4::{gtk::prelude::*, prelude::*};

/// Separate transport surface with an icon, status, switch and content area.
#[relm4::widget_template(pub)]
impl WidgetTemplate for TransportBlock {
    type Init = (&'static str, &'static str);

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "topbar-network-block",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,

            gtk::Box {
                add_css_class: "topbar-network-setting",
                set_spacing: 12,

                gtk::Image {
                    set_icon_name: Some(init.1),
                    set_valign: gtk::Align::Center,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_spacing: 3,

                    gtk::Label {
                        add_css_class: "topbar-menu-title",
                        set_xalign: 0.0,
                        set_label: init.0,
                    },

                    #[name = "subtitle"]
                    gtk::Label {
                        add_css_class: "topbar-menu-subtitle",
                        set_xalign: 0.0,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_max_width_chars: 32,
                    },
                },

                #[name = "toggle"]
                gtk::Switch {
                    set_valign: gtk::Align::Center,
                },
            },

            #[name = "content"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,
            },
        }
    }
}

impl TransportBlock {
    /// Borrows the transport block's GTK root.
    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}

impl AsRef<gtk::Widget> for TransportBlock {
    /// Supplies the GTK widget handle used for composition.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

/// Compact network entry with bounded text and a horizontal action group.
#[relm4::widget_template(pub)]
impl WidgetTemplate for DetailRow {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "topbar-network-row",
            set_spacing: 12,

            #[name = "icon"]
            gtk::Image {
                set_valign: gtk::Align::Center,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 3,

                #[name = "title"]
                gtk::Label {
                    add_css_class: "topbar-menu-title",
                    set_xalign: 0.0,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_max_width_chars: 24,
                },

                #[name = "subtitle"]
                gtk::Label {
                    add_css_class: "topbar-menu-subtitle",
                    set_xalign: 0.0,
                    set_wrap: true,
                    set_wrap_mode: gtk::pango::WrapMode::WordChar,
                    set_max_width_chars: 32,
                },
            },

            #[name = "actions"]
            gtk::Box {
                set_valign: gtk::Align::Center,
                set_spacing: 8,
            },
        }
    }
}

impl DetailRow {
    /// Borrows the entry's GTK root.
    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}

impl AsRef<gtk::Widget> for DetailRow {
    /// Supplies the widget handle used by Relm4 factories.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

/// Inline password or confirmation surface within the network popover.
#[relm4::widget_template(pub)]
impl WidgetTemplate for InlinePrompt {
    type Init = ();

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "topbar-network-prompt",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            #[name = "title"]
            gtk::Label {
                add_css_class: "topbar-menu-title",
                set_xalign: 0.0,
                set_wrap: true,
                set_max_width_chars: 36,
            },
        }
    }
}

impl InlinePrompt {
    /// Borrows the inline prompt's GTK root.
    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }
}

impl AsRef<gtk::Widget> for InlinePrompt {
    /// Supplies the GTK widget handle used for composition.
    fn as_ref(&self) -> &gtk::Widget {
        self.widget().upcast_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gtk::test]
    fn detail_rows_wrap_secondary_text_without_expanding_titles() {
        let row = DetailRow::init(());

        assert!(row.subtitle.wraps());
        assert_eq!(row.title.ellipsize(), gtk::pango::EllipsizeMode::End);
        assert!(row.widget().has_css_class("topbar-network-row"));
        assert_eq!(row.actions.orientation(), gtk::Orientation::Horizontal);
    }
}
