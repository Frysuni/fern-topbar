use super::Input;
use crate::backend::notifications::Notification;
#[cfg(test)]
use crate::backend::notifications::Urgency;
use crate::features::notifications::icon;
use crate::features::notifications::urgency_class;
use crate::ui::{
    core::{Button, motion},
    icon_names,
};
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryView};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

const TEXT_WIDTH_CHARS: i32 = 30;

pub struct NotificationRow {
    item: Notification,
}

#[relm4::factory(pub)]
impl FactoryComponent for NotificationRow {
    type Init = Notification;
    type Input = ();
    type Output = Input;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Overlay {
            #[watch]
            set_css_classes: &["topbar-notification-row", urgency_class(self.item.urgency)],

            add_controller = gtk::GestureClick {
                set_button: gtk::gdk::BUTTON_SECONDARY,
                set_propagation_phase: gtk::PropagationPhase::Capture,
                connect_pressed[sender, id] => move |gesture, _, _, _| {
                    let _ = sender.output(Input::Dismiss(id));

                    gesture.set_state(gtk::EventSequenceState::Claimed);
                },
            },

            #[name = "open"]
            #[wrap(Some)]
            #[template]
            set_child = &Button {
                add_css_class: "topbar-notification-open",
                #[watch]
                set_class_active: ("activatable", self.item.default_action),
                set_has_frame: false,
                set_hexpand: true,
                #[watch]
                set_cursor_from_name: self.item.default_action.then_some("pointer"),
                #[watch]
                set_tooltip_text: self.item.default_action.then_some("Open application"),
                connect_clicked[sender, id] => move |_| {
                    let _ = sender.output(Input::Activate(id));
                },

                gtk::Box {
                    add_css_class: "topbar-notification-content",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_margin_end: 32,

                    gtk::Image {
                        add_css_class: "topbar-notification-icon",
                        set_valign: gtk::Align::Start,
                        #[watch]
                        set_from_gicon: &icon::resolve(&self.item),
                    },

                    gtk::Box {
                        add_css_class: "topbar-notification-copy",
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,

                        gtk::Label {
                            add_css_class: "topbar-notification-app",
                            set_halign: gtk::Align::Start,
                            set_max_width_chars: TEXT_WIDTH_CHARS,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_single_line_mode: true,
                            #[watch]
                            set_label: &self.item.app,
                            #[watch]
                            set_tooltip_text: (!self.item.app.is_empty())
                                .then_some(self.item.app.as_str()),
                            #[watch]
                            set_visible: !self.item.app.is_empty(),
                        },

                        gtk::Label {
                            add_css_class: "topbar-notification-title",
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::WordChar,
                            set_max_width_chars: TEXT_WIDTH_CHARS,
                            set_lines: 2,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            #[watch]
                            set_label: &self.item.summary,
                            #[watch]
                            set_tooltip_text: Some(&self.item.summary),
                        },

                        gtk::Label {
                            add_css_class: "topbar-notification-body",
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::WordChar,
                            set_max_width_chars: TEXT_WIDTH_CHARS,
                            set_lines: 3,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            #[watch]
                            set_label: &self.item.body,
                            #[watch]
                            set_tooltip_text: (!self.item.body.is_empty())
                                .then_some(self.item.body.as_str()),
                            #[watch]
                            set_visible: !self.item.body.is_empty(),
                        },
                    },
                },
            },

            #[name = "dismiss"]
            #[template]
            add_overlay = &Button {
                add_css_class: "topbar-dismiss",
                add_css_class: "topbar-notification-dismiss",
                set_valign: gtk::Align::Start,
                set_halign: gtk::Align::End,
                set_margin_top: 10,
                set_margin_end: 10,
                set_icon_name: icon_names::CLOSE,
                set_tooltip_text: Some("Dismiss notification"),
                connect_clicked[sender, id] => move |_| {
                    let _ = sender.output(Input::Dismiss(id));
                },
            },
        }
    }

    fn init_model(item: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { item }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let id = self.item.id;
        let widgets = view_output!();

        motion::animate_appearance(&root);

        widgets
    }
}

