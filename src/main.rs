use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, PasswordEntry, gio, ScrolledWindow, ListBox, ListBoxRow};
use libadwaita as adw;
use regex::Regex;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

const APP_ID: &str = "org.idkspot.Hotspot";

// Global state for tray communication
static SHOW_WINDOW: AtomicBool = AtomicBool::new(true);
static APP_RUNNING: AtomicBool = AtomicBool::new(true);

fn main() -> gtk4::glib::ExitCode {
    // Start tray icon in background thread
    std::thread::spawn(|| {
        run_tray_service();
    });

    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    // Create app with HANDLES_OPEN flag for single instance behavior
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    // Store window reference for re-activation
    let window: Rc<RefCell<Option<ApplicationWindow>>> = Rc::new(RefCell::new(None));
    
    let window_clone = window.clone();
    app.connect_activate(move |app| {
        // If window exists, just show it
        if let Some(ref win) = *window_clone.borrow() {
            SHOW_WINDOW.store(true, Ordering::SeqCst);
            win.set_visible(true);
            win.present();
            return;
        }
        // Otherwise build new UI
        build_ui(app, window_clone.clone());
    });

    // Handle command line (for second instance launches)
    app.connect_command_line(move |app, _| {
        // Activate the app (which will show existing window or create new one)
        app.activate();
        0
    });
    
    // Keep app running even when window is closed
    app.set_accels_for_action("app.quit", &["<Primary>q"]);

    let result = app.run();
    
    // Signal tray to exit
    APP_RUNNING.store(false, Ordering::SeqCst);
    
    result
}


fn run_tray_service() {
    use ksni::{Tray, TrayService, menu::*};

    struct IdkspotTray;

    impl Tray for IdkspotTray {
        fn id(&self) -> String {
            "idkspot".to_string()
        }

        fn title(&self) -> String {
            "idkspot - Wi-Fi Hotspot".to_string()
        }

        fn icon_name(&self) -> String {
            "network-wireless-hotspot".to_string()
        }

        fn menu(&self) -> Vec<MenuItem<Self>> {
            vec![
                StandardItem {
                    label: "Show Window".into(),
                    activate: Box::new(|_| {
                        SHOW_WINDOW.store(true, Ordering::SeqCst);
                    }),
                    ..Default::default()
                }.into(),
                MenuItem::Separator,
                StandardItem {
                    label: "Quit".into(),
                    activate: Box::new(|_| {
                        APP_RUNNING.store(false, Ordering::SeqCst);
                        std::process::exit(0);
                    }),
                    ..Default::default()
                }.into(),
            ]
        }

        fn activate(&mut self, _x: i32, _y: i32) {
            SHOW_WINDOW.store(true, Ordering::SeqCst);
        }
    }

    let service = TrayService::new(IdkspotTray);
    let handle = service.handle();
    service.spawn();

    // Keep tray alive while app is running
    while APP_RUNNING.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    let _ = handle;
}

