use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

pub struct KeyboardLayout {
    layout: Option<String>,
}

#[relm4::component(pub)]
impl Component for KeyboardLayout {
    type Init = Option<String>;
    type Input = Option<String>;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Label {
            add_css_class: "topbar-layout",
            #[watch]
            set_visible: model.layout.is_some(),
            #[watch]
            set_label: model.layout.as_deref().unwrap_or(""),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { layout: init };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, layout: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.layout = layout;
    }
}
