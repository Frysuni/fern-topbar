use super::super::backend::{DeviceInfo, Snapshot, WifiNetwork, WifiProfile};
use crate::ui::icon_names;

#[derive(Clone, Debug)]
pub struct View {
    pub visible: bool,
    pub icon: &'static str,
    pub summary: String,
    pub tooltip: String,
    pub wifi_enabled: bool,
    pub wifi_available: bool,
    pub wifi_hardware_enabled: bool,
    pub wired_enabled: bool,
    pub wired_connected: bool,
    pub wired_available: bool,
    pub error: Option<String>,
    pub busy: bool,
    pub networks: Vec<WifiNetwork>,
    pub profiles: Vec<WifiProfile>,
    pub devices: Vec<DeviceInfo>,
}

impl View {
    pub fn wifi_empty_message(&self) -> &'static str {
        if !self.wifi_available {
            "No Wi-Fi adapter detected"
        } else if !self.wifi_hardware_enabled {
            "Unblock Wi-Fi using the hardware switch"
        } else if !self.wifi_enabled {
            "Turn on Wi-Fi to see nearby networks"
        } else if self.busy {
            "Looking for networks…"
        } else {
            "No networks found. Try scanning again."
        }
    }

    pub fn wifi_subtitle(&self) -> &'static str {
        if !self.wifi_available {
            "No Wi-Fi device"
        } else if !self.wifi_hardware_enabled {
            "Blocked by hardware switch"
        } else if !self.wifi_enabled {
            "Off"
        } else if self.networks.iter().any(|network| network.active) {
            "Connected"
        } else {
            "Not connected"
        }
    }

    pub fn wired_subtitle(&self) -> &'static str {
        if self.wired_connected {
            "Connected"
        } else if !self.wired_available {
            "No Ethernet device"
        } else if !self.wired_enabled {
            "Off"
        } else {
            "Not connected"
        }
    }

    pub fn wifi_sensitive(&self) -> bool {
        !self.busy && self.wifi_available && self.wifi_hardware_enabled
    }
}

impl Default for View {
    fn default() -> Self {
        status_view(Snapshot::default(), None, false)
    }
}

pub fn status_view(status: Snapshot, error: Option<String>, visible: bool) -> View {
    let wifi_enabled = status.wifi_enabled && status.wifi_available && status.wifi_hardware_enabled;
    let (icon, label) = match status.connection_kind.as_str() {
        "802-11-wireless" if wifi_enabled => (icon_names::NETWORK_WIFI, "Wi-Fi"),
        "802-3-ethernet" if status.wired_connected => (icon_names::NETWORK_WIRED, "Ethernet"),
        _ if status.wired_connected => (icon_names::NETWORK_WIRED, "Ethernet"),
        _ if status.wired_enabled => (icon_names::NETWORK_WIRED, "Ethernet on"),
        _ if wifi_enabled => (icon_names::NETWORK_WIFI, "Wi-Fi on"),
        _ => (icon_names::NETWORK_OFF, "Offline"),
    };

    let tooltip = if status.connection_name.is_empty() {
        label.to_string()
    } else {
        format!("Connected: {}", status.connection_name)
    };

    View {
        visible,
        icon,
        summary: label.into(),
        tooltip,
        wifi_enabled,
        wifi_available: status.wifi_available,
        wifi_hardware_enabled: status.wifi_hardware_enabled,
        wired_enabled: status.wired_enabled,
        wired_connected: status.wired_connected,
        wired_available: status.wired_available,
        error,
        busy: false,
        networks: status.networks,
        profiles: status.profiles,
        devices: status.devices,
    }
}
