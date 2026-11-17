//! The editing tabs for a character.

use crate::app::{Doc, Tab};
use crate::engine::{self, PowerCost};
use crate::model::*;
use crate::rules::{self, CostSpec, ModCost, ModDef, ModKind, PowerDef, RankSpec, Rules};
use crate::ui::theme::{self, Palette};
use egui::{RichText, Ui};
use std::collections::HashSet;

pub const RULES_NOTES: &str = "\
How the two rule sets differ in this app (summarised from the Combined Updates; see the books for full text):

• Roles (Blaster, Controller, Defender, Fighter, Scout, Tank) exist only in the errata. Each gives +1d to rolls its Role covers and lets you spend 1 Resolve to make 6s explode. The core book instead has the Specialty Power (Street/Low Level games).
• Power list changes: Evasion → Dodge (plus a new Parry), Power Absorption/Mimicry → Absorption/Mimicry, Form split into Form, Energy/Gaseous/Liquid/Solid, Light/Effect → Light/Sound/Effect, Animal Speech and Fast Healing are separate Powers, Port is new, Transformation sub-powers stand alone.
• Many costs changed. Examples: Regeneration 6 → 12 HP (heals per minute, with Fast/Slow options); Quick Change 3 → 1 HP; Strike is 1 HP per 2 ranks (Parrying Pro raises it to 1 per rank); Martial Arts/Blink use the average of Agility and Might as baseline; Super Senses are mostly flat-cost Default Rank effects.
• Generic Pros & Cons were reworked: Area/Zone +4, Charges/Uses −1 to −6, Conditional/Limited/Shutdown up to −6, Overkill lowers cost by 1 per rank AND raises that Trait's cap by 3d.
• Summoning costs ½ Threat HP per rank in the errata (core: 1 HP per rank per 2d Threat, rounded up). Max Threat: cap −4d (errata) vs cap −6d (core).
• Resolve: in the core book, Powers that can't attack, defend or affect others (Flight, Running, senses, Detection...) are ignored. In the errata only Talents, Talent-boosting Expertise, Detection and Acute Sense are ignored; Shapeshifting raises effective physical Abilities for Resolve.
• Teleportation may exceed the Trait Cap in the errata.
• Flaws: the core book caps Heroes at 6 Flaws; the errata has no stated maximum. Extra Flaws beyond 3 cost 3 HP in both.
• Pet/Sidekick: errata costs 6 HP per power level; core gives 10 HP of companion per HP spent.
• Edge: both books' character chapters use Perception + the higher of Agility or Intellect. The errata combat chapter says Agility or Willpower; a per-character option lets you use that wording.

Costs this app applies everywhere: Hero Points are tracked in half-points so '1 per 2 ranks' rounds up once per Power; no Power costs less than 1 HP (or 1 HP per 2 ranks), except Powers listed at 0 HP. Anything unusual can be given a GM-approved cost override (Rule Uno).";

#[derive(Default)]
pub struct UiState {
    pub picker_open: bool,
    pub search: String,
    pub category: String,
    pub expanded: HashSet<u64>,
    pub custom_perk: String,
    pub custom_flaw: String,
    pub award_amount: i32,
    pub award_note: String,
    pub award_date: String,
    pub dice: crate::ui::play::Roller,
    pub add_perk: String,
    pub add_flaw: String,
}

pub fn show(ui: &mut Ui, tab: Tab, doc: &mut Doc, st: &mut UiState, dark: bool) {
    let pal = theme::palette(dark);
    match tab {
        Tab::Overview => overview(ui, doc, &pal, dark),
        Tab::Traits => traits(ui, doc, &pal, dark),
        Tab::Powers => powers(ui, doc, st, &pal, dark),
        Tab::PerksFlaws => perks_flaws(ui, doc, st, &pal, dark),
        Tab::Story => story(ui, doc, &pal, dark),
        Tab::Advancement => advancement(ui, doc, st, &pal, dark),
        Tab::Play => crate::ui::play::show(ui, doc, &mut st.dice, &pal, dark),
    }
}

fn hint(ui: &mut Ui, pal: &Palette, text: &str) {
    ui.label(RichText::new(text).size(12.0).color(pal.muted));
}

fn rank_editor(ui: &mut Ui, v: &mut i32, min: i32, max: i32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if ui.add_enabled(*v > min, egui::Button::new("−").small()).clicked() {
            *v -= 1;
            changed = true;
        }
        changed |= ui.add(egui::DragValue::new(v).range(min..=max).suffix("d").speed(0.1)).changed();
        if ui.add_enabled(*v < max, egui::Button::new("+").small()).clicked() {
            *v += 1;
            changed = true;
        }
    });
    changed
}

fn opt_i32(ui: &mut Ui, label: &str, v: &mut Option<i32>, default: i32, range: std::ops::RangeInclusive<i32>) {
    ui.horizontal(|ui| {
        let mut on = v.is_some();
        if ui.checkbox(&mut on, label).changed() {
            *v = if on { Some(default) } else { None };
        }
        if let Some(x) = v {
            ui.add(egui::DragValue::new(x).range(range));
        }
    });
}

// ---------------------------------------------------------------- Overview