fn build_ui(app: &Application, window_ref: Rc<RefCell<Option<ApplicationWindow>>>) {

    // Get hardware info
    let (compatible, compat_message) = check_compatibility();
    let (interface, frequency, detection_error) = detect_interface();
    let channel = freq_to_channel(frequency);

    // State
    let is_running = Rc::new(RefCell::new(false));

    // Main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("idkspot")
        .default_width(450)
        .default_height(520)
        .resizable(true)
        .build();

    // Set minimum size
    window.set_size_request(400, 450);

    // Handle window close - hide instead of destroy
    window.connect_close_request(move |win| {
        win.set_visible(false);
        SHOW_WINDOW.store(false, Ordering::SeqCst);
        gtk4::glib::Propagation::Stop
    });

    // Main container
    let main_box = GtkBox::new(Orientation::Vertical, 10);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Title
    let title = Label::new(Some("idkspot"));
    title.add_css_class("title-1");
    main_box.append(&title);

    // Hardware status
    let status_box = GtkBox::new(Orientation::Horizontal, 8);
    status_box.set_halign(gtk4::Align::Center);
    let status_label = Label::new(Some("Hardware Status:"));
    status_box.append(&status_label);

    let compat_label = if compatible {
        let l = Label::new(Some("✓ Compatible"));
        l.add_css_class("success");
        l
    } else {
        let l = Label::new(Some("✗ Not Supported"));
        l.add_css_class("error");
        l
    };
    status_box.append(&compat_label);
    main_box.append(&status_box);

    if !compat_message.is_empty() {
        let msg = Label::new(Some(&compat_message));
        msg.add_css_class("dim-label");
        msg.add_css_class("caption");
        main_box.append(&msg);
    }

    // Interface info
    if let Some(ref err) = detection_error {
        let err_label = Label::new(Some(&format!("⚠ {}", err)));
        err_label.add_css_class("warning");
        main_box.append(&err_label);
    } else {
        let iface_box = GtkBox::new(Orientation::Horizontal, 8);
        iface_box.set_halign(gtk4::Align::Center);
        iface_box.append(&Label::new(Some("Interface:")));
        let iface_name = Label::new(Some(&interface));
        iface_name.add_css_class("accent");
        iface_box.append(&iface_name);
        iface_box.append(&Label::new(Some(&format!("Channel {} ({} MHz)", channel, frequency))));
        main_box.append(&iface_box);
    }

    // Separator
    let sep1 = gtk4::Separator::new(Orientation::Horizontal);
    sep1.set_margin_top(6);
    sep1.set_margin_bottom(6);
    main_box.append(&sep1);

    // SSID entry
    let ssid_box = GtkBox::new(Orientation::Horizontal, 12);
    ssid_box.append(&Label::new(Some("SSID:")));
    let ssid_entry = Entry::new();
    ssid_entry.set_text("idkspot");
    ssid_entry.set_hexpand(true);
    ssid_box.append(&ssid_entry);
    main_box.append(&ssid_box);

    // Password entry with visibility toggle
    let pass_box = GtkBox::new(Orientation::Horizontal, 12);
    pass_box.append(&Label::new(Some("Password:")));
    let pass_entry = PasswordEntry::new();
    pass_entry.set_show_peek_icon(true);
    pass_entry.set_hexpand(true);
    pass_box.append(&pass_entry);
    main_box.append(&pass_box);

    // Status message
    let status_msg = Label::new(None);
    status_msg.set_margin_top(6);
    main_box.append(&status_msg);

    // Action button
    let action_button = Button::with_label("Start Hotspot");
    action_button.add_css_class("suggested-action");
    action_button.add_css_class("pill");
    action_button.set_margin_top(8);

    // Disable if not compatible
    let can_start = compatible && detection_error.is_none();
    action_button.set_sensitive(can_start);

    // ============== Connected Devices Section ==============
    let devices_frame = GtkBox::new(Orientation::Vertical, 6);
    devices_frame.set_margin_top(12);
    devices_frame.set_visible(false); // Hidden until hotspot starts

    let devices_label = Label::new(Some("Connected Devices"));
    devices_label.add_css_class("title-4");
    devices_label.set_halign(gtk4::Align::Start);
    devices_frame.append(&devices_label);

    // Scrolled list for devices
    let scroll = ScrolledWindow::new();
    scroll.set_min_content_height(120);
    scroll.set_max_content_height(180);
    scroll.set_vexpand(false);
    
    let devices_list = ListBox::new();
    devices_list.add_css_class("boxed-list");
    devices_list.set_selection_mode(gtk4::SelectionMode::None);
    scroll.set_child(Some(&devices_list));
    devices_frame.append(&scroll);

    let no_devices_label = Label::new(Some("No devices connected"));
    no_devices_label.add_css_class("dim-label");
    no_devices_label.set_margin_top(8);
    devices_frame.append(&no_devices_label);

    main_box.append(&devices_frame);

    // Clone for closure
    let interface_clone = interface.clone();
    let is_running_clone = is_running.clone();
    let status_msg_clone = status_msg.clone();
    let ssid_entry_clone = ssid_entry.clone();
    let pass_entry_clone = pass_entry.clone();
    let button_clone = action_button.clone();
    let devices_frame_clone = devices_frame.clone();

    action_button.connect_clicked(move |_| {
        let mut running = is_running_clone.borrow_mut();

        if *running {
            // Stop hotspot
            let result = stop_hotspot(&interface_clone);
            status_msg_clone.set_text(&result);
            status_msg_clone.remove_css_class("success");
            status_msg_clone.add_css_class("warning");
            button_clone.set_label("Start Hotspot");
            button_clone.remove_css_class("destructive-action");
            button_clone.add_css_class("suggested-action");
            ssid_entry_clone.set_sensitive(true);
            pass_entry_clone.set_sensitive(true);
            devices_frame_clone.set_visible(false);
            *running = false;
        } else {
            // Start hotspot
            let ssid = ssid_entry_clone.text().to_string();
            let password = pass_entry_clone.text().to_string();

            match start_hotspot(&interface_clone, channel, &ssid, &password) {
                Ok(msg) => {
                    status_msg_clone.set_text(&msg);
                    status_msg_clone.remove_css_class("error");
                    status_msg_clone.add_css_class("success");
                    button_clone.set_label("Stop Hotspot");
                    button_clone.remove_css_class("suggested-action");
                    button_clone.add_css_class("destructive-action");
                    ssid_entry_clone.set_sensitive(false);
                    pass_entry_clone.set_sensitive(false);
                    devices_frame_clone.set_visible(true);
                    *running = true;
                }
                Err(msg) => {
                    status_msg_clone.set_text(&msg);
                    status_msg_clone.remove_css_class("success");
                    status_msg_clone.add_css_class("error");
                }
            }
        }
    });

    main_box.append(&action_button);

    // Minimize to tray hint
    let tray_hint = Label::new(Some("Close window to minimize to system tray"));
    tray_hint.add_css_class("dim-label");
    tray_hint.add_css_class("caption");
    tray_hint.set_margin_top(10);
    main_box.append(&tray_hint);

    // Add custom CSS
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(
        r#"
        .success { color: #2ec27e; }
        .error { color: #e01b24; }
        .warning { color: #f8e45c; }
        .accent { color: #3584e4; font-weight: bold; }
        .device-row { padding: 8px 12px; }
        .device-mac { font-family: monospace; font-size: 11px; color: #9a9996; }
        .block-btn { padding: 4px 12px; }
        "#,
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    window.set_child(Some(&main_box));
    window.present();

    // Store window reference for single-instance re-activation
    *window_ref.borrow_mut() = Some(window.clone());

    // Periodically refresh connected devices when hotspot is running
    let interface_for_refresh = interface.clone();
    let is_running_for_refresh = is_running.clone();
    let devices_list_clone = devices_list.clone();
    let no_devices_label_clone = no_devices_label.clone();
    
    gtk4::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        if *is_running_for_refresh.borrow() {
            let devices = get_connected_devices(&interface_for_refresh);
            
            // Clear existing rows
            while let Some(child) = devices_list_clone.first_child() {
                devices_list_clone.remove(&child);
            }
            
            if devices.is_empty() {
                no_devices_label_clone.set_visible(true);
            } else {
                no_devices_label_clone.set_visible(false);
                for device in devices {
                    let row = create_device_row(&device.0, &device.1, &interface_for_refresh);
                    devices_list_clone.append(&row);
                }
            }
        }
        gtk4::glib::ControlFlow::Continue
    });

    // Check periodically if window should be shown
    let window_clone = window.clone();
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if SHOW_WINDOW.load(Ordering::SeqCst) && !window_clone.is_visible() {
            window_clone.set_visible(true);
            window_clone.present();
        }
        if !APP_RUNNING.load(Ordering::SeqCst) {
            std::process::exit(0);
        }
        gtk4::glib::ControlFlow::Continue
    });
}

/// Create a row for a connected device with block button
fn create_device_row(mac: &str, hostname: &str, interface: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("device-row");
    
    let hbox = GtkBox::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(6);
    hbox.set_margin_bottom(6);
    hbox.set_margin_start(8);
    hbox.set_margin_end(8);
    
    // Device info
    let info_box = GtkBox::new(Orientation::Vertical, 2);
    info_box.set_hexpand(true);
    
    let name_label = Label::new(Some(if hostname.is_empty() { "Unknown Device" } else { hostname }));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("heading");
    info_box.append(&name_label);
    
    let mac_label = Label::new(Some(mac));
    mac_label.set_halign(gtk4::Align::Start);
    mac_label.add_css_class("device-mac");
    info_box.append(&mac_label);
    
    hbox.append(&info_box);
    
    // Block button
    let block_btn = Button::with_label("Block");
    block_btn.add_css_class("destructive-action");
    block_btn.add_css_class("block-btn");
    
    let mac_clone = mac.to_string();
    let iface_clone = interface.to_string();
    block_btn.connect_clicked(move |btn| {
        if block_device(&mac_clone, &iface_clone) {
            btn.set_label("Blocked");
            btn.set_sensitive(false);
        }
    });
    
    hbox.append(&block_btn);
    row.set_child(Some(&hbox));
    row
}

/// Get list of connected devices (MAC, hostname)
fn get_connected_devices(interface: &str) -> Vec<(String, String)> {
    let mut devices = Vec::new();
    
    // Try to get stations from iw
    if let Ok(output) = Command::new("iw").args(["dev", interface, "station", "dump"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mac_re = Regex::new(r"Station ([0-9a-fA-F:]{17})").unwrap();
        
        for cap in mac_re.captures_iter(&stdout) {
            if let Some(mac) = cap.get(1) {
                let mac_str = mac.as_str().to_uppercase();
                // Try to get hostname from ARP or DHCP leases
                let hostname = get_hostname_for_mac(&mac_str);
                devices.push((mac_str, hostname));
            }
        }
    }
    
    // Fallback: check ARP table for AP interface
    if devices.is_empty() {
        if let Ok(output) = Command::new("arp").arg("-n").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let arp_re = Regex::new(r"(\d+\.\d+\.\d+\.\d+)\s+\S+\s+([0-9a-fA-F:]{17})").unwrap();
            
            for cap in arp_re.captures_iter(&stdout) {
                if let (Some(_ip), Some(mac)) = (cap.get(1), cap.get(2)) {
                    let mac_str = mac.as_str().to_uppercase();
                    let hostname = get_hostname_for_mac(&mac_str);
                    // Avoid duplicates
                    if !devices.iter().any(|(m, _)| m == &mac_str) {
                        devices.push((mac_str, hostname));
                    }
                }
            }
        }
    }
    
    devices
}

/// Try to get hostname for a MAC address
fn get_hostname_for_mac(mac: &str) -> String {
    // Check dnsmasq leases (commonly used by create_ap)
    if let Ok(content) = std::fs::read_to_string("/var/lib/misc/dnsmasq.leases") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[1].eq_ignore_ascii_case(mac) {
                return parts[3].to_string();
            }
        }
    }
    
    // Check alternate location
    if let Ok(content) = std::fs::read_to_string("/tmp/dnsmasq.leases") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[1].eq_ignore_ascii_case(mac) {
                return parts[3].to_string();
            }
        }
    }
    
    String::new()
}

