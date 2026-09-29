mod view;

use super::{FeatureDefinition, FeatureId, FeatureMountContext, MountedFeature};
use crate::runtime::Task;
use crate::ui::icon_names;
use relm4::{Component, ComponentController, Controller, gtk};

/// Describes availability and mounting without starting the feature.
pub fn definition() -> FeatureDefinition {
    FeatureDefinition {
        available: |services| services.availability.subscribe(FeatureId::Battery),
        mount,
    }
}

/// Owns the component and every resource required by this mounted feature.
pub struct Mounted {
    controller: Controller<view::Battery>,
    // Aborted on unmount so the event subscription cannot outlive its controller.
    _forwarder: Task,
}

impl Mounted {
    pub fn widget(&self) -> &gtk::Widget {
        self.controller.widget().as_ref()
    }
}

fn mount(context: FeatureMountContext) -> MountedFeature {
    let component = view::Battery::builder().launch(()).detach();
    let input = component.sender().clone();
    let mut receiver = context.battery.subscribe();
    let forwarder = Task::spawn(async move {
        loop {
            let status = receiver.borrow_and_update().clone();
            let view = status.map(|status| {
                let icon = if status.power_state.is_charging() {
                    icon_names::BATTERY_CHARGING
                } else if status.percent < 15 {
                    icon_names::BATTERY_EMPTY
                } else {
                    icon_names::BATTERY
                };

                let power = status
                    .watts
                    .map_or_else(|| "— W".into(), |watts| format!("{watts:.1} W"));
                let label = format!("{}% · {power}", status.percent);
                let tooltip = match status.watts {
                    Some(_) if status.power_state.is_charging() => {
                        format!("Battery charging power: {power}")
                    }
                    Some(_) => format!("Battery power draw: {power}"),
                    None => "Battery power data unavailable".into(),
                };

                view::View {
                    icon,
                    label,
                    tooltip,
                }
            });

            if input.send(view::Input::Changed(view)).is_err() || receiver.changed().await.is_err()
            {
                break;
            }
        }
    });

    MountedFeature::Battery(Mounted {
        controller: component,
        _forwarder: forwarder,
    })
}
