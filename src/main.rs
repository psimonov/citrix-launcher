#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use citrix_vdi_launcher::automation::{self, LaunchEvent, LaunchRequest};
use citrix_vdi_launcher::config::AppConfig;
use eframe::egui;
use std::sync::mpsc::{self, Receiver};

fn main() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/icon-256.png"))
        .expect("embedded application icon");
    eframe::run_native(
        "Citrix VDI Launcher",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([650.0, 590.0])
                .with_min_inner_size([570.0, 500.0])
                .with_icon(icon),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(LauncherApp::new(cc)))),
    )
}

struct LauncherApp {
    config: AppConfig,
    password: String,
    secret: String,
    otp: String,
    status: String,
    show_settings: bool,
    running: bool,
    events: Option<Receiver<LaunchEvent>>,
}
impl LauncherApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (config, warning) = AppConfig::load();
        let password = config.load_password().unwrap_or_default();
        let secret = config.load_secret().unwrap_or_default();
        Self {
            show_settings: !config.is_ready(),
            config,
            password,
            secret,
            otp: String::new(),
            status: warning.unwrap_or_else(|| "Готово к подключению".into()),
            running: false,
            events: None,
        }
    }
    fn save(&mut self) -> anyhow::Result<()> {
        self.config.save_with_secrets(&self.password, &self.secret)
    }
    fn launch(&mut self) {
        if let Err(e) = self.save() {
            self.status = format!("Не удалось сохранить настройки: {e:#}");
            return;
        }
        if self.config.username.trim().is_empty() || self.password.is_empty() {
            self.status = "Укажите логин и пароль в настройках".into();
            self.show_settings = true;
            return;
        }
        if self.secret.trim().is_empty() && self.otp.trim().is_empty() {
            self.status = "Введите OTP".into();
            return;
        }
        let request = LaunchRequest {
            config: self.config.clone(),
            password: self.password.clone(),
            secret: self.secret.clone(),
            manual_otp: self.otp.trim().to_owned(),
        };
        let (tx, rx) = mpsc::channel();
        self.events = Some(rx);
        self.running = true;
        self.status = "Подключение к Citrix Gateway…".into();
        std::thread::spawn(move || automation::run(request, tx));
    }
}
impl eframe::App for LauncherApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.events {
            while let Ok(event) = rx.try_recv() {
                match event {
                    LaunchEvent::Status(s) => self.status = s,
                    LaunchEvent::Finished(r) => {
                        self.running = false;
                        self.otp.clear();
                        self.status = match r {
                            Ok(()) => "VDI передана в Citrix Workspace".into(),
                            Err(e) => format!("Ошибка: {e:#}"),
                        };
                    }
                }
            }
        }
        ui.heading("Citrix VDI Launcher");
        ui.add_space(8.0);
        ui.label(format!("Рабочий стол: {}", self.config.vdi_name));
        ui.label(&self.status);
        ui.add_space(12.0);
        if self.secret.trim().is_empty() {
            ui.horizontal(|ui| {
                ui.label("OTP:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.otp)
                        .password(true)
                        .desired_width(200.0)
                        .hint_text("Код из Яндекс ID"),
                );
            });
        } else {
            ui.label("OTP будет рассчитан автоматически из защищённого секрета.");
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.running, egui::Button::new("Подключиться"))
                .clicked()
            {
                self.launch();
            }
            if ui
                .button(if self.show_settings {
                    "Скрыть настройки"
                } else {
                    "Настройки"
                })
                .clicked()
            {
                self.show_settings = !self.show_settings;
            }
        });
        if self.running {
            ui.add(egui::Spinner::new());
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(200));
        }
        if self.show_settings {
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("settings")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        field(ui, "StoreFront URL", &mut self.config.storefront_url, false);
                        field(ui, "Название VDI", &mut self.config.vdi_name, false);
                        field(ui, "Логин", &mut self.config.username, false);
                        field(ui, "Пароль", &mut self.password, true);
                        field(ui, "TOTP-секрет", &mut self.secret, true);
                        field(ui, "Путь Citrix", &mut self.config.citrix_path, false);
                    });
                ui.horizontal(|ui| {
                    if ui.button("Найти Citrix автоматически").clicked() {
                        self.status = match self.config.refresh_citrix_path() {
                            Some(p) => format!("Citrix найден: {}", p.display()),
                            None => "Citrix Workspace не найден".into(),
                        };
                    }
                });
                ui.small("Системная тема выбирается автоматически. Браузер не используется.");
                if ui.button("Сохранить настройки").clicked() {
                    self.status = match self.save() {
                        Ok(()) => "Настройки сохранены".into(),
                        Err(e) => format!("Ошибка сохранения: {e:#}"),
                    };
                }
            });
        }
    }
}
fn field(ui: &mut egui::Ui, label: &str, value: &mut String, secret: bool) {
    ui.label(label);
    ui.add(
        egui::TextEdit::singleline(value)
            .password(secret)
            .desired_width(410.0),
    );
    ui.end_row();
}
