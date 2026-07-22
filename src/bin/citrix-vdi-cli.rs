use anyhow::{Context, Result, bail};
use citrix_vdi_launcher::{
    automation::{self, LaunchEvent, LaunchRequest},
    config::{self, AppConfig},
};
use clap::{Args, Parser, Subcommand};
use std::{
    io::{self, Write},
    process::ExitCode,
    sync::mpsc,
    thread,
};

#[derive(Parser)]
#[command(
    name = "citrix-vdi",
    version,
    about = "Подключение к Citrix VDI без браузера"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Подключиться, используя сохранённые настройки
    Connect {
        #[arg(long)]
        otp: Option<String>,
    },
    /// Показать или изменить конфигурацию
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Найти установленный Citrix Workspace
    DetectCitrix,
}
#[derive(Subcommand)]
enum ConfigCommand {
    Show,
    Path,
    Set(SetArgs),
}
#[derive(Args)]
struct SetArgs {
    #[arg(long)]
    storefront: Option<String>,
    #[arg(long)]
    vdi: Option<String>,
    #[arg(long)]
    username: Option<String>,
    #[arg(long)]
    citrix: Option<String>,
    #[arg(long)]
    password: Option<String>,
    #[arg(long = "totp-secret")]
    totp_secret: Option<String>,
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Ошибка: {e:#}");
            ExitCode::FAILURE
        }
    }
}
fn execute() -> Result<()> {
    match Cli::parse().command {
        Command::Connect { otp } => connect(otp),
        Command::DetectCitrix => {
            let p = config::discover_citrix().context("Citrix Workspace не найден")?;
            println!("{}", p.display());
            Ok(())
        }
        Command::Config { command } => match command {
            ConfigCommand::Path => {
                println!("{}", AppConfig::config_path()?.display());
                Ok(())
            }
            ConfigCommand::Show => {
                let (c, w) = AppConfig::load();
                if let Some(w) = w {
                    eprintln!("Предупреждение: {w}")
                }
                println!(
                    "StoreFront: {}\nVDI: {}\nЛогин: {}\nCitrix: {}\nПароль сохранён: {}\nTOTP-секрет сохранён: {}",
                    c.storefront_url,
                    c.vdi_name,
                    c.username,
                    c.citrix_path,
                    !c.load_password()?.is_empty(),
                    !c.load_secret()?.is_empty()
                );
                Ok(())
            }
            ConfigCommand::Set(args) => set_config(args),
        },
    }
}
fn set_config(args: SetArgs) -> Result<()> {
    let (mut c, _) = AppConfig::load();
    let mut password = c.load_password()?;
    let mut secret = c.load_secret()?;
    if let Some(v) = args.storefront {
        c.storefront_url = v
    }
    if let Some(v) = args.vdi {
        c.vdi_name = v
    }
    if let Some(v) = args.username {
        c.username = v
    }
    if let Some(v) = args.citrix {
        c.citrix_path = v
    }
    if let Some(v) = args.password {
        password = v
    }
    if let Some(v) = args.totp_secret {
        secret = v
    }
    if c.citrix_path.is_empty() {
        c.refresh_citrix_path();
    }
    c.save_with_secrets(&password, &secret)?;
    println!(
        "Настройки сохранены: {}",
        AppConfig::config_path()?.display()
    );
    Ok(())
}
fn connect(otp: Option<String>) -> Result<()> {
    let (c, w) = AppConfig::load();
    if let Some(w) = w {
        eprintln!("Предупреждение: {w}")
    }
    let password = c.load_password()?;
    let secret = c.load_secret()?;
    if c.username.trim().is_empty() || password.is_empty() {
        bail!("Сначала задайте логин и пароль через `citrix-vdi-cli config set`");
    }
    let manual = match (secret.is_empty(), otp) {
        (true, Some(v)) => v,
        (true, None) => {
            print!("OTP: ");
            io::stdout().flush()?;
            let mut v = String::new();
            io::stdin().read_line(&mut v)?;
            v.trim().to_owned()
        }
        (_, Some(v)) => v,
        (_, None) => String::new(),
    };
    let request = LaunchRequest {
        config: c,
        password,
        secret,
        manual_otp: manual,
    };
    let (tx, rx) = mpsc::channel();
    let worker = thread::spawn(move || automation::run(request, tx));
    for event in rx {
        match event {
            LaunchEvent::Status(s) => println!("{s}"),
            LaunchEvent::Finished(result) => result?,
        }
    }
    worker
        .join()
        .map_err(|_| anyhow::anyhow!("Поток подключения аварийно завершён"))?;
    Ok(())
}