fn overview(ui: &mut Ui, doc: &mut Doc, pal: &Palette, dark: bool) {
    let c = &mut doc.ch;
    let ruleset = c.ruleset;
    ui.columns(2, |cols| {
        theme::card(&mut cols[0], dark, |ui| {
            theme::section_title(ui, "Identity");
            egui::Grid::new("identity").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label("Hero name");
                ui.add(egui::TextEdit::singleline(&mut c.story.hero_name).desired_width(f32::INFINITY));
                ui.end_row();
                ui.label("Real name");
                ui.add(egui::TextEdit::singleline(&mut c.story.real_name).desired_width(f32::INFINITY));
                ui.end_row();
                ui.label("Player");
                ui.add(egui::TextEdit::singleline(&mut c.story.player).desired_width(f32::INFINITY));
                ui.end_row();
                ui.label("Concept");
                ui.add(egui::TextEdit::singleline(&mut c.story.concept).hint_text("One-line pitch").desired_width(f32::INFINITY));
                ui.end_row();
                ui.label("Type");
                ui.horizontal(|ui| {
                    for t in CharType::ALL {
                        ui.selectable_value(&mut c.char_type, t, t.label());
                    }
                });
                ui.end_row();
                ui.label("Motivation");
                let list: &[&str] = if matches!(c.char_type, CharType::Villain | CharType::Foe) { &VILLAIN_MOTIVATIONS } else { &MOTIVATIONS };
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("motivation")
                        .selected_text(if c.story.motivation.is_empty() { "Choose…" } else { c.story.motivation.as_str() })
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            for m in list {
                                ui.selectable_value(&mut c.story.motivation, m.to_string(), *m);
                            }
                        });
                    ui.add(egui::TextEdit::singleline(&mut c.story.motivation).hint_text("or type your own").desired_width(140.0));
                });
                ui.end_row();
            });
        });
        theme::card(&mut cols[1], dark, |ui| {
            theme::section_title(ui, "Game Setup");
            egui::Grid::new("setup").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                ui.label("Rules");
                ui.horizontal(|ui| {
                    theme::badge(ui, ruleset.label(), theme::ruleset_color(ruleset));
                    ui.label(RichText::new("(Edit ▸ Convert ruleset…)").small().color(pal.muted));
                });
                ui.end_row();
                ui.label("Power level");
                egui::ComboBox::from_id_salt("pl")
                    .selected_text(format!("{} — {} HP, {}d cap", c.power_level.name(ruleset), c.power_level.hero_points(), c.power_level.trait_cap()))
                    .width(260.0)
                    .show_ui(ui, |ui| {
                        for pl in PowerLevel::ALL {
                            ui.selectable_value(&mut c.power_level, pl, format!("{} — {} HP, {}d cap", pl.name(ruleset), pl.hero_points(), pl.trait_cap()));
                        }
                    });
                ui.end_row();
                ui.label("Starting HP");
                opt_i32(ui, "GM override", &mut c.options.starting_hp_override, c.power_level.hero_points(), 0..=2000);
                ui.end_row();
                ui.label("Trait Cap");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("capmode").selected_text(c.options.cap_mode.label()).show_ui(ui, |ui| {
                        for m in CapMode::ALL {
                            ui.selectable_value(&mut c.options.cap_mode, m, m.label());
                        }
                    });
                    if c.options.cap_mode == CapMode::Soft {
                        ui.add(egui::DragValue::new(&mut c.options.cap_bonus).range(-6..=24).prefix("+").suffix("d"));
                    }
                });
                ui.end_row();
                if rules::rules(ruleset).config().has_roles && c.char_type == CharType::Hero {
                    ui.label("Role");
                    ui.vertical(|ui| {
                        egui::ComboBox::from_id_salt("role").selected_text(c.role.label()).width(160.0).show_ui(ui, |ui| {
                            for r in Role::ALL {
                                ui.selectable_value(&mut c.role, r, r.label()).on_hover_text(r.applies_to());
                            }
                        });
                        if c.role != Role::None {
                            ui.label(RichText::new(format!("+1d to {}", c.role.applies_to())).small().color(pal.muted));
                        }
                    });
                    ui.end_row();
                }
                ui.label("Package");
                let mut pkg = c.package;
                egui::ComboBox::from_id_salt("pkg").selected_text(pkg.label()).width(260.0).show_ui(ui, |ui| {
                    for p in Package::ALL {
                        ui.selectable_value(&mut pkg, p, p.label());
                    }
                });
                if pkg != c.package {
                    c.apply_package(pkg);
                }
                ui.end_row();
            });
            ui.add_space(6.0);
            ui.label(RichText::new("TABLE OPTIONS").small().strong().color(pal.muted));
            ui.checkbox(&mut c.options.more_health, "More Health (sum of the two Abilities)");
            ui.checkbox(&mut c.options.lethal_damage, "Lethal Damage rules in play");
            if let Some(label) = rules::rules(ruleset).config().edge_alt_label {
                ui.checkbox(&mut c.options.edge_uses_willpower, label);
            }
            ui.checkbox(&mut c.options.creation_complete, "Character creation complete (Flaws beyond 3 are bought in play)");
            ui.collapsing("Manual overrides", |ui| {
                opt_i32(ui, "Edge", &mut c.options.edge_override, doc.summary.edge, 0..=200);
                opt_i32(ui, "Health", &mut c.options.health_override, doc.summary.health, 0..=200);
                opt_i32(ui, "Resolve", &mut c.options.resolve_override, doc.summary.resolve.unwrap_or(0), 0..=99);
            });
        });
    });
    ui.add_space(10.0);
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "At a Glance");
        let c = &doc.ch;
        let s = &doc.summary;
        let abilities: Vec<GlanceItem> = ABILITIES
            .iter()
            .enumerate()
            .map(|(i, a)| GlanceItem {
                name: a.to_string(),
                value: format!("{}d", c.abilities[i].rank),
                color: pal.text,
                hover: String::new(),
            })
            .collect();
        glance_grid(ui, "glance_abilities", &abilities, pal);
        ui.add_space(10.0);
        let rules = rules::rules(c.ruleset);
        if c.powers.is_empty() {
            hint(ui, pal, "No Powers yet — add some on the Powers tab.");
        }
        let powers: Vec<GlanceItem> = c
            .powers
            .iter()
            .zip(&s.power_costs)
            .map(|(p, pc)| GlanceItem {
                name: engine::power_label(p),
                value: pc.rank.map(|r| format!("{r}d")).unwrap_or_default(),
                color: if pc.unknown { theme::BAD } else { pal.text },
                hover: engine::power_def(rules, p).map(|d| d.summary.clone()).unwrap_or_default(),
            })
            .collect();
        glance_grid(ui, "glance_powers", &powers, pal);
    });
}

/// One cell of the At a Glance grid.
struct GlanceItem {
    name: String,
    /// Shown right-aligned at the top of the cell (e.g. "10d"); may be empty.
    value: String,
    color: egui::Color32,
    hover: String,
}

