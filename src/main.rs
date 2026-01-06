use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use regex::Regex;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 400.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "idkspot",
        options,
        Box::new(|_cc| Ok(Box::new(IdkspotApp::new()))),
    )
}

struct IdkspotApp {
    compatible: bool,
    compat_message: String,
    interface: String,
    channel: u32,
    frequency: u32,
    ssid: String,
    password: String,
    status_message: String,
    detection_error: Option<String>,
    is_running: Arc<Mutex<bool>>,
    child_process: Arc<Mutex<Option<Child>>>,
}

impl IdkspotApp {
    fn new() -> Self {
        let (compatible, compat_message) = check_compatibility();
        let (interface, frequency, detection_error) = detect_interface();
        let channel = freq_to_channel(frequency);

        Self {
            compatible,
            compat_message,
            interface,
            channel,
            frequency,
            ssid: "idkspot".to_string(),
            password: String::new(),
            status_message: String::new(),
            detection_error,
            is_running: Arc::new(Mutex::new(false)),
            child_process: Arc::new(Mutex::new(None)),
        }
    }
}

/// Configure GNOME Adwaita dark theme
fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Adwaita Dark colors
    let bg_dark = Color32::from_rgb(36, 36, 36);        // #242424 - window bg
    let bg_darker = Color32::from_rgb(30, 30, 30);      // #1e1e1e - headerbar
    let bg_view = Color32::from_rgb(48, 48, 48);        // #303030 - view bg
    let fg_color = Color32::from_rgb(255, 255, 255);    // white text
    let fg_dim = Color32::from_rgb(154, 153, 150);      // #9a9996 - dim text
    let accent = Color32::from_rgb(53, 132, 228);       // #3584e4 - GNOME blue
    let accent_hover = Color32::from_rgb(98, 160, 234); // #62a0ea
    let destructive = Color32::from_rgb(224, 27, 36);   // #e01b24 - red
    let success = Color32::from_rgb(46, 194, 126);      // #2ec27e - green
    let border = Color32::from_rgb(54, 54, 54);         // subtle border

    // Backgrounds
    visuals.panel_fill = bg_dark;
    visuals.window_fill = bg_dark;
    visuals.extreme_bg_color = bg_darker;
    visuals.faint_bg_color = bg_view;

    // Adwaita-style rounding (12px for buttons, 6px for inputs)
    let button_rounding = Rounding::same(6.0);

    visuals.widgets.noninteractive.rounding = button_rounding;
    visuals.widgets.inactive.rounding = button_rounding;
    visuals.widgets.hovered.rounding = button_rounding;
    visuals.widgets.active.rounding = button_rounding;
    visuals.widgets.open.rounding = button_rounding;

    // Widget backgrounds
    visuals.widgets.noninteractive.bg_fill = bg_view;
    visuals.widgets.inactive.bg_fill = bg_view;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(58, 58, 58);
    visuals.widgets.active.bg_fill = Color32::from_rgb(68, 68, 68);

    // Borders
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_hover);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, accent);

    // Text colors
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, fg_dim);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, fg_color);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, fg_color);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, fg_color);

    // Selection and accent
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = Stroke::new(1.0, accent);
    visuals.hyperlink_color = accent;

    // Window styling
    visuals.window_rounding = Rounding::same(12.0);
    visuals.window_stroke = Stroke::new(1.0, border);

    ctx.set_visuals(visuals);
}


