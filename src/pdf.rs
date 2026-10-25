//! Character sheet PDF export (US Letter), using the PDF built-in Helvetica fonts so no font
//! files are needed on any platform.

use crate::engine::{self, Summary};
use crate::model::*;
use crate::rules::{self, Ruleset};
use printpdf::path::PaintMode;
use printpdf::*;

const PAGE_W: f32 = 215.9;
const PAGE_H: f32 = 279.4;
const MARGIN: f32 = 12.0;
const PT_TO_MM: f32 = 0.352_778;

// Helvetica advance widths (1/1000 em) for ASCII 32..=126.
const HELV: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556,
    556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833,
    722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556,
    556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334,
    260, 334, 584,
];
// Helvetica-Bold advance widths for ASCII 32..=126.
const HELV_B: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556, 556, 556, 556, 556, 556,
    556, 556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833,
    722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611,
    556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389,
    280, 389, 584,
];

fn text_width_mm(s: &str, size_pt: f32, bold: bool) -> f32 {
    let table = if bold { &HELV_B } else { &HELV };
    let units: u32 = s
        .chars()
        .map(|c| {
            let i = c as u32;
            if (32..=126).contains(&i) { table[(i - 32) as usize] as u32 } else { 556 }
        })
        .sum();
    units as f32 / 1000.0 * size_pt * PT_TO_MM
}

/// Keep characters the WinAnsi encoding can represent; replace the rest.
fn clean(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\t' => ' ',
            c if (c as u32) < 0x100 && !c.is_control() => c,
            '‘' | '’' | '“' | '”' | '–' | '—' | '…' | '•' | '€' | '™' | 'Œ' | 'œ' | 'Š' | 'š' | 'Ž' | 'ž' | 'Ÿ' => c,
            '−' => '-',
            _ => '?',
        })
        .collect()
}

fn wrap(text: &str, width_mm: f32, size: f32, bold: bool) -> Vec<String> {
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if text_width_mm(&candidate, size, bold) <= width_mm || line.is_empty() {
                // Hard-break words that are longer than the line on their own.
                if line.is_empty() && text_width_mm(word, size, bold) > width_mm {
                    let mut chunk = String::new();
                    for ch in word.chars() {
                        let t = format!("{chunk}{ch}");
                        if text_width_mm(&t, size, bold) > width_mm && !chunk.is_empty() {
                            lines.push(std::mem::take(&mut chunk));
                        }
                        chunk.push(ch);
                    }
                    line = chunk;
                } else {
                    line = candidate;
                }
            } else {
                lines.push(std::mem::replace(&mut line, word.to_string()));
            }
        }
        lines.push(line);
    }
    lines
}

#[derive(Clone, Copy)]
struct Rgb3(f32, f32, f32);
const INK: Rgb3 = Rgb3(0.11, 0.12, 0.16);
const ACCENT: Rgb3 = Rgb3(0.86, 0.20, 0.18);
const MUTED: Rgb3 = Rgb3(0.42, 0.44, 0.50);
const RULE: Rgb3 = Rgb3(0.80, 0.81, 0.84);
const TINT: Rgb3 = Rgb3(0.95, 0.95, 0.96);
const WHITE: Rgb3 = Rgb3(1.0, 1.0, 1.0);

fn col(c: Rgb3) -> Color {
    Color::Rgb(Rgb::new(c.0, c.1, c.2, None))
}

struct Writer {
    doc: PdfDocumentReference,
    layer: PdfLayerReference,
    regular: IndirectFontRef,
    bold: IndirectFontRef,
    oblique: IndirectFontRef,
    /// Cursor: distance from the top of the page in mm.
    y: f32,
    page_no: u32,
    footer: String,
}

impl Writer {
    fn new(title: &str, footer: String) -> Result<Writer, String> {
        let (doc, page, layer) = PdfDocument::new(clean(title), Mm(PAGE_W), Mm(PAGE_H), "Sheet");
        let regular = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
        let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;
        let oblique = doc.add_builtin_font(BuiltinFont::HelveticaOblique).map_err(|e| e.to_string())?;
        let layer = doc.get_page(page).get_layer(layer);
        let w = Writer { doc, layer, regular, bold, oblique, y: MARGIN, page_no: 1, footer };
        w.draw_footer();
        Ok(w)
    }