/// Lay items out in a fixed grid: four equal columns (two on narrow windows). Long names wrap
/// inside their own cell, and every cell in a row is drawn at the height of the tallest one.
fn glance_grid(ui: &mut Ui, id: &str, items: &[GlanceItem], pal: &Palette) {
    if items.is_empty() {
        return;
    }
    let gap = 8.0;
    let pad = egui::vec2(10.0, 6.0);
    // Measure the width that is actually visible: on narrow windows the tab can be laid out wider
    // than the viewport, and columns past the visible edge would be hidden.
    let avail = ui.available_width().min(ui.clip_rect().right() - ui.cursor().left() - 4.0).max(160.0);
    let cols = if avail >= 640.0 {
        4
    } else if avail >= 470.0 {
        3
    } else {
        2
    };
    let col_w = ((avail - gap * (cols as f32 - 1.0)) / cols as f32).floor();
    let name_font = egui::FontId::proportional(14.0);
    let value_font = egui::FontId::proportional(14.0);
    for (row_i, row) in items.chunks(cols).enumerate() {
        let top = ui.cursor().top();
        let left = ui.cursor().left();
        // Reserve a background slot per cell; it's filled in once the row height is known.
        let bg: Vec<egui::layers::ShapeIdx> = row.iter().map(|_| ui.painter().add(egui::Shape::Noop)).collect();
        let mut heights = Vec::with_capacity(row.len());
        for (k, item) in row.iter().enumerate() {
            let x = left + k as f32 * (col_w + gap);
            let value = ui.painter().layout_no_wrap(item.value.clone(), value_font.clone(), item.color);
            let value_w = if item.value.is_empty() { 0.0 } else { value.size().x + 8.0 };
            let name_w = (col_w - 2.0 * pad.x - value_w).max(20.0);
            let name = ui.painter().layout(item.name.clone(), name_font.clone(), item.color, name_w);
            let h = name.size().y.max(value.size().y) + 2.0 * pad.y;
            let origin = egui::pos2(x + pad.x, top + pad.y);
            ui.painter().galley(origin, name, item.color);
            if !item.value.is_empty() {
                let vx = x + col_w - pad.x - value.size().x;
                ui.painter().galley(egui::pos2(vx, top + pad.y), value, item.color);
            }
            heights.push(h);
        }
        let row_h = heights.iter().cloned().fold(0.0, f32::max);
        for (k, item) in row.iter().enumerate() {
            let x = left + k as f32 * (col_w + gap);
            let rect = egui::Rect::from_min_size(egui::pos2(x, top), egui::vec2(col_w, row_h));
            ui.painter().set(bg[k], egui::Shape::rect_filled(rect, 6.0, pal.card_alt));
            if !item.hover.is_empty() {
                ui.interact(rect, ui.scope_id().with((id, row_i, k)), egui::Sense::hover()).on_hover_text(&item.hover);
            }
        }
        let row_rect = egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(avail, row_h));
        ui.advance_cursor_after_rect(row_rect);
        ui.add_space(gap - ui.spacing().item_spacing.y);
    }
}

// ---------------------------------------------------------------- Traits

