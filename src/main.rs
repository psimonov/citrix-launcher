#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use citrix_vdi_launcher::automation::{self, LaunchCommand, LaunchEvent, LaunchRequest};
use citrix_vdi_launcher::config::{AppConfig, KnownDesktop};
use eframe::egui;
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, Sender};

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
    /// Kept for the lifetime of the session so several desktops can be launched
    /// from a single sign-in. Dropped when settings change.
    commands: Option<Sender<LaunchCommand>>,
    desktops: Vec<KnownDesktop>,
    selected: Option<usize>,
    /// Index of the leftmost visible card.
    carousel_offset: usize,
    /// Desktops with a Citrix session open right now, refreshed every second.
    open_desktops: HashSet<String>,
    preview: bool,
    settings_can_scroll: bool,
    file_dialog_result: Option<Receiver<Option<std::path::PathBuf>>>,
    last_otp_complete: bool,
    session_monitor: SessionMonitor,
    /// Set when "close after launch" is on: the moment the window should quit,
    /// delayed just enough for the final status to be readable.
    close_at: Option<std::time::Instant>,
}

/// Watches which desktops currently have a Citrix session open.
struct SessionMonitor {
    system: sysinfo::System,
    last_check: std::time::Instant,
}

impl SessionMonitor {
    fn new() -> Self {
        Self {
            system: sysinfo::System::new(),
            last_check: std::time::Instant::now() - std::time::Duration::from_secs(1),
        }
    }

    /// Once per second, the command lines of live Citrix session processes.
    fn poll(&mut self) -> Option<Vec<String>> {
        if self.last_check.elapsed() < std::time::Duration::from_secs(1) {
            return None;
        }
        self.last_check = std::time::Instant::now();
        // `refresh_processes` does not collect command lines, which is exactly
        // what the per-desktop match needs; ask for them and nothing else.
        // `OnlyIfNotSet` reads each command line once, when the process appears.
        self.system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::nothing().with_cmd(sysinfo::UpdateKind::OnlyIfNotSet),
        );
        Some(
            self.system
                .processes()
                .values()
                .filter(|process| is_citrix_session_process(&process.name().to_string_lossy()))
                .map(|process| {
                    process
                        .cmd()
                        .iter()
                        .map(|argument| argument.to_string_lossy().to_lowercase())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect(),
        )
    }
}

/// Keys whose ICA file is still referenced by a live Citrix process.
///
/// The launcher itself starts Citrix with the desktop's own ICA file, so the
/// path is something this application controls rather than a client-specific
/// detail. That keeps the match meaningful on every platform. When a client
/// does not expose the path, the result is empty and callers fall back to the
/// aggregate "any session process" signal.
fn running_keys<'a>(
    keys: impl IntoIterator<Item = &'a String>,
    command_lines: &[String],
) -> HashSet<String> {
    keys.into_iter()
        .filter(|key| {
            let file = automation::ica_file_name(key).to_lowercase();
            command_lines
                .iter()
                .any(|command_line| command_line.contains(&file))
        })
        .cloned()
        .collect()
}