impl eframe::App for IdkspotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply custom theme
        configure_visuals(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(15.0);

            // Title - GNOME style
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("idkspot")
                        .size(24.0)
                        .color(Color32::WHITE),
                );
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // Compatibility status
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Hardware Status:").size(14.0));
                ui.add_space(5.0);
                if self.compatible {
                    ui.label(
                        egui::RichText::new("✓ Compatible")
                            .size(14.0)
                            .color(Color32::from_rgb(46, 194, 126)),  // #2ec27e
                    );
                } else {
                    ui.label(
                        egui::RichText::new("✗ Hardware Not Supported")
                            .size(14.0)
                            .color(Color32::from_rgb(224, 27, 36)),  // #e01b24
                    );
                }
            });

            if !self.compat_message.is_empty() {
                ui.add_space(3.0);
                ui.label(
                    egui::RichText::new(&self.compat_message)
                        .size(11.0)
                        .color(Color32::GRAY),
                );
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(10.0);

            // Interface detection
            if let Some(ref err) = self.detection_error {
                ui.label(
                    egui::RichText::new(format!("⚠ {}", err))
                        .size(13.0)
                        .color(Color32::from_rgb(248, 228, 92)),  // #f8e45c - Adwaita warning
                );
            } else {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Interface:").size(14.0));
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new(&self.interface)
                            .size(14.0)
                            .strong()
                            .color(Color32::from_rgb(53, 132, 228)),  // #3584e4
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("Channel {} ({} MHz)", self.channel, self.frequency))
                            .size(13.0)
                            .color(Color32::GRAY),
                    );
                });
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(15.0);

            // Input fields
            let is_running = *self.is_running.lock().unwrap();
            let enabled = self.compatible && self.detection_error.is_none() && !is_running;

            ui.add_enabled_ui(enabled, |ui| {
                egui::Grid::new("inputs")
                    .num_columns(2)
                    .spacing([15.0, 12.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("SSID:").size(14.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.ssid)
                                .desired_width(280.0)
                                .font(egui::TextStyle::Body),
                        );
                        ui.end_row();

                        ui.label(egui::RichText::new("Password:").size(14.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.password)
                                .password(true)
                                .desired_width(280.0)
                                .font(egui::TextStyle::Body),
                        );
                        ui.end_row();
                    });
            });

            ui.add_space(25.0);

            // IGNITE / STOP button
            ui.vertical_centered(|ui| {
                if is_running {
                    // STOP button - Adwaita destructive red
                    let stop_button = egui::Button::new(
                        egui::RichText::new("Stop Hotspot")
                            .size(16.0)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(224, 27, 36))  // #e01b24
                    .min_size(Vec2::new(200.0, 42.0))
                    .rounding(Rounding::same(6.0));

                    if ui.add(stop_button).clicked() {
                        self.status_message = stop_hotspot(&self.interface);
                        *self.is_running.lock().unwrap() = false;
                    }
                } else {
                    // IGNITE button - Adwaita suggested blue
                    let can_ignite = self.compatible && self.detection_error.is_none();
                    let button_color = if can_ignite {
                        Color32::from_rgb(53, 132, 228)  // #3584e4
                    } else {
                        Color32::from_rgb(80, 80, 80)
                    };

                    let ignite_button = egui::Button::new(
                        egui::RichText::new("Start Hotspot")
                            .size(16.0)
                            .color(Color32::WHITE),
                    )
                    .fill(button_color)
                    .min_size(Vec2::new(200.0, 42.0))
                    .rounding(Rounding::same(6.0));

                    if ui.add_enabled(can_ignite, ignite_button).clicked() {
                        let result = start_hotspot(
                            &self.interface,
                            self.channel,
                            &self.ssid,
                            &self.password,
                        );
                        match result {
                            Ok(msg) => {
                                self.status_message = msg;
                                *self.is_running.lock().unwrap() = true;
                            }
                            Err(msg) => {
                                self.status_message = msg;
                            }
                        }
                    }
                }
            });

            // Status message
            if !self.status_message.is_empty() {
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                let color = if self.status_message.starts_with("Error") {
                    Color32::from_rgb(224, 27, 36)    // #e01b24 - error red
                } else if self.status_message.contains("stopped") {
                    Color32::from_rgb(248, 228, 92)   // #f8e45c - warning yellow
                } else {
                    Color32::from_rgb(46, 194, 126)   // #2ec27e - success green
                };

                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(&self.status_message).size(13.0).color(color));
                });
            }

            ui.add_space(10.0);
        });

        // Request repaint to update UI state
        ctx.request_repaint();
    }
}

