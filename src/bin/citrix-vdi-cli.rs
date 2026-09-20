use anyhow::{Context, Result, bail};
use citrix_vdi_launcher::{
    automation::{self, LaunchCommand, LaunchEvent, LaunchRequest},
    config::{self, AppConfig, KnownDesktop},
};
use clap::{Args, Parser, Subcommand};
use std::{
    io::{self, Write},
    process::ExitCode,
    sync::mpsc::{Receiver, Sender},
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
    /// Подключиться к рабочему столу из настроек
    Connect {
        #[arg(long)]
        otp: Option<String>,
    },
    /// Запустить один или несколько рабочих столов за один вход
    Launch {
        /// Названия рабочих столов; без аргументов используется стол из настроек
        names: Vec<String>,
        #[arg(long)]
        otp: Option<String>,
    },
    /// Показать рабочие столы, доступные в Citrix StoreFront
    Desktops {
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
    /// Закрывать GUI после передачи рабочего стола в Citrix Workspace
    #[arg(long = "close-after-launch")]
    close_after_launch: Option<bool>,
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
        Command::Connect { otp } => launch(Vec::new(), otp),
        Command::Launch { names, otp } => launch(names, otp),
        Command::Desktops { otp } => desktops(otp),
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
                    "StoreFront: {}\nVDI: {}\nЛогин: {}\nCitrix: {}\nПароль сохранён: {}\nTOTP-секрет сохранён: {}\nЗакрывать после запуска: {}",
                    c.storefront_url,
                    c.vdi_name,
                    c.username,
                    c.citrix_path,
                    !c.load_password()?.is_empty(),
                    !c.load_secret()?.is_empty(),
                    c.close_after_launch
                );
                if !c.known_desktops.is_empty() {
                    println!("Известные рабочие столы:");
                    for desktop in &c.known_desktops {
                        println!("  {}", desktop.name);
                    }
                }
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
    if let Some(v) = args.close_after_launch {
        c.close_after_launch = v
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

/// A worker plus the one-time code needed for its first sign-in.
struct Worker {
    config: AppConfig,
    commands: Sender<LaunchCommand>,
    events: Receiver<LaunchEvent>,
    manual_otp: String,
}

fn start(otp: Option<String>) -> Result<Worker> {
    let (config, warning) = AppConfig::load();
    if let Some(warning) = warning {
        eprintln!("Предупреждение: {warning}")
    }
    let password = config.load_password()?;
    let secret = config.load_secret()?;
    if config.username.trim().is_empty() || password.is_empty() {
        bail!("Сначала задайте логин и пароль через `citrix-vdi-cli config set`");
    }
    let manual_otp = match (secret.trim().is_empty(), otp) {
        (_, Some(v)) => v,
        (true, None) => {
            print!("OTP: ");
            io::stdout().flush()?;
            let mut v = String::new();
            io::stdin().read_line(&mut v)?;
            v.trim().to_owned()
        }
        (false, None) => String::new(),
    };
    let (tx, events) = std::sync::mpsc::channel();
    let commands = automation::spawn(
        LaunchRequest {
            config: config.clone(),
            password,
            secret,
        },
        tx,
    );
    Ok(Worker {
        config,
        commands,
        events,
        manual_otp,
    })
}

impl Worker {
    /// Run one command to completion, printing progress as it arrives.
    fn wait(&mut self) -> Result<()> {
        loop {
            let Ok(event) = self.events.recv() else {
                bail!("Поток подключения аварийно завершён")
            };
            match event {
                LaunchEvent::Status(s) => println!("{s}"),
                LaunchEvent::Desktops(list) => self.remember(list),
                LaunchEvent::Launched { .. } => {}
                LaunchEvent::Finished(result) => return result,
            }
        }
    }
    fn remember(&mut self, desktops: Vec<KnownDesktop>) {
        if self.config.remember_desktops(desktops)
            && let Err(error) = self.config.save()
        {
            eprintln!("Предупреждение: не удалось сохранить список столов: {error:#}");
        }
    }
    fn send(&self, command: LaunchCommand) -> Result<()> {
        self.commands
            .send(command)
            .map_err(|_| anyhow::anyhow!("Поток подключения аварийно завершён"))
    }
}

fn launch(names: Vec<String>, otp: Option<String>) -> Result<()> {
    let mut worker = start(otp)?;
    let targets = if names.is_empty() {
        let default = worker.config.vdi_name.trim().to_owned();
        if default.is_empty() {
            bail!("Укажите рабочий стол или задайте его через `citrix-vdi-cli config set --vdi`");
        }
        vec![default]
    } else {
        names
    };
    // One sign-in serves every target: the worker keeps the session open.
    let manual_otp = worker.manual_otp.clone();
    for target in targets {
        worker.send(LaunchCommand::Launch {
            key: target.clone(),
            manual_otp: manual_otp.clone(),
        })?;
        worker
            .wait()
            .with_context(|| format!("Запуск рабочего стола «{target}»"))?;
    }
    Ok(())
}

fn desktops(otp: Option<String>) -> Result<()> {
    let mut worker = start(otp)?;
    let manual_otp = worker.manual_otp.clone();
    worker.send(LaunchCommand::Connect { manual_otp })?;
    worker.wait()?;
    for desktop in &worker.config.known_desktops {
        println!("{}", desktop.name);
    }
    Ok(())
}