/// Processes that exist only while a desktop session is open.
///
/// Verified on Windows: `Citrix.DesktopViewer.App.exe` and `wfica32.exe` appear
/// with the session and disappear when the desktop is disconnected, and the
/// former carries the ICA path. `wfcrun32.exe` is deliberately excluded — it is
/// the connection manager, it survives the session it started, and it keeps a
/// stale ICA path in its command line.
///
/// The macOS (`Citrix Viewer`) and Linux (`wfica`) entries are the executables
/// the launcher starts itself and have not been verified on those platforms
/// yet; if one of them turns out to outlive its session, it belongs here no
/// more than `wfcrun32.exe` does.
fn is_citrix_session_process(name: &str) -> bool {
    [
        "Citrix.DesktopViewer.App.exe",
        "Citrix.DesktopViewer.App",
        "wfica32.exe",
        "wfica32",
        "wfica",
        "Citrix Viewer",
    ]
    .iter()
    .any(|candidate| name.eq_ignore_ascii_case(candidate))
}
impl LauncherApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let preview = cfg!(debug_assertions) && std::env::var_os("CITRIX_UI_PREVIEW").is_some();
        if preview
            && std::env::var("CITRIX_UI_PREVIEW_THEME")
                .is_ok_and(|theme| theme.eq_ignore_ascii_case("light"))
        {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }
        configure_style(&cc.egui_ctx);
        let (config, password, secret, warning) = if preview {
            let mut config = AppConfig::default();
            config.storefront_url = "https://gateway.example/".into();
            config.username = "user".into();
            config.known_desktops = preview_desktops();
            config.close_after_launch = std::env::var_os("CITRIX_UI_PREVIEW_CLOSE").is_some();
            config.vdi_name = config
                .known_desktops
                .first()
                .map(|desktop| desktop.name.clone())
                .unwrap_or_else(|| "MY-DESKTOP".into());
            (config, "preview-password".into(), String::new(), None)
        } else {
            let (config, warning) = AppConfig::load();
            let password = config.load_password().unwrap_or_default();
            let secret = config.load_secret().unwrap_or_default();
            (config, password, secret, warning)
        };
        let desktops = config.desktop_choices();
        let selected = selected_index(&config, &desktops);
        let mut app = Self {
            show_settings: !preview && !config.is_ready(),
            config,
            password,
            secret,
            otp: String::new(),
            status: warning.unwrap_or_else(|| "Готово к подключению".into()),
            running: false,
            events: None,
            commands: None,
            desktops,
            selected,
            carousel_offset: 0,
            open_desktops: HashSet::new(),
            preview,
            settings_can_scroll: true,
            file_dialog_result: None,
            last_otp_complete: false,
            session_monitor: SessionMonitor::new(),
            close_at: None,
        };
        if preview
            && std::env::var("CITRIX_UI_PREVIEW_STATE")
                .is_ok_and(|state| state.eq_ignore_ascii_case("settings"))
        {
            app.show_settings = true;
        }
        if preview
            && std::env::var("CITRIX_UI_PREVIEW_STATE")
                .is_ok_and(|state| state.eq_ignore_ascii_case("error"))
        {
            app.status = "Ошибка: не удалось завершить подключение к рабочему столу. Проверьте параметры Citrix Gateway, доступность сети и сохранённые настройки, затем повторите попытку. Если проблема сохраняется, обратитесь к администратору инфраструктуры и сообщите время возникновения ошибки. Дополнительные данные не требуются.".into();
        }
        app
    }
    fn save(&mut self) -> anyhow::Result<()> {
        if self.preview {
            return Ok(());
        }
        self.config.save_with_secrets(&self.password, &self.secret)
    }
    /// Desktop the buttons act on. `None` disables connecting.
    fn selected_desktop(&self) -> Option<&KnownDesktop> {
        self.selected.and_then(|index| self.desktops.get(index))
    }

    fn select(&mut self, index: usize) {
        if let Some(desktop) = self.desktops.get(index) {
            self.selected = Some(index);
            self.config.vdi_name = desktop.name.clone();
        }
    }

    /// Replace the cached list after a sign-in, keeping the selection on the
    /// same desktop when it is still published.
    fn apply_desktops(&mut self, desktops: Vec<KnownDesktop>) {
        if self.config.remember_desktops(desktops)
            && !self.preview
            && let Err(error) = self.config.save()
        {
            self.status = format!("Не удалось сохранить список столов: {error:#}");
        }
        self.desktops = self.config.desktop_choices();
        self.selected = selected_index(&self.config, &self.desktops);
        self.clamp_carousel();
    }

    fn clamp_carousel(&mut self) {
        self.carousel_offset = self
            .carousel_offset
            .min(self.desktops.len().saturating_sub(1));
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
        let Some(key) = self.selected_desktop().map(|desktop| desktop.key.clone()) else {
            self.status = "Выберите рабочий стол".into();
            return;
        };
        if self.secret.trim().is_empty() && self.otp.trim().is_empty() {
            self.status = "Введите OTP".into();
            return;
        }
        if self.commands.is_none() {
            let (tx, rx) = mpsc::channel();
            self.events = Some(rx);
            self.commands = Some(automation::spawn(
                LaunchRequest {
                    config: self.config.clone(),
                    password: self.password.clone(),
                    secret: self.secret.clone(),
                },
                tx,
            ));
        }
        let command = LaunchCommand::Launch {
            key,
            manual_otp: self.otp.trim().to_owned(),
        };
        match self.commands.as_ref().map(|tx| tx.send(command)) {
            Some(Ok(())) => {
                self.running = true;
                self.status = "Подключение к Citrix Gateway…".into();
            }
            _ => {
                // The worker died; the next attempt starts a fresh one.
                self.end_session();
                self.status = "Поток подключения недоступен. Повторите попытку".into();
            }
        }
    }

    /// Forget the authenticated session, e.g. after settings changed.
    fn end_session(&mut self) {
        self.commands = None;
        self.events = None;
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
        if let Some(deadline) = self.close_at {
            if std::time::Instant::now() >= deadline {
                // Citrix owns the session now, so quitting cannot disturb it.
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ui.ctx()
                    .request_repaint_after(deadline - std::time::Instant::now());
            }
        }
        if !self.preview
            && let Some(command_lines) = self.session_monitor.poll()
        {
            let open = running_keys(
                self.desktops.iter().map(|desktop| &desktop.key),
                &command_lines,
            );
            // The last desktop was closed: the previous "opened" status is stale.
            if open.is_empty() && !self.open_desktops.is_empty() && !self.running {
                self.status = "Готово к подключению".into();
            }
            self.open_desktops = open;
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));
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
        let mut incoming = Vec::new();
        if let Some(rx) = &self.events {
            while let Ok(event) = rx.try_recv() {
                incoming.push(event);
            }
        }
        for event in incoming {
            match event {
                LaunchEvent::Status(s) => self.status = s,
                LaunchEvent::Desktops(list) => self.apply_desktops(list),
                // The card state comes from the process table, not from here.
                LaunchEvent::Launched { .. } => {}
                LaunchEvent::Finished(r) => {
                    self.running = false;
                    self.otp.clear();
                    self.last_otp_complete = false;
                    self.status = match r {
                        Ok(()) => {
                            if self.config.close_after_launch && !self.preview {
                                self.close_at = Some(
                                    std::time::Instant::now()
                                        + std::time::Duration::from_millis(1200),
                                );
                            }
                            "Рабочий стол открыт в Citrix Workspace".into()
                        }
                        Err(e) => format!("Ошибка: {e:#}"),
                    };
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
        let counter = (!self.show_settings)
            .then(|| self.carousel_counter())
            .flatten();
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Citrix VDI Launcher")
                    .size(22.0)
                    .strong(),
            );
            if let Some(counter) = counter {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(counter)
                            .size(13.0)
                            .color(palette.secondary_text),
                    );
                });
            }
        });

        ui.add_space(18.0);
        if self.show_settings {
            self.settings_content(ui, palette);
            return;
        }

        self.desktop_carousel(ui, palette);

        ui.add_space(12.0);
        ui.horizontal_top(|ui| {
            let state = UiState::from_status(&self.status, self.running);
            status_indicator(ui, state.color(palette), self.running);
            let status_width = (ui.available_width() - 22.0).max(120.0);
            ui.allocate_ui_with_layout(
                egui::vec2(status_width, 52.0),
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
            // Connecting is the only launch action: a card click merely selects.
            let ready = otp_ready && self.selected_desktop().is_some();
            let connect_response = ui.add_enabled(!self.running && ready, connect);
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
                        .fill(palette.button)
                        .stroke(egui::Stroke::new(1.0, palette.button_border))
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
                if back_button(ui, controls_enabled, palette).clicked() {
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
                    field(
                        ui,
                        "StoreFront URL",
                        &mut self.config.storefront_url,
                        false,
                        palette,
                    );
                    field(
                        ui,
                        "Рабочий стол по умолчанию",
                        &mut self.config.vdi_name,
                        false,
                        palette,
                    );
                    field(ui, "Логин", &mut self.config.username, false, palette);
                    field(ui, "Пароль", &mut self.password, true, palette);
                    field(ui, "TOTP-секрет", &mut self.secret, true, palette);
                    if path_field(
                        ui,
                        &mut self.config.citrix_path,
                        palette,
                        self.file_dialog_result.is_some(),
                    ) {
                        self.browse_for_citrix();
                    }
                    ui.add_space(2.0);
                    checkbox_field(
                        ui,
                        &mut self.config.close_after_launch,
                        "Закрывать приложение после запуска",
                        "Рабочий стол в Citrix Workspace продолжит работу",
                        palette,
                    );
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
                // Credentials or gateway may have changed: sign in again.
                self.end_session();
                self.desktops = self.config.desktop_choices();
                self.selected = selected_index(&self.config, &self.desktops);
                self.clamp_carousel();
            }
            if ui
                .add_enabled(
                    controls_enabled,
                    egui::Button::new("Найти Citrix")
                        .fill(palette.button)
                        .stroke(egui::Stroke::new(1.0, palette.button_border))
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

    /// "3 / 7" for the header: position of the selected desktop, and how many
    /// exist in total. Hidden when there is nothing to choose between.
    fn carousel_counter(&self) -> Option<String> {
        let total = self.desktops.len();
        if total < 2 {
            return None;
        }
        Some(match self.selected {
            Some(index) => format!("{} / {total}", index + 1),
            None => format!("— / {total}"),
        })
    }

    fn desktop_carousel(&mut self, ui: &mut egui::Ui, palette: Palette) {
        const CARD_HEIGHT: f32 = 118.0;
        const CARD_MIN_WIDTH: f32 = 236.0;
        const CARD_GAP: f32 = 12.0;
        const ARROW_WIDTH: f32 = 14.0;
        // Grid rule: card, gutter, arrow, gutter, window edge. The outer gutter
        // is the frame margin, so only the inner one is added here.
        const ARROW_GUTTER: f32 = 24.0;

        let total = self.desktops.len();
        if total == 0 {
            empty_desktop_card(ui, palette, CARD_HEIGHT);
            return;
        }
        let full_width = ui.available_width();
        let fit = |width: f32| {
            (((width + CARD_GAP) / (CARD_MIN_WIDTH + CARD_GAP)).floor() as usize).max(1)
        };
        let mut visible = fit(full_width).min(total);
        let arrows = visible < total;
        let cards_width = if arrows {
            full_width - 2.0 * (ARROW_WIDTH + ARROW_GUTTER)
        } else {
            full_width
        };
        if arrows {
            visible = fit(cards_width).min(total);
        }
        let max_offset = total - visible;
        self.carousel_offset = self.carousel_offset.min(max_offset);
        // Keep the selected card in view when selection moves by keyboard.
        if let Some(index) = self.selected {
            if index < self.carousel_offset {
                self.carousel_offset = index;
            } else if index >= self.carousel_offset + visible {
                self.carousel_offset = index + 1 - visible;
            }
        }
        let card_width = ((cards_width - CARD_GAP * (visible.saturating_sub(1)) as f32)
            / visible as f32)
            .max(0.0);

        let mut clicked = None;
        let mut step: i32 = 0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if arrows {
                if arrow(
                    ui,
                    palette,
                    CARD_HEIGHT,
                    ARROW_WIDTH,
                    false,
                    self.carousel_offset > 0,
                ) {
                    step = -1;
                }
                ui.add_space(ARROW_GUTTER);
            }
            for slot in 0..visible {
                let index = self.carousel_offset + slot;
                let desktop = &self.desktops[index];
                if desktop_card(
                    ui,
                    palette,
                    egui::vec2(card_width, CARD_HEIGHT),
                    &desktop.name,
                    self.open_desktops.contains(&desktop.key),
                    self.selected == Some(index),
                    !self.running,
                ) {
                    clicked = Some(index);
                }
                if slot + 1 < visible {
                    ui.add_space(CARD_GAP);
                }
            }
            if arrows {
                ui.add_space(ARROW_GUTTER);
                if arrow(
                    ui,
                    palette,
                    CARD_HEIGHT,
                    ARROW_WIDTH,
                    true,
                    self.carousel_offset < max_offset,
                ) {
                    step = 1;
                }
            }
        });

        if !self.running && ui.memory(|memory| memory.focused().is_none()) {
            let (left, right) = ui.input(|input| {
                (
                    input.key_pressed(egui::Key::ArrowLeft),
                    input.key_pressed(egui::Key::ArrowRight),
                )
            });
            let current = self.selected.unwrap_or(0);
            if left && current > 0 {
                clicked = Some(current - 1);
            } else if right && current + 1 < total {
                clicked = Some(current + 1);
            }
        }
        if let Some(index) = clicked {
            self.select(index);
        }
        if step < 0 {
            self.carousel_offset = self.carousel_offset.saturating_sub(1);
        } else if step > 0 {
            self.carousel_offset = (self.carousel_offset + 1).min(max_offset);
        }
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
            "Подключение передано Citrix. Приложение можно закрыть"
        } else if self.secret.trim().is_empty() && self.otp.len() < 6 {
            "Введите код из приложения-аутентификатора"
        } else {
            "Можно подключиться к рабочему столу"
        }
    }
}