    fn draw_footer(&self) {
        let text = format!("{}   ·   page {}", self.footer, self.page_no);
        self.text_at(&text, MARGIN, PAGE_H - 7.0, 7.0, Font::Regular, MUTED);
    }

    fn new_page(&mut self) {
        let (page, layer) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Sheet");
        self.layer = self.doc.get_page(page).get_layer(layer);
        self.page_no += 1;
        self.y = MARGIN;
        self.draw_footer();
    }

    fn ensure(&mut self, h: f32) {
        if self.y + h > PAGE_H - MARGIN - 4.0 {
            self.new_page();
        }
    }

    /// Draw text with its baseline at `top + size` (top measured from the page top).
    fn text_at(&self, text: &str, x: f32, top: f32, size: f32, font: Font, color: Rgb3) {
        let f = match font {
            Font::Regular => &self.regular,
            Font::Bold => &self.bold,
            Font::Italic => &self.oblique,
        };
        self.layer.set_fill_color(col(color));
        let baseline = PAGE_H - top - size * PT_TO_MM * 0.8;
        self.layer.use_text(clean(text), size, Mm(x), Mm(baseline), f);
    }

    fn rect(&self, x: f32, top: f32, w: f32, h: f32, fill: Rgb3) {
        self.layer.set_fill_color(col(fill));
        let r = Rect::new(Mm(x), Mm(PAGE_H - top - h), Mm(x + w), Mm(PAGE_H - top)).with_mode(PaintMode::Fill);
        self.layer.add_rect(r);
    }

    fn outline(&self, x: f32, top: f32, w: f32, h: f32, color: Rgb3, thickness: f32) {
        self.layer.set_outline_color(col(color));
        self.layer.set_outline_thickness(thickness);
        let r = Rect::new(Mm(x), Mm(PAGE_H - top - h), Mm(x + w), Mm(PAGE_H - top)).with_mode(PaintMode::Stroke);
        self.layer.add_rect(r);
    }

    fn hline(&self, x1: f32, x2: f32, top: f32, color: Rgb3, thickness: f32) {
        self.layer.set_outline_color(col(color));
        self.layer.set_outline_thickness(thickness);
        let line = Line {
            points: vec![(Point::new(Mm(x1), Mm(PAGE_H - top)), false), (Point::new(Mm(x2), Mm(PAGE_H - top)), false)],
            is_closed: false,
        };
        self.layer.add_line(line);
    }

    /// Section heading with an accent bar.
    fn heading(&mut self, title: &str) {
        self.ensure(14.0);
        self.y += 2.5;
        self.rect(MARGIN, self.y, 2.2, 5.2, ACCENT);
        self.text_at(&title.to_uppercase(), MARGIN + 4.0, self.y + 0.2, 10.5, Font::Bold, INK);
        self.y += 6.6;
        self.hline(MARGIN, PAGE_W - MARGIN, self.y, RULE, 0.4);
        self.y += 1.8;
    }

    /// Wrapped paragraph at x with width w; returns height used.
    fn paragraph(&mut self, text: &str, x: f32, w: f32, size: f32, font: Font, color: Rgb3) {
        let lh = size * PT_TO_MM * 1.32;
        for line in wrap(text, w, size, matches!(font, Font::Bold)) {
            self.ensure(lh);
            self.text_at(&line, x, self.y, size, font, color);
            self.y += lh;
        }
    }

    fn labeled(&mut self, label: &str, value: &str) {
        if value.trim().is_empty() {
            return;
        }
        self.ensure(9.0);
        self.text_at(label, MARGIN, self.y, 7.0, Font::Bold, MUTED);
        self.y += 3.4;
        self.paragraph(value, MARGIN, PAGE_W - 2.0 * MARGIN, 9.0, Font::Regular, INK);
        self.y += 1.5;
    }
}

#[derive(Clone, Copy)]
enum Font {
    Regular,
    Bold,
    Italic,
}

