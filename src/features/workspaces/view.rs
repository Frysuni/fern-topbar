use crate::backend::{self, wm::Workspace};
use relm4::factory::FactoryVecDeque;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

pub struct Workspaces {
    dots: FactoryVecDeque<dot::Dot>,
    workspaces: Vec<Workspace>,
    output: Option<String>,
    commands: Option<tokio::sync::mpsc::UnboundedSender<backend::Command>>,
}

pub struct WorkspacesInit {
    pub wm_commands: Option<tokio::sync::mpsc::UnboundedSender<backend::Command>>,
}

#[derive(Debug)]
pub enum Input {
    WorkspacesChanged(Vec<Workspace>),
    OutputChanged(Option<String>),
    DotClicked(u64),
}

#[relm4::component(pub)]
impl Component for Workspaces {
    type Init = WorkspacesInit;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "topbar-workspaces",

            #[local_ref]
            dots -> gtk::Box {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let WorkspacesInit { wm_commands } = init;

        let dots = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), Input::DotClicked);

        let model = Self {
            dots,
            workspaces: Vec::new(),
            output: None,
            commands: wm_commands,
        };

        let dots = model.dots.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::WorkspacesChanged(workspaces) => {
                self.workspaces = workspaces;
                self.refresh_dots();
            }
            Input::OutputChanged(output) => {
                if self.output != output {
                    self.output = output;
                    self.refresh_dots();
                }
            }
            Input::DotClicked(id) => self.focus_workspace(id),
        }
    }
}

impl Workspaces {
    fn refresh_dots(&mut self) {
        let workspaces = self
            .workspaces
            .iter()
            .filter(|workspace| workspace.output == self.output)
            .map(|workspace| {
                dot::WorkspaceDot::new(
                    workspace.id,
                    workspace
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Workspace {}", workspace.index)),
                    workspace.active,
                    workspace.urgent,
                )
            });

        let mut dots = self.dots.guard();
        dots.clear();

        for workspace in workspaces {
            dots.push_back(workspace);
        }
    }

    fn focus_workspace(&self, id: u64) {
        if let Some(commands) = &self.commands {
            let _ = commands.send(backend::Command::FocusWorkspace(id));
        }
    }
}

mod dot {
    use crate::ui::core::Button;
    use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender};
    use relm4::gtk;
    use relm4::gtk::prelude::*;

    #[derive(Clone, Debug)]
    pub struct WorkspaceDot {
        id: u64,
        title: String,
        active: bool,
        urgent: bool,
    }

    impl WorkspaceDot {
        pub fn new(id: u64, title: String, active: bool, urgent: bool) -> Self {
            Self {
                id,
                title,
                active,
                urgent,
            }
        }
    }

    pub struct Dot {
        workspace: WorkspaceDot,
    }

    #[relm4::factory(pub)]
    impl FactoryComponent for Dot {
        type Init = WorkspaceDot;
        type Input = ();
        type Output = u64;
        type CommandOutput = ();
        type ParentWidget = gtk::Box;

        view! {
            #[root]
            #[template]
            Button {
                #[watch]
                set_css_classes: self.css_classes(),

                set_tooltip_text: Some(&self.workspace.title),

                connect_clicked[sender, id] => move |_| {
                    let _ = sender.output(id);
                },

                gtk::Box {
                    add_css_class: "workspace-dot-mark",
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                }
            }
        }

        fn init_model(
            workspace: Self::Init,
            _index: &DynamicIndex,
            _sender: FactorySender<Self>,
        ) -> Self {
            Self { workspace }
        }

        fn init_widgets(
            &mut self,
            _index: &DynamicIndex,
            root: Self::Root,
            _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
            sender: FactorySender<Self>,
        ) -> Self::Widgets {
            let id = self.workspace.id;
            let widgets = view_output!();

            widgets
        }
    }

    impl Dot {
        fn css_classes(&self) -> &'static [&'static str] {
            match (self.workspace.active, self.workspace.urgent) {
                (true, true) => &["workspace-dot-button", "active", "urgent"],
                (true, false) => &["workspace-dot-button", "active"],
                (false, true) => &["workspace-dot-button", "urgent"],
                (false, false) => &["workspace-dot-button"],
            }
        }
    }
}
