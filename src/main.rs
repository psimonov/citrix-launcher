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
    preview: bool,
    settings_can_scroll: bool,
    file_dialog_result: Option<Receiver<Option<std::path::PathBuf>>>,
    last_otp_complete: bool,
}
impl LauncherApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);
        let preview = cfg!(debug_assertions) && std::env::var_os("CITRIX_UI_PREVIEW").is_some();
        let (config, password, secret, warning) = if preview {
            let mut config = AppConfig::default();
            config.storefront_url = "https://gateway.example/".into();
            config.vdi_name = "MY-DESKTOP".into();
            config.username = "user".into();
            (config, "preview-password".into(), String::new(), None)
        } else {
            let (config, warning) = AppConfig::load();
            let password = config.load_password().unwrap_or_default();
            let secret = config.load_secret().unwrap_or_default();
            (config, password, secret, warning)
        };
        Self {
            show_settings: !preview && !config.is_ready(),
            config,
            password,
            secret,
            otp: String::new(),
            status: warning.unwrap_or_else(|| "Готово к подключению".into()),
            running: false,
            events: None,
            preview,
            settings_can_scroll: true,
            file_dialog_result: None,
            last_otp_complete: false,
        }
    }
    fn save(&mut self) -> anyhow::Result<()> {
        if self.preview {
            return Ok(());
        }
        self.config.save_with_secrets(&self.password, &self.secret)
    }
    fn launch(&mut self) {
        if self.preview {
            let (tx, rx) = mpsc::channel();
            self.events = Some(rx);
            self.running = true;
            self.status = "Одноразовый код введён".into();
            std::thread::spawn(move || {
                for message in [
                    "Открытие Citrix Gateway…",
                    "Проверка логина, пароля и одноразового кода…",
                    "Авторизация выполнена. Открытие Citrix StoreFront…",
                    "Сеанс Citrix StoreFront создан. Загрузка рабочих столов…",
                    "Рабочий стол найден",
                    "Подготовка рабочего стола к запуску…",
                    "Получение файла запуска ICA…",
                    "Открытие Citrix Workspace…",
                ] {
                    let _ = tx.send(LaunchEvent::Status(message.into()));
                    std::thread::sleep(std::time::Duration::from_millis(650));
                }
                let _ = tx.send(LaunchEvent::Finished(Ok(())));
            });
            return;
        }
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

    fn browse_for_citrix(&mut self) {
        if self.file_dialog_result.is_some() {
            return;
        }
        let current = self.config.citrix_path.clone();
        let (tx, rx) = mpsc::channel();
        self.file_dialog_result = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(pick_citrix_executable(&current));
        });
    }
}
impl eframe::App for LauncherApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.file_dialog_result {
            match rx.try_recv() {
                Ok(Some(path)) => {
                    self.config.citrix_path = path.to_string_lossy().into_owned();
                    self.status = "Путь к Citrix Workspace выбран".into();
                    self.file_dialog_result = None;
                }
                Ok(None) => {
                    self.status = "Выбор файла отменён".into();
                    self.file_dialog_result = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = "Не удалось открыть окно выбора файла".into();
                    self.file_dialog_result = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }
        if let Some(rx) = &self.events {
            while let Ok(event) = rx.try_recv() {
                match event {
                    LaunchEvent::Status(s) => self.status = s,
                    LaunchEvent::Finished(r) => {
                        self.running = false;
                        self.otp.clear();
                        self.last_otp_complete = false;
                        self.status = match r {
                            Ok(()) => "Рабочий стол открыт в Citrix Workspace".into(),
                            Err(e) => format!("Ошибка: {e:#}"),
                        };
                    }
                }
            }
        }
        let palette = Palette::new(ui.visuals().dark_mode);
        let available = ui.available_size();
        egui::Frame::new()
            .fill(palette.background)
            .inner_margin(24)
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(
                    (available.x - 48.0).max(0.0),
                    (available.y - 48.0).max(0.0),
                ));
                self.content(ui, palette);
            });
    }
}

