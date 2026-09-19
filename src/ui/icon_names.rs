// The icon bundler also generates names without GTK's symbolic suffix. We use
// the suffixed names below and include its output for the resource bytes/prefix.
#[allow(
    dead_code,
    reason = "bundler emits unsuffixed icon constants alongside resource data"
)]
mod bundled {
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}

pub use bundled::{GRESOURCE_BYTES, RESOURCE_PREFIX};

// Shipped Relm4 icons are stored as symbolic SVGs. GTK needs the full
// icon name to resolve them from the resource path.
pub const SPEAKER_CROSS: &str = "volume-off-fill-symbolic";
pub const SPEAKER_MIN: &str = "volume-mute-fill-symbolic";
pub const SPEAKER_MID: &str = "volume-down-fill-symbolic";
pub const SPEAKER_MAX: &str = "volume-up-fill-symbolic";
pub const MIC: &str = "mic-fill-symbolic";
pub const MIC_MUTED: &str = "mic-off-fill-symbolic";
pub const BRIGHTNESS: &str = "brightness-high-fill-symbolic";
pub const NETWORK_WIFI: &str = "wifi-fill-symbolic";
pub const NETWORK_WIRED: &str = "lan-fill-symbolic";
pub const NETWORK_OFF: &str = "wifi-off-fill-symbolic";
pub const BATTERY: &str = "battery-full-fill-symbolic";
pub const BATTERY_EMPTY: &str = "battery-low-fill-symbolic";
pub const BATTERY_CHARGING: &str = "battery-charging-full-fill-symbolic";
pub const NOTIFICATIONS: &str = "notifications-fill-symbolic";
pub const CHEVRON_LEFT: &str = "chevron-left-fill-symbolic";
pub const CHEVRON_RIGHT: &str = "chevron-right-fill-symbolic";
pub const CLOSE: &str = "close-fill-symbolic";
pub const CHECK: &str = "check-fill-symbolic";
pub const RADIO_ON: &str = "radio-button-checked-fill-symbolic";
pub const RADIO_OFF: &str = "radio-button-unchecked-fill-symbolic";
