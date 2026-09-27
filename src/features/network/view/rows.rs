use super::super::backend::{DeviceInfo, Security, WifiNetwork, WifiProfile};
use super::{Input, Network, View, templates::DetailRow};
use crate::ui::{
    core::{Button, motion},
    icon_names,
};
use relm4::{
    factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque, FactoryView},
    gtk::prelude::*,
    prelude::*,
};

pub struct Rows {
    pub wifi_devices: FactoryVecDeque<NetworkRow>,
    pub wired_devices: FactoryVecDeque<NetworkRow>,
    pub networks: FactoryVecDeque<NetworkRow>,
    pub profiles: FactoryVecDeque<NetworkRow>,
}

impl Rows {
    pub fn new(sender: &ComponentSender<Network>) -> Self {
        let create = || {
            FactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |input| input)
        };

        Self {
            wifi_devices: create(),
            wired_devices: create(),
            networks: create(),
            profiles: create(),
        }
    }

    pub fn update(&mut self, status: &View) {
        reconcile(
            &mut self.wifi_devices,
            status
                .devices
                .iter()
                .filter(|device| device.wireless)
                .map(|device| RowData::device(device, status.busy))
                .collect(),
        );

        reconcile(
            &mut self.wired_devices,
            status
                .devices
                .iter()
                .filter(|device| !device.wireless)
                .map(|device| RowData::device(device, status.busy))
                .collect(),
        );

        reconcile(
            &mut self.networks,
            status
                .networks
                .iter()
                .map(|network| RowData::network(network, status.busy))
                .collect(),
        );

        reconcile(
            &mut self.profiles,
            status
                .profiles
                .iter()
                .map(|profile| RowData::profile(profile, status.busy))
                .collect(),
        );
    }
}

fn reconcile(rows: &mut FactoryVecDeque<NetworkRow>, items: Vec<RowData>) {
    let mut rows = rows.guard();
    let mut index = 0;

    while index < rows.len() {
        if items.iter().any(|item| item.key == rows[index].data.key) {
            index += 1;
        } else {
            rows.remove(index);
        }
    }

    for (target, item) in items.into_iter().enumerate() {
        if let Some(current) = (0..rows.len()).find(|&index| rows[index].data.key == item.key) {
            rows.move_to(current, target);
            rows[target].data = item;
        } else {
            rows.insert(target, item);
        }
    }
}