impl LauncherApp {
    fn content(&mut self, ui: &mut egui::Ui, palette: Palette) {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("Citrix VDI Launcher")
                        .size(22.0)
                        .strong(),
                );
            });
        });

        ui.add_space(18.0);
        if self.show_settings {
            self.settings_content(ui, palette);
            return;
        }

        card(ui, palette, |ui| {
            ui.label(
                egui::RichText::new("РАБОЧИЙ СТОЛ")
                    .size(11.0)
                    .strong()
                    .color(palette.secondary_text),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(if self.config.vdi_name.trim().is_empty() {
                    "VDI не настроен"
                } else {
                    &self.config.vdi_name
                })
                .size(19.0)
                .strong(),
            );
            ui.add_space(8.0);
            ui.horizontal_top(|ui| {
                let state = UiState::from_status(&self.status, self.running);
                status_indicator(ui, state.color(palette), self.running);
                let status_width = (ui.available_width() - 22.0).max(120.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(status_width, 72.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(&self.status).size(14.0).strong())
                                .wrap(),
                        );
                        ui.add_space(4.0);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(self.next_step())
                                    .size(13.0)
                                    .color(palette.secondary_text),
                            )
                            .wrap(),
                        );
                    },
                );
            });
        });

        ui.add_space(14.0);
        if self.secret.trim().is_empty() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Одноразовый код").size(14.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} из 6", self.otp.len()))
                            .size(13.0)
                            .color(palette.secondary_text),
                    );
                });
            });
            ui.add_space(6.0);
            otp_input(ui, &mut self.otp, palette, !self.running);
            let otp_complete = self.otp.len() == 6;
            if otp_complete != self.last_otp_complete {
                self.last_otp_complete = otp_complete;
                if otp_complete {
                    self.status = "Одноразовый код введён".into();
                } else {
                    self.status = "Введите одноразовый код".into();
                }
            }
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(if self.otp.len() == 6 {
                    "Код введён полностью"
                } else {
                    "Введите 6 цифр из приложения-аутентификатора"
                })
                .size(13.0)
                .color(if self.otp.len() == 6 {
                    palette.success
                } else {
                    palette.secondary_text
                }),
            );
        } else {
            ui.horizontal(|ui| {
                success_check_icon(ui, palette.success);
                ui.label(
                    egui::RichText::new("Одноразовый код будет рассчитан автоматически")
                        .color(palette.secondary_text),
                );
            });
        }

        let action_space = (ui.available_height() - 42.0).max(14.0);
        ui.add_space(action_space);
        ui.horizontal(|ui| {
            let connect = egui::Button::new(
                egui::RichText::new(if self.running {
                    "     Подключение…"
                } else {
                    "Подключиться"
                })
                .size(15.0)
                .strong()
                .color(egui::Color32::WHITE),
            )
            .fill(palette.accent)
            .corner_radius(8)
            .min_size(egui::vec2(0.0, 40.0));
            let otp_ready = !self.secret.trim().is_empty() || self.otp.len() == 6;
            let connect_response = ui.add_enabled(!self.running && otp_ready, connect);
            if self.running {
                paint_spinner(ui, &connect_response, 18.0, egui::Color32::WHITE);
            }
            if connect_response.clicked() {
                self.launch();
            }
            if ui
                .add_enabled(
                    !self.running,
                    egui::Button::new("Настройки")
                        .corner_radius(8)
                        .min_size(egui::vec2(0.0, 40.0)),
                )
                .clicked()
            {
                self.show_settings = !self.show_settings;
            }
        });

        if self.running {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(200));
        }
    }

    fn settings_content(&mut self, ui: &mut egui::Ui, palette: Palette) {
        let controls_enabled = !self.running && self.file_dialog_result.is_none();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("Настройки подключения")
                        .size(19.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new("Измените параметры и сохраните их")
                        .size(13.0)
                        .color(palette.secondary_text),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if back_button(ui, controls_enabled).clicked() {
                    self.show_settings = false;
                }
            });
        });
        ui.add_space(12.0);
        let form_height = (ui.available_height() - 96.0).max(100.0);
        let can_scroll = self.settings_can_scroll;
        settings_card(ui, palette, |ui| {
            let output = egui::ScrollArea::vertical()
                .max_height(form_height)
                .min_scrolled_height(form_height)
                .scroll_source(if can_scroll {
                    egui::scroll_area::ScrollSource::default()
                } else {
                    egui::scroll_area::ScrollSource::NONE
                })
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if !controls_enabled {
                        ui.disable();
                    }
                    let content_top = ui.cursor().top();
                    field(ui, "StoreFront URL", &mut self.config.storefront_url, false);
                    field(ui, "Название VDI", &mut self.config.vdi_name, false);
                    field(ui, "Логин", &mut self.config.username, false);
                    field(ui, "Пароль", &mut self.password, true);
                    field(ui, "TOTP-секрет", &mut self.secret, true);
                    if path_field(
                        ui,
                        &mut self.config.citrix_path,
                        palette,
                        self.file_dialog_result.is_some(),
                    ) {
                        self.browse_for_citrix();
                    }
                    ui.cursor().top() - content_top
                });
            let can_scroll = output.inner > output.inner_rect.height() + 0.5;
            if self.settings_can_scroll != can_scroll {
                self.settings_can_scroll = can_scroll;
                ui.ctx().request_repaint();
            }
        });
        let action_space = (ui.available_height() - 42.0).max(12.0);
        ui.add_space(action_space);
        ui.horizontal(|ui| {
            let save = egui::Button::new(
                egui::RichText::new("Сохранить")
                    .size(15.0)
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .fill(palette.accent)
            .corner_radius(8)
            .min_size(egui::vec2(0.0, 40.0));
            if ui.add_enabled(controls_enabled, save).clicked() {
                self.status = match self.save() {
                    Ok(()) => "Настройки сохранены".into(),
                    Err(e) => format!("Ошибка сохранения: {e:#}"),
                };
            }
            if ui
                .add_enabled(
                    controls_enabled,
                    egui::Button::new("Найти Citrix")
                        .corner_radius(8)
                        .min_size(egui::vec2(0.0, 40.0)),
                )
                .clicked()
            {
                self.status = match self.config.refresh_citrix_path() {
                    Some(p) => format!("Citrix найден: {}", p.display()),
                    None => "Citrix Workspace не найден".into(),
                };
            }
        });
    }

    fn next_step(&self) -> &'static str {
        if self.running && self.status.starts_with("Одноразовый") {
            "Код готов, начинается безопасный вход"
        } else if self.running && self.status.starts_with("Проверка логина") {
            "Данные переданы, ожидайте результат авторизации"
        } else if self.running && self.status.contains("Gateway") {
            "Ожидайте завершения авторизации"
        } else if self.running && self.status.contains("StoreFront") {
            "Авторизация завершена, загружаются ресурсы"
        } else if self.running && self.status.contains("Workspace") {
            "Citrix Workspace принимает рабочий стол"
        } else if self.running {
            "Подготавливается выбранный рабочий стол"
        } else if self.status.starts_with("Ошибка") || self.status.contains("не найден")
        {
            "Проверьте настройки и повторите попытку"
        } else if self.status.contains("открыт в Citrix Workspace") {
            "Подключение передано Citrix; launcher можно закрыть"
        } else if self.secret.trim().is_empty() && self.otp.len() < 6 {
            "Введите код из приложения-аутентификатора"
        } else {
            "Можно подключиться к рабочему столу"
        }
    }
}

