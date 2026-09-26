use super::super::backend::MenuEntry;
use crate::ui::{core::Button, icon_names};
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::gtk;
use relm4::gtk::prelude::*;

const MENU_ENTRY_ICON_SIZE: i32 = 16;

#[derive(Clone, Debug)]
pub struct RowData {
    id: i32,
    label: String,
    icon: String,
    icon_data: Option<Vec<u8>>,
    shortcut: Option<String>,
    enabled: bool,
    separator: bool,
    toggle: Option<(bool, bool)>,
    opens_submenu: bool,
}

impl From<&MenuEntry> for RowData {
    fn from(entry: &MenuEntry) -> Self {
        Self {
            id: entry.id,
            label: entry.label.clone(),
            icon: entry.icon.clone(),
            icon_data: entry.icon_data.clone(),
            shortcut: entry.shortcut.clone(),
            enabled: entry.enabled,
            separator: entry.separator,
            toggle: entry.toggle,
            opens_submenu: entry.submenu || !entry.children.is_empty(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    Open(i32),
    Click(i32),
}

#[derive(Debug)]
pub enum Input {
    Activate,
}

pub struct MenuRow {
    entry: RowData,
}

#[relm4::factory(pub)]
impl FactoryComponent for MenuRow {
    type Init = RowData;
    type Input = Input;
    type Output = Action;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Separator {
                add_css_class: "topbar-tray-separator",
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                #[watch]
                set_visible: self.entry.separator,
            },
            #[template]
            Button {
                add_css_class: "topbar-tray-menu-row",
                #[watch]
                set_sensitive: self.entry.enabled,
                #[watch]
                set_visible: !self.entry.separator,
                connect_clicked => Input::Activate,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,

                    gtk::Image {
                        set_pixel_size: MENU_ENTRY_ICON_SIZE,
                        #[watch]
                        set_visible: self.has_icon_slot(),
                        #[watch]
                        set_menu_entry_icon: &self.entry,
                    },
                    gtk::Label {
                        add_css_class: "topbar-tray-label",
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        #[watch]
                        set_label: &self.entry.label,
                    },
                    gtk::Label {
                        add_css_class: "topbar-tray-shortcut",
                        #[watch]
                        set_label: self.entry.shortcut.as_deref().unwrap_or(""),
                        #[watch]
                        set_visible: self.entry.shortcut.is_some(),
                    },
                    gtk::Image {
                        add_css_class: "topbar-tray-chevron",
                        set_icon_name: Some(icon_names::CHEVRON_RIGHT),
                        #[watch]
                        set_visible: self.entry.opens_submenu,
                    },
                },
            },
        }
    }

    fn init_model(entry: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { entry }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        match message {
            Input::Activate => {
                let action = if self.entry.opens_submenu {
                    Action::Open(self.entry.id)
                } else {
                    Action::Click(self.entry.id)
                };

                let _ = sender.output(action);
            }
        }
    }
}

impl MenuRow {
    fn has_icon_slot(&self) -> bool {
        self.entry.toggle.is_some() || self.entry.icon_data.is_some() || !self.entry.icon.is_empty()
    }

    pub fn id(&self) -> i32 {
        self.entry.id
    }

    pub fn replace(&mut self, entry: &MenuEntry) {
        self.entry = RowData::from(entry);
    }
}

trait MenuEntryIconExt {
    fn set_menu_entry_icon(&self, entry: &RowData);
}

impl MenuEntryIconExt for gtk::Image {
    fn set_menu_entry_icon(&self, entry: &RowData) {
        self.clear();

        if let Some((radio, checked)) = entry.toggle {
            let name = match (radio, checked) {
                (true, true) => icon_names::RADIO_ON,
                (true, false) => icon_names::RADIO_OFF,
                (false, true) => icon_names::CHECK,
                (false, false) => "",
            };

            if !name.is_empty() {
                self.set_icon_name(Some(name));
            }
        } else if let Some(data) = &entry.icon_data
            && let Ok(texture) = gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from(data))
        {
            self.set_paintable(Some(&texture));
        } else if !entry.icon.is_empty() {
            self.set_icon_name(Some(&entry.icon));
        }
    }
}
