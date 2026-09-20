use crate::{
    config::{AppConfig, KnownDesktop},
    network::{self, Desktop, GatewaySession},
};
use anyhow::{Context, Result, bail};
use std::{
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use totp_rs::{Algorithm, Secret, TOTP};

const TOTP_STEP: u64 = 30;

pub struct LaunchRequest {
    pub config: AppConfig,
    pub password: String,
    pub secret: String,
}

pub enum LaunchCommand {
    /// Sign in, if needed, and report the available desktops.
    Connect { manual_otp: String },
    /// Launch one desktop, signing in first if no session is open yet.
    Launch { key: String, manual_otp: String },
}

pub enum LaunchEvent {
    Status(String),
    /// Desktops offered by StoreFront, sent after every successful sign-in.
    Desktops(Vec<KnownDesktop>),
    Launched {
        key: String,
    },
    /// The current command is over; the worker stays alive for the next one.
    Finished(Result<()>),
}

/// Start a worker that keeps one authenticated session alive.
///
/// The session lives inside the worker thread, so several desktops can be
/// launched from a single sign-in. Dropping the returned sender stops the
/// worker and discards the session.
pub fn spawn(request: LaunchRequest, events: Sender<LaunchEvent>) -> Sender<LaunchCommand> {
    let (commands, rx) = std::sync::mpsc::channel();
    thread::spawn(move || run(request, rx, events));
    commands
}

fn run(request: LaunchRequest, commands: Receiver<LaunchCommand>, events: Sender<LaunchEvent>) {
    let mut worker = Worker {
        request,
        session: None,
        desktops: Vec::new(),
        last_otp_step: None,
    };
    for command in commands {
        let result = match command {
            LaunchCommand::Connect { manual_otp } => worker.connect(&manual_otp, &events),
            LaunchCommand::Launch { key, manual_otp } => worker.launch(&key, &manual_otp, &events),
        };
        let _ = events.send(LaunchEvent::Finished(result));
    }
}

struct Worker {
    request: LaunchRequest,
    session: Option<GatewaySession>,
    desktops: Vec<Desktop>,
    last_otp_step: Option<u64>,
}

impl Worker {
    fn connect(&mut self, manual_otp: &str, events: &Sender<LaunchEvent>) -> Result<()> {
        validate(&self.request.config)?;
        if self.session.is_none() {
            self.authenticate(manual_otp, events)?;
        }
        Ok(())
    }

    fn launch(&mut self, key: &str, manual_otp: &str, events: &Sender<LaunchEvent>) -> Result<()> {
        validate(&self.request.config)?;
        if self.session.is_none() {
            self.authenticate(manual_otp, events)?;
        }
        match self.launch_once(key, events) {
            Err(error) if expired(&error) && self.can_reauthenticate() => {
                // The gateway dropped the session. With a stored seed this is
                // recoverable without involving the user at all.
                status(events, "Сеанс истёк, выполняется повторный вход…");
                self.session = None;
                self.authenticate("", events)?;
                self.launch_once(key, events)
            }
            result => result,
        }
    }

    fn authenticate(&mut self, manual_otp: &str, events: &Sender<LaunchEvent>) -> Result<()> {
        let otp = self.next_otp(manual_otp, events)?;
        let progress = |message: &str| status(events, message);
        let session = network::authenticate(
            &self.request.config.storefront_url,
            &self.request.config.username,
            &self.request.password,
            &otp,
            &progress,
        )?;
        let desktops = network::list_desktops(&session)?;
        if desktops.is_empty() {
            bail!("Citrix StoreFront не вернул ни одного рабочего стола");
        }
        let _ = events.send(LaunchEvent::Desktops(
            desktops
                .iter()
                .map(|desktop| KnownDesktop {
                    key: desktop.key().to_owned(),
                    name: desktop.name.clone(),
                })
                .collect(),
        ));
        self.session = Some(session);
        self.desktops = desktops;
        Ok(())
    }

    fn launch_once(&self, key: &str, events: &Sender<LaunchEvent>) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .context("Нет активного сеанса Citrix StoreFront")?;
        status(events, "Поиск рабочего стола в Citrix StoreFront…");
        let desktop = self.find(key)?;
        status(events, "Рабочий стол найден");
        let data = self.request.config.data_dir()?;
        let progress = |message: &str| status(events, message);
        network::launch_resource(
            session,
            desktop,
            &self.request.config.citrix_path,
            &data.join(ica_file_name(desktop.key())),
            &progress,
        )?;
        let _ = events.send(LaunchEvent::Launched {
            key: desktop.key().to_owned(),
        });
        Ok(())
    }

    /// A cached key identifies a desktop exactly; anything else is treated as a
    /// user-entered name and resolved by the strict matcher.
    fn find(&self, key: &str) -> Result<&Desktop> {
        let wanted = key.trim();
        if let Some(desktop) = self.desktops.iter().find(|desktop| desktop.key() == wanted) {
            return Ok(desktop);
        }
        network::find_desktop(&self.desktops, wanted)
    }

    fn can_reauthenticate(&self) -> bool {
        !self.request.secret.trim().is_empty()
    }

    fn next_otp(&mut self, manual_otp: &str, events: &Sender<LaunchEvent>) -> Result<String> {
        if !self.can_reauthenticate() {
            let otp = manual_otp.trim().to_owned();
            check_otp(&otp)?;
            status(events, "Одноразовый код введён");
            return Ok(otp);
        }
        // NetScaler rejects a code that was already spent, so a second sign-in
        // inside the same 30-second window has to wait for the next one.
        if let Some(previous) = self.last_otp_step {
            let remaining = seconds_until_step_after(previous);
            if remaining > 0 {
                status(events, "Ожидание нового одноразового кода…");
                thread::sleep(Duration::from_secs(remaining));
            }
        }
        let otp = generate_totp(&self.request.secret)?;
        check_otp(&otp)?;
        self.last_otp_step = Some(current_step());
        status(events, "Одноразовый код рассчитан автоматически");
        Ok(otp)
    }
}