fn success_check_icon(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
    let center = rect.center();
    let painter = ui.painter();
    painter.circle_filled(center, 7.0, color);
    let stroke = egui::Stroke::new(1.8, egui::Color32::WHITE);
    painter.line_segment(
        [
            center + egui::vec2(-3.3, 0.0),
            center + egui::vec2(-0.8, 2.6),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(-0.8, 2.6),
            center + egui::vec2(3.8, -2.8),
        ],
        stroke,
    );
}

fn field(ui: &mut egui::Ui, label: &str, value: &mut String, secret: bool) {
    ui.label(egui::RichText::new(label).size(13.0).strong());
    ui.add_sized(
        [ui.available_width(), 40.0],
        egui::TextEdit::singleline(value)
            .password(secret)
            .font(egui::FontId::proportional(16.0))
            .vertical_align(egui::Align::Center)
            .margin(egui::Margin::symmetric(10, 0)),
    );
    ui.add_space(9.0);
}

fn path_field(ui: &mut egui::Ui, value: &mut String, palette: Palette, browsing: bool) -> bool {
    ui.label(
        egui::RichText::new("Путь до Citrix Workspace")
            .size(13.0)
            .strong(),
    );
    let width = ui.available_width();
    let mut browse_clicked = false;
    egui::Frame::new()
        .fill(ui.visuals().text_edit_bg_color())
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(8)
        .inner_margin(egui::Margin {
            left: 10,
            right: 4,
            top: 4,
            bottom: 4,
        })
        .show(ui, |ui| {
            ui.set_width((width - 14.0).max(0.0));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let font_id = egui::TextStyle::Button.resolve(ui.style());
                let text_color = ui.visuals().text_color();
                let browse_label = if browsing {
                    "     Открытие…"
                } else {
                    "Обзор…"
                };
                let text_width = ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(browse_label.into(), font_id, text_color)
                        .size()
                        .x
                });
                let button_width = text_width + ui.spacing().button_padding.x * 2.0;
                let input_width = (ui.available_width() - button_width - 8.0).max(80.0);
                ui.add_enabled_ui(false, |ui| {
                    ui.add_sized(
                        [input_width, 32.0],
                        egui::TextEdit::singleline(value)
                            .font(egui::FontId::proportional(16.0))
                            .vertical_align(egui::Align::Center)
                            .margin(egui::Margin::ZERO)
                            .frame(egui::Frame::NONE),
                    );
                });
                let browse_response = ui.add(
                    egui::Button::new(browse_label)
                        .corner_radius(5)
                        .min_size(egui::vec2(0.0, 32.0)),
                );
                if browsing {
                    paint_spinner(ui, &browse_response, 14.0, ui.visuals().text_color());
                }
                browse_clicked = browse_response.clicked();
            });
        });
    browse_clicked
}

