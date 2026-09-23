use super::{Volume, VolumeControlOptions, backend};
use crate::ui::core::{
    Button, MenuButtonStyle, MenuPopover, PanelMenuButton, PercentScale, PercentValue,
    PopoverScope, PopoverStyle, PopupRegistration,
};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

#[derive(Clone, Debug)]
pub struct Sound {
    pub level: u8,
    pub muted: bool,
}

impl Sound {
    fn volume(&self) -> Volume {
        Volume {
            level: f64::from(self.level) / 100.0,
            muted: self.muted,
        }
    }
}

pub struct VolumeControl {
    _popup: Option<PopupRegistration>,
    spec: VolumeControlOptions,
    sound: Option<Sound>,
    controls: backend::Controls,
    pending: Option<u8>,
}

pub struct VolumeControlInit {
    pub popovers: PopoverScope,
    pub spec: VolumeControlOptions,
    pub controls: backend::Controls,
}

#[derive(Debug)]
pub enum Input {
    Sound(Option<Sound>),
    SliderChanged(u8),
    ToggleMute,
    Popup(bool),
    Scroll(i8),
}

#[relm4::component(pub)]
impl Component for VolumeControl {
    type Init = VolumeControlInit;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        #[template]
        PanelMenuButton(MenuButtonStyle::Icon) {
            #[watch]
            set_css_classes: &model.button_css_classes(),
            #[watch]
            set_visible: model.sound.is_some(),
            #[watch]
            set_icon_name: model.icon(),
            set_tooltip_text: Some(model.spec.button_tooltip),
            connect_active_notify[sender] => move |button| sender.input(Input::Popup(button.is_active())),
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
            set_popover = &MenuPopover(PopoverStyle::Mixer) {
                gtk::Box {
                    add_css_class: model.spec.menu_class,
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box {
                        add_css_class: model.spec.row_class,
                        set_spacing: 8,

                        #[template]
                        Button {
                            #[watch]
                            set_css_classes: model.mute_button_css_classes(),
                            set_tooltip_text: Some(model.spec.mute_tooltip),
                            #[watch]
                            set_icon_name: model.icon(),
                            connect_clicked => Input::ToggleMute,
                        },

                        #[name = "scale"]
                        #[template]
                        PercentScale(0) {
                            set_round_digits: 0,
                            set_tooltip_text: Some(model.spec.scale_tooltip),
                            connect_change_value[sender] => move |_, _, value| {
                                sender.input(Input::SliderChanged(value.round().clamp(0.0, 100.0) as u8));

                                gtk::glib::Propagation::Proceed
                            },
                        },

                        #[template]
                        PercentValue {
                            #[watch]
                            set_label: &model.sound.as_ref().map_or(String::new(), |sound| format!("{}%", sound.level)),
                        },
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
        let VolumeControlInit {
            spec,
            controls,
            popovers,
        } = init;

        let mut model = Self {
            _popup: None,
            spec,
            sound: None,
            controls,
            pending: None,
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
        _root: &Self::Root,
    ) {
        match message {
            Input::Sound(sound) => {
                if !self.apply_sound_update(widgets, sound) {
                    return;
                }
            }
            Input::SliderChanged(level) => self.apply_slider_change(level),
            Input::ToggleMute => self.controls.toggle_mute(self.spec.device),
            Input::Popup(open) => {
                if !open {
                    self.pending = None;
                }
            }
            Input::Scroll(delta) => self.apply_scroll(delta),
        }

        self.update_view(widgets, sender);
    }
}

impl VolumeControl {
    fn apply_sound_update(
        &mut self,
        widgets: &mut <Self as Component>::Widgets,
        sound: Option<Sound>,
    ) -> bool {
        if let Some(expected) = self.pending {
            match &sound {
                Some(current) if current.level != expected => return false,
                _ => self.pending = None,
            }
        }

        if let Some(sound) = &sound {
            widgets.scale.widget().set_value(f64::from(sound.level));
        }

        self.sound = sound;

        true
    }

    fn apply_slider_change(&mut self, level: u8) {
        if let Some(sound) = &mut self.sound
            && sound.level != level
        {
            sound.level = level;
            self.pending = Some(level);
            self.controls.set_volume(self.spec.device, level);
        }
    }

    fn apply_scroll(&mut self, delta: i8) {
        if let Some(sound) = &mut self.sound {
            sound.level = sound.level.saturating_add_signed(delta).min(100);
            self.pending = Some(sound.level);
            self.controls.set_volume(self.spec.device, sound.level);
        }
    }

    fn button_css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["topbar-feature-button", self.spec.button_class];

        if self.is_muted() {
            classes.push("muted");
        }

        classes
    }

    fn mute_button_css_classes(&self) -> &'static [&'static str] {
        if self.is_muted() {
            &["flat", "topbar-mute-button", "muted"]
        } else {
            &["flat", "topbar-mute-button"]
        }
    }

    fn is_muted(&self) -> bool {
        self.sound
            .as_ref()
            .is_some_and(|sound| sound.volume().is_muted())
    }

    fn icon(&self) -> &'static str {
        self.sound.as_ref().map_or(self.spec.default_icon, |sound| {
            (self.spec.icon)(sound.volume())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::sound::{
            AudioDevice,
            backend::tests::{Message, controls},
        },
        ui::icon_names,
    };

    #[gtk::test]
    fn optimistic_level_changes_update_effective_mute_without_latching_it() {
        let (controls, commands) = controls();
        let component = VolumeControl::builder()
            .launch(VolumeControlInit {
                popovers: PopoverScope::default(),
                controls,
                spec: VolumeControlOptions {
                    device: AudioDevice::Output,
                    default_icon: icon_names::SPEAKER_MAX,
                    button_tooltip: "Volume",
                    mute_tooltip: "Mute",
                    scale_tooltip: "Volume",
                    button_class: "topbar-audio-button",
                    menu_class: "topbar-audio-menu",
                    row_class: "topbar-audio-row",
                    icon: |_| icon_names::SPEAKER_MAX,
                },
            })
            .detach();

        let context = gtk::glib::MainContext::default();
        let send = |input| {
            component.emit(input);

            while context.pending() {
                context.iteration(false);
            }
        };

        send(Input::Sound(Some(Sound {
            level: 5,
            muted: false,
        })));
        assert!(!component.widget().widget().has_css_class("muted"));
        assert!(commands.try_recv().is_err());

        send(Input::SliderChanged(0));
        assert!(component.widget().widget().has_css_class("muted"));
        assert!(matches!(
            commands.try_recv(),
            Ok(Message::SetVolume(AudioDevice::Output, 0))
        ));

        // A stale server snapshot must not undo the optimistic slider change.
        send(Input::Sound(Some(Sound {
            level: 5,
            muted: false,
        })));
        assert_eq!(component.model().sound.as_ref().unwrap().level, 0);
        assert_eq!(component.model().pending, Some(0));

        send(Input::Sound(Some(Sound {
            level: 0,
            muted: false,
        })));
        assert_eq!(component.model().pending, None);

        send(Input::SliderChanged(5));
        assert!(!component.widget().widget().has_css_class("muted"));
        assert!(matches!(
            commands.try_recv(),
            Ok(Message::SetVolume(AudioDevice::Output, 5))
        ));

        send(Input::Sound(Some(Sound {
            level: 5,
            muted: true,
        })));
        assert!(component.widget().widget().has_css_class("muted"));

        send(Input::Scroll(5));
        assert!(component.widget().widget().has_css_class("muted"));
        assert_eq!(component.model().sound.as_ref().unwrap().level, 10);
        assert!(matches!(
            commands.try_recv(),
            Ok(Message::SetVolume(AudioDevice::Output, 10))
        ));

        send(Input::Sound(None));
        assert!(!component.widget().widget().get_visible());
        assert_eq!(component.model().pending, None);

        send(Input::SliderChanged(50));
        assert!(commands.try_recv().is_err());
    }
}
