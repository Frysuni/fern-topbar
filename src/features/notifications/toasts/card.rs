use super::{CARD_TRANSITION_MS, Notification, same_notification};
use crate::backend::notifications::Urgency;
use crate::features::notifications::icon;
use crate::features::notifications::urgency_class;
use crate::ui::{
    core::{Button, motion},
    icon_names,
};
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use std::time::Duration;

const TEXT_WIDTH_CHARS: i32 = 36;

pub struct ToastCard {
    notification: Notification,
    expanded: bool,
    can_expand: bool,
    layout_ready: bool,
    hovered: bool,
    leaving: bool,
}

#[derive(Debug)]
pub enum Input {
    Hover(bool),
    ToggleExpansion,
    CheckLayout,
    Reveal,
    Replace(Notification),
    Leave,
}

#[derive(Debug)]
pub enum Output {
    Activate(u32),
    Dismiss(u32),
    Remove(u32),
    LayoutChanged,
    LayoutReady(u32),
}

#[relm4::factory(pub)]
impl FactoryComponent for ToastCard {
    type Init = Notification;
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Revealer {
            set_transition_type: gtk::RevealerTransitionType::SlideUp,
            set_transition_duration: 0,
            #[watch]
            set_reveal_child: !self.leaving,

            #[name = "card"]
            gtk::Box {
                #[watch]
                set_css_classes: &self.card_classes(),
                set_orientation: gtk::Orientation::Horizontal,

                add_controller = gtk::EventControllerMotion {
                    connect_enter[sender] => move |_, _, _| sender.input(Input::Hover(true)),
                    connect_leave[sender] => move |_| sender.input(Input::Hover(false)),
                },
                add_controller = gtk::GestureClick {
                    set_button: gtk::gdk::BUTTON_SECONDARY,
                    set_propagation_phase: gtk::PropagationPhase::Capture,
                    connect_pressed[sender, id] => move |gesture, _, _, _| {
                        let _ = sender.output(Output::Dismiss(id));

                        gesture.set_state(gtk::EventSequenceState::Claimed);
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,

                    #[template]
                    Button {
                        add_css_class: "topbar-toast-open",
                        #[watch]
                        set_class_active: ("activatable", self.notification.default_action),
                        set_has_frame: false,
                        set_hexpand: true,
                        #[watch]
                        set_cursor_from_name: self.notification.default_action.then_some("pointer"),
                        #[watch]
                        set_tooltip_text: self.notification.default_action.then_some("Open application"),
                        connect_clicked[sender, id] => move |_| {
                            let _ = sender.output(Output::Activate(id));
                        },

                        gtk::Box {
                            add_css_class: "topbar-toast-content",
                            set_orientation: gtk::Orientation::Horizontal,

                            gtk::Image {
                                add_css_class: "topbar-toast-icon",
                                set_valign: gtk::Align::Start,
                                #[watch]
                                set_from_gicon: &icon::resolve(&self.notification),
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 3,
                                set_hexpand: true,

                                gtk::Label {
                                    add_css_class: "topbar-toast-app",
                                    set_halign: gtk::Align::Start,
                                    set_max_width_chars: TEXT_WIDTH_CHARS,
                                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                                    set_single_line_mode: true,
                                    #[watch]
                                    set_label: &self.notification.app,
                                    #[watch]
                                    set_tooltip_text: (!self.notification.app.is_empty())
                                        .then_some(self.notification.app.as_str()),
                                    #[watch]
                                    set_visible: !self.notification.app.is_empty(),
                                },

                                #[name = "title"]
                                gtk::Label {
                                    add_css_class: "topbar-toast-title",
                                    set_halign: gtk::Align::Fill,
                                    set_wrap: true,
                                    set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                    set_max_width_chars: TEXT_WIDTH_CHARS,
                                    set_xalign: 0.0,
                                    #[watch]
                                    set_label: &self.notification.summary,
                                    #[watch]
                                    set_lines: if self.expanded { -1 } else { 2 },
                                    #[watch]
                                    set_ellipsize: if self.expanded {
                                        gtk::pango::EllipsizeMode::None
                                    } else {
                                        gtk::pango::EllipsizeMode::End
                                    },
                                    #[watch]
                                    set_tooltip_text: Some(&self.notification.summary),
                                },

                                #[name = "body"]
                                gtk::Label {
                                    add_css_class: "topbar-toast-body",
                                    set_halign: gtk::Align::Fill,
                                    set_wrap: true,
                                    set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                    set_max_width_chars: TEXT_WIDTH_CHARS,
                                    set_xalign: 0.0,
                                    #[watch]
                                    set_label: &self.notification.body,
                                    #[watch]
                                    set_lines: if self.expanded { -1 } else { 3 },
                                    #[watch]
                                    set_ellipsize: if self.expanded {
                                        gtk::pango::EllipsizeMode::None
                                    } else {
                                        gtk::pango::EllipsizeMode::End
                                    },
                                    #[watch]
                                    set_tooltip_text: (!self.notification.body.is_empty())
                                        .then_some(self.notification.body.as_str()),
                                    #[watch]
                                    set_visible: !self.notification.body.is_empty(),
                                },
                            },
                        },
                    },

                    #[template]
                    Button {
                        add_css_class: "topbar-toast-hint",
                        set_halign: gtk::Align::Start,
                        set_cursor_from_name: Some("pointer"),
                        #[watch]
                        set_visible: self.expanded || self.can_expand,
                        #[watch]
                        set_label: if self.expanded { "Show less" } else { "Show more" },
                        connect_clicked => Input::ToggleExpansion,
                    },
                },

                #[template]
                Button {
                    add_css_class: "topbar-toast-close",
                    set_icon_name: icon_names::CLOSE,
                    set_tooltip_text: Some("Dismiss notification"),
                    set_valign: gtk::Align::Start,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked[sender, id] => move |_| {
                        let _ = sender.output(Output::Dismiss(id));
                    },
                },
            },
        }
    }