fn pick_citrix_executable(current: &str) -> Option<std::path::PathBuf> {
    let current = std::path::Path::new(current.trim());
    let initial_directory = if current.is_file() {
        current.parent()
    } else if current.is_dir() {
        Some(current)
    } else {
        None
    };
    let mut dialog = rfd::FileDialog::new().set_title("Выберите Citrix Workspace");
    if let Some(directory) = initial_directory {
        dialog = dialog.set_directory(directory);
    }
    dialog.pick_file()
}

fn back_button(ui: &mut egui::Ui, enabled: bool) -> egui::Response {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new("     Назад")
            .corner_radius(8)
            .min_size(egui::vec2(0.0, 36.0)),
    );
    let center = egui::pos2(response.rect.left() + 21.0, response.rect.center().y);
    let stroke = egui::Stroke::new(1.5, ui.style().interact(&response).fg_stroke.color);
    ui.painter().line_segment(
        [
            center + egui::vec2(-5.0, 0.0),
            center + egui::vec2(5.0, 0.0),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            center + egui::vec2(-5.0, 0.0),
            center + egui::vec2(-1.0, -4.0),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            center + egui::vec2(-5.0, 0.0),
            center + egui::vec2(-1.0, 4.0),
        ],
        stroke,
    );
    response
}

#[derive(Clone, Copy)]
struct Palette {
    background: egui::Color32,
    card: egui::Color32,
    border: egui::Color32,
    secondary_text: egui::Color32,
    accent: egui::Color32,
    success: egui::Color32,
    error: egui::Color32,
    warning: egui::Color32,
}

impl Palette {
    fn new(dark: bool) -> Self {
        if dark {
            Self {
                background: egui::Color32::from_rgb(20, 23, 29),
                card: egui::Color32::from_rgb(29, 33, 41),
                border: egui::Color32::from_rgb(51, 57, 68),
                secondary_text: egui::Color32::from_rgb(163, 171, 184),
                accent: egui::Color32::from_rgb(64, 126, 255),
                success: egui::Color32::from_rgb(65, 190, 118),
                error: egui::Color32::from_rgb(239, 92, 92),
                warning: egui::Color32::from_rgb(236, 173, 73),
            }
        } else {
            Self {
                background: egui::Color32::from_rgb(244, 246, 249),
                card: egui::Color32::WHITE,
                border: egui::Color32::from_rgb(218, 223, 231),
                secondary_text: egui::Color32::from_rgb(94, 104, 120),
                accent: egui::Color32::from_rgb(38, 103, 230),
                success: egui::Color32::from_rgb(35, 153, 91),
                error: egui::Color32::from_rgb(205, 50, 64),
                warning: egui::Color32::from_rgb(184, 119, 20),
            }
        }
    }
}

#[derive(Clone, Copy)]
enum UiState {
    Ready,
    Busy,
    Success,
    Error,
}

impl UiState {
    fn from_status(status: &str, running: bool) -> Self {
        if running {
            Self::Busy
        } else if status.starts_with("Ошибка")
            || status.starts_with("Не удалось")
            || status.contains("не найден")
        {
            Self::Error
        } else if status.contains("передана")
            || status.contains("сохранены")
            || status.contains("найден:")
            || status.contains("введён")
            || status.contains("рассчитан")
            || status.contains("выполнена")
            || status.contains("создан")
            || status.contains("открыт")
            || status.contains("запущен")
            || status.contains("выбран")
        {
            Self::Success
        } else {
            Self::Ready
        }
    }

    fn color(self, palette: Palette) -> egui::Color32 {
        match self {
            Self::Ready => palette.accent,
            Self::Busy => palette.warning,
            Self::Success => palette.success,
            Self::Error => palette.error,
        }
    }
}

