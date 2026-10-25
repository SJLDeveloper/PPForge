//! Play tab: a clickable character sheet (left) feeding a Challenge Roll dice roller (right).

use crate::app::Doc;
use crate::engine::{self, Summary};
use crate::model::*;
use crate::rules::{self, Ruleset};
use crate::ui::theme::{self, Palette};
use egui::{Align, Color32, Layout, RichText, Sense, Stroke, Ui, vec2};

/// What the roller is currently rolling.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RollKey {
    Ability(usize),
    Talent(usize),
    Power(u64),
    Edge,
    Custom,
}

#[derive(Clone, Debug)]
pub struct RollResult {
    pub label: String,
    pub pool: i32,
    pub faces: Vec<u8>,
    /// Index where exploded (bonus) dice start.
    pub base_dice: usize,
    pub successes: i32,
    pub threshold: i32,
    pub opp_faces: Vec<u8>,
    pub auto: bool,
    pub rerolled: bool,
    pub resolve_spent: i32,
    pub defining: bool,
}

impl RollResult {
    pub fn margin(&self) -> i32 {
        self.successes - self.threshold
    }
    pub fn one_line(&self) -> String {
        let mut how = if self.auto { " (automatic)".to_string() } else { String::new() };
        if self.resolve_spent > 0 {
            how.push_str(&format!(", {} Resolve spent", self.resolve_spent));
        }
        format!(
            "{} {}d{}: {} successes vs {} → Margin {:+} — {}",
            self.label,
            self.pool,
            how,
            self.successes,
            self.threshold,
            self.margin(),
            outcome(self.margin()).1
        )
    }
}

pub struct Roller {
    pub key: RollKey,
    pub custom_rank: i32,
    /// GM bonus or penalty dice.
    pub modifier: i32,
    /// Resolve spent to add dice (1 die per point).
    pub resolve_dice: i32,
    /// Extra actions this page (-2d each).
    pub extra_actions: i32,
    pub half: bool,
    pub role: bool,
    pub role_explode: bool,
    pub explode_other: bool,
    pub defining: bool,
    pub one_per_even: bool,
    pub take_auto: bool,
    pub threshold: i32,
    pub opposed: bool,
    pub opp_dice: i32,
    pub last: Option<RollResult>,
    pub history: Vec<RollResult>,
    rng: u64,
}

impl Default for Roller {
    fn default() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42)
            | 1;
        Roller {
            key: RollKey::Ability(AGI),
            custom_rank: 4,
            modifier: 0,
            resolve_dice: 0,
            extra_actions: 0,
            half: false,
            role: false,
            role_explode: false,
            explode_other: false,
            defining: false,
            one_per_even: false,
            take_auto: false,
            threshold: 2,
            opposed: false,
            opp_dice: 4,
            last: None,
            history: Vec::new(),
            rng: seed,
        }
    }
}

/// Options that change the dice pool.
#[derive(Clone, Copy, Default, Debug)]
pub struct PoolOpts {
    pub half: bool,
    pub defining: bool,
    pub role: bool,
    pub modifier: i32,
    pub resolve_dice: i32,
    pub extra_actions: i32,
}

/// Dice pool and a human-readable breakdown. A Challenge Roll always gets at least 1 die.
pub fn pool_for(base: i32, o: PoolOpts) -> (i32, Vec<String>) {
    let mut parts = vec![format!("{base}d base")];
    let mut rank = base;
    if o.half {
        rank = (rank + 1) / 2;
        parts.push(format!("half → {rank}d"));
    }
    if o.defining {
        rank *= 2;
        parts.push(format!("Defining Moment ×2 → {rank}d"));
    }
    let mut pool = rank;
    if o.role {
        pool += 1;
        parts.push("+1d Role".into());
    }
    if o.modifier != 0 {
        pool += o.modifier;
        parts.push(format!("{:+}d modifier", o.modifier));
    }
    if o.resolve_dice > 0 {
        pool += o.resolve_dice;
        parts.push(format!("+{}d Resolve", o.resolve_dice));
    }
    if o.extra_actions > 0 {
        pool -= 2 * o.extra_actions;
        parts.push(format!("−{}d multiple actions", 2 * o.extra_actions));
    }
    if pool < 1 {
        parts.push("minimum 1d".into());
    }
    (pool.max(1), parts)
}