fn traits(ui: &mut Ui, doc: &mut Doc, pal: &Palette, dark: bool) {
    let c = &mut doc.ch;
    let rules = rules::rules(c.ruleset);
    let cap = doc.summary.trait_cap;
    let ab = c.package.ability_base();
    let tb = c.package.talent_base();
    if c.package != Package::None {
        hint(ui, pal, &format!("{} — Abilities start at {}d and Talents at {}d; you pay only for ranks above that.", c.package.label(), ab, tb));
    } else {
        hint(ui, pal, "Each rank costs 1 Hero Point. Ordinary people have 2d; 6d is peak human.");
    }
    ui.add_space(6.0);
    let mut abilities = |ui: &mut Ui| {
        theme::card(ui, dark, |ui| {
            theme::section_title(ui, "Abilities");
            trait_grid(ui, "abilities", &ABILITIES, &mut c.abilities, &doc.summary.ability_costs, ab.max(0), cap, rules, pal, Source::Innate);
        });
    };
    if ui.available_width() >= 900.0 {
        ui.horizontal_top(|ui| {
            abilities(ui);
            theme::card(ui, dark, |ui| {
                theme::section_title(ui, "Talents");
                trait_grid(ui, "talents", &TALENTS, &mut c.talents, &doc.summary.talent_costs, tb.max(0), cap, rules, pal, Source::Trained);
            });
        });
    } else {
        abilities(ui);
        ui.add_space(8.0);
        theme::card(ui, dark, |ui| {
            theme::section_title(ui, "Talents");
            trait_grid(ui, "talents", &TALENTS, &mut c.talents, &doc.summary.talent_costs, tb.max(0), cap, rules, pal, Source::Trained);
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn trait_grid(
    ui: &mut Ui,
    id: &str,
    names: &[&str],
    entries: &mut [TraitEntry],
    costs: &[i32],
    min: i32,
    cap: i32,
    rules: &'static Rules,
    pal: &Palette,
    default_source: Source,
) {
    egui::Grid::new(id).num_columns(5).spacing([10.0, 6.0]).striped(true).show(ui, |ui| {
        ui.label(RichText::new("TRAIT").small().color(pal.muted));
        ui.label(RichText::new("RANK").small().color(pal.muted));
        ui.label(RichText::new("SOURCE").small().color(pal.muted));
        ui.label(RichText::new("PROS/CONS").small().color(pal.muted));
        ui.label(RichText::new("HP").small().color(pal.muted));
        ui.end_row();
        for (i, name) in names.iter().enumerate() {
            let e = &mut entries[i];
            let over = e.rank > cap + e.mods.iter().filter_map(|m| rules.generic_mod(&m.name)).map(|m| m.trait_cap_bonus).max().unwrap_or(0);
            ui.label(RichText::new(*name).strong().color(if over { theme::BAD } else { pal.text }));
            let max = cap + 3;
            rank_editor(ui, &mut e.rank, min.max(0), max.max(min));
            egui::ComboBox::from_id_salt((id, i, "src"))
                .selected_text(RichText::new(e.source.label()).color(if e.source == default_source { pal.muted } else { pal.text }))
                .width(84.0)
                .show_ui(ui, |ui| {
                    for s in Source::ALL.iter().filter(|s| **s != Source::NoSource) {
                        ui.selectable_value(&mut e.source, *s, s.label());
                    }
                });
            let label = if e.mods.is_empty() { "＋".to_string() } else { format!("{} ✎", e.mods.len()) };
            ui.menu_button(label, |ui| {
                ui.set_min_width(360.0);
                ui.label(RichText::new(format!("{name}: Pros & Cons")).strong());
                hint(ui, pal, "Generic Pros & Cons can apply to Abilities and Talents (e.g. Overkill on Might, Item).");
                mod_list(ui, rules, None, &mut e.mods, (id, i), pal);
            });
            ui.label(costs[i].to_string());
            ui.end_row();
        }
    });
}

// ---------------------------------------------------------------- Modifiers

fn mod_cost_text(m: &ModDef) -> String {
    m.cost.describe()
}

fn mod_list(ui: &mut Ui, rules: &'static Rules, def: Option<&PowerDef>, mods: &mut Vec<ModPick>, id: impl std::hash::Hash + Copy + std::fmt::Debug, pal: &Palette) {
    let mut remove: Option<usize> = None;
    for (i, m) in mods.iter_mut().enumerate() {
        let md = rules.find_mod(def, &m.name, m.generic).cloned();
        ui.horizontal_wrapped(|ui| {
            let (color, tag) = match md.as_ref().map(|d| d.kind) {
                Some(ModKind::Pro) => (theme::GOOD, "PRO"),
                Some(ModKind::Con) => (theme::WARN, "CON"),
                None => (theme::BAD, "?"),
            };
            ui.label(RichText::new(tag).small().strong().color(color));
            let r = ui.label(RichText::new(&m.name).strong());
            if let Some(d) = &md {
                r.on_hover_text(format!("{}\n{}", mod_cost_text(d), d.summary));
                if let Some(choices) = d.cost.choices() {
                    let current = m.value.filter(|v| choices.contains(v)).unwrap_or(choices[0]);
                    let mut v = current;
                    egui::ComboBox::from_id_salt((id, i, "mv")).selected_text(format!("{v:+}")).width(56.0).show_ui(ui, |ui| {
                        for c in &choices {
                            ui.selectable_value(&mut v, *c, format!("{c:+}"));
                        }
                    });
                    if Some(v) != m.value {
                        m.value = Some(v);
                    }
                } else if matches!(d.cost, ModCost::Special { .. }) && !matches!(d.name.as_str(), "Overkill" | "Weak") {
                    let mut v = m.value.unwrap_or(0);
                    if ui.add(egui::DragValue::new(&mut v).range(-30..=30).prefix("HP ")).changed() {
                        m.value = Some(v);
                    }
                } else {
                    ui.label(RichText::new(d.cost.describe()).small().color(pal.muted));
                }
                if d.repeatable {
                    ui.add(egui::DragValue::new(&mut m.count).range(1..=12).prefix("×"));
                }
            } else {
                ui.label(RichText::new("not in these rules").small().color(theme::BAD));
            }
            ui.add(egui::TextEdit::singleline(&mut m.note).hint_text("details").desired_width(120.0));
            if ui.small_button("✕").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        mods.remove(i);
    }
    ui.horizontal(|ui| {
        if let Some(d) = def
            && !d.mods.is_empty() {
                ui.menu_button(format!("＋ {} option", d.name), |ui| {
                    ui.set_min_width(320.0);
                    for m in &d.mods {
                        let txt = format!("{}  ({})", m.name, m.cost.describe());
                        if ui.button(txt).on_hover_text(&m.summary).clicked() {
                            mods.push(ModPick::new(&m.name, false));
                            ui.close();
                        }
                    }
                });
            }
        ui.menu_button("＋ Generic Pro/Con", |ui| {
            ui.set_min_width(360.0);
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for kind in [ModKind::Pro, ModKind::Con] {
                    ui.label(RichText::new(if kind == ModKind::Pro { "PROS" } else { "CONS" }).small().strong().color(pal.muted));
                    for m in rules.data.generic_mods.iter().filter(|m| m.kind == kind) {
                        let txt = format!("{}  ({})", m.name, m.cost.describe());
                        if ui.button(txt).on_hover_text(&m.summary).clicked() {
                            mods.push(ModPick::new(&m.name, true));
                            ui.close();
                        }
                    }
                }
            });
        });
    });
}

// ---------------------------------------------------------------- Powers

fn picker(ui: &mut Ui, c: &mut Character, st: &mut UiState, pal: &Palette, dark: bool) {
    let rules = rules::rules(c.ruleset);
    let max_w = ui.available_width() - 30.0;
    theme::card(ui, dark, |ui| {
        ui.set_max_width(max_w);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Add a Power").strong().size(15.0));
            ui.add(egui::TextEdit::singleline(&mut st.search).hint_text("Search name or effect…").desired_width(240.0));
            let mut cats: Vec<String> = rules.data.powers.iter().filter_map(|p| p.category.clone()).collect();
            cats.sort();
            cats.dedup();
            egui::ComboBox::from_id_salt("cat")
                .selected_text(if st.category.is_empty() { "All groups" } else { st.category.as_str() })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut st.category, String::new(), "All groups");
                    for cat in cats {
                        ui.selectable_value(&mut st.category, cat.clone(), cat);
                    }
                });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    st.picker_open = false;
                }
                if ui.button("＋ Custom Power").on_hover_text("Build your own Power (Rule Uno)").clicked() {
                    let def = PowerDef {
                        name: "Custom Power".into(),
                        category: None,
                        range: "Self".into(),
                        rank: RankSpec::Power,
                        cost: CostSpec::PerRank { value: 1 },
                        affects_resolve: true,
                        summary: String::new(),
                        mods: Vec::new(),
                        ignores_trait_cap: false,
                    };
                    let uid = c.add_power(PowerEntry { def: "Custom Power".into(), ranks: 4, custom: Some(def), ..Default::default() });
                    st.expanded.insert(uid);
                }
            });
        });
        ui.add_space(4.0);
        let q = st.search.to_lowercase();
        egui::ScrollArea::vertical().id_salt("picker").max_height(300.0).max_width(max_w).show(ui, |ui| {
            egui::Grid::new("picker_grid").num_columns(4).striped(true).spacing([12.0, 4.0]).max_col_width(240.0).show(ui, |ui| {
                for d in &rules.data.powers {
                    if !st.category.is_empty() && d.category.as_deref() != Some(st.category.as_str()) {
                        continue;
                    }
                    if !q.is_empty() && !d.name.to_lowercase().contains(&q) && !d.summary.to_lowercase().contains(&q) {
                        continue;
                    }
                    let owned = c.powers.iter().filter(|p| p.def == d.name).count();
                    if ui.button(RichText::new("Add").strong()).clicked() {
                        let mut p = PowerEntry { def: d.name.clone(), ..Default::default() };
                        if d.buys_ranks() {
                            p.ranks = if d.is_baseline() { 2 } else { 6 };
                        }
                        if let RankSpec::Baseline { base, .. } = &d.rank
                            && let Some(rest) = base.strip_prefix("choice:") {
                                p.base_trait = rest.split('|').next().unwrap_or_default().to_string();
                            }
                        if matches!(d.cost, CostSpec::PerPowerLevel { .. }) {
                            p.units = c.power_level.ordinal();
                        }
                        if let CostSpec::Choice { options, per_rank: false } = &d.cost
                            && c.options.lethal_damage && options.len() == 2 && options[1].label.contains("Lethal") {
                                p.option = 1;
                            }
                        if d.name == "Determination" || d.name == "Variant" || d.name == "Preparation" {
                            p.source = Source::NoSource;
                        }
                        let uid = c.add_power(p);
                        st.expanded.insert(uid);
                    }
                    let mut name = RichText::new(&d.name).strong();
                    if owned > 0 {
                        name = name.color(theme::ACCENT);
                    }
                    ui.label(name).on_hover_text(&d.summary);
                    ui.label(RichText::new(d.cost.describe()).color(pal.muted));
                    let meta = match &d.category {
                        Some(cat) => format!("{} · {}", d.range, cat),
                        None => d.range.clone(),
                    };
                    ui.label(RichText::new(meta).small().color(pal.muted))
                        .on_hover_text(format!("{}\n{} · {}", d.summary, d.rank_label(), d.cost.describe()));
                    ui.end_row();
                }
            });
        });
    });
}

