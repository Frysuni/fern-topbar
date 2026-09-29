use crate::ui::core::StatusBox;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct View {
    pub icon: &'static str,
    pub label: String,
    pub tooltip: String,
}

#[derive(Debug)]
pub enum Input {
    Changed(Option<View>),
}

pub struct Battery {
    status: Option<View>,
}

#[relm4::component(pub)]
impl Component for Battery {
    type Init = ();
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        #[template]
        StatusBox {
            #[watch]
            set_visible: model.status.is_some(),
            #[watch]
            set_tooltip_text: Some(&model.status.as_ref().map_or(String::new(), |status| status.tooltip.clone())),
            gtk::Image {
                #[watch]
                set_icon_name: model.status.as_ref().map(|status| status.icon),
            },
            gtk::Label {
                #[watch]
                set_label: model.status.as_ref().map_or("", |status| status.label.as_str()),
            },
        }
    }

    fn init(_: (), root: Self::Root, _sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = Self { status: None };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _: ComponentSender<Self>, _: &Self::Root) {
        match input {
            Input::Changed(status) => self.status = status,
        }
    }
}