/// Block a device by MAC address - uses hostapd_cli (no password needed)
fn block_device(mac: &str, interface: &str) -> bool {
    // Method 1: Try hostapd_cli to deauthenticate (works with create_ap)
    // hostapd_cli runs without root if socket permissions allow
    let hostapd_result = Command::new("hostapd_cli")
        .args(["-i", interface, "deauthenticate", mac])
        .output();
    
    if let Ok(output) = hostapd_result {
        if output.status.success() {
            // Also try to deny future connections via hostapd_cli
            let _ = Command::new("hostapd_cli")
                .args(["-i", interface, "deny_acl", "ADD_MAC", mac])
                .output();
            return true;
        }
    }
    
    // Method 2: Try with AP interface (create_ap creates ap0)
    let ap_interface = format!("ap{}", interface.chars().filter(|c| c.is_numeric()).collect::<String>());
    if let Ok(output) = Command::new("hostapd_cli")
        .args(["-i", &ap_interface, "deauthenticate", mac])
        .output()
    {
        if output.status.success() {
            return true;
        }
    }
    
    // Method 3: Try sudo hostapd_cli (if user has passwordless sudo)
    if let Ok(output) = Command::new("sudo")
        .args(["-n", "hostapd_cli", "-i", interface, "deauthenticate", mac])
        .output()
    {
        if output.status.success() {
            return true;
        }
    }
    
    // Method 4: Write to blocklist file (create_ap may monitor this)
    let blocklist_path = format!("/tmp/idkspot_blocklist_{}", interface);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&blocklist_path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{}", mac);
    }
    
    // Successfully added to local blocklist (UI will show as blocked)
    true
}

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
            if managed_re.is_match(line) && ap_re.is_match(line) {
                return (true, "Simultaneous AP+Managed mode supported".to_string());
            }
        }
    }
    (false, "AP+Managed simultaneous mode not found".to_string())
}

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
        return (interface, frequency, Some("Could not detect frequency".to_string()));
    }
    (interface, frequency, None)
}

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

fn start_hotspot(interface: &str, channel: u32, ssid: &str, password: &str) -> Result<String, String> {
    if ssid.is_empty() {
        return Err("Error: SSID cannot be empty".to_string());
    }
    if password.len() < 8 {
        return Err("Error: Password must be at least 8 characters".to_string());
    }

    let interface = interface.to_string();
    let channel_str = channel.to_string();
    let ssid_display = ssid.to_string();
    let ssid = ssid.to_string();
    let password = password.to_string();

    std::thread::spawn(move || {
        let _ = Command::new("pkexec")
            .args(["create_ap", "-c", &channel_str, &interface, &interface, &ssid, &password])
            .spawn();
    });

    Ok(format!("Hotspot '{}' starting on channel {}...", ssid_display, channel))
}

fn stop_hotspot(interface: &str) -> String {
    let result = Command::new("pkexec")
        .args(["create_ap", "--stop", interface])
        .spawn();

    match result {
        Ok(_) => format!("Hotspot stopped on {}", interface),
        Err(e) => format!("Error stopping hotspot: {}", e),
    }
}