fn powers(ui: &mut Ui, doc: &mut Doc, st: &mut UiState, pal: &Palette, dark: bool) {
    ui.horizontal(|ui| {
        let label = if st.picker_open { "Hide Power list" } else { "＋ Add Power" };
        if ui.add(egui::Button::new(RichText::new(label).strong()).fill(theme::ACCENT_DIM)).clicked() {
            st.picker_open = !st.picker_open;
        }
        if ui.button("Expand all").clicked() {
            st.expanded = doc.ch.powers.iter().map(|p| p.uid).collect();
        }
        if ui.button("Collapse all").clicked() {
            st.expanded.clear();
        }
        ui.label(RichText::new(format!("{} Powers · {} HP", doc.ch.powers.len(), doc.summary.cost_powers)).color(pal.muted));
    });
    ui.add_space(6.0);
    if st.picker_open {
        picker(ui, &mut doc.ch, st, pal, dark);
        ui.add_space(8.0);
    }
    if doc.ch.powers.is_empty() && !st.picker_open {
        hint(ui, pal, "This character has no Powers. Click “Add Power” to browse the list for the selected ruleset.");
    }

    let rules = rules::rules(doc.ch.ruleset);
    let mut action: Option<(usize, &'static str)> = None;
    let n = doc.ch.powers.len();
    // Values needed while editing (snapshot to avoid borrowing the whole character).
    let costs: Vec<PowerCost> = doc.summary.power_costs.clone();
    let trait_names: Vec<String> = ABILITIES
        .iter()
        .chain(TALENTS.iter())
        .map(|s| s.to_string())
        .chain(doc.ch.powers.iter().map(|p| p.def.clone()))
        .collect();
    let power_level = doc.ch.power_level;
    for (i, p) in doc.ch.powers.iter_mut().enumerate() {
        let pc = costs.get(i).cloned().unwrap_or_default();
        let uid = p.uid;
        let open = st.expanded.contains(&uid);
        let def_owned: Option<PowerDef> = engine::power_def(rules, p).cloned();
        let def = def_owned.as_ref();
        let frame_stroke = if pc.unknown { egui::Stroke::new(1.5, theme::BAD) } else { egui::Stroke::new(1.0, pal.border) };
        ui.push_id(("power_card", uid), |ui| egui::Frame::new().fill(pal.card).stroke(frame_stroke).corner_radius(10).inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if ui.button(if open { "▾" } else { "▸" }).clicked() {
                    if open {
                        st.expanded.remove(&uid);
                    } else {
                        st.expanded.insert(uid);
                    }
                }
                let title = ui.label(RichText::new(engine::power_label(p)).size(15.0).strong());
                if let Some(d) = def {
                    title.on_hover_text(&d.summary);
                }
                if let Some(r) = pc.rank {
                    theme::badge(ui, &format!("{r}d"), theme::ACCENT);
                }
                ui.label(RichText::new(p.source.label()).color(pal.muted));
                if !p.mods.is_empty() {
                    ui.label(RichText::new(engine::mods_label(rules, def, &p.mods)).small().color(pal.muted));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🗑").on_hover_text("Remove").clicked() {
                        action = Some((i, "del"));
                    }
                    if ui.small_button("⧉").on_hover_text("Duplicate").clicked() {
                        action = Some((i, "dup"));
                    }
                    if ui.add_enabled(i + 1 < n, egui::Button::new("↓").small()).clicked() {
                        action = Some((i, "down"));
                    }
                    if ui.add_enabled(i > 0, egui::Button::new("↑").small()).clicked() {
                        action = Some((i, "up"));
                    }
                    ui.label(RichText::new(format!("{} HP", pc.cost)).strong());
                    if pc.floor_applied {
                        ui.label(RichText::new("min").small().color(theme::WARN)).on_hover_text("Cons would take this below the minimum cost; the minimum applies");
                    }
                });
            });
            if !open {
                return;
            }
            ui.separator();
            if let Some(d) = def {
                ui.label(RichText::new(&d.summary).color(pal.muted));
                ui.label(RichText::new(format!("{} · {} · {}", d.range, d.rank_label(), d.cost.describe())).small().color(pal.muted));
            } else {
                ui.label(RichText::new(&pc.detail).color(theme::BAD));
            }
            ui.add_space(4.0);
            power_fields(ui, p, def, &pc, &trait_names, power_level, pal);
            ui.add_space(4.0);
            ui.label(RichText::new("PROS & CONS").small().strong().color(pal.muted));
            mod_list(ui, rules, def, &mut p.mods, ("pmods", uid), pal);
            ui.add_space(4.0);
            ui.add(egui::TextEdit::multiline(&mut p.notes).hint_text("Notes, special effects, limits…").desired_rows(2).desired_width(f32::INFINITY));
            ui.collapsing("Advanced", |ui| {
                opt_i32(ui, "GM-approved cost override (HP)", &mut p.cost_override, pc.cost, -50..=500);
                ui.horizontal(|ui| {
                    ui.label("Counts toward Resolve:");
                    let default = def.map(|d| d.affects_resolve).unwrap_or(true);
                    ui.selectable_value(&mut p.resolve_override, None, format!("Rules default ({})", if default { "yes" } else { "no" }));
                    ui.selectable_value(&mut p.resolve_override, Some(true), "Yes");
                    ui.selectable_value(&mut p.resolve_override, Some(false), "No");
                });
                ui.label(RichText::new(format!("Cost: {}", pc.detail)).small().color(pal.muted));
            });
        }));
        ui.add_space(6.0);
    }
    if let Some((i, a)) = action {
        match a {
            "del" => {
                doc.ch.powers.remove(i);
            }
            "dup" => {
                let mut p = doc.ch.powers[i].clone();
                p.uid = 0;
                let uid = doc.ch.add_power(p);
                let last = doc.ch.powers.len() - 1;
                let moved = doc.ch.powers.remove(last);
                doc.ch.powers.insert(i + 1, moved);
                st.expanded.insert(uid);
            }
            "up" => doc.ch.powers.swap(i, i - 1),
            "down" => doc.ch.powers.swap(i, i + 1),
            _ => {}
        }
    }
}

