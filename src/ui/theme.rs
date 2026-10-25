//! Visual theme: a dark "inked page" palette with a comic-red accent, plus a light variant.

use egui::{Color32, CornerRadius, Stroke, Visuals};

pub const ACCENT: Color32 = Color32::from_rgb(229, 72, 77);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(150, 46, 50);
pub const GOOD: Color32 = Color32::from_rgb(70, 167, 88);
pub const WARN: Color32 = Color32::from_rgb(230, 160, 40);
pub const BAD: Color32 = Color32::from_rgb(229, 72, 77);
pub const INFO: Color32 = Color32::from_rgb(90, 150, 230);
pub const CORE_BADGE: Color32 = Color32::from_rgb(90, 120, 200);
pub const ERRATA_BADGE: Color32 = Color32::from_rgb(160, 95, 205);

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub card: Color32,
    pub card_alt: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: Color32::from_rgb(15, 17, 21),
            panel: Color32::from_rgb(21, 23, 29),
            card: Color32::from_rgb(28, 31, 38),
            card_alt: Color32::from_rgb(34, 37, 45),
            border: Color32::from_rgb(48, 52, 62),
            text: Color32::from_rgb(230, 232, 238),
            muted: Color32::from_rgb(140, 146, 160),
        }
    } else {
        Palette {
            bg: Color32::from_rgb(240, 241, 244),
            panel: Color32::from_rgb(247, 248, 250),
            card: Color32::from_rgb(255, 255, 255),
            card_alt: Color32::from_rgb(244, 245, 248),
            border: Color32::from_rgb(218, 221, 228),
            text: Color32::from_rgb(28, 30, 36),
            muted: Color32::from_rgb(105, 110, 122),
        }
    }
}

pub fn apply(ctx: &egui::Context, dark: bool, scale: f32) {
    let p = palette(dark);
    let mut v = if dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = p.panel;
    v.window_fill = p.card;
    v.extreme_bg_color = if dark { Color32::from_rgb(12, 13, 17) } else { Color32::from_rgb(250, 250, 252) };
    v.faint_bg_color = p.card_alt;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(8);
    v.selection.bg_fill = if dark { ACCENT_DIM } else { Color32::from_rgb(250, 205, 207) };
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;
    v.override_text_color = Some(p.text);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(6);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    v.widgets.inactive.weak_bg_fill = p.card_alt;
    v.widgets.inactive.bg_fill = p.card_alt;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.bg_fill = ACCENT_DIM;
    ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx.set_visuals(v);
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 4.0);
        s.spacing.interact_size.y = 24.0;
    });
    ctx.set_zoom_factor(scale.clamp(0.7, 2.0));
}

/// A rounded card container.
pub fn card<R>(ui: &mut egui::Ui, dark: bool, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let p = palette(dark);
    egui::Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::same(14))
        .show(ui, add)
        .inner
}

pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.25))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.0).strong().color(color));
        });
}

pub fn ruleset_color(r: crate::rules::Ruleset) -> Color32 {
    match r {
        crate::rules::Ruleset::Core => CORE_BADGE,
        crate::rules::Ruleset::Errata => ERRATA_BADGE,
    }
}

pub fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 18.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 2.0, ACCENT);
        ui.label(egui::RichText::new(text).size(16.0).strong());
    });
    ui.add_space(4.0);
}
