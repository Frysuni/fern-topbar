use crate::backend::notifications::Notification;
#[cfg(test)]
use crate::backend::notifications::Urgency;
use crate::ui::{
    core::{
        Button, MenuButtonStyle, MenuPopover, PanelMenuButton, PopoverScope, PopoverStyle,
        PopupRegistration, UiScale,
    },
    icon_names,
};
use gtk4_layer_shell::LayerShell;
use relm4::factory::FactoryVecDeque;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use row::NotificationRow;

mod row;

const MAX_LIST_HEIGHT: i32 = 500;
const POPOVER_OFFSET: i32 = 12;
const POPOVER_CHROME_HEIGHT: i32 = 88;

pub struct Menu {
    _popup: Option<PopupRegistration>,
    items: Vec<Notification>,
    rows: FactoryVecDeque<NotificationRow>,
}

#[derive(Debug)]
pub enum Input {
    Items(Vec<Notification>),
    Activate(u32),
    Dismiss(u32),
    Clear,
    History,
    Popup(bool),
}

#[derive(Debug)]
pub enum Output {
    Activate(u32),
    Dismiss(u32),
    Clear,
    History,
    Popup(bool),
}

#[relm4::component(pub)]
impl Component for Menu {
    type Init = PopoverScope;
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    view! {
        #[root]
        #[template]
        PanelMenuButton(MenuButtonStyle::Icon) {
            set_icon_name: icon_names::NOTIFICATIONS,
            #[watch]
            set_tooltip_text: Some(&format!("{} notifications", model.items.len())),
            connect_active_notify[sender] => move |button| sender.input(Input::Popup(button.is_active())),
            #[wrap(Some)]
            #[template]
            set_popover = &MenuPopover(PopoverStyle::Menu) {
                gtk::Box {
                    add_css_class: "topbar-notification-menu",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 0,
                    gtk::Box {
                        add_css_class: "topbar-notification-header",
                        set_spacing: 8,
                        gtk::Label { set_label: "Notifications", add_css_class: "topbar-menu-title", set_hexpand: true, set_halign: gtk::Align::Start },
                        #[name = "history"]
                        #[template]
                        Button {
                            add_css_class: "topbar-toast-control",
                            add_css_class: "topbar-notification-action",
                            set_label: "View history · 0",
                            set_sensitive: false,
                            connect_clicked => Input::History,
                        },
                        #[template]
                        Button {
                            add_css_class: "topbar-notification-clear",
                            add_css_class: "topbar-notification-action",
                            set_label: "Clear all",
                            #[watch]
                            set_sensitive: !model.items.is_empty(),
                            connect_clicked => Input::Clear,
                        },
                    },
                    #[name = "scroll"]
                    gtk::ScrolledWindow {
                        set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                        set_propagate_natural_height: true,
                        gtk::Box {
                            add_css_class: "topbar-notification-list",
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 0,
                            gtk::Label {
                                add_css_class: "topbar-empty",
                                set_label: "No notifications",
                                #[watch]
                                set_visible: model.items.is_empty(),
                            },
                            #[local_ref]
                            rows -> gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 0,
                                #[watch]
                                set_visible: !model.items.is_empty(),
                            },
                        },
                    },
                },
            },
        }
    }
    fn init(
        popovers: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let rows = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), |message| message);
        let mut model = Self {
            _popup: None,
            items: Vec::new(),
            rows,
        };

        let rows = model.rows.widget();
        let widgets = view_output!();

        model._popup = Some(popovers.register_button(root.widget()));
        update_scroll_limit(root.widget(), &widgets.scroll);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            Input::Items(items) => {
                self.items = items;
                widgets
                    .history
                    .set_label(&format!("View history · {}", self.items.len()));
                widgets.history.set_sensitive(!self.items.is_empty());
                update_scroll_limit(root.widget(), &widgets.scroll);
                self.update_rows();
            }
            Input::Activate(id) => {
                let _ = sender.output(Output::Activate(id));
            }
            Input::Dismiss(id) => {
                let _ = sender.output(Output::Dismiss(id));
            }
            Input::Clear => {
                let _ = sender.output(Output::Clear);
            }
            Input::History => {
                if !self.items.is_empty() {
                    root.widget().popdown();

                    let _ = sender.output(Output::History);
                }
            }
            Input::Popup(open) => {
                if open {
                    update_scroll_limit(root.widget(), &widgets.scroll);
                }

                let _ = sender.output(Output::Popup(open));
            }
        }

        self.update_view(widgets, sender);
    }
}

impl Menu {
    fn update_rows(&mut self) {
        let visible: Vec<_> = self.items.iter().rev().take(8).cloned().collect();
        let mut rows = self.rows.guard();

        let mut index = 0;

        while index < rows.len() {
            if visible.iter().any(|item| item.id == rows[index].id()) {
                index += 1;
            } else {
                rows.remove(index);
            }
        }

        for (target, item) in visible.into_iter().enumerate() {
            if let Some(current) = (0..rows.len()).find(|&index| rows[index].id() == item.id) {
                rows.move_to(current, target);
                rows[target].replace(item);
            } else {
                rows.insert(target, item);
            }
        }
    }
}

fn update_scroll_limit(root: &gtk::MenuButton, scroll: &gtk::ScrolledWindow) {
    let scale = UiScale::for_widget(scroll);
    let monitor = root
        .root()
        .and_then(|root| root.downcast::<gtk::Window>().ok())
        .and_then(|window| {
            window.monitor().or_else(|| {
                gtk::prelude::WidgetExt::display(&window)
                    .monitors()
                    .item(0)
                    .and_downcast::<gtk::gdk::Monitor>()
            })
        });

    let height = monitor.map_or(MAX_LIST_HEIGHT, |monitor| {
        let panel_height = root.root().map_or(34, |root| root.height().max(34));

        scale
            .units(
                monitor.geometry().height()
                    - panel_height
                    - scale.pixels(POPOVER_OFFSET + POPOVER_CHROME_HEIGHT),
            )
            .clamp(1, MAX_LIST_HEIGHT)
    });

    scroll.set_max_content_height(height);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32, summary: &str) -> Notification {
        Notification {
            id,
            app: "Test".into(),
            icon: String::new(),
            desktop_entry: None,
            summary: summary.into(),
            body: String::new(),
            default_action: true,
            resident: false,
            urgency: Urgency::Normal,
            request_attention: false,
        }
    }

    #[gtk::test]
    fn menu_rows_keep_their_widgets_on_updates() {
        let (sender, _receiver) = relm4::channel::<Input>();
        let rows = FactoryVecDeque::builder()
            .launch_default()
            .forward(&sender, |message| message);
        let mut model = Menu {
            _popup: None,
            items: vec![item(1, "First"), item(2, "Second")],
            rows,
        };

        model.update_rows();

        assert_eq!(model.rows.len(), 2);
        assert_eq!(model.rows[0].id(), 2);

        let first = model.rows.widget().last_child().unwrap();

        model.items = vec![item(1, "Updated"), item(3, "Third")];
        model.update_rows();

        assert_eq!(model.rows.len(), 2);
        assert_eq!(model.rows[0].id(), 3);
        assert_eq!(first, model.rows.widget().last_child().unwrap());
        assert_eq!(model.rows[1].summary(), "Updated");
    }
}