impl NotificationRow {
    pub fn id(&self) -> u32 {
        self.item.id
    }

    pub fn replace(&mut self, item: Notification) {
        self.item = item;
    }

    #[cfg(test)]
    pub fn summary(&self) -> &str {
        &self.item.summary
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Menu, Output};
    use super::*;
    use crate::ui::core::PopoverScope;
    use futures_util::{
        FutureExt,
        future::{Either, select},
    };
    use std::{thread, time::Duration};

    #[gtk::test]
    fn activation_covers_the_card_and_dismiss_is_a_separate_overlay() {
        relm4::set_global_css(include_str!(concat!(env!("OUT_DIR"), "/styles.css")));
        relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);

        let (sender, receiver) = relm4::channel::<Output>();
        let component = Menu::builder()
            .launch(PopoverScope::default())
            .forward(&sender, |output| output);
        component.emit(Input::Items(vec![Notification {
            id: 7,
            app: "Test".into(),
            icon: String::new(),
            desktop_entry: None,
            summary: "Notification".into(),
            body: "Body".into(),
            default_action: true,
            resident: false,
            urgency: Urgency::Normal,
            request_attention: false,
        }]));

        let context = gtk::glib::MainContext::default();

        while context.pending() {
            context.iteration(false);
        }

        let rows = component.model().rows.widget().clone();
        let overlay = rows
            .first_child()
            .unwrap()
            .downcast::<gtk::Overlay>()
            .unwrap();

        let open = overlay.child().unwrap().downcast::<gtk::Button>().unwrap();
        let dismiss = open
            .next_sibling()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();

        assert_eq!(
            open.parent().as_ref(),
            Some(overlay.upcast_ref::<gtk::Widget>())
        );
        assert_eq!(
            dismiss.parent().as_ref(),
            Some(overlay.upcast_ref::<gtk::Widget>())
        );

        let button = component.widget().widget();
        let popover = button.popover().unwrap();
        let window = gtk::Window::new();
        window.set_child(Some(button));
        window.present();
        button.popup();

        for _ in 0..20 {
            while context.pending() {
                context.iteration(false);
            }

            thread::sleep(Duration::from_millis(10));
        }

        let hit = overlay
            .pick(
                3.0,
                f64::from(overlay.height()) / 2.0,
                gtk::PickFlags::DEFAULT,
            )
            .expect("Notification padding must be clickable");

        assert!(hit == open || hit.is_ancestor(&open));

        let bounds = open.compute_bounds(&overlay).unwrap();

        assert_eq!(bounds.x(), 0.0);
        assert_eq!(bounds.y(), 0.0);
        assert_eq!(bounds.width(), overlay.width() as f32);
        assert_eq!(bounds.height(), overlay.height() as f32);

        if let Ok(path) = std::env::var("NOTIFICATION_UI_PREVIEW") {
            let snapshot = gtk::Snapshot::new();
            let paintable = gtk::WidgetPaintable::new(Some(&popover));
            paintable.snapshot(&snapshot, popover.width().into(), popover.height().into());

            let node = snapshot.to_node().unwrap();
            let renderer = popover.native().unwrap().renderer().unwrap();

            renderer
                .render_texture(&node, None)
                .save_to_png(path)
                .unwrap();
        }

        let next = || {
            context.block_on(async {
                let received = Box::pin(receiver.recv());
                let deadline = Box::pin(gtk::glib::timeout_future(Duration::from_secs(1)));

                match select(received, deadline).await {
                    Either::Left((Some(output), _)) => output,
                    _ => panic!("Notification row did not emit its action"),
                }
            })
        };

        while receiver.recv().now_or_never().flatten().is_some() {}

        open.emit_clicked();

        assert!(matches!(next(), Output::Activate(7)));

        dismiss.emit_clicked();

        assert!(matches!(next(), Output::Dismiss(7)));
        assert!(receiver.recv().now_or_never().flatten().is_none());

        button.popdown();
        window.close();
    }
}
