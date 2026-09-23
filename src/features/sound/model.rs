#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub level: f64,
    pub muted: bool,
}

impl Volume {
    /// Returns the rounded percentage displayed by both sound controls.
    pub fn percent(self) -> u8 {
        (self.level * 100.0).round() as u8
    }

    /// Treats a displayed zero as silent without replacing the server mute flag.
    pub fn is_muted(self) -> bool {
        self.muted || self.percent() == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioDevice {
    Output,
    Input,
}

impl AudioDevice {
    pub const fn index(self) -> usize {
        match self {
            Self::Output => 0,
            Self::Input => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Volume;

    #[test]
    fn zero_percent_is_muted_even_without_the_server_mute_flag() {
        for level in [0.0, 0.001, 0.0049] {
            let volume = Volume {
                level,
                muted: false,
            };

            assert_eq!(volume.percent(), 0);
            assert!(volume.is_muted());
        }
    }

    #[test]
    fn positive_volume_respects_the_server_mute_flag() {
        for level in [0.005, 0.01, 0.5, 1.0] {
            assert!(
                !Volume {
                    level,
                    muted: false
                }
                .is_muted()
            );
            assert!(Volume { level, muted: true }.is_muted());
        }
    }
}
