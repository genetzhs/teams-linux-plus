//! User configuration, stored as JSON in `$XDG_CONFIG_HOME/teams-for-linux-rs/config.json`.
//!
//! Every field has a default so older or hand-edited files keep loading; unknown
//! keys are ignored rather than rejected.

use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

pub const DEFAULT_URL: &str = "https://teams.microsoft.com/v2/";
pub const DEFAULT_MAIL_URL: &str = "https://outlook.office.com/mail/";

/// Teams refuses to load in browsers it does not recognise, so present
/// ourselves as current Edge on Linux unless the user overrides it.
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GpuPolicy {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub url: String,
    pub mail_enabled: bool,
    pub mail_url: String,
    pub user_agent: String,
    pub start_minimized: bool,
    pub close_to_tray: bool,
    pub notifications: bool,
    pub notification_preview: bool,
    pub tray_badge: bool,
    pub zoom: f64,
    pub theme: Theme,
    pub low_memory: bool,
    pub gpu: GpuPolicy,
    pub custom_css: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL.into(),
            mail_enabled: true,
            mail_url: DEFAULT_MAIL_URL.into(),
            user_agent: String::new(),
            start_minimized: false,
            close_to_tray: true,
            notifications: true,
            notification_preview: true,
            tray_badge: true,
            zoom: 1.0,
            theme: Theme::System,
            low_memory: true,
            gpu: GpuPolicy::Auto,
            custom_css: String::new(),
        }
    }
}

impl Config {
    pub fn effective_user_agent(&self) -> &str {
        match self.user_agent.trim() {
            "" => DEFAULT_USER_AGENT,
            ua => ua,
        }
    }

    /// Clamp values that would otherwise produce an unusable window.
    pub fn sanitized(mut self) -> Self {
        self.zoom = if self.zoom.is_finite() { self.zoom.clamp(0.5, 3.0) } else { 1.0 };
        let https = |u: &str| url::Url::parse(u).is_ok_and(|u| u.scheme() == "https");
        if !https(&self.url) {
            self.url = DEFAULT_URL.into();
        }
        if !https(&self.mail_url) {
            self.mail_url = DEFAULT_MAIL_URL.into();
        }
        self
    }

    /// Whether switching from `self` to `next` only takes effect after a restart.
    pub fn needs_restart(&self, next: &Config) -> bool {
        self.url != next.url
            || self.mail_url != next.mail_url
            || self.user_agent != next.user_agent
            || self.low_memory != next.low_memory
            || self.gpu != next.gpu
            || self.custom_css != next.custom_css
    }
}

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("teams-for-linux-rs").join("config.json"))
}

pub fn load() -> Config {
    let Some(path) = path() else { return Config::default() };
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str::<Config>(&text)
            .map(Config::sanitized)
            .unwrap_or_else(|err| {
                eprintln!("[config] ignoring invalid config file: {err}");
                Config::default()
            }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Config::default(),
        Err(err) => {
            eprintln!("[config] cannot read config file: {err}");
            Config::default()
        }
    }
}

pub fn save(config: &Config) -> io::Result<()> {
    let path = path().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no config directory"))?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    // Write-then-rename so a crash never leaves a truncated config behind.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(config)?)?;
    fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_file_uses_defaults() {
        let cfg: Config = serde_json::from_str(r#"{"zoom": 1.25}"#).unwrap();
        assert_eq!(cfg.zoom, 1.25);
        assert_eq!(cfg.url, DEFAULT_URL);
        assert!(cfg.close_to_tray);
    }

    #[test]
    fn sanitize_rejects_bad_values() {
        let cfg = Config { zoom: 40.0, url: "http://evil.example".into(), ..Config::default() }.sanitized();
        assert_eq!(cfg.zoom, 3.0);
        assert_eq!(cfg.url, DEFAULT_URL);
    }

    #[test]
    fn restart_detection() {
        let a = Config::default();
        assert!(!a.needs_restart(&Config { zoom: 1.5, notifications: false, ..a.clone() }));
        assert!(a.needs_restart(&Config { low_memory: false, ..a.clone() }));
    }
}
