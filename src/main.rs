use eframe::egui;
use regex::Regex;
use std::process::Command;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 350.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "idkspot",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(IdkspotApp::new()))
        }),
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
        }
    }
}

impl eframe::App for IdkspotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("🔥 idkspot");
                ui.add_space(10.0);
            });

            ui.separator();

            // Compatibility status
            ui.horizontal(|ui| {
                ui.label("Hardware Status:");
                if self.compatible {
                    ui.colored_label(egui::Color32::GREEN, "✓ Compatible");
                } else {
                    ui.colored_label(egui::Color32::RED, "✗ Hardware Not Supported");
                }
            });

            if !self.compat_message.is_empty() {
                ui.small(&self.compat_message);
            }

            ui.add_space(10.0);
            ui.separator();

            // Interface detection
            if let Some(ref err) = self.detection_error {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠ {}", err));
            } else {
                ui.horizontal(|ui| {
                    ui.label("Using Interface:");
                    ui.strong(&self.interface);
                    ui.label(format!("on Channel {} ({} MHz)", self.channel, self.frequency));
                });
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // Input fields
            let enabled = self.compatible && self.detection_error.is_none();

            ui.add_enabled_ui(enabled, |ui| {
                egui::Grid::new("inputs")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("SSID:");
                        ui.add(egui::TextEdit::singleline(&mut self.ssid).desired_width(250.0));
                        ui.end_row();

                        ui.label("Password:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.password)
                                .password(true)
                                .desired_width(250.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(20.0);

                // IGNITE button
                ui.vertical_centered(|ui| {
                    let button = egui::Button::new(
                        egui::RichText::new("🚀 IGNITE")
                            .size(24.0)
                            .strong(),
                    )
                    .min_size(egui::vec2(200.0, 50.0));

                    if ui.add(button).clicked() {
                        self.status_message = execute_create_ap(
                            &self.interface,
                            self.channel,
                            &self.ssid,
                            &self.password,
                        );
                    }
                });
            });

            // Status message
            if !self.status_message.is_empty() {
                ui.add_space(15.0);
                ui.separator();
                ui.add_space(5.0);

                let color = if self.status_message.starts_with("Error") {
                    egui::Color32::RED
                } else {
                    egui::Color32::LIGHT_BLUE
                };
                ui.colored_label(color, &self.status_message);
            }
        });
    }
}

/// Check Wi-Fi card compatibility by parsing `iw list` output
fn check_compatibility() -> (bool, String) {
    let output = match Command::new("iw").arg("list").output() {
        Ok(o) => o,
        Err(e) => return (false, format!("Failed to run iw list: {}", e)),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Find "valid interface combinations" section and check for simultaneous AP+managed
    let mut in_valid_section = false;

    // Regex patterns that match modes anywhere inside #{ ... } blocks
    // e.g., matches "managed" in "#{ managed }" or "#{ managed, AP }"
    // and "ap" in "#{ AP }" or "#{ AP, P2P-client, P2P-GO }"
    let managed_re = Regex::new(r"(?i)#\{[^}]*\bmanaged\b[^}]*\}").unwrap();
    let ap_re = Regex::new(r"(?i)#\{[^}]*\bap\b[^}]*\}").unwrap();

    for line in stdout.lines() {
        if line.contains("valid interface combinations") {
            in_valid_section = true;
            continue;
        }

        if in_valid_section {
            // End of section detection
            if !line.starts_with('\t') && !line.starts_with(' ') && !line.is_empty() {
                in_valid_section = false;
                continue;
            }

            // Check if this line has both managed and AP modes (in any #{ } block)
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
        return (interface, frequency, Some("Could not detect frequency (interface may not be connected)".to_string()));
    }

    (interface, frequency, None)
}

/// Convert frequency (MHz) to channel number
fn freq_to_channel(freq: u32) -> u32 {
    match freq {
        // 2.4 GHz band
        2412 => 1,
        2417 => 2,
        2422 => 3,
        2427 => 4,
        2432 => 5,
        2437 => 6,
        2442 => 7,
        2447 => 8,
        2452 => 9,
        2457 => 10,
        2462 => 11,
        2467 => 12,
        2472 => 13,
        2484 => 14,
        // 5 GHz band (common channels)
        5180 => 36,
        5200 => 40,
        5220 => 44,
        5240 => 48,
        5260 => 52,
        5280 => 56,
        5300 => 60,
        5320 => 64,
        5500 => 100,
        5520 => 104,
        5540 => 108,
        5560 => 112,
        5580 => 116,
        5600 => 120,
        5620 => 124,
        5640 => 128,
        5660 => 132,
        5680 => 136,
        5700 => 140,
        5720 => 144,
        5745 => 149,
        5765 => 153,
        5785 => 157,
        5805 => 161,
        5825 => 165,
        // Fallback calculation
        f if f >= 2412 && f <= 2484 => (f - 2407) / 5,
        f if f >= 5180 && f <= 5825 => (f - 5000) / 5,
        _ => 0,
    }
}

/// Execute create_ap command with pkexec
fn execute_create_ap(interface: &str, channel: u32, ssid: &str, password: &str) -> String {
    if ssid.is_empty() {
        return "Error: SSID cannot be empty".to_string();
    }
    if password.len() < 8 {
        return "Error: Password must be at least 8 characters".to_string();
    }

    let result = Command::new("pkexec")
        .args([
            "create_ap",
            "-c",
            &channel.to_string(),
            interface,
            interface,
            ssid,
            password,
        ])
        .spawn();

    match result {
        Ok(_) => format!("Hotspot '{}' starting on channel {}...", ssid, channel),
        Err(e) => format!("Error: Failed to start hotspot: {}", e),
    }
}