/// Successes for a set of faces: 2s and 4s are 1, 6s are 2 (or 1 when "checking your swing").
pub fn count_successes(faces: &[u8], one_per_even: bool) -> i32 {
    faces
        .iter()
        .map(|f| match f {
            2 | 4 => 1,
            6 => {
                if one_per_even {
                    1
                } else {
                    2
                }
            }
            _ => 0,
        })
        .sum()
}

/// Narrative and traditional result for a Margin.
pub fn outcome(margin: i32) -> (&'static str, &'static str, Color32) {
    match margin {
        m if m <= -2 => ("Opponent narrates", "Complete failure", theme::BAD),
        -1 | 0 => ("Opponent narrates, you embellish", "Failure with a silver lining", theme::WARN),
        1 | 2 => ("You narrate, opponent embellishes", "Success with a complication", theme::INFO),
        _ => ("You narrate", "Complete success", theme::GOOD),
    }
}

impl Roller {
    /// Developer hook for automated screenshots: pre-load a Might roll.
    pub fn demo(&mut self, rank: i32) {
        self.key = RollKey::Ability(MIG);
        self.threshold = 3;
        self.explode_other = true;
        let r = self.roll("Might".into(), rank, false);
        self.history.push(r.clone());
        self.last = Some(r);
    }

    fn d6(&mut self) -> u8 {
        // xorshift64*
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        ((x.wrapping_mul(0x2545F4914F6CDD1D) >> 33) % 6) as u8 + 1
    }

    /// Roll `n` dice; with `explode`, each 6 adds another die (capped for safety).
    fn roll_dice(&mut self, n: i32, explode: bool) -> (Vec<u8>, usize) {
        let mut faces = Vec::new();
        for _ in 0..n.max(0) {
            faces.push(self.d6());
        }
        let base = faces.len();
        if explode {
            let mut pending = faces.iter().filter(|f| **f == 6).count();
            while pending > 0 && faces.len() < 400 {
                pending -= 1;
                let f = self.d6();
                if f == 6 {
                    pending += 1;
                }
                faces.push(f);
            }
        }
        (faces, base)
    }

    fn explodes(&self) -> bool {
        self.defining || self.role_explode || self.explode_other
    }

    fn opts(&self, role_available: bool) -> PoolOpts {
        PoolOpts {
            half: self.half,
            defining: self.defining,
            role: self.role && role_available,
            modifier: self.modifier,
            resolve_dice: self.resolve_dice,
            extra_actions: self.extra_actions,
        }
    }

    /// Resolve this roll would cost before rolling.
    fn resolve_cost(&self, role_available: bool) -> i32 {
        self.resolve_dice + if self.role_explode && role_available && !self.defining { 1 } else { 0 }
    }

    fn roll(&mut self, label: String, base: i32, role_available: bool) -> RollResult {
        let (pool, _) = pool_for(base, self.opts(role_available));
        let (faces, base_dice, successes) = if self.take_auto {
            (Vec::new(), 0, pool / 2)
        } else {
            let (f, b) = self.roll_dice(pool, self.explodes());
            let s = count_successes(&f, self.one_per_even);
            (f, b, s)
        };
        let (threshold, opp_faces) = if self.opposed {
            let (f, _) = self.roll_dice(self.opp_dice.max(1), false);
            (count_successes(&f, self.one_per_even), f)
        } else {
            (self.threshold, Vec::new())
        };
        RollResult {
            label,
            pool,
            faces,
            base_dice,
            successes,
            threshold,
            opp_faces,
            auto: self.take_auto,
            rerolled: false,
            resolve_spent: self.resolve_cost(role_available),
            defining: self.defining,
        }
    }

    /// Reroll the whole pool (1 Resolve) and keep the better result.
    fn reroll(&mut self) -> Option<RollResult> {
        let prev = self.last.clone()?;
        if prev.auto {
            return None;
        }
        let (faces, base_dice) = self.roll_dice(prev.pool, prev.defining || self.role_explode || self.explode_other);
        let successes = count_successes(&faces, self.one_per_even);
        let mut r = RollResult { faces, base_dice, successes, rerolled: true, resolve_spent: 1, ..prev.clone() };
        if r.successes < prev.successes {
            // Keep the better result, but record that a reroll happened.
            r = RollResult { rerolled: true, resolve_spent: 1, ..prev };
        }
        Some(r)
    }
}