/// Render a character sheet as PDF bytes.
pub fn render(c: &Character) -> Result<Vec<u8>, String> {
    let rules = rules::rules(c.ruleset);
    let s: Summary = engine::evaluate(c);
    let footer = format!(
        "{} — Prowlers & Paragons Ultimate Edition — {}",
        c.display_name(),
        if c.ruleset == Ruleset::Core { "Core rules" } else { "Core rules + Combined Updates errata" }
    );
    let mut w = Writer::new(&c.display_name(), footer)?;
    let content_w = PAGE_W - 2.0 * MARGIN;

    // ---------- Header band ----------
    let band_h = 30.0;
    w.rect(0.0, 0.0, PAGE_W, band_h, INK);
    w.rect(0.0, band_h, PAGE_W, 1.4, ACCENT);
    w.text_at(&c.display_name(), MARGIN, 6.0, 24.0, Font::Bold, WHITE);
    let mut sub = Vec::new();
    if !c.story.real_name.trim().is_empty() {
        sub.push(c.story.real_name.trim().to_string());
    }
    sub.push(c.char_type.label().to_string());
    sub.push(c.power_level.name(c.ruleset).to_string());
    if c.ruleset == Ruleset::Errata && c.role != Role::None {
        sub.push(format!("{} Role", c.role.label()));
    }
    if !c.story.motivation.trim().is_empty() {
        sub.push(format!("Motivation: {}", c.story.motivation.trim()));
    }
    w.text_at(&sub.join("   |   "), MARGIN, 17.5, 9.5, Font::Regular, Rgb3(0.85, 0.86, 0.9));
    let rs = c.ruleset.label().to_uppercase();
    let rw = text_width_mm(&rs, 7.5, true);
    w.rect(PAGE_W - MARGIN - rw - 6.0, 6.5, rw + 6.0, 5.6, ACCENT);
    w.text_at(&rs, PAGE_W - MARGIN - rw - 3.0, 7.3, 7.5, Font::Bold, WHITE);
    if !c.story.player.trim().is_empty() {
        let p = format!("Player: {}", c.story.player.trim());
        let pw = text_width_mm(&p, 8.0, false);
        w.text_at(&p, PAGE_W - MARGIN - pw, 24.0, 8.0, Font::Regular, Rgb3(0.75, 0.76, 0.8));
    }
    w.y = band_h + 6.0;

    // ---------- Stat boxes ----------
    let mut boxes: Vec<(String, String)> = vec![
        ("EDGE".into(), s.edge.to_string()),
        ("HEALTH".into(), s.health.to_string()),
    ];
    if let Some(r) = s.resolve {
        boxes.push(("RESOLVE".into(), r.to_string()));
    }
    if c.char_type.uses_budget() {
        boxes.push(("HERO POINTS".into(), format!("{}/{}", s.total_spent, s.hp_available)));
    }
    boxes.push(("TRAIT CAP".into(), format!("{}d", s.trait_cap)));
    let gap = 3.0;
    let bw = (content_w - gap * (boxes.len() as f32 - 1.0)) / boxes.len() as f32;
    for (i, (label, value)) in boxes.iter().enumerate() {
        let x = MARGIN + i as f32 * (bw + gap);
        w.rect(x, w.y, bw, 17.0, TINT);
        w.rect(x, w.y, bw, 1.2, ACCENT);
        let vw = text_width_mm(value, 18.0, true);
        w.text_at(value, x + (bw - vw) / 2.0, w.y + 3.2, 18.0, Font::Bold, INK);
        let lw = text_width_mm(label, 7.0, true);
        w.text_at(label, x + (bw - lw) / 2.0, w.y + 12.2, 7.0, Font::Bold, MUTED);
    }
    w.y += 21.0;
    if c.char_type == CharType::Hero {
        let hl = format!(
            "Starting Resolve: {} from highest rank ({}d {}) + {} from Determination/Plot Hooks/Conditions",
            s.resolve_base, s.highest_rank, s.highest_from, s.resolve_bonus
        );
        w.text_at(&hl, MARGIN, w.y, 7.0, Font::Italic, MUTED);
        w.y += 4.5;
    }

    // ---------- Play tracker boxes ----------
    if c.char_type == CharType::Hero {
        w.heading("In Play");
        let labels = ["Current Health", "Current Resolve", "Defining Moments", "Hero Points Unspent"];
        let bw = (content_w - 9.0) / 4.0;
        for (i, l) in labels.iter().enumerate() {
            let x = MARGIN + i as f32 * (bw + 3.0);
            w.outline(x, w.y, bw, 14.0, RULE, 0.6);
            w.text_at(l, x + 1.5, w.y + 1.0, 6.8, Font::Bold, MUTED);
        }
        w.y += 16.0;
    }

    // ---------- Abilities & Talents (two columns) ----------
    w.heading("Abilities & Talents");
    let col_w = (content_w - 8.0) / 2.0;
    let row_h = 5.4;
    let top = w.y;
    let draw_trait = |w: &mut Writer, x: f32, y: f32, name: &str, t: &TraitEntry, show_source: bool, shade: bool| {
        if shade {
            w.rect(x, y - 0.6, col_w, row_h, TINT);
        }
        w.text_at(name, x + 1.5, y + 0.4, 9.5, Font::Regular, INK);
        let r = format!("{}d", t.rank);
        let rw = text_width_mm(&r, 10.0, true);
        w.text_at(&r, x + col_w - rw - 1.5, y + 0.2, 10.0, Font::Bold, INK);
        let mut extra = Vec::new();
        if show_source {
            extra.push(t.source.label().to_string());
        }
        let m = engine::mods_label(rules, None, &t.mods);
        if !m.is_empty() {
            extra.push(m);
        }
        if !extra.is_empty() {
            let e = extra.join(" · ");
            let ew = text_width_mm(&e, 7.0, false);
            w.text_at(&e, x + col_w - rw - 3.5 - ew, y + 1.2, 7.0, Font::Italic, MUTED);
        }
    };
    for (i, t) in c.abilities.iter().enumerate() {
        let show = t.source != Source::Innate;
        draw_trait(&mut w, MARGIN, top + i as f32 * row_h, ABILITIES[i], t, show, i % 2 == 0);
    }
    for (i, t) in c.talents.iter().enumerate() {
        let show = t.source != Source::Trained;
        draw_trait(&mut w, MARGIN + col_w + 8.0, top + i as f32 * row_h, TALENTS[i], t, show, i % 2 == 0);
    }
    // Derived notes under abilities column.
    let mut ny = top + 6.0 * row_h + 3.0;
    let pkg = if c.package == Package::None { "No package".to_string() } else { c.package.label().to_string() };
    w.text_at(&pkg, MARGIN + 1.5, ny, 7.5, Font::Italic, MUTED);
    ny += 4.0;
    let health_rule = if c.options.more_health { "Toughness + higher of Might/Willpower" } else { "avg of Toughness and higher of Might/Willpower" };
    w.text_at(&format!("Health: {health_rule}"), MARGIN + 1.5, ny, 7.5, Font::Italic, MUTED);
    w.y = top + 12.0 * row_h + 2.0;

    // ---------- Powers ----------
    w.heading("Powers");
    let cx = [MARGIN, MARGIN + 52.0, MARGIN + 66.0, MARGIN + 84.0, PAGE_W - MARGIN - 12.0];
    let header = ["POWER", "RANK", "SOURCE", "PROS / CONS / NOTES", "COST"];
    for (i, h) in header.iter().enumerate() {
        w.text_at(h, cx[i] + 1.0, w.y, 6.8, Font::Bold, MUTED);
    }
    w.y += 4.2;
    if c.powers.is_empty() {
        w.text_at("None", MARGIN + 1.0, w.y, 9.0, Font::Italic, MUTED);
        w.y += 5.0;
    }
    // Group by Source like the book's stat blocks.
    let mut order: Vec<usize> = (0..c.powers.len()).collect();
    order.sort_by_key(|&i| (c.powers[i].source as u8, engine::power_label(&c.powers[i])));
    for (row, &i) in order.iter().enumerate() {
        let p = &c.powers[i];
        let pc = &s.power_costs[i];
        let def = engine::power_def(rules, p);
        let mut notes = engine::mods_label(rules, def, &p.mods);
        let mut extra = Vec::new();
        if let Some(d) = def {
            if let crate::rules::CostSpec::Choice { options, .. } = &d.cost
                && let Some(o) = options.get(p.option) {
                    extra.push(o.label.clone());
                }
            if matches!(d.cost, crate::rules::CostSpec::PerUnit { .. }) && p.units != 1 {
                extra.push(format!("×{}", p.units));
            }
            if matches!(d.cost, crate::rules::CostSpec::PerPowerLevel { .. }) {
                extra.push(PowerLevel::from_ordinal(p.units).name(c.ruleset).to_string());
            }
            if d.name == "Summoning" {
                extra.push(format!("{}d Threat", p.threat));
            }
            if !p.base_trait.is_empty() && matches!(&d.rank, crate::rules::RankSpec::Baseline { base, .. } if base.starts_with("choice:") || base == "special") {
                extra.push(format!("based on {}", p.base_trait));
            }
        }
        if !extra.is_empty() {
            notes = if notes.is_empty() { extra.join(", ") } else { format!("{}; {}", extra.join(", "), notes) };
        }
        if !p.notes.trim().is_empty() {
            notes = if notes.is_empty() { p.notes.trim().to_string() } else { format!("{notes}. {}", p.notes.trim()) };
        }
        let name_lines = wrap(&engine::power_label(p), cx[1] - cx[0] - 2.0, 9.0, true);
        let note_lines = wrap(&notes, cx[4] - cx[3] - 3.0, 7.8, false);
        let lines = name_lines.len().max(note_lines.len()).max(1);
        let h = lines as f32 * 3.9 + 1.8;
        w.ensure(h);
        if row % 2 == 0 {
            w.rect(MARGIN, w.y - 0.8, content_w, h, TINT);
        }
        for (k, l) in name_lines.iter().enumerate() {
            w.text_at(l, cx[0] + 1.0, w.y + k as f32 * 3.9, 9.0, Font::Bold, INK);
        }
        let rank = pc.rank.map(|r| format!("{r}d")).unwrap_or_else(|| "—".into());
        w.text_at(&rank, cx[1] + 1.0, w.y, 9.0, Font::Bold, INK);
        w.text_at(p.source.label(), cx[2] + 1.0, w.y + 0.3, 8.0, Font::Regular, INK);
        for (k, l) in note_lines.iter().enumerate() {
            w.text_at(l, cx[3] + 1.0, w.y + 0.3 + k as f32 * 3.9, 7.8, Font::Regular, INK);
        }
        let cost = pc.cost.to_string();
        let cw = text_width_mm(&cost, 9.0, false);
        w.text_at(&cost, PAGE_W - MARGIN - cw - 1.0, w.y, 9.0, Font::Regular, INK);
        w.y += h;
    }
    w.y += 1.0;

    // ---------- Perks & Flaws ----------
    w.heading("Perks");
    if c.perks.is_empty() {
        w.text_at("None", MARGIN + 1.0, w.y, 9.0, Font::Italic, MUTED);
        w.y += 5.0;
    }
    for (i, p) in c.perks.iter().enumerate() {
        let def = rules.perk(&p.def);
        let mut name = p.def.clone();
        if let Some(d) = def {
            match &d.cost {
                crate::rules::CostSpec::PerUnit { .. } if p.units != 1 => name.push_str(&format!(" ×{}", p.units)),
                crate::rules::CostSpec::Choice { options, .. } => {
                    if let Some(o) = options.get(p.option) {
                        name = o.label.clone();
                    }
                }
                crate::rules::CostSpec::PerPowerLevel { .. } => {
                    name.push_str(&format!(" ({})", PowerLevel::from_ordinal(p.units).name(c.ruleset)))
                }
                _ => {}
            }
        }
        let line = if p.note.trim().is_empty() { name } else { format!("{name} — {}", p.note.trim()) };
        w.ensure(5.0);
        let x0 = w.y;
        w.paragraph(&line, MARGIN + 1.0, content_w - 16.0, 9.0, Font::Regular, INK);
        let cost = s.perk_costs[i].to_string();
        let cw = text_width_mm(&cost, 9.0, false);
        w.text_at(&cost, PAGE_W - MARGIN - cw - 1.0, x0, 9.0, Font::Regular, INK);
    }

    w.heading("Flaws");
    if c.flaws.is_empty() {
        w.text_at("None", MARGIN + 1.0, w.y, 9.0, Font::Italic, MUTED);
        w.y += 5.0;
    }
    for f in &c.flaws {
        let mut tags = Vec::new();
        if let Some(d) = rules.flaw(&f.def)
            && d.kind != crate::rules::FlawKind::Ordinary && d.kind != crate::rules::FlawKind::Special {
                tags.push(d.kind.label().to_string());
            }
        if engine::flaw_grants_issue_resolve(rules, f) {
            tags.push("+1 Resolve per issue".into());
        }
        let mut line = f.def.clone();
        if !f.note.trim().is_empty() {
            line.push_str(&format!(" ({})", f.note.trim()));
        }
        if !tags.is_empty() {
            line.push_str(&format!("  [{}]", tags.join(", ")));
        }
        w.paragraph(&line, MARGIN + 1.0, content_w - 2.0, 9.0, Font::Regular, INK);
    }
    if s.cost_flaws > 0 {
        w.text_at(&format!("Extra Flaws cost {} HP", s.cost_flaws), MARGIN + 1.0, w.y, 7.5, Font::Italic, MUTED);
        w.y += 4.0;
    }

    // ---------- Finishing touches ----------
    let st = &c.story;
    let has_story = !st.quote.trim().is_empty()
        || st.costume.iter().chain(st.civilian.iter()).any(|x| !x.trim().is_empty())
        || st.connections.iter().any(|x| !x.name.trim().is_empty())
        || st.details.iter().any(|x| !x.trim().is_empty())
        || !st.origin.trim().is_empty()
        || !st.gear.trim().is_empty()
        || !st.concept.trim().is_empty();
    if has_story {
        w.heading("Finishing Touches");
        if !st.quote.trim().is_empty() {
            w.paragraph(&format!("“{}”", st.quote.trim()), MARGIN + 4.0, content_w - 8.0, 11.0, Font::Italic, INK);
            w.y += 2.0;
        }
        w.labeled("CONCEPT", &st.concept);
        let list = |xs: &[String; 3]| xs.iter().filter(|x| !x.trim().is_empty()).map(|x| x.trim().to_string()).collect::<Vec<_>>().join(" • ");
        w.labeled("IN COSTUME", &list(&st.costume));
        w.labeled("IN CIVILIAN CLOTHES", &list(&st.civilian));
        let conns: Vec<String> = st
            .connections
            .iter()
            .filter(|x| !x.name.trim().is_empty())
            .map(|x| if x.description.trim().is_empty() { x.name.trim().to_string() } else { format!("{}: {}", x.name.trim(), x.description.trim()) })
            .collect();
        w.labeled("CONNECTIONS", &conns.join("\n"));
        let details: Vec<String> = st.details.iter().filter(|x| !x.trim().is_empty()).map(|x| format!("• {}", x.trim())).collect();
        w.labeled("DETAILS", &details.join("\n"));
        w.labeled("ORIGIN", &st.origin);
        w.labeled("GEAR", &st.gear);
        w.labeled("NOTES", &st.notes);
    }

    // ---------- Advancement ----------
    if !c.awards.is_empty() {
        w.heading("Advancement");
        for a in &c.awards {
            let line = format!("{:+} HP   {}   {}", a.amount, a.date.trim(), a.note.trim());
            w.paragraph(&line, MARGIN + 1.0, content_w - 2.0, 8.5, Font::Regular, INK);
        }
        w.text_at(&format!("Earned in play: {} HP (total available {})", s.hp_earned, s.hp_available), MARGIN + 1.0, w.y, 7.5, Font::Italic, MUTED);
        w.y += 4.0;
    }

    w.doc.save_to_bytes().map_err(|e| e.to_string())
}

pub fn export(c: &Character, path: &std::path::Path) -> Result<(), String> {
    let bytes = render(c)?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_long_text() {
        let lines = wrap("one two three four five six seven eight nine ten", 20.0, 9.0, false);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| text_width_mm(l, 9.0, false) <= 20.0 + 0.01 || !l.contains(' ')));
    }

    #[test]
    fn renders_pdf_for_empty_and_full_characters() {
        for r in Ruleset::ALL {
            let c = Character::new(r);
            let bytes = render(&c).unwrap();
            assert!(bytes.starts_with(b"%PDF"));
        }
    }
}
