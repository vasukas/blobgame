use crate::{common::*, control::input::InputSettings};
pub use serde::{Deserialize, Serialize};

// TODO: platform-dependent default path to save settings

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub master_volume: f32,
    pub fullscreen: bool,
    pub difficulty: Difficulty,
    pub input: InputSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            master_volume: 0.6,
            fullscreen: false,
            difficulty: Difficulty::Hard,
            input: default(),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Difficulty {
    Hard,
}

// desktop
#[cfg(not(target_arch = "wasm32"))]
impl Settings {
    const FILE: &'static str = "user.cfg.ron";

    pub fn save(&self) {
        match std::fs::write(Self::FILE, self.save_ron()) {
            Ok(_) => (),
            Err(error) => log::error!(
                "Failed to save settings (file: \"{}\") - {}",
                Self::FILE,
                error
            ),
        }
    }

    pub fn load() -> Option<Self> {
        Self::load_ron(&std::fs::read_to_string(Self::FILE).ok()?)
    }
}

// wasm
#[cfg(target_arch = "wasm32")]
impl Settings {
    const NAME: &'static str = "blobfight-settings";
    const EXPIRE: Duration = Duration::from_secs(60 * 60 * 24 * 30); // 30 days

    pub fn save(&self) {
        wasm_cookies::set(
            Self::NAME,
            &Self::encode(self.save_ron().as_bytes()),
            &wasm_cookies::CookieOptions::default().expires_after(Self::EXPIRE),
        );
    }

    pub fn load() -> Option<Self> {
        match wasm_cookies::get(Self::NAME) {
            Some(Ok(data)) => Self::load_ron(std::str::from_utf8(&Self::decode(&data)?).ok()?),
            Some(Err(error)) => {
                log::warn!("cookie read error: {}", error);
                None
            }
            None => None,
        }
    }

    // base64 or something similiar
    fn encode(data: &[u8]) -> String {
        data.into_iter()
            .flat_map(|c| [c >> 4, c & 15])
            .map(|c| char::from_digit(c.into(), 16).unwrap())
            .fold(String::with_capacity(data.len() * 2), |mut s, c| {
                s.push(c);
                s
            })
    }
    fn decode(data: &str) -> Option<Vec<u8>> {
        use itertools::Itertools;
        if data.len() % 2 != 0 {
            log::error!("Invalid length");
            return None;
        }
        data.chars()
            .tuples()
            .try_fold(
                Vec::with_capacity(data.len() / 2),
                |mut s, (c1, c2)| match char::to_digit(c1, 16).zip(char::to_digit(c2, 16)) {
                    Some((n1, n2)) => {
                        let n = (n1 << 4) | n2;
                        s.push(n as u8);
                        Ok(s)
                    }
                    None => {
                        log::error!("Invalid character");
                        Err(())
                    }
                },
            )
            .ok()
    }
}

impl Settings {
    fn save_ron(&self) -> String {
        ron::ser::to_string(self).unwrap()
    }
    fn load_ron(s: &str) -> Option<Self> {
        ron::from_str(s).ok()
    }
}