/// Name and rank for the current roll key.
fn key_info(c: &Character, s: &Summary, key: RollKey, custom: i32) -> Option<(String, i32)> {
    match key {
        RollKey::Ability(i) => Some((ABILITIES[i].to_string(), c.abilities[i].rank)),
        RollKey::Talent(i) => Some((TALENTS[i].to_string(), c.talents[i].rank)),
        RollKey::Power(uid) => {
            let i = c.powers.iter().position(|p| p.uid == uid)?;
            let r = s.power_costs.get(i)?.rank?;
            Some((engine::power_label(&c.powers[i]), r))
        }
        RollKey::Edge => Some(("Edge".into(), s.edge)),
        RollKey::Custom => Some(("Custom roll".into(), custom)),
    }
}

const THRESHOLDS: [(&str, i32); 9] = [
    ("Easy", 0),
    ("Average", 1),
    ("Hard", 2),
    ("Daunting", 3),
    ("Brutal", 4),
    ("Inhuman", 6),
    ("Superhuman", 8),
    ("Legendary", 10),
    ("Godlike", 12),
];

pub fn show(ui: &mut Ui, doc: &mut Doc, st: &mut Roller, pal: &Palette, dark: bool) {
    let s = doc.summary.clone();
    let c = &mut doc.ch;
    let width = ui.available_width();
    if width >= 860.0 {
        let gap = 12.0;
        let left_w = ((width - gap) * 0.56).floor();
        let right_w = width - gap - left_w;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(left_w);
                sheet(ui, c, &s, st, pal, dark);
            });
            ui.allocate_ui_with_layout(vec2(right_w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(right_w);
                roller(ui, c, &s, st, pal, dark);
            });
        });
    } else {
        roller(ui, c, &s, st, pal, dark);
        ui.add_space(10.0);
        sheet(ui, c, &s, st, pal, dark);
    }
    ui.add_space(10.0);
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Session Log");
        ui.add(
            egui::TextEdit::multiline(&mut c.play.session_log)
                .desired_rows(6)
                .desired_width(f32::INFINITY)
                .hint_text("What happened, Resolve earned, subplots… (use “Log” on a roll to add it here)"),
        );
    });
}

// ---------------------------------------------------------------- Resolve helpers

fn current_resolve(c: &Character, s: &Summary) -> Option<i32> {
    s.resolve.map(|start| c.play.current_resolve.unwrap_or(start))
}

fn set_resolve(c: &mut Character, s: &Summary, v: i32) {
    let v = v.max(0);
    c.play.current_resolve = if Some(v) == s.resolve { None } else { Some(v) };
}

// ---------------------------------------------------------------- Sheet (left)

/// A full-width clickable sheet row. Returns the click response.
fn row(ui: &mut Ui, label: &str, sub: &str, rank: Option<i32>, selected: bool, pal: &Palette) -> egui::Response {
    row_tagged(ui, label, "", sub, rank, selected, pal)
}

