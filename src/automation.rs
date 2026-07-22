use crate::{config::AppConfig, network};
use anyhow::{Context, Result, bail};
use std::{path::Path, sync::mpsc::Sender};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct LaunchRequest {
    pub config: AppConfig,
    pub password: String,
    pub secret: String,
    pub manual_otp: String,
}
pub enum LaunchEvent {
    Status(String),
    Finished(Result<()>),
}

pub fn run(request: LaunchRequest, tx: Sender<LaunchEvent>) {
    let result = run_inner(request, &tx);
    let _ = tx.send(LaunchEvent::Finished(result));
}

fn run_inner(req: LaunchRequest, tx: &Sender<LaunchEvent>) -> Result<()> {
    validate(&req.config)?;
    let automatic_otp = !req.secret.trim().is_empty();
    let otp = if automatic_otp {
        generate_totp(&req.secret)?
    } else {
        req.manual_otp
    };
    if otp.len() != 6 || !otp.chars().all(|c| c.is_ascii_digit()) {
        bail!("OTP должен состоять из 6 цифр");
    }
    status(
        tx,
        if automatic_otp {
            "Одноразовый код рассчитан автоматически"
        } else {
            "Одноразовый код введён"
        },
    );
    let progress = |message: &str| status(tx, message);
    let session = network::authenticate(
        &req.config.storefront_url,
        &req.config.username,
        &req.password,
        &otp,
        &progress,
    )?;
    let names = vec![req.config.vdi_name.clone()];
    let data = req.config.data_dir()?;
    network::launch_vdi(
        &session,
        &names,
        &req.config.citrix_path,
        &data.join("launch.ica"),
        &progress,
    )?;
    status(tx, "Рабочий стол открыт в Citrix Workspace");
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
    TOTP::new(Algorithm::SHA1, 6, 1, 30, bytes)
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
    if c.vdi_name.trim().is_empty() {
        bail!("Укажите название VDI")
    }
    Ok(())
}
fn status(tx: &Sender<LaunchEvent>, message: &str) {
    let _ = tx.send(LaunchEvent::Status(message.into()));
}

#[cfg(test)]
mod tests {
    use totp_rs::{Algorithm, TOTP};
    #[test]
    fn rfc6238_sha1_vector_six_digits() {
        let t = TOTP::new(Algorithm::SHA1, 6, 1, 30, b"12345678901234567890".to_vec()).unwrap();
        assert_eq!(t.generate(59), "287082");
    }
}
