use super::backend::{self, TrayUpdate};
use crate::ui::core::PopoverScope;
use item::{TrayItem, TrayItemInput};
use relm4::factory::FactoryVecDeque;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

mod icon;
mod item;
mod menu_row;

pub struct Tray {
    popovers: PopoverScope,
    items: FactoryVecDeque<TrayItem>,
    commands: backend::Controls,
}

pub struct TrayInit {
    pub popovers: PopoverScope,
    pub commands: backend::Controls,
}

#[derive(Debug)]
pub enum TrayInput {
    Items(TrayUpdate),
    Activate(String),
    Secondary(String),
    Open(String, i32),
    PageChanged(String, i32),
    Click(String, i32),
    Scroll(String, i32, bool),
}

#[relm4::component(pub)]
impl Component for Tray {
    type Init = TrayInit;
    type Input = TrayInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "topbar-tray",
            #[watch]
            set_visible: !model.items.is_empty(),

            #[local_ref]
            items -> gtk::Box {
                set_spacing: 2,
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let TrayInit { commands, popovers } = init;

        let items = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |message| message);

        let model = Self {
            popovers,
            items,
            commands,
        };

        let items = model.items.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            TrayInput::Items(update) => self.sync_items(update),
            TrayInput::Activate(id) => self.commands.send(backend::Command::Activate(id)),
            TrayInput::Secondary(id) => self.commands.send(backend::Command::SecondaryActivate(id)),
            TrayInput::Open(id, page) => self.open_item_menu(id, page),
            TrayInput::PageChanged(id, page) => {
                self.commands.send(backend::Command::OpenMenu(id, page))
            }
            TrayInput::Click(id, entry) => self.close_menu_and_click(id, entry),
            TrayInput::Scroll(id, delta, horizontal) => self
                .commands
                .send(backend::Command::Scroll(id, delta * 120, horizontal)),
        }

        self.update_view(widgets, sender);
    }
}

impl Tray {
    fn sync_items(&mut self, update: TrayUpdate) {
        let TrayUpdate {
            items,
            changed_items,
            changed_menus,
            changed_titles,
            changed_menu_rows,
        } = update;

        let item_ids = items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<std::collections::HashSet<_>>();

        let mut current = self.items.guard();
        let mut index = 0;

        while index < current.len() {
            if item_ids.contains(current[index].id()) {
                index += 1;
            } else {
                current.remove(index);
            }
        }

        for (target, item) in items.into_iter().enumerate() {
            if let Some(source) = (0..current.len()).find(|&index| current[index].id() == item.id) {
                if source != target {
                    current.move_to(source, target);
                }

                if changed_items.contains(&item.id) {
                    let menu_changed = changed_menus.contains(&item.id);
                    let title_changed = changed_titles.contains(&item.id);
                    let changed_rows = changed_menu_rows.get(&item.id);

                    current[target].replace(item, menu_changed, title_changed, changed_rows);
                }
            } else {
                current.insert(target, (item, self.popovers.clone()));
            }
        }
    }

    fn open_item_menu(&mut self, id: String, page: i32) {
        let Some(index) = self.item_index(&id) else {
            tracing::warn!(tray_item = %id, "tray item disappeared before its menu opened");
            return;
        };

        if !self.items[index].has_menu() {
            self.commands.send(backend::Command::ContextMenu(id));
            return;
        }

        self.items.send(index, TrayItemInput::OpenPage(page));
        self.items.send(index, TrayItemInput::Popup);

        self.commands
            .send(backend::Command::OpenMenu(id.clone(), page));
    }

    fn close_menu_and_click(&self, id: String, entry: i32) {
        if let Some(index) = self.item_index(&id) {
            self.items.send(index, TrayItemInput::Popdown);
        }

        self.commands.send(backend::Command::ClickMenu(id, entry));
    }

    fn item_index(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|item| item.id() == id)
    }
}