/// Check Wi-Fi card compatibility by parsing `iw list` output
fn check_compatibility() -> (bool, String) {
    let output = match Command::new("iw").arg("list").output() {
        Ok(o) => o,
        Err(e) => return (false, format!("Failed to run iw list: {}", e)),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut in_valid_section = false;
    let managed_re = Regex::new(r"(?i)#\{[^}]*\bmanaged\b[^}]*\}").unwrap();
    let ap_re = Regex::new(r"(?i)#\{[^}]*\bap\b[^}]*\}").unwrap();

    for line in stdout.lines() {
        if line.contains("valid interface combinations") {
            in_valid_section = true;
            continue;
        }

        if in_valid_section {
            if !line.starts_with('\t') && !line.starts_with(' ') && !line.is_empty() {
                in_valid_section = false;
                continue;
            }

            let has_managed = managed_re.is_match(line);
            let has_ap = ap_re.is_match(line);

            if has_managed && has_ap {
                return (true, "Simultaneous AP+Managed mode supported".to_string());
            }
        }
    }

    (false, "AP+Managed simultaneous mode not found".to_string())
}

/// Detect wireless interface and frequency from `iw dev`
fn detect_interface() -> (String, u32, Option<String>) {
    let output = match Command::new("iw").arg("dev").output() {
        Ok(o) => o,
        Err(e) => return (String::new(), 0, Some(format!("Failed to run iw dev: {}", e))),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    let iface_re = Regex::new(r"Interface\s+(\w+)").unwrap();
    let freq_re = Regex::new(r"channel\s+\d+\s+\((\d+)\s+MHz\)").unwrap();

    let mut interface = String::new();
    let mut frequency: u32 = 0;

    for line in stdout.lines() {
        if let Some(caps) = iface_re.captures(line) {
            interface = caps.get(1).map_or("", |m| m.as_str()).to_string();
        }
        if let Some(caps) = freq_re.captures(line) {
            if let Ok(f) = caps.get(1).map_or("0", |m| m.as_str()).parse() {
                frequency = f;
            }
        }
    }

    if interface.is_empty() {
        return (interface, frequency, Some("No wireless interface found".to_string()));
    }

    if frequency == 0 {
        return (interface, frequency, Some("Could not detect frequency (not connected?)".to_string()));
    }

    (interface, frequency, None)
}

/// Convert frequency (MHz) to channel number
fn freq_to_channel(freq: u32) -> u32 {
    match freq {
        2412 => 1, 2417 => 2, 2422 => 3, 2427 => 4, 2432 => 5,
        2437 => 6, 2442 => 7, 2447 => 8, 2452 => 9, 2457 => 10,
        2462 => 11, 2467 => 12, 2472 => 13, 2484 => 14,
        5180 => 36, 5200 => 40, 5220 => 44, 5240 => 48,
        5260 => 52, 5280 => 56, 5300 => 60, 5320 => 64,
        5500 => 100, 5520 => 104, 5540 => 108, 5560 => 112,
        5580 => 116, 5600 => 120, 5620 => 124, 5640 => 128,
        5660 => 132, 5680 => 136, 5700 => 140, 5720 => 144,
        5745 => 149, 5765 => 153, 5785 => 157, 5805 => 161, 5825 => 165,
        f if f >= 2412 && f <= 2484 => (f - 2407) / 5,
        f if f >= 5180 && f <= 5825 => (f - 5000) / 5,
        _ => 0,
    }
}

/// Start the hotspot using create_ap (spawns in background)
fn start_hotspot(interface: &str, channel: u32, ssid: &str, password: &str) -> Result<String, String> {
    if ssid.is_empty() {
        return Err("Error: SSID cannot be empty".to_string());
    }
    if password.len() < 8 {
        return Err("Error: Password must be at least 8 characters".to_string());
    }

    let interface = interface.to_string();
    let channel_str = channel.to_string();
    let ssid_display = ssid.to_string(); // For display message
    let ssid = ssid.to_string();
    let password = password.to_string();

    // Spawn in background thread to avoid blocking GUI
    std::thread::spawn(move || {
        let _ = Command::new("pkexec")
            .args([
                "create_ap",
                "-c",
                &channel_str,
                &interface,
                &interface,
                &ssid,
                &password,
            ])
            .spawn();
    });

    Ok(format!("🔥 Hotspot '{}' starting on channel {}...", ssid_display, channel))
}

/// Stop the hotspot using create_ap --stop
fn stop_hotspot(interface: &str) -> String {
    let result = Command::new("pkexec")
        .args(["create_ap", "--stop", interface])
        .spawn();

    match result {
        Ok(_) => format!("⏹ Hotspot stopped on {}", interface),
        Err(e) => format!("Error stopping hotspot: {}", e),
    }
}
