//! Parse application arguments before GTK, so validation works without a display.

use std::{ffi::OsString, path::PathBuf};

pub const HELP: &str = "Usage: topbar [--config PATH | -c PATH]\n       topbar validate [--config PATH | -c PATH]\n\nOptions:\n  -c, --config PATH  Configuration file (overrides TOPBAR_CONFIG)\n  -h, --help         Show this help\n\nWithout an override, reads $XDG_CONFIG_HOME/topbar/config.json\n(or ~/.config/topbar/config.json). Changes are applied automatically.\n";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub config: Option<PathBuf>,
    pub validate: bool,
    pub help: bool,
}

impl Options {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            if arg == "-h" || arg == "--help" {
                options.help = true;
            } else if arg == "validate" && !options.validate {
                options.validate = true;
            } else if arg == "-c" || arg == "--config" {
                let value = args.next().ok_or("--config requires a file path")?;
                options.set_config(value)?;
            } else if let Some(value) = arg.as_encoded_bytes().strip_prefix(b"--config=") {
                // Preserve non-UTF-8 Unix paths without lossy string conversion.
                use std::os::unix::ffi::OsStringExt;
                options.set_config(OsString::from_vec(value.to_vec()))?;
            } else {
                return Err(format!("unknown argument: {}", arg.to_string_lossy()));
            }
        }
        Ok(options)
    }

    fn set_config(&mut self, value: OsString) -> Result<(), String> {
        if value.is_empty()
            || value == "--help"
            || value == "-h"
            || value == "--config"
            || value == "-c"
        {
            return Err("--config requires a file path".into());
        }
        if self.config.is_some() {
            return Err("--config may only be specified once".into());
        }
        self.config = Some(value.into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(args.iter().map(OsString::from))
    }

    #[test]
    fn accepts_config_before_or_after_validate() {
        for args in [
            vec!["validate", "-c", "/tmp/my config.json"],
            vec!["--config", "/tmp/my config.json", "validate"],
            vec!["validate", "--config=/tmp/my config.json"],
        ] {
            assert_eq!(
                parse(&args).unwrap(),
                Options {
                    config: Some("/tmp/my config.json".into()),
                    validate: true,
                    help: false,
                }
            );
        }
        assert_eq!(parse(&[]).unwrap(), Options::default());
        assert!(parse(&["--help"]).unwrap().help);
    }

    #[test]
    fn rejects_unknown_arguments_and_missing_or_duplicate_paths() {
        for args in [
            vec!["unknown"],
            vec!["--config"],
            vec!["-c", "--help"],
            vec!["--config="],
            vec!["-c", "a", "-c", "b"],
            vec!["validate", "validate"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn accepts_non_utf8_paths() {
        use std::os::unix::ffi::OsStringExt;
        let path = OsString::from_vec(b"/tmp/\xff.json".to_vec());
        let options = Options::parse([OsString::from("-c"), path.clone()]).unwrap();
        assert_eq!(options.config, Some(path.into()));
    }
}
