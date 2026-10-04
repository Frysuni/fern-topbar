use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "topbar-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_topbar"));
        command
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("DBUS_SESSION_BUS_ADDRESS")
            .env_remove("TOPBAR_CONFIG")
            .env_remove("_TOPBAR_RELOAD_PID")
            .env_remove("_TOPBAR_RELOAD_CONFIG")
            .env("XDG_CONFIG_HOME", &self.0);
        command
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn validation_works_without_display_and_cli_path_overrides_environment() {
    let fixture = Fixture::new();
    let valid = fixture.write("valid config.json", include_str!("../config.example.json"));
    let invalid = fixture.write("invalid.json", "{");
    let output = fixture
        .command()
        .arg("validate")
        .arg("-c")
        .arg(&valid)
        .env("TOPBAR_CONFIG", invalid)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(valid.to_str().unwrap()));
    assert!(
        fixture
            .command()
            .arg("validate")
            .env("TOPBAR_CONFIG", valid)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn validation_reports_parse_semantic_and_missing_file_errors() {
    let fixture = Fixture::new();
    for contents in [
        "{",
        r#"{"scale":0}"#,
        r#"{"unknown":true}"#,
        r#"{"features":{"start":[{"name":"clock","mode":true},{"name":"clock","mode":false}]}}"#,
        r#"{"reveal":{"on_hover":{"enabled":false,"on_fullscreen":true}}}"#,
    ] {
        let path = fixture.write("bad.json", contents);
        let output = fixture
            .command()
            .arg("validate")
            .arg("--config")
            .arg(path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid"));
    }
    let output = fixture
        .command()
        .arg("validate")
        .arg("-c")
        .arg(fixture.0.join("missing.json"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read"));
}

#[test]
fn default_path_help_and_usage_errors_work_offline() {
    let fixture = Fixture::new();
    assert!(
        fixture
            .command()
            .arg("validate")
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::create_dir(fixture.0.join("topbar")).unwrap();
    fixture.write("topbar/config.json", r#"{"scale":0}"#);
    assert_eq!(
        fixture
            .command()
            .arg("validate")
            .output()
            .unwrap()
            .status
            .code(),
        Some(1)
    );
    let help = fixture.command().arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("validate"));
    assert_eq!(
        fixture.command().arg("-c").output().unwrap().status.code(),
        Some(2)
    );
}
