use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::i18n;

/* ─────────────────────────────────────────────────────────────
   undervolt.rs — Intel 13th Gen Undervolt & Power Limits
   ───────────────────────────────────────────────────────────── */

fn is_cpu_locked(cpu: &str) -> bool {
    let name_part = cpu.split('·').next().unwrap_or(cpu).to_uppercase();
    if name_part.contains("AMD") { return false; } // AMD handled by ryzenadj in daemon
    if !name_part.contains("INTEL") { return false; }
    
    // Check for 12, 13, 14th gen
    if name_part.contains("-12") || name_part.contains("-13") || name_part.contains("-14") 
        || name_part.contains(" 12") || name_part.contains(" 13") || name_part.contains(" 14") {
        if name_part.contains("HK") || name_part.contains("HX") {
            return false;
        }
        if name_part.contains("H") || name_part.contains("P") || name_part.contains("U") {
            return true;
        }
    }
    // Check for Core Ultra
    if name_part.contains("ULTRA") {
        return true;
    }
    false
}

pub fn build_page() -> gtk::Box {
    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(16)
        .build();

    // ── Header ───────────────────────────────────────────────
    let hdr = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_bottom(2)
        .build();
    hdr.append(&gtk::Label::builder()
        .label(i18n::t("title_undervolt"))
        .css_classes(["page-title"])
        .halign(gtk::Align::Start)
        .build());
    hdr.append(&gtk::Label::builder()
        .label(i18n::t("uv_desc"))
        .css_classes(["os-section-desc"])
        .halign(gtk::Align::Start)
        .build());
    page.append(&hdr);

    // ── Protection / Warning Banner ───────────────────────────
    let warn_card = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .css_classes(["os-card"])
        .build();

    let warn_icon = gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .pixel_size(24)
        .valign(gtk::Align::Center)
        .build();
    warn_card.append(&warn_icon);

    let warn_text_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();
    let warn_title = gtk::Label::builder()
        .label(i18n::t("uv_lock_title"))
        .css_classes(["chip-title"])
        .halign(gtk::Align::Start)
        .build();
    warn_text_box.append(&warn_title);
    
    let warn_desc = gtk::Label::builder()
        .label(i18n::t("uv_lock_desc"))
        .css_classes(["os-section-desc"])
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    warn_text_box.append(&warn_desc);
    
    warn_card.append(&warn_text_box);

    let status_badge = gtk::Label::builder()
        .label(i18n::t("msr_protected"))
        .css_classes(["badge-warn"])
        .valign(gtk::Align::Center)
        .build();
    warn_card.append(&status_badge);

    page.append(&warn_card);

    // ── Voltage Offsets Section ───────────────────────────────
    let volt_group = adw::PreferencesGroup::builder()
        .title(i18n::t("voltage_offsets_group"))
        .description(i18n::t("voltage_offsets_desc"))
        .build();

    // 1. Core Voltage Offset — initialize then load from daemon
    let core_val = Rc::new(RefCell::new(0i32));
    let core_row = adw::ActionRow::builder()
        .title(i18n::t("core_offset_title"))
        .subtitle(i18n::t("core_offset_sub"))
        .build();
    let core_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, -150.0, 0.0, 5.0);
    core_scale.set_value(0.0);
    core_scale.set_hexpand(true);
    core_scale.set_size_request(200, -1);
    core_scale.set_valign(gtk::Align::Center);
    let core_lbl = gtk::Label::builder()
        .label("0 mV")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let core_lbl_clone = core_lbl.clone();
    let core_val_clone = core_val.clone();
    core_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        *core_val_clone.borrow_mut() = val;
        core_lbl_clone.set_label(&format!("{} mV", val));
    });
    core_row.add_suffix(&core_scale);
    core_row.add_suffix(&core_lbl);
    volt_group.add(&core_row);

    // 2. CPU Cache (Ring) Offset
    let cache_row = adw::ActionRow::builder()
        .title(i18n::t("cache_offset_title"))
        .subtitle(i18n::t("cache_offset_sub"))
        .build();
    let cache_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, -150.0, 0.0, 5.0);
    cache_scale.set_value(0.0);
    cache_scale.set_hexpand(true);
    cache_scale.set_size_request(200, -1);
    cache_scale.set_valign(gtk::Align::Center);
    let cache_lbl = gtk::Label::builder()
        .label("0 mV")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let cache_lbl_clone = cache_lbl.clone();
    cache_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        cache_lbl_clone.set_label(&format!("{} mV", val));
    });
    cache_row.add_suffix(&cache_scale);
    cache_row.add_suffix(&cache_lbl);
    volt_group.add(&cache_row);

    page.append(&volt_group);

    // ── Power Limits (PL1 / PL2 / Tau / TCC) ───────────────────
    let pwr_group = adw::PreferencesGroup::builder()
        .title(i18n::t("power_limits_title"))
        .description(i18n::t("power_limits_desc"))
        .build();

    // PL1 (Sürekli Güç Limiti)
    let pl1_row = adw::ActionRow::builder()
        .title(i18n::t("pl1_label"))
        .subtitle(i18n::t("pl1_sub"))
        .build();
    let pl1_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 25.0, 75.0, 1.0);
    pl1_scale.set_value(45.0);
    pl1_scale.set_hexpand(true);
    pl1_scale.set_size_request(200, -1);
    pl1_scale.set_valign(gtk::Align::Center);
    let pl1_lbl = gtk::Label::builder()
        .label("45 W")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let pl1_lbl_clone = pl1_lbl.clone();
    pl1_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        pl1_lbl_clone.set_label(&format!("{} W", val));
    });
    pl1_row.add_suffix(&pl1_scale);
    pl1_row.add_suffix(&pl1_lbl);
    pwr_group.add(&pl1_row);

    // PL2 (Kısa Süreli Turbo Güç Limiti)
    let pl2_row = adw::ActionRow::builder()
        .title(i18n::t("pl2_label"))
        .subtitle(i18n::t("pl2_sub"))
        .build();
    let pl2_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 45.0, 115.0, 1.0);
    pl2_scale.set_value(95.0);
    pl2_scale.set_hexpand(true);
    pl2_scale.set_size_request(200, -1);
    pl2_scale.set_valign(gtk::Align::Center);
    let pl2_lbl = gtk::Label::builder()
        .label("95 W")
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let pl2_lbl_clone = pl2_lbl.clone();
    pl2_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        pl2_lbl_clone.set_label(&format!("{} W", val));
    });
    pl2_row.add_suffix(&pl2_scale);
    pl2_row.add_suffix(&pl2_lbl);
    pwr_group.add(&pl2_row);

    // TCC Offset (Termal Tetikleme Sıcaklığı)
    let tcc_row = adw::ActionRow::builder()
        .title(i18n::t("tcc_label"))
        .subtitle(i18n::t("tcc_sub"))
        .build();
    let tcc_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 15.0, 1.0);
    tcc_scale.set_value(3.0);
    tcc_scale.set_hexpand(true);
    tcc_scale.set_size_request(200, -1);
    tcc_scale.set_valign(gtk::Align::Center);
    let tcc_lbl = gtk::Label::builder()
        .label(format!("97°C ({}: 3°C)", i18n::t("tcc_offset_str")))
        .css_classes(["os-monitor-val-sm"])
        .valign(gtk::Align::Center)
        .margin_start(8)
        .build();
    let tcc_lbl_clone = tcc_lbl.clone();
    tcc_scale.connect_value_changed(move |s| {
        let val = s.value() as i32;
        let max_temp = 100 - val;
        tcc_lbl_clone.set_label(&format!("{}°C ({}: {}°C)", max_temp, i18n::t("tcc_offset_str"), val));
    });
    tcc_row.add_suffix(&tcc_scale);
    tcc_row.add_suffix(&tcc_lbl);
    pwr_group.add(&tcc_row);

    page.append(&pwr_group);

    // ── Action Buttons Row ────────────────────────────────────
    let actions_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::End)
        .margin_top(4)
        .margin_bottom(12)
        .build();

    let reset_btn = gtk::Button::builder()
        .label(i18n::t("reset_defaults"))
        .css_classes(["ec-btn"])
        .build();
    let core_s_c = core_scale.clone();
    let cache_s_c = cache_scale.clone();
    let pl1_s_c = pl1_scale.clone();
    let pl2_s_c = pl2_scale.clone();
    let tcc_s_c = tcc_scale.clone();
    reset_btn.connect_clicked(move |_| {
        core_s_c.set_value(0.0);
        cache_s_c.set_value(0.0);
        pl1_s_c.set_value(45.0);
        pl2_s_c.set_value(95.0);
        tcc_s_c.set_value(0.0);
    });

    let apply_btn = gtk::Button::builder()
        .label(i18n::t("apply_save"))
        .css_classes(["suggested-action"])
        .build();

    let core_val_c = core_val.clone();
    let pl1_s_apply = pl1_scale.clone();
    let pl2_s_apply = pl2_scale.clone();
    let cache_s_apply = cache_scale.clone();
    let tcc_s_apply = tcc_scale.clone();
    apply_btn.connect_clicked(move |_| {
        let core_mv = *core_val_c.borrow();
        let cache_mv = cache_s_apply.value() as i32;
        let pl1 = pl1_s_apply.value() as i32;
        let pl2 = pl2_s_apply.value() as i32;
        let tcc = tcc_s_apply.value() as i32;
        crate::daemon_client::set_undervolt_sync(core_mv, cache_mv);
        crate::daemon_client::set_power_limits_sync(pl1, pl2);
        crate::daemon_client::set_tcc_offset_sync(tcc);
    });

    actions_box.append(&reset_btn);
    actions_box.append(&apply_btn);

    page.append(&actions_box);

    // ── Load current state from daemon (async) ────────────────
    let core_s_load = core_scale.clone();
    let cache_s_load = cache_scale.clone();
    let pl1_s_load = pl1_scale.clone();
    let pl2_s_load = pl2_scale.clone();
    let tcc_s_load = tcc_scale.clone();
    let badge_load = status_badge.clone();
    
    let cache_row_clone = cache_row.clone();
    let core_row_clone = core_row.clone();
    let core_scale_range = core_scale.clone();
    
    let warn_title_clone = warn_title.clone();
    let warn_desc_clone = warn_desc.clone();
    let volt_group_clone = volt_group.clone();

    glib::spawn_future_local(async move {
        // First check CPU lock
        let mut locked = false;
        let mut is_amd = false;
        if let Ok(hw_json) = crate::daemon_client::get_hardware_specs_async().await {
            if let Ok(hw) = serde_json::from_str::<crate::daemon_client::HardwareSpecs>(&hw_json) {
                is_amd = hw.cpu_spec.to_uppercase().contains("AMD");
                if is_cpu_locked(&hw.cpu_spec) {
                    locked = true;
                    warn_title_clone.set_label(i18n::t("uv_unsupported_title"));
                    warn_desc_clone.set_label(&i18n::t("uv_unsupported_desc").replace("{}", &hw.cpu_spec));
                    badge_load.set_label("Locked");
                    badge_load.set_css_classes(&["badge-err"]);
                    
                    volt_group_clone.set_sensitive(false);
                    // allow power limits and TCC offset for locked CPUs
                }
            }
        }

        if let Ok(json) = crate::daemon_client::get_undervolt_state_async().await {
            if let Ok(state) = serde_json::from_str::<serde_json::Value>(&json) {
                // Determine if daemon thinks it's AMD (fallback)
                if let Some(amd_flag) = state.get("is_amd").and_then(|v| v.as_bool()) {
                    is_amd = amd_flag;
                }

                if is_amd {
                    warn_title_clone.set_label("AMD Curve Optimizer & PBO");
                    if let Some(w) = state.get("warning").and_then(|v| v.as_str()) {
                        warn_desc_clone.set_label(w);
                        badge_load.set_label("RyzenAdj Missing");
                        badge_load.set_css_classes(&["badge-err"]);
                    } else {
                        warn_desc_clone.set_label("RyzenAdj is managing Curve Optimizer and Power Limits.");
                        badge_load.set_label("RyzenAdj OK");
                        badge_load.set_css_classes(&["badge-ok"]);
                    }
                    core_scale_range.set_range(-30.0, 30.0);
                    cache_row_clone.set_visible(false);
                    core_row_clone.set_title("Curve Optimizer (All Core)");
                    core_row_clone.set_subtitle("Negative values = Undervolt. Range: -30 to +30.");
                } else if !locked {
                    badge_load.set_label("MSR OK");
                    badge_load.set_css_classes(&["badge-ok"]);
                }

                if let Some(core) = state.get("offsets").and_then(|v| v.get("core")).and_then(|v| v.as_f64()) {
                    core_s_load.set_value(core);
                }
                // Also parse old state format just in case
                if let Some(core) = state.get("core").and_then(|v| v.as_f64()) {
                    core_s_load.set_value(core);
                }
                
                if let Some(cache) = state.get("offsets").and_then(|v| v.get("cache")).and_then(|v| v.as_f64()) {
                    cache_s_load.set_value(cache);
                }
                if let Some(cache) = state.get("cache").and_then(|v| v.as_f64()) {
                    cache_s_load.set_value(cache);
                }

                if let Some(tcc) = state.get("tcc_offset").and_then(|v| v.as_f64()) {
                    tcc_s_load.set_value(tcc);
                }
            }
        }
        
        // Load Power Limits separately
        if let Ok(json) = crate::daemon_client::get_power_profile_raw_async().await {
            if let Ok(state) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(pl1) = state.get("pl1_w").and_then(|v| v.as_f64()) {
                    pl1_s_load.set_value(pl1);
                }
                if let Some(pl2) = state.get("pl2_w").and_then(|v| v.as_f64()) {
                    pl2_s_load.set_value(pl2);
                }
            }
        }
    });

    page
}
