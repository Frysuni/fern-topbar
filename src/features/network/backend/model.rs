use std::fmt::{self, Formatter};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub wifi_enabled: bool,
    pub wifi_hardware_enabled: bool,
    pub wifi_available: bool,
    pub wired_connected: bool,
    pub wired_enabled: bool,
    pub wired_available: bool,
    pub connection_kind: Option<ConnectionKind>,
    pub connection_name: String,
    pub networks: Vec<WifiNetwork>,
    pub profiles: Vec<WifiProfile>,
    pub devices: Vec<DeviceInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionKind {
    Wifi,
    Ethernet,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceState {
    Connected,
    Connecting,
    Disconnected,
    Disconnecting,
    Failed,
    Unknown,
}

#[derive(Clone)]
pub struct Password(pub String);

impl fmt::Debug for Password {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("Password([redacted])")
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Updated(Snapshot),
    Error(String),
    Busy(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Security {
    Open,
    Psk,
    Sae,
    Owe,
    Unsupported,
}

impl Security {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Psk => "WPA/WPA2",
            Self::Sae => "WPA3",
            Self::Owe => "Enhanced open",
            Self::Unsupported => "Unsupported security",
        }
    }

    pub fn needs_password(self) -> bool {
        matches!(self, Self::Psk | Self::Sae)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WifiNetwork {
    pub ssid: Vec<u8>,
    pub name: String,
    pub security: Security,
    pub strength: u8,
    pub device: String,
    pub access_point: String,
    pub active: bool,
    pub profile: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WifiProfile {
    pub path: String,
    pub name: String,
    pub ssid: Vec<u8>,
    pub security: Security,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub path: String,
    pub interface: String,
    pub wireless: bool,
    pub state: DeviceState,
    pub addresses: Vec<String>,
}

impl Snapshot {
    pub fn normalize(&mut self) {
        let networks = &mut self.networks;
        networks.sort_by(|left, right| {
            right
                .active
                .cmp(&left.active)
                .then(right.strength.cmp(&left.strength))
                .then(left.name.cmp(&right.name))
                .then(left.access_point.cmp(&right.access_point))
        });

        let mut seen = Vec::new();

        networks.retain(|network| {
            let key = (network.ssid.clone(), network.security);

            if seen.contains(&key) {
                false
            } else {
                seen.push(key);

                true
            }
        });

        self.profiles
            .sort_by(|left, right| left.name.cmp(&right.name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network(security: Security) -> WifiNetwork {
        WifiNetwork {
            ssid: b"Test".to_vec(),
            name: "Test".into(),
            security,
            strength: 20,
            device: "/device".into(),
            access_point: "/ap".into(),
            active: false,
            profile: None,
        }
    }

    #[test]
    fn duplicate_access_points_prefer_active_and_keep_security_distinct() {
        let mut active = network(Security::Psk);
        active.active = true;

        let mut stronger = network(Security::Psk);
        stronger.strength = 90;

        let mut snapshot = Snapshot {
            networks: vec![stronger, network(Security::Open), active],
            ..Snapshot::default()
        };

        snapshot.normalize();

        assert_eq!(snapshot.networks.len(), 2);
        assert!(snapshot.networks[0].active);
        assert_eq!(snapshot.networks[0].security, Security::Psk);
        assert_eq!(snapshot.networks[0].strength, 20);
        assert_eq!(snapshot.networks[1].security, Security::Open);
    }

    #[test]
    fn password_debug_output_never_contains_the_secret() {
        assert!(!format!("{:?}", Password("secret".into())).contains("secret"));
    }
}