/// Like `row`, with a small muted tag drawn on the same line after the label.
fn row_tagged(ui: &mut Ui, label: &str, tag: &str, sub: &str, rank: Option<i32>, selected: bool, pal: &Palette) -> egui::Response {
    let h = if sub.is_empty() { 24.0 } else { 34.0 };
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), h), if rank.is_some() { Sense::click() } else { Sense::hover() });
    let painter = ui.painter();
    let hovered = resp.hovered() && rank.is_some();
    let fill = if selected {
        theme::ACCENT_DIM
    } else if hovered {
        pal.card_alt
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 5.0, fill);
    if selected {
        painter.rect_stroke(rect, 5.0, Stroke::new(1.0, theme::ACCENT), egui::StrokeKind::Inside);
    }
    let text_color = if selected { Color32::WHITE } else { pal.text };
    let muted = if selected { Color32::from_gray(225) } else { pal.muted };
    let left = rect.left() + 8.0;
    let top_y = if sub.is_empty() { rect.center().y } else { rect.top() + 11.0 };
    let name_rect = painter.text(egui::pos2(left, top_y), egui::Align2::LEFT_CENTER, label, egui::FontId::proportional(14.0), text_color);
    if !tag.is_empty() {
        painter.text(egui::pos2(name_rect.right() + 6.0, top_y + 1.0), egui::Align2::LEFT_CENTER, tag, egui::FontId::proportional(11.0), muted);
    }
    if !sub.is_empty() {
        let max_chars = ((rect.width() - 70.0) / 6.2).max(10.0) as usize;
        let mut t: String = sub.chars().take(max_chars).collect();
        if sub.chars().count() > max_chars {
            t.push('…');
        }
        painter.text(egui::pos2(left, rect.bottom() - 9.0), egui::Align2::LEFT_CENTER, t, egui::FontId::proportional(11.0), muted);
    }
    let rank_text = match rank {
        Some(r) => format!("{r}d"),
        None => "—".into(),
    };
    painter.text(
        egui::pos2(rect.right() - 8.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        rank_text,
        egui::FontId::proportional(16.0),
        if rank.is_some() { text_color } else { muted },
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

fn label_small(ui: &mut Ui, pal: &Palette, t: &str) {
    ui.label(RichText::new(t).size(11.0).strong().color(pal.muted));
}

fn stat_tile(ui: &mut Ui, pal: &Palette, label: &str, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new().fill(pal.card_alt).corner_radius(8).inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
        ui.vertical(|ui| {
            label_small(ui, pal, label);
            ui.horizontal(|ui| add(ui));
        });
    });
}