/// Index of the configured desktop inside the offered list.
fn selected_index(config: &AppConfig, desktops: &[KnownDesktop]) -> Option<usize> {
    let selected = config.vdi_name.trim();
    if selected.is_empty() {
        return None;
    }
    desktops
        .iter()
        .position(|desktop| desktop.key == selected || desktop.name == selected)
}

fn preview_desktops() -> Vec<KnownDesktop> {
    const NAMES: [&str; 8] = [
        "CR-IMG017-FAT",
        "CR-IMG022-DEV",
        "CR-IMG031-TEST",
        "CR-IMG044-ANALYTICS",
        "CR-IMG051-BUILD",
        "CR-IMG066-QA",
        "CR-IMG072-SANDBOX",
        "CR-IMG089-RESERVE",
    ];
    let count = std::env::var("CITRIX_UI_PREVIEW_DESKTOPS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .min(NAMES.len());
    NAMES[..count]
        .iter()
        .map(|name| KnownDesktop {
            key: format!("Controller.{name}"),
            name: (*name).to_owned(),
        })
        .collect()
}

fn blend(base: egui::Color32, tint: egui::Color32, amount: f32) -> egui::Color32 {
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * amount).round() as u8;
    egui::Color32::from_rgb(
        mix(base.r(), tint.r()),
        mix(base.g(), tint.g()),
        mix(base.b(), tint.b()),
    )
}

