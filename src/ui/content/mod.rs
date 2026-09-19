//! Feature composition for the panel's start, center and end slots.

use super::{BAR_HEIGHT, core::PopoverScope};
use crate::features::{EnabledFeature, EnabledFeatures, FeatureMountContext, FeatureServices};
use crate::features::{FeatureId, availability::Availability};
use feature::{FeatureGroup, FeatureSlot};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

mod feature;

/// Owns mounted feature groups and applies their availability changes.
pub struct PanelContent {
    start: FeatureGroup,
    center: FeatureGroup,
    end: FeatureGroup,
}

/// Mount configuration, services and the parent panel's shared popover scope.
pub struct PanelContentInit {
    pub features: EnabledFeatures,
    pub services: FeatureServices,
    pub popovers: PopoverScope,
}

/// Availability and layout messages received from mounted feature slots.
#[derive(Debug)]
pub enum Input {
    AvailabilityChanged(FeatureId, Availability),
    ContentChanged,
}

#[relm4::component(pub)]
impl Component for PanelContent {
    type Init = PanelContentInit;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::CenterBox {
            add_css_class: "topbar-content",
            set_size_request: (-1, BAR_HEIGHT),
            #[wrap(Some)]
            set_start_widget = &gtk::Box { #[local_ref] start -> gtk::Box {} },
            #[wrap(Some)]
            set_center_widget = &gtk::Box { #[local_ref] center -> gtk::Box {} },
            #[wrap(Some)]
            set_end_widget = &gtk::Box { #[local_ref] end -> gtk::Box {} },
        }
    }

    /// Creates every configured feature once; readiness controls its slot visibility.
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let PanelContentInit {
            features,
            services,
            popovers,
        } = init;

        let mount_group = |features: Vec<EnabledFeature>| {
            let mut mounted = Vec::with_capacity(features.len());

            for feature in features {
                let context = FeatureMountContext::new(&services, popovers.clone());

                let updates = (feature.definition.available)(&services);
                let component = (feature.definition.mount)(context);

                mounted.push(FeatureSlot::new(
                    feature,
                    component,
                    updates,
                    sender.input_sender().clone(),
                ));
            }

            FeatureGroup::new(mounted)
        };

        let model = Self {
            start: mount_group(features.start),
            center: mount_group(features.center),
            end: mount_group(features.end),
        };

        let start = model.start.widget().clone();
        let center = model.center.widget().clone();
        let end = model.end.widget().clone();
        end.add_css_class("topbar-end");

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    /// Updates feature visibility and group layout without interpreting feature data.
    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _: &Self::Root) {
        match message {
            Input::AvailabilityChanged(name, state) => {
                self.start.update(name, state);
                self.center.update(name, state);
                self.end.update(name, state);
            }
            Input::ContentChanged => {
                self.start.refresh();
                self.center.refresh();
                self.end.refresh();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{backend::wm::state::WindowManagerState, ui::monitor::MonitorSelection};
    use crate::{
        config::{Auto, FeatureMode, FeatureOptions, Features},
        features,
    };

    fn children(widget: &gtk::Widget) -> Vec<gtk::Widget> {
        let mut children = Vec::new();
        let mut child = widget.first_child();

        while let Some(widget) = child {
            child = widget.next_sibling();
            children.push(widget);
        }

        children
    }
}