fn sheet(ui: &mut Ui, c: &mut Character, s: &Summary, st: &mut Roller, pal: &Palette, dark: bool) {
    let rules = rules::rules(c.ruleset);
    theme::card(ui, dark, |ui| {
        // Header.
        ui.horizontal(|ui| {
            ui.label(RichText::new(c.display_name()).size(20.0).strong());
            if !c.story.real_name.trim().is_empty() {
                ui.label(RichText::new(c.story.real_name.trim()).color(pal.muted));
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{} · {}", c.char_type.label(), c.power_level.name(c.ruleset))).color(pal.muted));
            if c.ruleset == Ruleset::Errata && c.role != Role::None {
                theme::badge(ui, &format!("{} Role", c.role.label()), theme::ACCENT);
            }
            if !c.story.motivation.trim().is_empty() {
                ui.label(RichText::new(format!("Motivation: {}", c.story.motivation.trim())).color(pal.muted));
            }
        });
        ui.add_space(8.0);

        // Status strip.
        ui.horizontal_wrapped(|ui| {
            let edge_sel = st.key == RollKey::Edge;
            stat_tile(ui, pal, "EDGE", |ui| {
                let t = RichText::new(s.edge.to_string()).size(22.0).strong();
                let t = if edge_sel { t.color(theme::ACCENT) } else { t };
                if ui.add(egui::Button::new(t).frame(false)).on_hover_text("Click to roll Edge (random initiative)").clicked() {
                    st.key = RollKey::Edge;
                }
            });
            let max_h = s.health;
            let mut h = c.play.current_health.unwrap_or(max_h);
            stat_tile(ui, pal, "HEALTH", |ui| {
                if ui.small_button("−").clicked() {
                    h -= 1;
                }
                let col = if h <= 0 { theme::BAD } else if h * 2 <= max_h { theme::WARN } else { pal.text };
                ui.label(RichText::new(format!("{h}/{max_h}")).size(22.0).strong().color(col));
                if ui.small_button("+").clicked() {
                    h = (h + 1).min(max_h);
                }
                if ui.small_button("Full").clicked() {
                    h = max_h;
                }
            });
            c.play.current_health = if h == max_h { None } else { Some(h) };
            if let Some(r) = current_resolve(c, s) {
                let mut r2 = r;
                stat_tile(ui, pal, "RESOLVE", |ui| {
                    if ui.small_button("−").clicked() {
                        r2 -= 1;
                    }
                    ui.label(RichText::new(r2.max(0).to_string()).size(22.0).strong());
                    if ui.small_button("+").clicked() {
                        r2 += 1;
                    }
                    if ui.small_button("New issue").on_hover_text("Resolve resets to starting Resolve at the start of every issue").clicked() {
                        r2 = s.resolve.unwrap_or(0);
                        c.play.issues_played += 1;
                    }
                });
                if r2 != r {
                    set_resolve(c, s, r2);
                }
            }
            stat_tile(ui, pal, "DEFINING MOMENTS", |ui| {
                ui.add(egui::DragValue::new(&mut c.play.defining_moments).range(0..=12));
            });
        });
        ui.add_space(4.0);
        ui.label(RichText::new("Click any Ability, Talent or Power to load it into the roller.").size(11.5).color(pal.muted));
        ui.add_space(6.0);

        // Abilities.
        label_small(ui, pal, "ABILITIES");
        ui.columns(2, |cols| {
            for (i, a) in ABILITIES.iter().enumerate() {
                let col = &mut cols[i % 2];
                let t = &c.abilities[i];
                let tag = if t.source != Source::Innate { t.source.label() } else { "" };
                if row_tagged(col, a, tag, "", Some(t.rank), st.key == RollKey::Ability(i), pal).clicked() {
                    st.key = RollKey::Ability(i);
                }
            }
        });
        ui.add_space(8.0);

        // Talents.
        label_small(ui, pal, "TALENTS");
        ui.columns(2, |cols| {
            for (i, a) in TALENTS.iter().enumerate() {
                let col = &mut cols[i % 2];
                let t = &c.talents[i];
                let tag = if t.source != Source::Trained { t.source.label() } else { "" };
                if row_tagged(col, a, tag, "", Some(t.rank), st.key == RollKey::Talent(i), pal).clicked() {
                    st.key = RollKey::Talent(i);
                }
            }
        });
        ui.add_space(8.0);

        // Powers.
        label_small(ui, pal, "POWERS");
        if c.powers.is_empty() {
            ui.label(RichText::new("None").color(pal.muted));
        }
        for (i, p) in c.powers.iter().enumerate() {
            let pc = s.power_costs.get(i);
            let rank = pc.and_then(|x| x.rank);
            let def = engine::power_def(rules, p);
            let mut sub = vec![p.source.label().to_string()];
            if let Some(d) = def {
                sub.push(d.range.clone());
            }
            let mods = engine::mods_label(rules, def, &p.mods);
            if !mods.is_empty() {
                sub.push(mods);
            }
            let resp = row(ui, &engine::power_label(p), &sub.join(" · "), rank, st.key == RollKey::Power(p.uid), pal);
            let resp = match def {
                Some(d) if !d.summary.is_empty() => resp.on_hover_text(&d.summary),
                _ => resp,
            };
            if resp.clicked() && rank.is_some() {
                st.key = RollKey::Power(p.uid);
            }
        }
        ui.add_space(8.0);

        // Flaws & Perks.
        ui.columns(2, |cols| {
            let ui = &mut cols[0];
            label_small(ui, pal, "FLAWS");
            if c.flaws.is_empty() {
                ui.label(RichText::new("None").color(pal.muted));
            }
            let mut earn = false;
            for f in &c.flaws {
                ui.horizontal_wrapped(|ui| {
                    let per_issue = engine::flaw_grants_issue_resolve(rules, f);
                    let mut t = f.def.clone();
                    if !f.note.trim().is_empty() {
                        t.push_str(&format!(" ({})", f.note.trim()));
                    }
                    ui.label(t);
                    if per_issue {
                        ui.label(RichText::new("+1/issue").size(10.5).color(theme::INFO));
                    } else if s.resolve.is_some()
                        && ui.small_button("+1").on_hover_text("This Flaw caused trouble: earn 1 Resolve").clicked()
                    {
                        earn = true;
                    }
                });
            }
            if earn
                && let Some(r) = current_resolve(c, s) {
                    set_resolve(c, s, r + 1);
                }
            let ui = &mut cols[1];
            label_small(ui, pal, "PERKS");
            if c.perks.is_empty() {
                ui.label(RichText::new("None").color(pal.muted));
            }
            for p in &c.perks {
                let t = if p.note.trim().is_empty() { p.def.clone() } else { format!("{} ({})", p.def, p.note.trim()) };
                ui.label(t);
            }
        });
        if !c.story.gear.trim().is_empty() {
            ui.add_space(6.0);
            label_small(ui, pal, "GEAR");
            ui.label(c.story.gear.trim());
        }
    });
}

// ---------------------------------------------------------------- Roller (right)

fn die_face(ui: &mut Ui, face: u8, exploded: bool, one_per_even: bool, pal: &Palette) {
    let (rect, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
    let (fill, text) = match face {
        6 => (theme::ACCENT, Color32::WHITE),
        2 | 4 => (theme::GOOD, Color32::WHITE),
        _ => (pal.card_alt, pal.muted),
    };
    let p = ui.painter();
    p.rect_filled(rect, 6.0, fill);
    if exploded {
        p.rect_stroke(rect, 6.0, Stroke::new(2.0, theme::WARN), egui::StrokeKind::Inside);
    }
    p.text(rect.center(), egui::Align2::CENTER_CENTER, face.to_string(), egui::FontId::proportional(16.0), text);
    if face == 6 && !one_per_even {
        p.text(rect.right_top() + vec2(-4.0, 6.0), egui::Align2::CENTER_CENTER, "×2", egui::FontId::proportional(8.0), Color32::WHITE);
    }
}

fn roller(ui: &mut Ui, c: &mut Character, s: &Summary, st: &mut Roller, pal: &Palette, dark: bool) {
    let rules = rules::rules(c.ruleset);
    let role_available = rules.config().has_roles && c.role != Role::None && c.char_type == CharType::Hero;
    let resolve_now = current_resolve(c, s);
    // If the selected Power disappeared, fall back to Agility.
    let info = key_info(c, s, st.key, st.custom_rank).or_else(|| {
        st.key = RollKey::Ability(AGI);
        key_info(c, s, st.key, st.custom_rank)
    });
    let (label, base) = info.unwrap_or(("Agility".into(), 2));

    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Challenge Roll");
        // Selected trait.
        egui::Frame::new().fill(pal.card_alt).corner_radius(8).inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    label_small(ui, pal, "ROLLING");
                    ui.label(RichText::new(&label).size(18.0).strong());
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if st.key == RollKey::Custom {
                        ui.add(egui::DragValue::new(&mut st.custom_rank).range(1..=60).suffix("d"));
                    } else {
                        ui.label(RichText::new(format!("{base}d")).size(26.0).strong().color(theme::ACCENT));
                        if ui.small_button("Custom").on_hover_text("Roll any number of dice").clicked() {
                            st.custom_rank = base;
                            st.key = RollKey::Custom;
                        }
                    }
                });
            });
        });
        ui.add_space(8.0);

        // Threshold.
        label_small(ui, pal, "THRESHOLD");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut st.opposed, false, "Static");
            ui.selectable_value(&mut st.opposed, true, "Opposed roll");
        });
        if st.opposed {
            ui.horizontal(|ui| {
                ui.label("Opponent rolls");
                ui.add(egui::DragValue::new(&mut st.opp_dice).range(1..=60).suffix("d"));
                ui.label(RichText::new("(their successes become your Threshold)").size(11.0).color(pal.muted));
            });
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                for (name, v) in THRESHOLDS {
                    let sel = st.threshold == v;
                    let txt = RichText::new(format!("{name} ({v})")).size(12.0);
                    if ui.add(egui::Button::new(if sel { txt.color(Color32::WHITE) } else { txt }).selected(sel)).clicked() {
                        st.threshold = v;
                    }
                }
                ui.add(egui::DragValue::new(&mut st.threshold).range(0..=40).prefix("= "));
            });
        }
        ui.add_space(8.0);

        // Modifiers.
        label_small(ui, pal, "MODIFIERS");
        egui::Grid::new("roll_mods").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            ui.label("Bonus / penalty");
            ui.horizontal(|ui| {
                if ui.small_button("−").clicked() {
                    st.modifier -= 1;
                }
                ui.add(egui::DragValue::new(&mut st.modifier).range(-30..=30).suffix("d"));
                if ui.small_button("+").clicked() {
                    st.modifier += 1;
                }
                if st.modifier != 0 && ui.small_button("0").clicked() {
                    st.modifier = 0;
                }
            });
            ui.end_row();
            if let Some(r) = resolve_now {
                ui.label("Spend Resolve for dice");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut st.resolve_dice).range(0..=r.max(0)).prefix("+").suffix("d"));
                    ui.label(RichText::new(format!("{r} available")).size(11.0).color(pal.muted));
                });
                ui.end_row();
            }
            ui.label("Extra actions this page");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut st.extra_actions).range(0..=6));
                ui.label(RichText::new("−2d each").size(11.0).color(pal.muted));
            });
            ui.end_row();
        });
        ui.checkbox(&mut st.half, "Use half rank").on_hover_text("e.g. half an Ability in place of a Talent, or half Agility for stealth");
        if role_available {
            ui.checkbox(&mut st.role, format!("{} Role applies (+1d)", c.role.label())).on_hover_text(c.role.applies_to());
            if st.role {
                ui.checkbox(&mut st.role_explode, "Spend 1 Resolve: 6s explode (Role)");
            }
        }
        if c.char_type == CharType::Hero {
            ui.checkbox(&mut st.defining, "Defining Moment (double rank, 6s explode)");
        }
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut st.explode_other, "6s explode (other)");
            ui.checkbox(&mut st.one_per_even, "Checking your swing (6 = 1)");
        });
        ui.checkbox(&mut st.take_auto, "Take automatic successes (1 per 2 dice, no roll)");

        let (pool, parts) = pool_for(base, st.opts(role_available));
        let cost = st.resolve_cost(role_available);
        ui.add_space(6.0);
        ui.label(RichText::new(parts.join("  ")).size(11.5).color(pal.muted));
        let affordable = resolve_now.map(|r| r >= cost).unwrap_or(cost == 0);
        ui.horizontal(|ui| {
            let mut txt = format!("Roll {pool}d");
            if cost > 0 {
                txt.push_str(&format!("  (−{cost} Resolve)"));
            }
            let btn = egui::Button::new(RichText::new(txt).size(16.0).strong().color(Color32::WHITE)).fill(theme::ACCENT_DIM).min_size(vec2(160.0, 34.0));
            if ui.add_enabled(affordable, btn).clicked() {
                let r = st.roll(label.clone(), base, role_available);
                if cost > 0
                    && let Some(now) = resolve_now {
                        set_resolve(c, s, now - cost);
                    }
                st.resolve_dice = 0;
                st.history.insert(0, r.clone());
                st.history.truncate(12);
                st.last = Some(r);
            }
            let can_reroll = st.last.as_ref().map(|r| !r.auto).unwrap_or(false) && resolve_now.map(|r| r >= 1).unwrap_or(false);
            if ui
                .add_enabled(can_reroll, egui::Button::new("Reroll (−1 Resolve)"))
                .on_hover_text("Reroll the whole pool and keep the better result")
                .clicked()
                && let Some(r) = st.reroll() {
                    if let Some(now) = current_resolve(c, s) {
                        set_resolve(c, s, now - 1);
                    }
                    if let Some(h) = st.history.first_mut() {
                        *h = r.clone();
                    }
                    st.last = Some(r);
                }
        });
        if !affordable {
            ui.label(RichText::new("Not enough Resolve for this roll").size(11.5).color(theme::WARN));
        }

        // Result.
        if let Some(r) = st.last.clone() {
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(4.0);
            if r.auto {
                ui.label(RichText::new(format!("Automatic: {}d → {} successes", r.pool, r.successes)).color(pal.muted));
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                    for (i, f) in r.faces.iter().enumerate() {
                        die_face(ui, *f, i >= r.base_dice, st.one_per_even, pal);
                    }
                });
            }
            if !r.opp_faces.is_empty() {
                let faces: Vec<String> = r.opp_faces.iter().map(|f| f.to_string()).collect();
                ui.label(RichText::new(format!("Opponent: {} → {} successes", faces.join(" "), r.threshold)).size(12.0).color(pal.muted));
            }
            let m = r.margin();
            let (narr, trad, col) = outcome(m);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{}", r.successes)).size(30.0).strong().color(col));
                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("successes vs Threshold {}", r.threshold)).color(pal.muted));
                    ui.label(RichText::new(format!("Margin {m:+}")).size(18.0).strong().color(col));
                });
            });
            ui.label(RichText::new(trad).size(15.0).strong().color(col));
            ui.label(narr);
            if m >= 1 {
                let effect = match m / 2 {
                    0 => "too small for a special effect".to_string(),
                    1 => "a special effect lasting 1 page".to_string(),
                    n => format!("a special effect lasting {n} pages"),
                };
                ui.label(
                    RichText::new(format!("As an attack: {m} damage, or {effect}."))
                        .size(11.5)
                        .color(pal.muted),
                );
            }
            if r.rerolled {
                ui.label(RichText::new("Rerolled — better result kept").size(11.5).color(pal.muted));
            }
            if r.defining {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Defining Moment: afterward the Hero is out for the scene and the Trait drops by 1. From the 3rd, roll 2d6 — below the total means retirement.")
                        .size(11.5)
                        .color(theme::WARN),
                );
                if ui.button("Record Defining Moment").clicked() {
                    c.play.defining_moments += 1;
                    st.defining = false;
                }
            }
            ui.horizontal(|ui| {
                if ui.small_button("Log").on_hover_text("Add this roll to the session log").clicked() {
                    if !c.play.session_log.is_empty() && !c.play.session_log.ends_with('\n') {
                        c.play.session_log.push('\n');
                    }
                    c.play.session_log.push_str(&r.one_line());
                    c.play.session_log.push('\n');
                }
            });
        }

        // History.
        if st.history.len() > 1 {
            ui.add_space(8.0);
            ui.collapsing(format!("Recent rolls ({})", st.history.len()), |ui| {
                for r in st.history.iter().skip(1) {
                    let col = outcome(r.margin()).2;
                    ui.label(RichText::new(r.one_line()).size(11.5).color(col));
                }
                if ui.small_button("Clear").clicked() {
                    let keep = st.history.first().cloned();
                    st.history.clear();
                    st.history.extend(keep);
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successes_follow_the_core_rule() {
        // Book example: 1,2,2,3,3,3,3,4,5,5,6,6 = 7 successes.
        assert_eq!(count_successes(&[1, 2, 2, 3, 3, 3, 3, 4, 5, 5, 6, 6], false), 7);
        // Hiro: 1,1,3,3,4,5 = 1 success.
        assert_eq!(count_successes(&[1, 1, 3, 3, 4, 5], false), 1);
        assert_eq!(count_successes(&[6, 6], true), 2);
    }

    #[test]
    fn pool_modifiers() {
        let (p, _) = pool_for(9, PoolOpts { half: true, ..Default::default() });
        assert_eq!(p, 5, "half of 9d rounds up, like the book's Flicker example");
        let (p, _) = pool_for(6, PoolOpts { defining: true, role: true, ..Default::default() });
        assert_eq!(p, 13);
        let (p, _) = pool_for(12, PoolOpts { extra_actions: 1, modifier: 2, ..Default::default() });
        assert_eq!(p, 12, "Gatecrasher: -2d for 2 actions, +2d all-out attack");
        let (p, _) = pool_for(2, PoolOpts { modifier: -5, ..Default::default() });
        assert_eq!(p, 1, "always at least one die");
    }

    #[test]
    fn outcomes_match_table() {
        assert_eq!(outcome(-2).1, "Complete failure");
        assert_eq!(outcome(-1).1, "Failure with a silver lining");
        assert_eq!(outcome(0).1, "Failure with a silver lining");
        assert_eq!(outcome(2).1, "Success with a complication");
        assert_eq!(outcome(3).1, "Complete success");
    }

    #[test]
    fn exploding_dice_add_dice_and_auto_takes_half() {
        let mut r = Roller { explode_other: true, ..Default::default() };
        for _ in 0..50 {
            let res = r.roll("x".into(), 12, false);
            let sixes_in_base = res.faces[..res.base_dice].iter().filter(|f| **f == 6).count();
            assert!(res.faces.len() >= res.base_dice + sixes_in_base);
            assert_eq!(res.base_dice, 12);
        }
        r.take_auto = true;
        let res = r.roll("x".into(), 12, false);
        assert_eq!(res.successes, 6);
        assert!(res.faces.is_empty());
    }

    #[test]
    fn reroll_never_lowers_successes() {
        let mut r = Roller::default();
        for _ in 0..30 {
            let first = r.roll("x".into(), 8, false);
            r.last = Some(first.clone());
            let second = r.reroll().unwrap();
            assert!(second.successes >= first.successes);
            assert!(second.rerolled);
        }
    }
}