fn truncated(
    ui: &mut egui::Ui,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(width);
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

/// One desktop card. Returns true when the user picked it.
fn desktop_card(
    ui: &mut egui::Ui,
    palette: Palette,
    size: egui::Vec2,
    name: &str,
    launched: bool,
    selected: bool,
    enabled: bool,
) -> bool {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let hovered = response.hovered() && enabled;
    let fill = if selected {
        blend(palette.card, palette.accent, 0.14)
    } else if hovered {
        blend(palette.card, palette.accent, 0.05)
    } else {
        palette.card
    };
    let stroke = if selected {
        egui::Stroke::new(2.0, palette.accent)
    } else {
        egui::Stroke::new(1.0, palette.border)
    };
    let text_width = (size.x - 32.0).max(0.0);
    let caption = truncated(
        ui,
        if selected {
            "ВЫБРАН"
        } else {
            "РАБОЧИЙ СТОЛ"
        },
        egui::FontId::proportional(11.0),
        if selected {
            palette.accent
        } else {
            palette.secondary_text
        },
        text_width,
    );
    let title = truncated(
        ui,
        name,
        egui::FontId::proportional(19.0),
        ui.visuals().text_color(),
        text_width,
    );
    let state = truncated(
        ui,
        if launched {
            "Запущен"
        } else {
            "Не запущен"
        },
        egui::FontId::proportional(13.0),
        if launched {
            palette.success
        } else {
            palette.secondary_text
        },
        (text_width - 20.0).max(0.0),
    );
    let painter = ui.painter();
    painter.rect(rect, 12.0, fill, stroke, egui::StrokeKind::Inside);
    let left = rect.left() + 16.0;
    painter.galley(
        egui::pos2(left, rect.top() + 16.0),
        caption,
        palette.secondary_text,
    );
    painter.galley(
        egui::pos2(left, rect.top() + 38.0),
        title,
        ui.visuals().text_color(),
    );
    let state_y = rect.bottom() - 30.0;
    if launched {
        let center = egui::pos2(left + 7.0, state_y + 8.0);
        painter.circle_filled(center, 7.0, palette.success);
        let tick = egui::Stroke::new(1.8, egui::Color32::WHITE);
        painter.line_segment(
            [
                center + egui::vec2(-3.3, 0.0),
                center + egui::vec2(-0.8, 2.6),
            ],
            tick,
        );
        painter.line_segment(
            [
                center + egui::vec2(-0.8, 2.6),
                center + egui::vec2(3.8, -2.8),
            ],
            tick,
        );
        painter.galley(egui::pos2(left + 20.0, state_y), state, palette.success);
    } else {
        painter.galley(egui::pos2(left, state_y), state, palette.secondary_text);
    }
    response.clicked()
}

fn empty_desktop_card(ui: &mut egui::Ui, palette: Palette, height: f32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let width = (rect.width() - 32.0).max(0.0);
    let title = truncated(
        ui,
        "Рабочий стол не настроен",
        egui::FontId::proportional(19.0),
        ui.visuals().text_color(),
        width,
    );
    let hint = truncated(
        ui,
        "Укажите название VDI в настройках",
        egui::FontId::proportional(13.0),
        palette.secondary_text,
        width,
    );
    let painter = ui.painter();
    painter.rect(
        rect,
        12.0,
        palette.card,
        egui::Stroke::new(1.0, palette.border),
        egui::StrokeKind::Inside,
    );
    painter.galley(
        egui::pos2(rect.left() + 16.0, rect.top() + 38.0),
        title,
        ui.visuals().text_color(),
    );
    painter.galley(
        egui::pos2(rect.left() + 16.0, rect.bottom() - 30.0),
        hint,
        palette.secondary_text,
    );
}

/// Narrow full-height carousel arrow. Returns true when clicked.
fn arrow(
    ui: &mut egui::Ui,
    palette: Palette,
    height: f32,
    width: f32,
    forward: bool,
    enabled: bool,
) -> bool {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), sense);
    let color = if !enabled {
        palette.placeholder
    } else if response.hovered() {
        palette.accent
    } else {
        palette.secondary_text
    };
    let center = rect.center();
    let half_width = width / 2.0 - 1.0;
    let half_height = 14.0;
    let tip = egui::pos2(
        center.x + if forward { half_width } else { -half_width },
        center.y,
    );
    let back_x = center.x + if forward { -half_width } else { half_width };
    let stroke = egui::Stroke::new(2.0, color);
    ui.painter()
        .line_segment([egui::pos2(back_x, center.y - half_height), tip], stroke);
    ui.painter()
        .line_segment([egui::pos2(back_x, center.y + half_height), tip], stroke);
    response.clicked()
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