pub struct RowData {
    key: String,
    title: String,
    subtitle: String,
    icon: &'static str,
    active: bool,
    enabled: bool,
    primary: Option<(&'static str, Input)>,
    secondary: Option<(&'static str, Input)>,
}

impl RowData {
    fn device(device: &DeviceInfo, busy: bool) -> Self {
        let state = match device.state {
            100 => "Connected",
            40..=90 => "Connecting",
            110 => "Disconnecting",
            120 => "Failed",
            _ => "Disconnected",
        };

        let mut details = vec![state.to_string()];
        details.extend(device.addresses.iter().cloned());

        Self {
            key: device.path.clone(),
            title: device.interface.clone(),
            subtitle: details.join("\n"),
            icon: if device.wireless {
                icon_names::NETWORK_WIFI
            } else {
                icon_names::NETWORK_WIRED
            },
            active: device.state == 100,
            enabled: !busy,
            primary: (40..=110)
                .contains(&device.state)
                .then(|| ("Disconnect", Input::Disconnect(device.path.clone()))),
            secondary: None,
        }
    }

    fn network(network: &WifiNetwork, busy: bool) -> Self {
        let mut details = vec![
            network.security.label().to_string(),
            format!("{}%", network.strength),
        ];

        if network.active {
            details.push("Connected".into());
        } else if network.profile.is_some() {
            details.push("Saved".into());
        }

        Self {
            key: format!("{:?}:{:?}", network.ssid, network.security),
            title: network.name.clone(),
            subtitle: details.join(" · "),
            icon: icon_names::NETWORK_WIFI,
            active: network.active,
            enabled: !busy && (network.active || network.security != Security::Unsupported),
            primary: Some(if network.active {
                ("Disconnect", Input::Disconnect(network.device.clone()))
            } else {
                ("Connect", Input::Select(network.clone()))
            }),
            secondary: (!network.active
                && network.profile.is_some()
                && network.security.needs_password())
            .then(|| ("Password…", Input::EditPassword(network.clone()))),
        }
    }

    fn profile(profile: &WifiProfile, busy: bool) -> Self {
        Self {
            key: profile.path.clone(),
            title: profile.name.clone(),
            subtitle: profile.security.label().into(),
            icon: icon_names::NETWORK_WIFI,
            active: false,
            enabled: !busy,
            primary: Some(("Forget", Input::Forget(profile.path.clone()))),
            secondary: None,
        }
    }
}

pub struct NetworkRow {
    data: RowData,
}

#[derive(Debug)]
pub enum RowInput {
    Primary,
    Secondary,
}

#[relm4::factory(pub)]
impl FactoryComponent for NetworkRow {
    type Init = RowData;
    type Input = RowInput;
    type Output = Input;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        #[template]
        DetailRow {
            #[watch]
            set_class_active: ("connected", self.data.active),

            #[template_child]
            icon {
                #[watch]
                set_icon_name: Some(self.data.icon),
            },

            #[template_child]
            title {
                #[watch]
                set_label: &self.data.title,
                #[watch]
                set_tooltip_text: Some(&self.data.title),
            },

            #[template_child]
            subtitle {
                #[watch]
                set_label: &self.data.subtitle,
            },

            #[template_child]
            actions {
                #[template]
                Button {
                    add_css_class: "topbar-network-action",
                    #[watch]
                    set_visible: self.data.primary.is_some(),
                    #[watch]
                    set_sensitive: self.data.enabled,
                    #[watch]
                    set_label: self.data.primary.as_ref().map_or("", |action| action.0),
                    connect_clicked => RowInput::Primary,
                },

                #[template]
                Button {
                    add_css_class: "topbar-network-action",
                    add_css_class: "topbar-network-icon-action",
                    #[watch]
                    set_visible: self.data.secondary.is_some(),
                    #[watch]
                    set_sensitive: self.data.enabled,
                    #[watch]
                    set_tooltip_text: self.data.secondary.as_ref().map(|action| action.0),
                    set_icon_name: "dialog-password-symbolic",
                    connect_clicked => RowInput::Secondary,
                },
            },
        }
    }

    fn init_model(data: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { data }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        motion::animate_appearance(root.widget());

        widgets
    }

    fn update(&mut self, input: Self::Input, sender: FactorySender<Self>) {
        if !self.data.enabled {
            return;
        }

        let action = match input {
            RowInput::Primary => &self.data.primary,
            RowInput::Secondary => &self.data.secondary,
        };

        if let Some((_, input)) = action {
            let _ = sender.output(input.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_networks_disconnect_without_removing_the_saved_profile() {
        let network = WifiNetwork {
            ssid: b"Home".to_vec(),
            name: "Home".into(),
            security: Security::Psk,
            strength: 90,
            device: "/wifi".into(),
            access_point: "/ap".into(),
            active: true,
            profile: Some("/saved".into()),
        };

        let row = RowData::network(&network, false);

        assert!(matches!(
            row.primary,
            Some(("Disconnect", Input::Disconnect(path))) if path == "/wifi"
        ));
        assert!(row.secondary.is_none());
    }

    #[gtk::test]
    fn updates_preserve_row_widgets_and_remove_missing_networks() {
        let (sender, _receiver) = relm4::channel::<Input>();
        let mut rows = FactoryVecDeque::builder()
            .launch_default()
            .forward(&sender, |input| input);
        let mut network = WifiNetwork {
            ssid: b"Test".to_vec(),
            name: "Test".into(),
            security: Security::Psk,
            strength: 50,
            device: "/device".into(),
            access_point: "/ap".into(),
            active: false,
            profile: None,
        };

        reconcile(&mut rows, vec![RowData::network(&network, false)]);

        let widget = rows.widget().first_child().unwrap();

        network.strength = 75;
        network.access_point = "/another_ap".into();
        reconcile(&mut rows, vec![RowData::network(&network, true)]);

        assert_eq!(rows.widget().first_child().unwrap(), widget);
        assert_eq!(rows.guard()[0].data.subtitle, "WPA/WPA2 · 75%");
        assert!(!rows.guard()[0].data.enabled);

        reconcile(&mut rows, Vec::new());

        assert!(rows.widget().first_child().is_none());
    }
}
