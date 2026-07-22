use crate::crypto;
use anyhow::{Context, Result};
#[cfg(windows)]
use base64::{Engine, engine::general_purpose::STANDARD};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub storefront_url: String,
    pub vdi_name: String,
    pub username: String,
    pub citrix_path: String,
    protected_password: String,
    protected_secret: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            storefront_url: String::new(),
            vdi_name: String::new(),
            username: String::new(),
            citrix_path: discover_citrix()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            protected_password: String::new(),
            protected_secret: String::new(),
        }
    }
}

impl AppConfig {
    pub fn load() -> (Self, Option<String>) {
        match Self::load_inner() {
            Ok(v) => (v, None),
            Err(e) => (Self::default(), Some(format!("Настройки сброшены: {e:#}"))),
        }
    }
    fn load_inner() -> Result<Self> {
        let path = config_path()?;
        #[cfg(windows)]
        if !path.exists() {
            let legacy = [
                directories::ProjectDirs::from("ru", "Local", "CitrixVdiLauncher")
                    .map(|p| p.config_dir().join("config.json")),
                directories::ProjectDirs::from("", "", "CitrixVdiLauncher")
                    .map(|p| p.config_dir().join("config.json")),
            ];
            if let Some(old) = legacy.into_iter().flatten().find(|p| p.exists()) {
                fs::create_dir_all(path.parent().unwrap())?;
                fs::copy(old, &path)?;
            }
        }
        if !path.exists() {
            let config = Self::default();
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, serde_json::to_vec_pretty(&config)?)?;
            return Ok(config);
        }
        let mut config: Self = serde_json::from_str(
            &fs::read_to_string(&path).with_context(|| format!("Чтение {}", path.display()))?,
        )
        .context("Некорректный config.json")?;
        if (config.citrix_path.trim().is_empty() || !Path::new(&config.citrix_path).exists())
            && let Some(found) = discover_citrix()
        {
            config.citrix_path = found.to_string_lossy().into_owned();
        }
        fs::write(&path, serde_json::to_vec_pretty(&config)?)?;
        Ok(config)
    }
    pub fn save_with_secrets(&mut self, password: &str, secret: &str) -> Result<()> {
        #[cfg(windows)]
        {
            self.protected_password = encode(password)?;
            self.protected_secret = encode(secret.trim())?;
        }
        #[cfg(not(windows))]
        {
            crypto::store("password", password)?;
            crypto::store("totp-secret", secret.trim())?;
            self.protected_password = if password.is_empty() {
                String::new()
            } else {
                "system-keyring-v1".into()
            };
            self.protected_secret = if secret.trim().is_empty() {
                String::new()
            } else {
                "system-keyring-v1".into()
            };
        }
        let path = config_path()?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, serde_json::to_vec_pretty(self)?).context("Запись настроек")
    }
    pub fn load_password(&self) -> Result<String> {
        #[cfg(windows)]
        {
            decode(&self.protected_password)
        }
        #[cfg(not(windows))]
        {
            crypto::load("password")
        }
    }
    pub fn load_secret(&self) -> Result<String> {
        #[cfg(windows)]
        {
            decode(&self.protected_secret)
        }
        #[cfg(not(windows))]
        {
            crypto::load("totp-secret")
        }
    }
    pub fn is_ready(&self) -> bool {
        !self.username.is_empty() && !self.protected_password.is_empty()
    }
    pub fn data_dir(&self) -> Result<PathBuf> {
        Ok(base_dirs()?.data_local_dir().join(app_dir()))
    }
    pub fn config_path() -> Result<PathBuf> {
        config_path()
    }
    pub fn refresh_citrix_path(&mut self) -> Option<PathBuf> {
        let found = discover_citrix()?;
        self.citrix_path = found.to_string_lossy().into_owned();
        Some(found)
    }
}

pub fn discover_citrix() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        if let Some(p) = env::var_os("ProgramFiles(x86)") {
            candidates.push(PathBuf::from(p).join(r"Citrix\ICA Client\wfcrun32.exe"));
        }
        if let Some(p) = env::var_os("ProgramFiles") {
            candidates.push(PathBuf::from(p).join(r"Citrix\ICA Client\wfcrun32.exe"));
        }
        if let Some(p) = env::var_os("LOCALAPPDATA") {
            candidates.push(PathBuf::from(p).join(r"Citrix\ICA Client\wfcrun32.exe"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        candidates.extend(
            [
                "/Applications/Citrix Workspace.app/Contents/MacOS/Citrix Viewer",
                "/Applications/Citrix Viewer.app/Contents/MacOS/Citrix Viewer",
            ]
            .into_iter()
            .map(PathBuf::from),
        );
    }
    #[cfg(target_os = "linux")]
    {
        candidates.extend(
            [
                "/opt/Citrix/ICAClient/wfica",
                "/usr/lib/ICAClient/wfica",
                "/usr/bin/wfica",
            ]
            .into_iter()
            .map(PathBuf::from),
        );
    }
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            for name in platform_binary_names() {
                candidates.push(dir.join(name));
            }
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}

fn platform_binary_names() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &["wfcrun32.exe", "wfica32.exe"]
    }
    #[cfg(target_os = "macos")]
    {
        &["Citrix Viewer"]
    }
    #[cfg(target_os = "linux")]
    {
        &["wfica"]
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        &[]
    }
}

fn config_path() -> Result<PathBuf> {
    Ok(base_dirs()?
        .config_dir()
        .join(app_dir())
        .join("config.json"))
}
fn base_dirs() -> Result<BaseDirs> {
    BaseDirs::new().context("Не найден системный каталог настроек")
}
fn app_dir() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "citrix-vdi-launcher"
    }
    #[cfg(not(target_os = "linux"))]
    {
        "CitrixVdiLauncher"
    }
}
#[cfg(windows)]
fn encode(v: &str) -> Result<String> {
    if v.is_empty() {
        Ok(String::new())
    } else {
        Ok(STANDARD.encode(crypto::protect(v.as_bytes())?))
    }
}
#[cfg(windows)]
fn decode(v: &str) -> Result<String> {
    if v.is_empty() {
        return Ok(String::new());
    }
    String::from_utf8(crypto::unprotect(
        &STANDARD.decode(v).context("Некорректное DPAPI-поле")?,
    )?)
    .context("DPAPI вернул не UTF-8")
}