fn field(ui: &mut egui::Ui, label: &str, value: &mut String, secret: bool, palette: Palette) {
    ui.label(egui::RichText::new(label).size(13.0).strong());
    let response = ui.add_sized(
        [ui.available_width(), 40.0],
        egui::TextEdit::singleline(value)
            .password(secret)
            .font(egui::FontId::proportional(16.0))
            .vertical_align(egui::Align::Center)
            .frame(
                egui::Frame::new()
                    .fill(palette.input)
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .inner_margin(egui::Margin::symmetric(10, 0))
                    .corner_radius(8),
            ),
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            8.0,
            egui::Stroke::new(1.0, palette.accent),
            egui::StrokeKind::Inside,
        );
    }
    ui.add_space(9.0);
}

/// A checkbox drawn in the same visual language as the text fields: rounded
/// box, palette border, accent fill when checked.
fn checkbox_field(ui: &mut egui::Ui, value: &mut bool, label: &str, hint: &str, palette: Palette) {
    const BOX: f32 = 18.0;
    const GAP: f32 = 10.0;
    let text_color = ui.visuals().text_color();
    let width = ui.available_width();
    let caption = truncated(
        ui,
        label,
        egui::FontId::proportional(13.0),
        text_color,
        (width - BOX - GAP).max(0.0),
    );
    let row_height = caption.size().y.max(BOX);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, row_height), egui::Sense::click());
    let toggled_by_key = response.has_focus()
        && ui.input(|input| {
            input.key_pressed(egui::Key::Space) || input.key_pressed(egui::Key::Enter)
        });
    if response.clicked() || toggled_by_key {
        *value = !*value;
        response.request_focus();
    }
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let box_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.center().y - BOX / 2.0),
        egui::vec2(BOX, BOX),
    );
    let border = if *value || response.hovered() || response.has_focus() {
        palette.accent
    } else {
        palette.border
    };
    let painter = ui.painter();
    painter.rect(
        box_rect,
        5.0,
        if *value {
            palette.accent
        } else {
            palette.input
        },
        egui::Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    if *value {
        let center = box_rect.center();
        let tick = egui::Stroke::new(2.0, egui::Color32::WHITE);
        painter.line_segment(
            [
                center + egui::vec2(-4.0, 0.2),
                center + egui::vec2(-1.3, 3.0),
            ],
            tick,
        );
        painter.line_segment(
            [
                center + egui::vec2(-1.3, 3.0),
                center + egui::vec2(4.3, -3.2),
            ],
            tick,
        );
    }
    painter.galley(
        egui::pos2(
            box_rect.right() + GAP,
            rect.center().y - caption.size().y / 2.0,
        ),
        caption,
        text_color,
    );
    ui.add_space(3.0);
    ui.label(
        egui::RichText::new(hint)
            .size(12.0)
            .color(palette.secondary_text),
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
        .fill(palette.input)
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
                        .fill(palette.button)
                        .stroke(egui::Stroke::new(1.0, palette.button_border))
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

fn back_button(ui: &mut egui::Ui, enabled: bool, palette: Palette) -> egui::Response {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new("     Назад")
            .fill(palette.button)
            .stroke(egui::Stroke::new(1.0, palette.button_border))
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
    input: egui::Color32,
    border: egui::Color32,
    secondary_text: egui::Color32,
    placeholder: egui::Color32,
    button: egui::Color32,
    button_border: egui::Color32,
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
                input: egui::Color32::from_rgb(8, 10, 13),
                border: egui::Color32::from_rgb(51, 57, 68),
                secondary_text: egui::Color32::from_rgb(163, 171, 184),
                placeholder: egui::Color32::from_rgb(92, 99, 111),
                button: egui::Color32::from_rgb(57, 62, 72),
                button_border: egui::Color32::from_rgb(82, 89, 101),
                accent: egui::Color32::from_rgb(64, 126, 255),
                success: egui::Color32::from_rgb(65, 190, 118),
                error: egui::Color32::from_rgb(239, 92, 92),
                warning: egui::Color32::from_rgb(236, 173, 73),
            }
        } else {
            Self {
                background: egui::Color32::from_rgb(244, 246, 249),
                card: egui::Color32::WHITE,
                input: egui::Color32::from_rgb(248, 249, 251),
                border: egui::Color32::from_rgb(204, 211, 221),
                secondary_text: egui::Color32::from_rgb(94, 104, 120),
                placeholder: egui::Color32::from_rgb(128, 138, 153),
                button: egui::Color32::from_rgb(226, 231, 238),
                button_border: egui::Color32::from_rgb(181, 191, 203),
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
        .fill(palette.input)
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
        palette.placeholder,
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
        if !style.visuals.dark_mode {
            style.visuals.disabled_alpha = 0.65;
        }
        for widget in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = 8.into();
            widget.expansion = 0.0;
            widget.fg_stroke.width = 1.0;
            widget.bg_stroke.width = 1.0;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{is_citrix_session_process, running_keys};
    use std::collections::HashSet;

    #[test]
    fn recognizes_cross_platform_ica_session_processes() {
        for name in [
            "Citrix.DesktopViewer.App.exe",
            "wfica32.exe",
            "WFICA32",
            "wfica",
            "Citrix Viewer",
        ] {
            assert!(is_citrix_session_process(name), "{name}");
        }
        // wfcrun32.exe outlives the session it started, so treating it as a
        // session process makes a launched desktop look permanently open.
        for name in ["Receiver", "SelfService", "concentr", "wfcrun32.exe"] {
            assert!(!is_citrix_session_process(name), "{name}");
        }
    }

    #[test]
    fn keeps_only_desktops_whose_ica_file_is_still_open() {
        // Synthetic keys in the StoreFront shape "CONTROLLER-NAME $ID": Citrix
        // lower-cases the ICA path in its command line, and the launcher
        // replaces every non-alphanumeric character when naming the file.
        let launched: HashSet<String> = [
            "CTRL-01-ALPHA $B200-11-22CD33EF-0001",
            "CTRL-01-BETA $C300-12-44AB55CD-0002",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let command_lines = vec![
            r#""c:\program files (x86)\citrix\ica client\citrix.desktopviewer.app.exe" "c:\users\u\appdata\local\citrixvdilauncher\launch-ctrl-01-beta--c300-12-44ab55cd-0002.ica" /extsessionid:4237317593"#.to_owned(),
            r#""c:\program files (x86)\citrix\ica client\wfica32.exe" mfservice00080486002"#.to_owned(),
        ];
        let running = running_keys(&launched, &command_lines);
        assert_eq!(
            running,
            HashSet::from(["CTRL-01-BETA $C300-12-44AB55CD-0002".to_owned()]),
            "only the desktop whose ICA file is open stays marked"
        );
    }

    #[test]
    fn reports_nothing_when_command_lines_carry_no_ica_path() {
        let launched: HashSet<String> = HashSet::from(["Controller.ALPHA".to_owned()]);
        let running = running_keys(&launched, &["citrix viewer".to_owned()]);
        assert!(running.is_empty(), "callers then keep the previous marks");
    }
}
