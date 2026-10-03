#![windows_subsystem = "windows"]
use eframe::egui;
use std::fs;
use std::sync::Arc;

mod editor;
mod file_io;
mod formatter;

fn main() -> eframe::Result<()> {
    #[cfg(windows)]
    {
        let ico_path = "assets/app.ico";
        if std::path::Path::new(ico_path).exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(ico_path);
            res.set("FileDescription", "Mini Editor");
            res.compile().unwrap();
        }
    }

    // 设置透明图标
    // use std::sync::Arc;
    use eframe::egui::{IconData, ViewportBuilder};
    let transparent_icon = Arc::new(IconData {
        rgba: vec![0; 4], // RGBA = 0,0,0,0
        width: 1,
        height: 1,
    });

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(transparent_icon) // 关键
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "",
        native_options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = egui::Color32::from_rgb(255, 255, 255);
            visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);
            visuals.extreme_bg_color = egui::Color32::from_rgb(245, 245, 245);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(200, 200, 200);
            visuals.widgets.inactive.fg_stroke = egui::Stroke::NONE;
            // 0.36: Rounding -> CornerRadius
            visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(170, 170, 170);
            visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(140, 140, 140);
            visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
            visuals.text_cursor.stroke.color = egui::Color32::BLACK;
            cc.egui_ctx.set_visuals(visuals);

            let mut fonts = egui::FontDefinitions::default();
            configure_editor_fonts(&mut fonts);
            cc.egui_ctx.set_fonts(fonts);

            let mut style = (*cc.egui_ctx.style_of(egui::Theme::Light)).clone();
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::new(editor::EDITOR_FONT_SIZE, egui::FontFamily::Monospace),
            );
            style.spacing.item_spacing = egui::vec2(0.0, 0.0);
            style.spacing.scroll.bar_width = 8.0;
            cc.egui_ctx.set_style_of(egui::Theme::Light, style);
            Ok(Box::new(editor::MiniEditor::new(cc.egui_ctx.clone())))
        }),
    )
}

struct FontFamilyPreference {
    monospace: Vec<String>,
    proportional: Vec<String>,
}

fn preferred_font_ordering() -> FontFamilyPreference {
    FontFamilyPreference {
        monospace: vec![
            "Consolas".to_owned(),
            "Cascadia Mono".to_owned(),
            "Cascadia Code".to_owned(),
            "Consola".to_owned(),
            "Lucida Console".to_owned(),
            "Courier New".to_owned(),
            "Microsoft YaHei Mono".to_owned(),
            "monospace".to_owned(),
        ],
        proportional: vec![
            "Microsoft YaHei".to_owned(),
            "Microsoft YaHei UI".to_owned(),
            "PingFang SC".to_owned(),
            "Segoe UI".to_owned(),
            "sans-serif".to_owned(),
        ],
    }
}

fn load_font_data(paths: &[&str]) -> Option<Vec<u8>> {
    for path in paths {
        if let Ok(data) = fs::read(path) {
            return Some(data);
        }
    }
    None
}

fn configure_editor_fonts(fonts: &mut egui::FontDefinitions) {
    let preferred = preferred_font_ordering();

    if let Some(data) = load_font_data(&[
        "C:/Windows/Fonts/CascadiaMono.ttf",
        "C:/Windows/Fonts/CascadiaCode.ttf",
        "C:/Windows/Fonts/consola.ttf",
        "C:/Windows/Fonts/consolab.ttf",
        "C:/Windows/Fonts/Consolas.ttf",
    ]) {
        let key = "editor_mono".to_owned();
        fonts
            .font_data
            .insert(key.clone(), egui::FontData::from_owned(data).into());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, key);
    }

    if let Some(data) = load_font_data(&[
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/msyh.ttf",
        "C:/Windows/Fonts/msyhui.ttf",
    ]) {
        let key = "editor_chinese".to_owned();
        fonts
            .font_data
            .insert(key.clone(), egui::FontData::from_owned(data).into());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, key.clone());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(1, key.clone());
    }

    if fonts
        .families
        .get(&egui::FontFamily::Monospace)
        .and_then(|list| list.first())
        .is_none()
    {
        let families = &preferred.monospace;
        for name in families {
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, name.clone());
            break;
        }
    }

    if fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .and_then(|list| list.first())
        .is_none()
    {
        let families = &preferred.proportional;
        for name in families {
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, name.clone());
            break;
        }
    }
}