fn power_fields(ui: &mut Ui, p: &mut PowerEntry, def: Option<&PowerDef>, pc: &PowerCost, trait_names: &[String], pl: PowerLevel, pal: &Palette) {
    egui::Grid::new(("pf", p.uid)).num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
        if let Some(custom) = p.custom.as_mut() {
            ui.label("Name");
            ui.add(egui::TextEdit::singleline(&mut custom.name).desired_width(220.0));
            ui.end_row();
            ui.label("Range");
            egui::ComboBox::from_id_salt(("crange", p.uid)).selected_text(custom.range.clone()).show_ui(ui, |ui| {
                for r in ["Self", "Touch", "Ranged", "Zone", "Special"] {
                    ui.selectable_value(&mut custom.range, r.to_string(), r);
                }
            });
            ui.end_row();
            ui.label("Cost");
            ui.horizontal(|ui| {
                let mut kind = match custom.cost {
                    CostSpec::PerRank { .. } => 0,
                    CostSpec::PerTwoRanks => 1,
                    _ => 2,
                };
                let before = kind;
                ui.selectable_value(&mut kind, 0, "Per rank");
                ui.selectable_value(&mut kind, 1, "1 per 2 ranks");
                ui.selectable_value(&mut kind, 2, "Flat");
                if kind != before {
                    custom.cost = match kind {
                        0 => CostSpec::PerRank { value: 1 },
                        1 => CostSpec::PerTwoRanks,
                        _ => CostSpec::Flat { value: 3 },
                    };
                    custom.rank = if kind == 2 { RankSpec::Default } else { RankSpec::Power };
                }
                match &mut custom.cost {
                    CostSpec::PerRank { value } => {
                        ui.add(egui::DragValue::new(value).range(1..=10).suffix(" HP/rank"));
                    }
                    CostSpec::Flat { value } => {
                        ui.add(egui::DragValue::new(value).range(0..=100).suffix(" HP"));
                    }
                    _ => {}
                }
            });
            ui.end_row();
            ui.label("Description");
            ui.add(egui::TextEdit::singleline(&mut custom.summary).desired_width(f32::INFINITY));
            ui.end_row();
        }
        ui.label("Descriptor");
        ui.add(egui::TextEdit::singleline(&mut p.descriptor).hint_text("e.g. Fire, Claws, Trick Arrows").desired_width(220.0));
        ui.end_row();
        ui.label("Source");
        egui::ComboBox::from_id_salt(("src", p.uid)).selected_text(p.source.label()).show_ui(ui, |ui| {
            for s in Source::ALL {
                ui.selectable_value(&mut p.source, s, s.label());
            }
        });
        ui.end_row();
        let Some(d) = def else { return };
        if d.buys_ranks() {
            if let Some(b) = pc.baseline {
                ui.label("Bought ranks");
                ui.horizontal(|ui| {
                    rank_editor(ui, &mut p.ranks, 0, 40);
                    ui.label(RichText::new(format!("baseline {b}d + {} = {}d", p.ranks, b + p.ranks)).color(pal.muted));
                });
            } else {
                ui.label("Rank");
                rank_editor(ui, &mut p.ranks, 0, 40);
            }
            ui.end_row();
        } else if let Some(b) = pc.baseline {
            ui.label("Rank");
            ui.label(format!("{b}d (equals its baseline)"));
            ui.end_row();
        }
        if let RankSpec::Baseline { base, .. } = &d.rank {
            let choices: Option<Vec<String>> = if let Some(rest) = base.strip_prefix("choice:") {
                Some(rest.split('|').map(|s| s.to_string()).collect())
            } else if base == "special" {
                Some(trait_names.iter().filter(|n| **n != d.name).cloned().collect())
            } else {
                None
            };
            if let Some(choices) = choices {
                ui.label(if d.name == "Boost" { "Boosted Trait" } else if d.name == "Expertise" { "Parent Trait" } else { "Baseline Trait" });
                egui::ComboBox::from_id_salt(("base", p.uid))
                    .selected_text(if p.base_trait.is_empty() { "Choose…" } else { p.base_trait.as_str() })
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for ch in choices {
                            ui.selectable_value(&mut p.base_trait, ch.clone(), ch);
                        }
                    });
                ui.end_row();
            }
        }
        match &d.cost {
            CostSpec::Choice { options, .. } => {
                ui.label("Version");
                let sel = options.get(p.option).map(|o| format!("{} ({})", o.label, o.value)).unwrap_or_default();
                egui::ComboBox::from_id_salt(("opt", p.uid)).selected_text(sel).width(220.0).show_ui(ui, |ui| {
                    for (k, o) in options.iter().enumerate() {
                        ui.selectable_value(&mut p.option, k, format!("{} ({})", o.label, o.value));
                    }
                });
                ui.end_row();
            }
            CostSpec::PerUnit { unit, value } => {
                ui.label("Quantity");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut p.units).range(0..=40));
                    ui.label(RichText::new(format!("× {value} HP per {unit}")).color(pal.muted));
                });
                ui.end_row();
            }
            CostSpec::PerPowerLevel { .. } => {
                ui.label("Power level");
                let cur = PowerLevel::from_ordinal(p.units);
                egui::ComboBox::from_id_salt(("ppl", p.uid)).selected_text(format!("{:?}", cur)).show_ui(ui, |ui| {
                    for l in PowerLevel::ALL {
                        if l.ordinal() <= pl.ordinal() {
                            ui.selectable_value(&mut p.units, l.ordinal(), format!("{:?} ({} HP)", l, l.hero_points()));
                        }
                    }
                });
                ui.end_row();
            }
            CostSpec::Special { .. } => match d.name.as_str() {
                "Boost" => {
                    ui.label("Boosted Trait cost");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut p.boost_cost).range(0..=5).suffix(" HP/rank"));
                        ui.label(RichText::new("0 = 1 HP per 2 ranks").small().color(pal.muted));
                    });
                    ui.end_row();
                }
                "Summoning" => {
                    ui.label("Minion Threat");
                    rank_editor(ui, &mut p.threat, 1, 24);
                    ui.end_row();
                }
                _ => {}
            },
            _ => {}
        }
    });
}

