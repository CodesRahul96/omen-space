use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use crate::i18n;
use crate::daemon_client;

pub fn build_page(_window: &adw::ApplicationWindow) -> gtk::Box {
    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(20)
        .build();

    // ── 1. Page Header ────────────────────────────────────────────────────────
    let hdr = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_bottom(4)
        .build();

    let title_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .build();

    title_row.append(&gtk::Label::builder()
        .label(i18n::t("title_overlay"))
        .css_classes(["page-title"])
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build());

    let status_badge = gtk::Label::builder()
        .label(i18n::t("overlay_status_active"))
        .css_classes(["badge-ok"])
        .valign(gtk::Align::Center)
        .build();
    title_row.append(&status_badge);

    hdr.append(&title_row);
    hdr.append(&gtk::Label::builder()
        .label(i18n::t("overlay_desc"))
        .css_classes(["os-section-desc"])
        .halign(gtk::Align::Start)
        .build());
    page.append(&hdr);

    // ── 2. Hero Launch Card ───────────────────────────────────────────────────
    let hero_group = adw::PreferencesGroup::builder()
        .title(i18n::t("overlay_group"))
        .build();

    let launch_row = adw::ActionRow::builder()
        .title(i18n::t("overlay_launch_now"))
        .subtitle(i18n::t("overlay_launch_now_sub"))
        .build();

    let launch_btn = gtk::Button::builder()
        .label(i18n::t("overlay_launch_btn"))
        .icon_name("preferences-desktop-display-symbolic")
        .css_classes(["suggested-action", "pill"])
        .valign(gtk::Align::Center)
        .build();

    launch_btn.connect_clicked(|_| {
        let rt = daemon_client::get_runtime();
        rt.spawn(async {
            let _ = daemon_client::send_toggle_overlay_signal().await;
        });
    });

    launch_row.add_suffix(&launch_btn);
    hero_group.add(&launch_row);
    page.append(&hero_group);

    // ── 3. Keybindings & Shortcuts Cheatsheet ─────────────────────────────────
    let shortcuts_group = adw::PreferencesGroup::builder()
        .title(i18n::t("overlay_shortcuts_title"))
        .description(i18n::t("overlay_shortcuts_sub"))
        .build();

    let sc_items = [
        ("Shift + F2", "Global Hotkey", "Toggles the on-screen overlay HUD anywhere, including in games", "badge-ok"),
        ("1 / 2 / 3", "Power Modes", "1: Quiet (Eco) • 2: Balanced (Default) • 3: Performance (Max Power)", "badge-accent"),
        ("Q / W / E", "Fan Modes", "Q: Auto (Smart Curve) • W: Max (100% Turbo) • E: Custom Preset", "badge-accent"),
        ("Esc", "Close HUD", "Dismisses overlay and returns immediate focus to game or app", "badge-warn"),
    ];

    for (key, title, desc, badge_style) in sc_items.iter() {
        let row = adw::ActionRow::builder()
            .title(*title)
            .subtitle(*desc)
            .build();

        let badge = gtk::Label::builder()
            .label(*key)
            .css_classes([*badge_style])
            .valign(gtk::Align::Center)
            .build();

        row.add_suffix(&badge);
        shortcuts_group.add(&row);
    }

    page.append(&shortcuts_group);

    // ── 4. Capabilities & Features Card ───────────────────────────────────────
    let feat_group = adw::PreferencesGroup::builder()
        .title(i18n::t("overlay_preview_title"))
        .description(i18n::t("overlay_preview_desc"))
        .build();

    let feat_telemetry = adw::ActionRow::builder()
        .title("Real-Time Cyber Telemetry")
        .subtitle("Displays live CPU & GPU temperatures (°C), wattage (W), dual fan RPMs, and memory usage without taking focus away from games.")
        .build();
    feat_group.add(&feat_telemetry);

    let feat_sync = adw::ActionRow::builder()
        .title("Bidirectional System Synchronization")
        .subtitle("Instant real-time sync with OMEN Space GUI, OMEN Tray, CLI, and Hardware Daemon.")
        .build();
    feat_group.add(&feat_sync);

    let feat_lock = adw::ActionRow::builder()
        .title("Zero-Lag Resident Architecture")
        .subtitle("Resident GTK4 / Libadwaita single-instance daemon guarantees instant (<10ms) overlay appearance.")
        .build();
    feat_group.add(&feat_lock);

    page.append(&feat_group);

    page
}