    fn init_model(
        notification: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self {
            notification,
            expanded: false,
            can_expand: false,
            layout_ready: false,
            hovered: false,
            leaving: false,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &gtk::Widget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let id = self.notification.id;
        let widgets = view_output!();

        let input = sender.input_sender().clone();

        widgets.card.add_tick_callback(move |widget, _| {
            if widget.width() == 0 {
                return gtk::glib::ControlFlow::Continue;
            }

            let _ = input.send(Input::CheckLayout);

            gtk::glib::ControlFlow::Break
        });

        let input = sender.input_sender().clone();

        widgets
            .card
            .connect_notify_local(Some("width"), move |card, _| {
                if card.width() > 0 {
                    let _ = input.send(Input::CheckLayout);
                }
            });

        root.set_transition_duration(CARD_TRANSITION_MS);
        // The host reveals the card after checking text layout and fitting the viewport.
        widgets.card.set_opacity(0.0);

        widgets
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        let previous_expanded = self.expanded;
        let previous_can_expand = self.can_expand;
        let replaced = matches!(&message, Input::Replace(_));
        let first_layout = matches!(&message, Input::CheckLayout) && !self.layout_ready;

        match message {
            Input::Hover(hovered) => self.hovered = hovered,
            Input::ToggleExpansion => {
                if self.can_expand || self.expanded {
                    self.can_expand = true;
                    self.expanded = !self.expanded;
                }
            }
            Input::Reveal => {
                motion::reveal(&widgets.card);
                return;
            }
            Input::CheckLayout => {
                self.layout_ready = true;

                if !self.expanded {
                    self.can_expand = widgets.title.layout().is_ellipsized()
                        || (widgets.body.is_visible() && widgets.body.layout().is_ellipsized());
                }
            }
            Input::Replace(notification) => {
                self.notification = notification;
                self.expanded = false;
                self.can_expand = false;
                self.leaving = false;
            }
            Input::Leave => {
                if !self.leaving {
                    self.leaving = true;

                    let output = sender.output_sender().clone();
                    let id = self.notification.id;

                    gtk::glib::timeout_add_local_once(
                        Duration::from_millis(u64::from(CARD_TRANSITION_MS)),
                        move || {
                            let _ = output.send(Output::Remove(id));
                        },
                    );
                }
            }
        }

        self.update_view(widgets, sender.clone());

        if first_layout {
            let _ = sender.output(Output::LayoutReady(self.notification.id));
        } else if previous_expanded != self.expanded
            || previous_can_expand != self.can_expand
            || replaced
        {
            let _ = sender.output(Output::LayoutChanged);
        }
    }
}

impl ToastCard {
    fn card_classes(&self) -> Vec<&'static str> {
        let mut classes = vec![
            "topbar-toast-card",
            urgency_class(self.notification.urgency),
        ];