// ---------------------------------------------------------------- Perks & Flaws

fn perks_flaws(ui: &mut Ui, doc: &mut Doc, st: &mut UiState, pal: &Palette, dark: bool) {
    let c = &mut doc.ch;
    let rules = rules::rules(c.ruleset);
    let perk_costs = doc.summary.perk_costs.clone();
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Perks");
        hint(ui, pal, "Social and societal advantages bought at a flat cost.");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("addperk")
                .selected_text(if st.add_perk.is_empty() { "Choose a Perk…" } else { st.add_perk.as_str() })
                .width(220.0)
                .show_ui(ui, |ui| {
                    for p in &rules.data.perks {
                        ui.selectable_value(&mut st.add_perk, p.name.clone(), format!("{}  ({})", p.name, p.cost.describe())).on_hover_text(&p.summary);
                    }
                });
            if ui.add_enabled(!st.add_perk.is_empty(), egui::Button::new("Add")).clicked() {
                let mut e = PerkEntry { def: st.add_perk.clone(), ..Default::default() };
                if let Some(d) = rules.perk(&e.def)
                    && matches!(d.cost, CostSpec::PerPowerLevel { .. }) {
                        e.units = (c.power_level.ordinal() - 1).max(1);
                    }
                c.perks.push(e);
            }
            ui.separator();
            ui.add(egui::TextEdit::singleline(&mut st.custom_perk).hint_text("Custom Perk name").desired_width(160.0));
            if ui.add_enabled(!st.custom_perk.trim().is_empty(), egui::Button::new("Add custom")).clicked() {
                c.perks.push(PerkEntry { def: st.custom_perk.trim().to_string(), cost_override: Some(1), ..Default::default() });
                st.custom_perk.clear();
            }
        });
        ui.add_space(6.0);
        let mut remove = None;
        for (i, p) in c.perks.iter_mut().enumerate() {
            let def = rules.perk(&p.def);
            ui.horizontal_wrapped(|ui| {
                let r = ui.label(RichText::new(&p.def).strong());
                if let Some(d) = def {
                    r.on_hover_text(&d.summary);
                    match &d.cost {
                        CostSpec::PerUnit { unit, .. } => {
                            ui.add(egui::DragValue::new(&mut p.units).range(0..=60));
                            ui.label(RichText::new(unit).small().color(pal.muted));
                        }
                        CostSpec::Choice { options, .. } => {
                            egui::ComboBox::from_id_salt(("perkopt", i)).selected_text(options.get(p.option).map(|o| o.label.clone()).unwrap_or_default()).show_ui(ui, |ui| {
                                for (k, o) in options.iter().enumerate() {
                                    ui.selectable_value(&mut p.option, k, format!("{} ({} HP)", o.label, o.value));
                                }
                            });
                        }
                        CostSpec::PerPowerLevel { .. } => {
                            egui::ComboBox::from_id_salt(("perkpl", i)).selected_text(PowerLevel::from_ordinal(p.units).name(c.ruleset)).show_ui(ui, |ui| {
                                for l in PowerLevel::ALL {
                                    ui.selectable_value(&mut p.units, l.ordinal(), l.name(c.ruleset));
                                }
                            });
                        }
                        _ => {}
                    }
                } else {
                    let mut v = p.cost_override.unwrap_or(0);
                    if ui.add(egui::DragValue::new(&mut v).range(0..=100).suffix(" HP")).changed() {
                        p.cost_override = Some(v);
                    }
                }
                ui.add(egui::TextEdit::singleline(&mut p.note).hint_text("details (who, what, where)").desired_width(260.0));
                ui.label(RichText::new(format!("{} HP", perk_costs.get(i).copied().unwrap_or(0))).strong());
                if ui.small_button("✕").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            c.perks.remove(i);
        }
    });
    ui.add_space(10.0);
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Flaws");
        hint(ui, pal, "Heroes need at least 1 Flaw and start with up to 3 (free). Extra Flaws cost 3 HP each in play. Plot Hooks and Conditions grant 1 Resolve at the start of every issue.");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("addflaw")
                .selected_text(if st.add_flaw.is_empty() { "Choose a Flaw…" } else { st.add_flaw.as_str() })
                .width(240.0)
                .show_ui(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                        for f in &rules.data.flaws {
                            let tag = if f.kind == rules::FlawKind::Ordinary { String::new() } else { format!("  — {}", f.kind.label()) };
                            ui.selectable_value(&mut st.add_flaw, f.name.clone(), format!("{}{}", f.name, tag)).on_hover_text(&f.summary);
                        }
                    });
                });
            if ui.add_enabled(!st.add_flaw.is_empty(), egui::Button::new("Add")).clicked() {
                c.flaws.push(FlawEntry { def: st.add_flaw.clone(), ..Default::default() });
            }
            ui.separator();
            ui.add(egui::TextEdit::singleline(&mut st.custom_flaw).hint_text("Custom Flaw name").desired_width(160.0));
            if ui.add_enabled(!st.custom_flaw.trim().is_empty(), egui::Button::new("Add custom")).clicked() {
                c.flaws.push(FlawEntry { def: st.custom_flaw.trim().to_string(), ..Default::default() });
                st.custom_flaw.clear();
            }
        });
        ui.add_space(6.0);
        let mut remove = None;
        for (i, f) in c.flaws.iter_mut().enumerate() {
            let def = rules.flaw(&f.def);
            ui.horizontal_wrapped(|ui| {
                if i >= 3 {
                    theme::badge(ui, "3 HP", theme::WARN);
                }
                let r = ui.label(RichText::new(&f.def).strong());
                if let Some(d) = def {
                    r.on_hover_text(&d.summary);
                    if d.kind != rules::FlawKind::Ordinary {
                        theme::badge(ui, d.kind.label(), theme::INFO);
                    }
                }
                ui.add(egui::TextEdit::singleline(&mut f.note).hint_text("specifics").desired_width(260.0));
                let default = def.map(|d| d.kind.grants_issue_resolve()).unwrap_or(false);
                let mut on = f.issue_resolve.unwrap_or(default);
                if ui.checkbox(&mut on, "+1 Resolve/issue").on_hover_text("Plot Hooks and Conditions (including Severe versions) grant 1 Resolve at the start of each issue").changed() {
                    f.issue_resolve = if on == default { None } else { Some(on) };
                }
                if ui.small_button("✕").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            c.flaws.remove(i);
        }
    });
}

