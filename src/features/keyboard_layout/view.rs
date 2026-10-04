use crate::{
    backend::wm::{Command, KeyboardLayouts},
    ui::core::{
        MenuButtonStyle, MenuPopover, PanelMenuButton, PopoverScope, PopoverStyle,
        PopupRegistration,
    },
};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use std::{cell::Cell, rc::Rc, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

pub struct KeyboardLayout {
    layouts: Option<KeyboardLayouts>,
    wm_commands: Option<UnboundedSender<Command>>,
    rows: gtk::Box,
    pending: Option<u8>,
    close_generation: Rc<Cell<u64>>,
    _popup: Option<PopupRegistration>,
}

pub struct KeyboardLayoutInit {
    pub layouts: Option<KeyboardLayouts>,
    pub wm_commands: Option<UnboundedSender<Command>>,
    pub popovers: PopoverScope,
}

#[derive(Debug)]
pub enum Input {
    LayoutsChanged(Option<KeyboardLayouts>),
    Select { index: u8, names: Vec<String> },
    Closed,
    FinishSelection(u64),
}

#[relm4::component(pub)]
impl Component for KeyboardLayout {
    type Init = KeyboardLayoutInit;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        #[template]
        PanelMenuButton(MenuButtonStyle::Labeled) {
            add_css_class: "topbar-layout",
            set_direction: gtk::ArrowType::None,
            set_tooltip_text: Some("Choose keyboard layout"),
            #[watch]
            set_visible: model.layouts.as_ref().and_then(KeyboardLayouts::current_name).is_some(),
            #[watch]
            set_label: model.layouts.as_ref().and_then(KeyboardLayouts::current_name).unwrap_or(""),
            #[wrap(Some)]
            #[template]
            set_popover = &MenuPopover(PopoverStyle::Menu) {
                #[local_ref]
                rows -> gtk::Box {},
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            layouts: init.layouts,
            wm_commands: init.wm_commands,
            rows: gtk::Box::new(gtk::Orientation::Vertical, 4),
            pending: None,
            close_generation: Rc::new(Cell::new(0)),
            _popup: None,
        };
        model.sync_rows(&sender);
        let rows = &model.rows;
        let widgets = view_output!();
        let popover = root.popover().expect("layout menu has a popover");
        let generation = model.close_generation.clone();
        let input = sender.input_sender().clone();
        popover.connect_visible_notify(move |popover| {
            // Invalidate delayed closes synchronously, even on a fast reopen.
            generation.set(generation.get().wrapping_add(1));
            if !popover.get_visible() {
                let _ = input.send(Input::Closed);
            }
        });
        model
            .rows
            .connect_map(|rows| rows.add_css_class("layout-menu-mapped"));
        model
            .rows
            .connect_unmap(|rows| rows.remove_css_class("layout-menu-mapped"));
        model._popup = Some(init.popovers.register_button(root.widget()));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            Input::LayoutsChanged(layouts) => {
                let names_changed =
                    self.layouts.as_ref().map(|s| &s.names) != layouts.as_ref().map(|s| &s.names);
                self.layouts = layouts;
                self.close_generation
                    .set(self.close_generation.get().wrapping_add(1));
                if names_changed {
                    self.pending = None;
                    self.sync_rows(&sender);
                } else {
                    self.sync_selection();
                }
                if self.layouts.is_none() {
                    root.popdown();
                } else if self.pending.is_some()
                    && self.pending == self.layouts.as_ref().map(|s| s.current_idx)
                {
                    self.finish_selection(&sender);
                }
            }
            Input::Select { index, names } => {
                // A queued click must not select a different layout after a config reload.
                let Some(layouts) = &self.layouts else {
                    return;
                };
                if layouts.names != names || usize::from(index) >= names.len() {
                    return;
                }
                self.close_generation
                    .set(self.close_generation.get().wrapping_add(1));
                self.pending = None;
                if index == layouts.current_idx {
                    self.finish_selection(&sender);
                } else if let Some(commands) = &self.wm_commands {
                    if commands.send(Command::SwitchKeyboardLayout(index)).is_ok() {
                        self.pending = Some(index);
                    } else {
                        tracing::warn!("cannot switch keyboard layout: WM command channel closed");
                    }
                }
            }
            Input::FinishSelection(generation) => {
                if generation == self.close_generation.get() {
                    self.pending = None;
                    root.popdown();
                }
            }
            Input::Closed => {
                self.pending = None;
                // Discard theme transitions and pointer state from the previous opening.
                self.sync_rows(&sender);
            }
        }
    }
}

impl KeyboardLayout {
    fn finish_selection(&self, sender: &ComponentSender<Self>) {
        let generation = self.close_generation.get();
        let input = sender.input_sender().clone();
        // CSS selection takes 40 ms; close after a short completion margin.
        let delay = if self.rows.settings().is_gtk_enable_animations() {
            Duration::from_millis(50)
        } else {
            Duration::ZERO
        };
        gtk::glib::timeout_add_local_once(delay, move || {
            let _ = input.send(Input::FinishSelection(generation));
        });
    }

    fn sync_rows(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.rows.first_child() {
            self.rows.remove(&child);
        }
        let Some(layouts) = &self.layouts else {
            return;
        };
        for (index, name) in layouts.names.iter().enumerate() {
            let Ok(index) = u8::try_from(index) else {
                break;
            };
            let row = gtk::Button::new();
            row.add_css_class("topbar-layout-choice");
            row.set_focus_on_click(false);
            row.set_sensitive(self.wm_commands.is_some());
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            let label = gtk::Label::new(Some(name));
            label.set_hexpand(true);
            label.set_xalign(0.0);
            let check = gtk::Image::from_icon_name(crate::ui::icon_names::CHECK);
            check.set_pixel_size(16);
            content.append(&label);
            content.append(&check);
            row.set_child(Some(&content));
            let input = sender.input_sender().clone();
            let names = layouts.names.clone();
            row.connect_clicked(move |_| {
                let _ = input.send(Input::Select {
                    index,
                    names: names.clone(),
                });
            });
            self.rows.append(&row);
        }
        self.sync_selection();
    }

    fn sync_selection(&self) {
        let mut child = self.rows.first_child();
        let mut index = 0;
        while let Some(row) = child {
            let selected = self
                .layouts
                .as_ref()
                .is_some_and(|s| usize::from(s.current_idx) == index);
            row.set_css_classes(if selected {
                &["topbar-layout-choice", "selected"]
            } else {
                &["topbar-layout-choice"]
            });
            if let Some(check) = row.first_child().and_then(|content| content.last_child()) {
                check.set_opacity(if selected { 1.0 } else { 0.0 });
            }
            child = row.next_sibling();
            index += 1;
        }
    }
}