fn status_indicator(ui: &mut egui::Ui, color: egui::Color32, busy: bool) {
    if busy {
        ui.add(egui::Spinner::new().size(12.0).color(color));
    } else {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 20.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(egui::pos2(rect.center().x, rect.top() + 8.0), 4.0, color);
    }
}

fn paint_spinner(
    ui: &mut egui::Ui,
    response: &egui::Response,
    left_offset: f32,
    color: egui::Color32,
) {
    let center = egui::pos2(response.rect.left() + left_offset, response.rect.center().y);
    let start = ui.input(|input| input.time) as f32 * 4.0;
    let points = (0..=10)
        .map(|index| {
            let angle = start + 4.6 * index as f32 / 10.0;
            center + 5.0 * egui::vec2(angle.cos(), angle.sin())
        })
        .collect();
    ui.painter()
        .add(egui::Shape::line(points, egui::Stroke::new(1.7, color)));
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(16));
}

fn otp_input(ui: &mut egui::Ui, otp: &mut String, palette: Palette, enabled: bool) {
    let otp_id = ui.make_persistent_id("otp-input");
    if ui.memory(|memory| memory.has_focus(otp_id)) {
        ui.input_mut(|input| {
            input.events.retain_mut(|event| match event {
                egui::Event::Text(text) | egui::Event::Paste(text) => {
                    text.retain(|character| character.is_ascii_digit());
                    !text.is_empty()
                }
                _ => true,
            });
        });
    }
    let font = egui::FontId::monospace(22.0);
    let glyph_width = ui.fonts_mut(|fonts| fonts.glyph_width(&font, '0'));
    let editor_width = glyph_width * 6.0 + 2.0;
    let field_width = (ui.available_width() - 4.0).max(0.0);
    let field = egui::Frame::new()
        .fill(ui.visuals().text_edit_bg_color())
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(8)
        .show(ui, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(field_width, 48.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let side_space = ((field_width - editor_width) / 2.0).max(0.0);
                    ui.add_space(side_space);
                    let response = ui
                        .add_enabled_ui(enabled, |ui| {
                            ui.add_sized(
                                [editor_width, 48.0],
                                egui::TextEdit::singleline(otp)
                                    .id(otp_id)
                                    .char_limit(6)
                                    .font(font.clone())
                                    .vertical_align(egui::Align::Center)
                                    .margin(egui::Margin::ZERO)
                                    .frame(egui::Frame::NONE),
                            )
                        })
                        .inner;
                    ui.add_space(side_space);
                    response
                },
            )
            .inner
        });
    let response = field.inner;
    let remaining = "0".repeat(6usize.saturating_sub(otp.len()));
    let ghost_x = response.rect.left() + glyph_width * otp.len() as f32;
    ui.painter().text(
        egui::pos2(ghost_x, response.rect.center().y),
        egui::Align2::LEFT_CENTER,
        remaining,
        font,
        palette.secondary_text.gamma_multiply(0.45),
    );
    let field_interaction = field.response.interact(if enabled {
        egui::Sense::click_and_drag()
    } else {
        egui::Sense::hover()
    });
    if enabled
        && (field_interaction.clicked()
            || field_interaction.dragged()
            || response.clicked()
            || response.dragged())
    {
        response.request_focus();
        if let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), response.id) {
            let end = egui::text::CCursor::new(otp.chars().count());
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(end)));
            state.store(ui.ctx(), response.id);
        }
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            field.response.rect,
            8.0,
            egui::Stroke::new(1.0, palette.accent),
            egui::StrokeKind::Inside,
        );
    }
}

fn card(ui: &mut egui::Ui, palette: Palette, content: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width();
    egui::Frame::new()
        .fill(palette.card)
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(12)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_width((width - 32.0).max(0.0));
            content(ui);
        });
}

fn settings_card(ui: &mut egui::Ui, palette: Palette, content: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width();
    egui::Frame::new()
        .fill(palette.card)
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(12)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.set_width((width - 36.0).max(0.0));
            content(ui);
        });
}

fn configure_style(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 9.0);
        style.spacing.text_edit_width = 320.0;
        style.spacing.scroll = egui::style::ScrollStyle::solid();
        style.spacing.scroll.bar_width = 10.0;
        style.spacing.scroll.bar_inner_margin = 16.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
        style.animation_time = 0.12;
        for widget in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = 8.into();
        }
    });
}