// ---------------------------------------------------------------- Story

fn story(ui: &mut Ui, doc: &mut Doc, pal: &Palette, dark: bool) {
    let s = &mut doc.ch.story;
    let multi = |ui: &mut Ui, v: &mut String, hint: &str, rows: usize| {
        ui.add(egui::TextEdit::multiline(v).hint_text(hint).desired_rows(rows).desired_width(f32::INFINITY));
    };
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Quote");
        ui.add(egui::TextEdit::singleline(&mut s.quote).hint_text("A catchphrase or one-time zinger").desired_width(f32::INFINITY));
    });
    ui.add_space(8.0);
    ui.columns(2, |cols| {
        theme::card(&mut cols[0], dark, |ui| {
            theme::section_title(ui, "In Costume");
            hint(ui, pal, "Three things people notice first.");
            for (i, v) in s.costume.iter_mut().enumerate() {
                ui.add(egui::TextEdit::singleline(v).hint_text(format!("{}.", i + 1)).desired_width(f32::INFINITY));
            }
        });
        theme::card(&mut cols[1], dark, |ui| {
            theme::section_title(ui, "In Civilian Clothes");
            hint(ui, pal, "Three things people notice first.");
            for (i, v) in s.civilian.iter_mut().enumerate() {
                ui.add(egui::TextEdit::singleline(v).hint_text(format!("{}.", i + 1)).desired_width(f32::INFINITY));
            }
        });
    });
    ui.add_space(8.0);
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Connections");
        hint(ui, pal, "At least three people who matter in the Hero's life.");
        let mut remove = None;
        for (i, cn) in s.connections.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut cn.name).hint_text("Name").desired_width(180.0));
                ui.add(egui::TextEdit::singleline(&mut cn.description).hint_text("Who they are to the Hero").desired_width(ui.available_width() - 40.0));
                if ui.small_button("✕").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            s.connections.remove(i);
        }
        if ui.button("＋ Connection").clicked() {
            s.connections.push(Connection::default());
        }
    });
    ui.add_space(8.0);
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Details");
        let labels = ["About the Hero", "About another Hero in the group", "About someone or something in the world"];
        for (i, v) in s.details.iter_mut().enumerate() {
            ui.label(RichText::new(labels[i]).small().color(pal.muted));
            multi(ui, v, "A plot hook the GM can use", 2);
        }
    });
    ui.add_space(8.0);
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Origin");
        multi(ui, &mut s.origin, "How did they get this way? A sentence or two is plenty.", 4);
        ui.add_space(6.0);
        theme::section_title(ui, "Gear");
        multi(ui, &mut s.gear, "Mundane gear the Hero carries", 2);
        ui.add_space(6.0);
        theme::section_title(ui, "Notes");
        multi(ui, &mut s.notes, "Anything else", 3);
    });
}

// ---------------------------------------------------------------- Advancement

fn advancement(ui: &mut Ui, doc: &mut Doc, st: &mut UiState, pal: &Palette, dark: bool) {
    let s = doc.summary.clone();
    let c = &mut doc.ch;
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Hero Points");
        ui.label(format!(
            "Starting {} HP + earned {} HP = {} HP available. Spent {} HP, {} remaining.",
            s.hp_starting,
            s.hp_earned,
            s.hp_available,
            s.total_spent,
            s.remaining()
        ));
        hint(ui, pal, "Heroes earn about 1 HP every 3 issues, plus 1 for truly noteworthy deeds or major life events. Spend them between stories at character-creation costs; raising a baseline Trait raises Powers built on it.");
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut st.award_amount).range(-50..=50).prefix("HP "));
            ui.add(egui::TextEdit::singleline(&mut st.award_date).hint_text("Issue / date").desired_width(110.0));
            ui.add(egui::TextEdit::singleline(&mut st.award_note).hint_text("Reason").desired_width(260.0));
            if ui.add_enabled(st.award_amount != 0, egui::Button::new("Record award")).clicked() {
                c.awards.push(Award { date: st.award_date.trim().to_string(), amount: st.award_amount, note: st.award_note.trim().to_string() });
                st.award_note.clear();
                st.award_amount = 1;
            }
        });
        if st.award_amount == 0 && c.awards.is_empty() {
            st.award_amount = 1;
        }
        ui.add_space(6.0);
        let mut remove = None;
        egui::Grid::new("awards").num_columns(4).striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
            for (i, a) in c.awards.iter_mut().enumerate() {
                ui.add(egui::DragValue::new(&mut a.amount).range(-50..=50));
                ui.add(egui::TextEdit::singleline(&mut a.date).desired_width(110.0));
                ui.add(egui::TextEdit::singleline(&mut a.note).desired_width(300.0));
                if ui.small_button("✕").clicked() {
                    remove = Some(i);
                }
                ui.end_row();
            }
        });
        if let Some(i) = remove {
            c.awards.remove(i);
        }
    });
    ui.add_space(8.0);
    theme::card(ui, dark, |ui| {
        theme::section_title(ui, "Trait Cap");
        ui.label(format!("Current Trait Cap: {}d ({} mode).", s.trait_cap, c.options.cap_mode.label()));
        hint(ui, pal, "Hard caps never change. Floating caps rise +1d for every 10 HP earned. Soft caps rise when the GM says so (set the bonus on the Overview tab).");
        ui.add_space(4.0);
        ui.checkbox(&mut c.options.creation_complete, "Character creation is complete");
        hint(ui, pal, "Retcons: as long as the total HP stays the same, you may rebuild the Hero with GM approval — just edit the Traits.");
    });
}