        for (active, class) in [
            (self.hovered, "hovered"),
            (self.expanded, "expanded"),
            (self.leaving, "leaving"),
        ] {
            if active {
                classes.push(class);
            }
        }

        classes
    }

    pub fn is_critical(&self) -> bool {
        self.notification.urgency == Urgency::Critical
    }

    pub fn id(&self) -> u32 {
        self.notification.id
    }

    pub fn layout_ready(&self) -> bool {
        self.layout_ready
    }

    pub fn leaving(&self) -> bool {
        self.leaving
    }

    pub fn matches(&self, notification: &Notification) -> bool {
        same_notification(&self.notification, notification)
    }

    #[cfg(test)]
    pub fn expanded(&self) -> bool {
        self.expanded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::notifications::toasts::TOAST_WIDTH;

    #[gtk::test]
    fn laid_out_text_controls_card_expansion() {
        relm4::set_global_css(include_str!(concat!(env!("OUT_DIR"), "/styles.css")));

        let mut cards: FactoryVecDeque<ToastCard> =
            FactoryVecDeque::builder().launch_default().detach();
        cards.guard().push_back(Notification {
            id: 1,
            app: String::new(),
            icon: String::new(),
            desktop_entry: None,
            summary: "Notification".into(),
            body: "漢字".repeat(60),
            default_action: true,
            resident: false,
            urgency: Urgency::Normal,
            request_attention: false,
        });

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_width_request(TOAST_WIDTH);
        container.append(cards.widget());

        let window = gtk::Window::new();
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_child(Some(&container));
        window.present();

        let main_loop = gtk::glib::MainLoop::new(None, false);
        let settle = || {
            let quit = main_loop.clone();

            gtk::glib::timeout_add_local_once(Duration::from_millis(100), move || quit.quit());
            main_loop.run();
        };

        settle();

        let card = cards.widget().first_child().unwrap().first_child().unwrap();
        let content = card.first_child().unwrap();
        let open = content
            .first_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();

        let text = open.child().unwrap().last_child().unwrap();
        let title = text
            .first_child()
            .unwrap()
            .next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();

        let body = title
            .next_sibling()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();

        let hint = content
            .last_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();

        assert!(cards[0].layout_ready());
        assert!(body.layout().is_ellipsized());
        assert!(hint.get_visible());
        assert_eq!(hint.label().as_deref(), Some("Show more"));

        let width = card.width();
        let collapsed_height = card.measure(gtk::Orientation::Vertical, width).1;

        hint.emit_clicked();
        settle();

        assert!(cards[0].expanded());
        assert_eq!(hint.label().as_deref(), Some("Show less"));
        assert_eq!(title.lines(), -1);
        assert_eq!(body.lines(), -1);
        assert!(!body.layout().is_ellipsized());
        assert!(card.measure(gtk::Orientation::Vertical, width).1 > collapsed_height);

        hint.emit_clicked();
        settle();

        assert!(!cards[0].expanded());
        assert_eq!(title.lines(), 2);
        assert_eq!(body.lines(), 3);
        assert!(body.layout().is_ellipsized());
        assert_eq!(hint.label().as_deref(), Some("Show more"));

        cards.send(
            0,
            Input::Replace(Notification {
                id: 1,
                app: String::new(),
                icon: String::new(),
                desktop_entry: None,
                summary: "Short".into(),
                body: "Fits".into(),
                default_action: true,
                resident: false,
                urgency: Urgency::Normal,
                request_attention: false,
            }),
        );
        settle();

        assert!(!cards[0].expanded());
        assert!(!body.layout().is_ellipsized());
        assert!(!hint.get_visible());

        window.destroy();
    }
}
