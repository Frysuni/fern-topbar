//! Content-based polling also handles editors that replace the file atomically.

use super::Settings;
use std::{fs, path::PathBuf};

pub struct Watcher {
    path: PathBuf,
    observed: Option<String>,
    pending: Option<String>,
    read_error: Option<String>,
    reloaded: bool,
}

impl Watcher {
    /// Replace the current executable with the same validated configuration.
    /// The explicit path keeps watching the original file after replacement.
    pub fn restart_command(&self) -> Result<std::process::Command, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("cannot locate topbar executable: {error}"))?;
        let mut command = std::process::Command::new(executable);
        command.arg("--config").arg(&self.path);
        command.env("_TOPBAR_RELOAD_PID", std::process::id().to_string());
        command.env(
            "_TOPBAR_RELOAD_CONFIG",
            self.observed.as_deref().unwrap_or("{}"),
        );
        Ok(command)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn was_reloaded(&self) -> bool {
        self.reloaded
    }

    pub(super) fn new(path: PathBuf, contents: Option<String>, reloaded: bool) -> Self {
        Self {
            path,
            observed: contents,
            pending: None,
            read_error: None,
            reloaded,
        }
    }

    /// Accept only contents seen on two successive polls. Missing/unreadable
    /// files leave the running configuration intact, including during a save.
    pub fn poll(&mut self) -> Option<Result<Settings, String>> {
        let contents = match fs::read_to_string(&self.path) {
            Ok(contents) => {
                self.read_error = None;
                contents
            }
            Err(error) => {
                self.pending = None;
                if error.kind() == std::io::ErrorKind::NotFound {
                    return None;
                }
                let error = format!("cannot read {}: {error}", self.path.display());
                if self.read_error.as_ref() == Some(&error) {
                    return None;
                }
                self.read_error = Some(error.clone());
                return Some(Err(error));
            }
        };

        if self.observed.as_ref() == Some(&contents) {
            self.pending = None;
            return None;
        }
        if self.pending.as_ref() != Some(&contents) {
            self.pending = Some(contents);
            return None;
        }

        self.pending = None;
        self.observed = Some(contents.clone());
        Some(super::parse(Some(&contents), &self.path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "topbar-config-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> PathBuf {
            self.0.join("config.json")
        }

        fn write(&self, contents: &str) {
            fs::write(self.path(), contents).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn creation_and_atomic_replacement_are_detected_once() {
        let file = Fixture::new();
        let mut watcher = Watcher::new(file.path(), None, false);
        assert!(watcher.poll().is_none());
        file.write("{}");
        assert!(watcher.poll().is_none());
        assert_eq!(watcher.poll().unwrap().unwrap().scale.factor(), 1.0);
        assert!(watcher.poll().is_none());

        let temporary = file.0.join("save.tmp");
        fs::write(&temporary, r#"{"scale":1.5}"#).unwrap();
        fs::rename(temporary, file.path()).unwrap();
        assert!(watcher.poll().is_none());
        assert_eq!(watcher.poll().unwrap().unwrap().scale.factor(), 1.5);
        assert!(watcher.poll().is_none());
    }

    #[test]
    fn incomplete_saves_are_debounced_and_invalid_configs_can_be_corrected() {
        let file = Fixture::new();
        let mut watcher = Watcher::new(file.path(), Some("{}".into()), false);
        file.write("{");
        assert!(watcher.poll().is_none());
        file.write(r#"{"scale":2}"#);
        assert!(watcher.poll().is_none());
        assert_eq!(watcher.poll().unwrap().unwrap().scale.factor(), 2.0);

        for invalid in ["{", r#"{"scale":0}"#, r#"{"unknown":true}"#] {
            file.write(invalid);
            assert!(watcher.poll().is_none());
            assert!(watcher.poll().unwrap().is_err());
            assert!(watcher.poll().is_none());
        }
        file.write("{}");
        assert!(watcher.poll().is_none());
        assert!(watcher.poll().unwrap().is_ok());
    }

    #[test]
    fn deletion_does_not_reset_settings_or_complete_a_pending_save() {
        let file = Fixture::new();
        let mut watcher = Watcher::new(file.path(), Some("{}".into()), false);
        file.write(r#"{"scale":2}"#);
        assert!(watcher.poll().is_none());
        fs::remove_file(file.path()).unwrap();
        assert!(watcher.poll().is_none());
        assert!(watcher.poll().is_none());
        file.write(r#"{"scale":2}"#);
        assert!(watcher.poll().is_none());
        assert_eq!(watcher.poll().unwrap().unwrap().scale.factor(), 2.0);
    }
}
