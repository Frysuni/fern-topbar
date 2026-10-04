use super::backend;
use crate::ui::{
    core::{
        MenuButtonStyle, MenuPopover, PanelMenuButton, PercentScale, PercentValue, PopoverScope,
        PopoverStyle, PopupRegistration,
    },
    icon_names,
};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

pub struct Brightness {
    show_percent: bool,
    _popup: Option<PopupRegistration>,
    value: Option<backend::Brightness>,
    commands: backend::Controls,
}

pub struct BrightnessInit {
    pub show_percent: bool,
    pub popovers: PopoverScope,
    pub commands: backend::Controls,
}

#[derive(Debug)]
pub enum Input {
    Changed(Option<backend::Brightness>),
    Slider(u8),
    Scroll(i8),
}

#[relm4::component(pub)]
impl Component for Brightness {
    type Init = BrightnessInit;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        #[template]
        PanelMenuButton(MenuButtonStyle::Icon) {
            #[watch]
            set_visible: model.value.is_some(),
            set_class_active: ("topbar-percent-button", model.show_percent),
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_spacing: 4,
                gtk::Image { set_icon_name: Some(icon_names::BRIGHTNESS) },
                gtk::Label {
                    set_visible: model.show_percent,
                    #[watch]
                    set_label: &model.value.as_ref().map_or(String::new(), |value| format!("{}%", value.percent)),
                },
            },
            set_tooltip_text: Some("Brightness · scroll to adjust"),
            add_controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL) {
                connect_scroll[sender] => move |_, _, dy| {
                    if dy != 0.0 {
                        sender.input(Input::Scroll(if dy < 0.0 { 5 } else { -5 }));
                    }

                    gtk::glib::Propagation::Stop
                },
            },
            #[wrap(Some)]
            #[template]
            set_popover = &MenuPopover(PopoverStyle::Menu) {
                gtk::Box {
                    add_css_class: "topbar-slider-row",
                    set_spacing: 8,
                    gtk::Image { set_icon_name: Some(icon_names::BRIGHTNESS) },
                    #[name = "scale"]
                    #[template]
                    PercentScale(1) {
                        connect_change_value[sender] => move |_, _, value| {
                            sender.input(Input::Slider(value.round().clamp(1.0, 100.0) as u8));

                            gtk::glib::Propagation::Proceed
                        },
                    },
                    #[template]
                    PercentValue {
                        #[watch]
                        set_label: &model.value.as_ref().map_or(String::new(), |value| format!("{}%", value.percent)),
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let BrightnessInit {
            commands,
            popovers,
            show_percent,
        } = init;
        let mut model = Self {
            show_percent,
            _popup: None,
            value: None,
            commands,
        };

        let widgets = view_output!();

        model._popup = Some(popovers.register_button(root.widget()));

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
            Input::Changed(value) => {
                if let Some(value) = &value {
                    widgets.scale.widget().set_value(f64::from(value.percent));
                }

                self.value = value;
            }
            Input::Slider(percent) => {
                if let Some(value) = &mut self.value
                    && value.percent != percent
                {
                    value.percent = percent;
                    self.commands.set(backend::Command::SetBrightness(percent));
                }
            }
            Input::Scroll(delta) => {
                self.commands.set(backend::Command::AdjustBrightness(delta));
            }
        }

        self.update_view(widgets, sender);
    }
}
