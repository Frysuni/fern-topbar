//! Configuration reloads and errors share one replaceable, short-lived alert.

use super::Alert;
use crate::backend::notifications::Urgency;
use std::time::Duration;

pub fn reloaded() -> Alert {
    Alert {
        icon: "dialog-information-symbolic".into(),
        summary: "Topbar: configuration reloaded".into(),
        body: "The new configuration has been applied.".into(),
        urgency: Urgency::Normal,
        duration: Some(Duration::from_secs(5)),
    }
}

pub fn invalid(error: &str) -> Alert {
    Alert {
        icon: "dialog-error-symbolic".into(),
        summary: "Topbar: invalid configuration".into(),
        body: format!("The current configuration is unchanged.\n{error}"),
        urgency: Urgency::Critical,
        duration: Some(Duration::from_secs(10)),
    }
}

/// No application alert service exists when startup configuration is invalid.
/// Deliver to the desktop's active server and explicitly close the critical
/// notification, since servers may ignore expiration for critical urgency.
pub async fn startup_error(error: &str) {
    use crate::backend::notifications::client::NotificationClient;

    let Ok(mut client) = NotificationClient::connect().await else {
        return;
    };
    let alert = invalid(error);
    let body = format!("Topbar could not start.\n{error}");
    let Ok(id) = client
        .notify(0, &alert.icon, &alert.summary, &body, alert.urgency, true)
        .await
    else {
        return;
    };
    tokio::time::sleep(Duration::from_secs(10)).await;
    let _ = client.close(id).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_errors_are_critical_for_ten_seconds() {
        let alert = invalid("invalid config.json: scale must be between 0.25 and 4");
        assert_eq!(alert.urgency, Urgency::Critical);
        assert_eq!(alert.duration, Some(Duration::from_secs(10)));
        assert!(alert.body.contains("config.json"));
    }
}