/// One ICA file per desktop: concurrent launches must not overwrite each other,
/// and relaunching the same desktop must not leave stale one-time tickets behind.
///
/// Citrix keeps this path in its process command line, so the name also lets the
/// GUI tell which desktop a running Citrix process belongs to.
pub fn ica_file_name(key: &str) -> String {
    let safe: String = key
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .take(48)
        .collect();
    format!("launch-{}.ica", safe.trim_matches('-'))
}

fn expired(error: &anyhow::Error) -> bool {
    let text = format!("{error:#}").to_ascii_lowercase();
    ["401", "403", "unauthorized", "tokenrequired"]
        .iter()
        .any(|marker| text.contains(marker))
}

fn current_step() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() / TOTP_STEP)
        .unwrap_or(0)
}

fn seconds_until_step_after(step: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    if now / TOTP_STEP > step {
        0
    } else {
        ((step + 1) * TOTP_STEP).saturating_sub(now)
    }
}

fn check_otp(otp: &str) -> Result<()> {
    if otp.len() != 6 || !otp.chars().all(|c| c.is_ascii_digit()) {
        bail!("OTP должен состоять из 6 цифр");
    }
    Ok(())
}

fn generate_totp(secret: &str) -> Result<String> {
    let cleaned: String = secret
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    let bytes = Secret::Encoded(cleaned)
        .to_bytes()
        .context("TOTP-секрет должен быть Base32")?;
    TOTP::new(Algorithm::SHA1, 6, 1, TOTP_STEP, bytes)
        .context("Некорректный TOTP-секрет")?
        .generate_current()
        .context("Расчёт TOTP")
}

fn validate(c: &AppConfig) -> Result<()> {
    if !Path::new(&c.citrix_path).is_file() {
        bail!("Citrix не найден: {}", c.citrix_path)
    }
    if !c.storefront_url.starts_with("https://") {
        bail!("StoreFront URL должен начинаться с https://")
    }
    Ok(())
}
fn status(tx: &Sender<LaunchEvent>, message: &str) {
    let _ = tx.send(LaunchEvent::Status(message.into()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use totp_rs::{Algorithm, TOTP};

    #[test]
    fn rfc6238_sha1_vector_six_digits() {
        let t = TOTP::new(Algorithm::SHA1, 6, 1, 30, b"12345678901234567890".to_vec()).unwrap();
        assert_eq!(t.generate(59), "287082");
    }

    #[test]
    fn ica_file_name_is_unique_per_desktop_and_filesystem_safe() {
        assert_eq!(
            ica_file_name("Controller.ALPHA-1"),
            "launch-Controller-ALPHA-1.ica"
        );
        assert_ne!(ica_file_name("alpha"), ica_file_name("beta"));
        assert!(!ica_file_name("../../etc/passwd").contains('/'));
    }

    #[test]
    fn detects_only_session_expiry_errors() {
        assert!(expired(&anyhow::anyhow!("StoreFront resources HTTP 401")));
        assert!(expired(&anyhow::anyhow!("unauthorized")));
        assert!(!expired(&anyhow::anyhow!("Citrix не найден: /missing")));
    }

    #[test]
    fn waits_for_a_fresh_totp_window_after_a_spent_code() {
        assert!(seconds_until_step_after(current_step()) <= TOTP_STEP);
        assert_eq!(seconds_until_step_after(current_step() - 1), 0);
    }
}
