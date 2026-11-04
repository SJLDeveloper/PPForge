//! Application shell: open characters, file operations, dialogs, menus and layout.

use crate::engine::{self, Severity, Summary};
use crate::model::*;
use crate::rules::Ruleset;
use crate::storage::{self, Settings};
use crate::ui::{tabs, theme};
use egui::{Color32, Key, KeyboardShortcut, Modifiers, RichText};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Overview,
    Traits,
    Powers,
    PerksFlaws,
    Story,
    Advancement,
    Play,
}

impl Tab {
    const ALL: [Tab; 7] = [Tab::Overview, Tab::Traits, Tab::Powers, Tab::PerksFlaws, Tab::Story, Tab::Advancement, Tab::Play];
    fn label(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Traits => "Abilities & Talents",
            Tab::Powers => "Powers",
            Tab::PerksFlaws => "Perks & Flaws",
            Tab::Story => "Finishing Touches",
            Tab::Advancement => "Advancement",
            Tab::Play => "Play",
        }
    }
}

/// One open character.
pub struct Doc {
    pub ch: Character,
    pub path: Option<PathBuf>,
    saved: Character,
    prev: Character,
    undo: Vec<Character>,
    redo: Vec<Character>,
    last_push: Instant,
    pub summary: Summary,
}

impl Doc {
    pub fn new(ch: Character, path: Option<PathBuf>) -> Doc {
        let summary = engine::evaluate(&ch);
        Doc {
            saved: if path.is_some() { ch.clone() } else { Character { story: Story { hero_name: "\u{0}".into(), ..Default::default() }, ..Default::default() } },
            prev: ch.clone(),
            ch,
            path,
            undo: Vec::new(),
            redo: Vec::new(),
            last_push: Instant::now() - Duration::from_secs(10),
            summary,
        }
    }
    pub fn dirty(&self) -> bool {
        self.ch != self.saved
    }
    fn mark_saved(&mut self) {
        self.saved = self.ch.clone();
    }
    /// Record an undo point if the character changed this frame (coalescing rapid edits).
    fn track_changes(&mut self) {
        if self.ch != self.prev {
            if self.last_push.elapsed() > Duration::from_millis(700) {
                self.undo.push(self.prev.clone());
                if self.undo.len() > 200 {
                    self.undo.remove(0);
                }
            }
            self.last_push = Instant::now();
            self.redo.clear();
            self.prev = self.ch.clone();
            self.summary = engine::evaluate(&self.ch);
        }
    }
    fn undo(&mut self) {
        if let Some(c) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.ch, c));
            self.prev = self.ch.clone();
            self.summary = engine::evaluate(&self.ch);
        }
    }
    fn redo(&mut self) {
        if let Some(c) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.ch, c));
            self.prev = self.ch.clone();
            self.summary = engine::evaluate(&self.ch);
        }
    }
}

struct Toast {
    text: String,
    color: Color32,
    until: Instant,
}

struct NewDialog {
    name: String,
    ruleset: Ruleset,
    level: PowerLevel,
    char_type: CharType,
    package: Package,
}

enum Pending {
    /// Close a document (ask to save if dirty).
    Close(usize),
    /// Quit the app.
    Quit,
}

pub struct App {
    pub settings: Settings,
    pub docs: Vec<Doc>,
    pub active: usize,
    pub tab: Tab,
    pub ui_state: tabs::UiState,
    toasts: Vec<Toast>,
    new_dialog: Option<NewDialog>,
    convert_dialog: Option<Ruleset>,
    pending: Option<Pending>,
    about_open: bool,
    rules_open: bool,
    library: Vec<(PathBuf, String)>,
    library_scan: Instant,
    allow_close: bool,
    theme_applied: Option<(bool, f32)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> App {
        let settings = Settings::load();
        let mut app = App {
            settings,
            docs: Vec::new(),
            active: 0,
            tab: Tab::Overview,
            ui_state: tabs::UiState::default(),
            toasts: Vec::new(),
            new_dialog: None,
            convert_dialog: None,
            pending: None,
            about_open: false,
            rules_open: false,
            library: Vec::new(),
            library_scan: Instant::now() - Duration::from_secs(60),
            allow_close: false,
            theme_applied: None,
        };
        theme::apply(&cc.egui_ctx, app.settings.dark_mode, app.settings.ui_scale);
        app.theme_applied = Some((app.settings.dark_mode, app.settings.ui_scale));
        // Files passed on the command line (e.g. double-clicked .ppchar files).
        for arg in std::env::args().skip(1) {
            let p = PathBuf::from(arg);
            if p.exists() {
                app.open_path(p);
            }
        }
        if app.docs.is_empty() && app.settings.reopen_last {
            for p in app.settings.last_open.clone() {
                if p.exists() {
                    app.open_path(p);
                }
            }
        }
        if app.docs.is_empty() {
            app.new_dialog = Some(NewDialog::default_for(app.settings.default_ruleset));
        }
        app.active = 0;
        // Developer hook used for automated screenshots: PPFORGE_START_TAB=Powers etc.
        if let Ok(t) = std::env::var("PPFORGE_START_TAB") {
            if let Some(tab) = Tab::ALL.iter().find(|x| format!("{:?}", x).eq_ignore_ascii_case(&t)) {
                app.tab = *tab;
            }
            if std::env::var("PPFORGE_DEMO_ROLL").is_ok() {
                let might = app.docs.first().map(|d| d.ch.abilities[crate::model::MIG].rank).unwrap_or(4);
                app.ui_state.dice.demo(might);
            }
            if std::env::var("PPFORGE_PICKER").is_ok() {
                app.ui_state.picker_open = true;
            }
            app.ui_state.expanded = app.docs.iter().flat_map(|d| d.ch.powers.iter().map(|p| p.uid)).collect();
        }
        app
    }

    fn toast(&mut self, text: impl Into<String>, color: Color32) {
        self.toasts.push(Toast { text: text.into(), color, until: Instant::now() + Duration::from_secs(4) });
    }

    pub fn doc(&mut self) -> Option<&mut Doc> {
        self.docs.get_mut(self.active)
    }

    fn open_path(&mut self, p: PathBuf) {
        if let Some(i) = self.docs.iter().position(|d| d.path.as_ref() == Some(&p)) {
            self.active = i;
            return;
        }
        match storage::load_character(&p) {
            Ok(c) => {
                self.settings.push_recent(&p);
                self.docs.push(Doc::new(c, Some(p)));
                self.active = self.docs.len() - 1;
            }
            Err(e) => self.toast(format!("Could not open {}: {e}", p.display()), theme::BAD),
        }
    }

    fn open_dialog(&mut self) {
        let dir = self.settings.library();
        let files = rfd::FileDialog::new()
            .set_title("Open characters")
            .add_filter("P&P character", &[storage::EXTENSION])
            .add_filter("JSON", &["json"])
            .set_directory(&dir)
            .pick_files();
        for f in files.unwrap_or_default() {
            self.open_path(f);
        }
    }

    /// Save the active document. Unsaved new characters go to the library folder by default.
    fn save(&mut self, idx: usize, save_as: bool) -> bool {
        let lib = self.settings.library();
        let Some(doc) = self.docs.get_mut(idx) else { return false };
        let mut path = doc.path.clone();
        if save_as || path.is_none() {
            let name = format!("{}.{}", storage::safe_file_name(&doc.ch.display_name()), storage::EXTENSION);
            let _ = std::fs::create_dir_all(&lib);
            let picked = rfd::FileDialog::new()
                .set_title("Save character")
                .add_filter("P&P character", &[storage::EXTENSION])
                .set_directory(&lib)
                .set_file_name(&name)
                .save_file();
            match picked {
                Some(mut p) => {
                    if p.extension().is_none() {
                        p.set_extension(storage::EXTENSION);
                    }
                    path = Some(p);
                }
                None => {
                    if save_as {
                        return false;
                    }
                    // No dialog available (or cancelled): fall back to the library folder only for first saves.
                    let mut p = lib.join(&name);
                    let mut n = 2;
                    while p.exists() {
                        p = lib.join(format!("{} ({n}).{}", storage::safe_file_name(&doc.ch.display_name()), storage::EXTENSION));
                        n += 1;
                    }
                    path = Some(p);
                }
            }
        }
        let path = path.unwrap();
        match storage::save_character(&doc.ch, &path) {
            Ok(()) => {
                doc.path = Some(path.clone());
                doc.mark_saved();
                self.settings.push_recent(&path);
                self.settings.save();
                self.library_scan = Instant::now() - Duration::from_secs(60);
                self.toast(format!("Saved {}", path.display()), theme::GOOD);
                true
            }
            Err(e) => {
                self.toast(format!("Save failed: {e}"), theme::BAD);
                false
            }
        }
    }

    fn export_pdf(&mut self) {
        let lib = self.settings.library();
        let Some(doc) = self.docs.get(self.active) else { return };
        let name = format!("{}.pdf", storage::safe_file_name(&doc.ch.display_name()));
        let dir = doc.path.as_ref().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or(lib);
        let _ = std::fs::create_dir_all(&dir);
        let picked = rfd::FileDialog::new()
            .set_title("Export character sheet")
            .add_filter("PDF", &["pdf"])
            .set_directory(&dir)
            .set_file_name(&name)
            .save_file();
        let path = match picked {
            Some(p) => p,
            None => return,
        };
        match crate::pdf::export(&doc.ch, &path) {
            Ok(()) => self.toast(format!("Exported {}", path.display()), theme::GOOD),
            Err(e) => self.toast(format!("PDF export failed: {e}"), theme::BAD),
        }
    }

    fn close_doc(&mut self, idx: usize) {
        if idx < self.docs.len() {
            self.docs.remove(idx);
            // Keep the same character selected when one above it in the list is closed.
            if idx < self.active {
                self.active -= 1;
            }
            if self.active >= self.docs.len() {
                self.active = self.docs.len().saturating_sub(1);
            }
        }
    }

    fn request_close(&mut self, idx: usize) {
        if self.docs.get(idx).map(|d| d.dirty()).unwrap_or(false) {
            self.pending = Some(Pending::Close(idx));
        } else {
            self.close_doc(idx);
        }
    }

    fn remember_open(&mut self) {
        self.settings.last_open = self.docs.iter().filter_map(|d| d.path.clone()).collect();
        self.settings.save();
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let sc = |m: Modifiers, k: Key| KeyboardShortcut::new(m, k);
        let shift_cmd = Modifiers::COMMAND | Modifiers::SHIFT;
        let act = |s: KeyboardShortcut| ctx.input_mut(|i| i.consume_shortcut(&s));
        if act(sc(shift_cmd, Key::S)) {
            self.save(self.active, true);
        }
        if act(sc(Modifiers::COMMAND, Key::S)) {
            self.save(self.active, false);
        }
        if act(sc(Modifiers::COMMAND, Key::O)) {
            self.open_dialog();
        }
        if act(sc(Modifiers::COMMAND, Key::N)) {
            self.new_dialog = Some(NewDialog::default_for(self.settings.default_ruleset));
        }
        if act(sc(Modifiers::COMMAND, Key::P)) {
            self.export_pdf();
        }
        if act(sc(Modifiers::COMMAND, Key::W)) {
            self.request_close(self.active);
        }
        if (act(sc(shift_cmd, Key::Z)) || act(sc(Modifiers::COMMAND, Key::Y)))
            && let Some(d) = self.doc() {
                d.redo();
            }
        if act(sc(Modifiers::COMMAND, Key::Z))
            && let Some(d) = self.doc() {
                d.undo();
            }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New character…   Ctrl+N").clicked() {
                    self.new_dialog = Some(NewDialog::default_for(self.settings.default_ruleset));
                    ui.close();
                }
                if ui.button("Open…   Ctrl+O").clicked() {
                    ui.close();
                    self.open_dialog();
                }
                ui.menu_button("Open recent", |ui| {
                    if self.settings.recent.is_empty() {
                        ui.label(RichText::new("No recent files").weak());
                    }
                    for p in self.settings.recent.clone() {
                        let label = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                        if ui.button(label).on_hover_text(p.display().to_string()).clicked() {
                            ui.close();
                            self.open_path(p);
                        }
                    }
                });
                ui.separator();
                let has = !self.docs.is_empty();
                if ui.add_enabled(has, egui::Button::new("Save   Ctrl+S")).clicked() {
                    ui.close();
                    self.save(self.active, false);
                }
                if ui.add_enabled(has, egui::Button::new("Save as…   Ctrl+Shift+S")).clicked() {
                    ui.close();
                    self.save(self.active, true);
                }
                if ui.add_enabled(has, egui::Button::new("Duplicate character")).clicked() {
                    ui.close();
                    if let Some(d) = self.docs.get(self.active) {
                        let mut c = d.ch.clone();
                        c.story.hero_name = format!("{} (copy)", c.display_name());
                        self.docs.push(Doc::new(c, None));
                        self.active = self.docs.len() - 1;
                    }
                }
                ui.separator();
                if ui.add_enabled(has, egui::Button::new("Export PDF sheet…   Ctrl+P")).clicked() {
                    ui.close();
                    self.export_pdf();
                }
                ui.separator();
                if ui.add_enabled(has, egui::Button::new("Close   Ctrl+W")).clicked() {
                    ui.close();
                    self.request_close(self.active);
                }
                if ui.button("Quit").clicked() {
                    ui.close();
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Edit", |ui| {
                let (u, r) = self.docs.get(self.active).map(|d| (!d.undo.is_empty(), !d.redo.is_empty())).unwrap_or((false, false));
                if ui.add_enabled(u, egui::Button::new("Undo   Ctrl+Z")).clicked() {
                    if let Some(d) = self.doc() {
                        d.undo();
                    }
                    ui.close();
                }
                if ui.add_enabled(r, egui::Button::new("Redo   Ctrl+Y")).clicked() {
                    if let Some(d) = self.doc() {
                        d.redo();
                    }
                    ui.close();
                }
                ui.separator();
                if ui.add_enabled(!self.docs.is_empty(), egui::Button::new("Convert ruleset…")).clicked() {
                    if let Some(d) = self.docs.get(self.active) {
                        self.convert_dialog = Some(match d.ch.ruleset {
                            Ruleset::Core => Ruleset::Errata,
                            Ruleset::Errata => Ruleset::Core,
                        });
                    }
                    ui.close();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut self.settings.dark_mode, "Dark mode").changed() {
                    self.settings.save();
                }
                ui.horizontal(|ui| {
                    ui.label("UI scale");
                    if ui.add(egui::Slider::new(&mut self.settings.ui_scale, 0.75..=1.75).step_by(0.05)).changed() {
                        self.settings.save();
                    }
                });
                ui.separator();
                ui.label("Default ruleset for new characters");
                for r in Ruleset::ALL {
                    if ui.radio_value(&mut self.settings.default_ruleset, r, r.label()).changed() {
                        self.settings.save();
                    }
                }
                ui.checkbox(&mut self.settings.reopen_last, "Reopen characters on launch");
                ui.separator();
                if ui.button("Change library folder…").clicked() {
                    ui.close();
                    if let Some(d) = rfd::FileDialog::new().set_directory(self.settings.library()).pick_folder() {
                        self.settings.library_dir = Some(d);
                        self.settings.save();
                        self.library_scan = Instant::now() - Duration::from_secs(60);
                    }
                }
                ui.label(RichText::new(self.settings.library().display().to_string()).small().weak());
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Ruleset differences").clicked() {
                    self.rules_open = true;
                    ui.close();
                }
                if ui.button("About PP Forge").clicked() {
                    self.about_open = true;
                    ui.close();
                }
            });
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let dark = self.settings.dark_mode;
        let pal = theme::palette(dark);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("PP FORGE").size(18.0).strong().color(theme::ACCENT));
        });
        ui.label(RichText::new("Prowlers & Paragons UE").small().color(pal.muted));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("＋ New").clicked() {
                self.new_dialog = Some(NewDialog::default_for(self.settings.default_ruleset));
            }
            if ui.button("Open…").clicked() {
                self.open_dialog();
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new("OPEN CHARACTERS").small().strong().color(pal.muted));
        let mut close: Option<usize> = None;
        let mut select: Option<usize> = None;
        egui::ScrollArea::vertical().id_salt("open_docs").max_height(ui.available_height() * 0.55).show(ui, |ui| {
            for (i, d) in self.docs.iter().enumerate() {
                let selected = i == self.active;
                let fill = if selected { pal.card_alt } else { pal.card };
                let stroke = if selected { egui::Stroke::new(1.5, theme::ACCENT) } else { egui::Stroke::new(1.0, pal.border) };
                // The card's click area is registered before its contents (UiBuilder::sense), so
                // the ✕ button inside it gets its own clicks instead of the card swallowing them.
                let card = ui.scope_builder(egui::UiBuilder::new().sense(egui::Sense::click()), |ui| {
                    // Text inside the card shouldn't be selectable, or it would catch clicks meant for the card.
                    ui.style_mut().interaction.selectable_labels = false;
                    egui::Frame::new()
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(8)
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let name = if d.dirty() { format!("● {}", d.ch.display_name()) } else { d.ch.display_name() };
                            ui.label(RichText::new(name).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").on_hover_text("Close").clicked() {
                                    close = Some(i);
                                }
                            });
                        });
                        ui.horizontal(|ui| {
                            theme::badge(ui, d.ch.ruleset.short(), theme::ruleset_color(d.ch.ruleset));
                            ui.label(RichText::new(d.ch.char_type.label()).small().color(pal.muted));
                            if d.ch.char_type.uses_budget() {
                                let rem = d.summary.remaining();
                                let c = if rem < 0 { theme::BAD } else if rem == 0 { theme::GOOD } else { pal.muted };
                                ui.label(RichText::new(format!("{} HP left", rem)).small().color(c));
                            }
                        });
                    });
                });
                if card.response.clicked() && close != Some(i) {
                    select = Some(i);
                }
                ui.add_space(4.0);
            }
        });
        if let Some(i) = select {
            self.active = i;
        }
        if let Some(i) = close {
            self.request_close(i);
        }

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new("LIBRARY").small().strong().color(pal.muted));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("⟳").on_hover_text("Refresh").clicked() {
                    self.library_scan = Instant::now() - Duration::from_secs(60);
                }
            });
        });
        if self.library_scan.elapsed() > Duration::from_secs(5) {
            self.library = storage::list_library(&self.settings.library());
            self.library_scan = Instant::now();
        }
        let mut open: Option<PathBuf> = None;
        egui::ScrollArea::vertical().id_salt("library").show(ui, |ui| {
            if self.library.is_empty() {
                ui.label(RichText::new("Saved characters appear here.").small().color(pal.muted));
            }
            for (p, name) in &self.library {
                let is_open = self.docs.iter().any(|d| d.path.as_ref() == Some(p));
                let txt = if is_open { RichText::new(format!("{name}  ✓")).color(pal.muted) } else { RichText::new(name.clone()) };
                if ui.add(egui::Button::new(txt).frame(false)).on_hover_text(p.display().to_string()).clicked() {
                    open = Some(p.clone());
                }
            }
        });
        if let Some(p) = open {
            self.open_path(p);
        }
    }

    fn summary_panel(&mut self, ui: &mut egui::Ui) {
        let dark = self.settings.dark_mode;
        let pal = theme::palette(dark);
        let Some(doc) = self.docs.get(self.active) else { return };
        let s = &doc.summary;
        let c = &doc.ch;
        let mut action = 0u8;
        ui.add_space(8.0);
        egui::ScrollArea::vertical().id_salt("summary_scroll").auto_shrink([false, false]).show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            theme::section_title(ui, "Build Summary");
            // Stat tiles.
            let mut tiles: Vec<(&str, String, Color32)> =
                vec![("EDGE", s.edge.to_string(), pal.text), ("HEALTH", s.health.to_string(), pal.text)];
            if let Some(r) = s.resolve {
                tiles.push(("RESOLVE", r.to_string(), pal.text));
            }
            tiles.push(("CAP", format!("{}d", s.trait_cap), pal.muted));
            let n = tiles.len();
            ui.columns(n, |cols| {
                for (i, (label, value, color)) in tiles.into_iter().enumerate() {
                    egui::Frame::new().fill(pal.card_alt).corner_radius(8).inner_margin(egui::Margin::symmetric(4, 6)).show(&mut cols[i], |ui| {
                        ui.set_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(value).size(20.0).strong().color(color));
                            ui.label(RichText::new(label).size(10.0).color(pal.muted));
                        });
                    });
                }
            });
            ui.add_space(10.0);
            if c.char_type.uses_budget() {
                let frac = if s.hp_available > 0 { s.total_spent as f32 / s.hp_available as f32 } else { 0.0 };
                let color = if s.total_spent > s.hp_available { theme::BAD } else if s.total_spent == s.hp_available { theme::GOOD } else { theme::ACCENT };
                ui.label(RichText::new("HERO POINTS").small().strong().color(pal.muted));
                ui.add(egui::ProgressBar::new(frac.min(1.0)).fill(color).text(format!("{} / {} spent", s.total_spent, s.hp_available)));
                ui.add_space(4.0);
                egui::Grid::new("cost_grid").num_columns(2).spacing([16.0, 2.0]).show(ui, |ui| {
                    let mut row = |l: &str, v: i32| {
                        ui.label(RichText::new(l).color(pal.muted));
                        ui.label(v.to_string());
                        ui.end_row();
                    };
                    if s.cost_package > 0 {
                        row("Package", s.cost_package);
                    }
                    row("Abilities", s.cost_abilities);
                    row("Talents", s.cost_talents);
                    row("Powers", s.cost_powers);
                    row("Perks", s.cost_perks);
                    if s.cost_flaws > 0 {
                        row("Extra Flaws", s.cost_flaws);
                    }
                    ui.label(RichText::new("Remaining").strong());
                    ui.label(RichText::new(s.remaining().to_string()).strong().color(color));
                    ui.end_row();
                });
            } else {
                ui.label(RichText::new(format!("{} HP of Traits (NPCs don't use a budget)", s.total_spent)).color(pal.muted));
            }
            if s.resolve.is_some() {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!(
                        "Resolve {} = {} (highest rank {}d: {}) + {} (Determination, Plot Hooks, Conditions)",
                        s.resolve.unwrap_or(0),
                        s.resolve_base,
                        s.highest_rank,
                        if s.highest_from.is_empty() { "—" } else { &s.highest_from },
                        s.resolve_bonus
                    ))
                    .small()
                    .color(pal.muted),
                );
            }
            ui.add_space(12.0);
            theme::section_title(ui, "Checks");
            if s.issues.is_empty() {
                ui.label(RichText::new("✔ Everything checks out").color(theme::GOOD));
            }
            for i in &s.issues {
                let (icon, color) = match i.severity {
                    Severity::Error => ("⛔", theme::BAD),
                    Severity::Warning => ("⚠", theme::WARN),
                    Severity::Info => ("ℹ", theme::INFO),
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(icon).color(color));
                    ui.label(RichText::new(&i.message).size(12.5));
                });
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Export PDF").clicked() {
                    action = 1;
                }
                if ui.button("Save").clicked() {
                    action = 2;
                }
            });
        });
        match action {
            1 => self.export_pdf(),
            2 => {
                self.save(self.active, false);
            }
            _ => {}
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        // New character.
        let mut create: Option<Character> = None;
        let mut cancel = false;
        if let Some(nd) = &mut self.new_dialog {
            let modal = egui::Modal::new(egui::Id::unique("new_char")).show(ctx, |ui| {
                ui.set_width(420.0);
                ui.heading("New character");
                ui.add_space(6.0);
                egui::Grid::new("new_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label("Name");
                    ui.add(egui::TextEdit::singleline(&mut nd.name).hint_text("Hero name").desired_width(260.0));
                    ui.end_row();
                    ui.label("Rules");
                    ui.vertical(|ui| {
                        for r in Ruleset::ALL {
                            ui.radio_value(&mut nd.ruleset, r, r.label());
                        }
                    });
                    ui.end_row();
                    ui.label("Type");
                    ui.horizontal(|ui| {
                        for t in CharType::ALL {
                            ui.selectable_value(&mut nd.char_type, t, t.label());
                        }
                    });
                    ui.end_row();
                    ui.label("Power level");
                    egui::ComboBox::from_id_salt("nd_pl").selected_text(format!(
                        "{} ({} HP, {}d cap)",
                        nd.level.name(nd.ruleset),
                        nd.level.hero_points(),
                        nd.level.trait_cap()
                    )).width(260.0).show_ui(ui, |ui| {
                        for pl in PowerLevel::ALL {
                            ui.selectable_value(&mut nd.level, pl, format!("{} ({} HP, {}d cap)", pl.name(nd.ruleset), pl.hero_points(), pl.trait_cap()));
                        }
                    });
                    ui.end_row();
                    ui.label("Package");
                    egui::ComboBox::from_id_salt("nd_pkg").selected_text(nd.package.label()).width(260.0).show_ui(ui, |ui| {
                        for p in Package::ALL {
                            ui.selectable_value(&mut nd.package, p, p.label());
                        }
                    });
                    ui.end_row();
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("Create").strong()).fill(theme::ACCENT_DIM)).clicked() {
                        let mut c = Character::new(nd.ruleset);
                        c.story.hero_name = if nd.name.trim().is_empty() { "New Hero".into() } else { nd.name.trim().to_string() };
                        c.char_type = nd.char_type;
                        c.power_level = nd.level;
                        c.apply_package(nd.package);
                        create = Some(c);
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
            if modal.should_close() {
                cancel = true;
            }
        }
        if let Some(c) = create {
            self.docs.push(Doc::new(c, None));
            self.active = self.docs.len() - 1;
            self.tab = Tab::Overview;
            self.new_dialog = None;
        } else if cancel {
            self.new_dialog = None;
        }

        // Ruleset conversion.
        if let Some(target) = self.convert_dialog {
            let mut done: Option<bool> = None;
            let name = self.docs.get(self.active).map(|d| d.ch.display_name()).unwrap_or_default();
            egui::Modal::new(egui::Id::unique("convert")).show(ctx, |ui| {
                ui.set_width(440.0);
                ui.heading("Convert ruleset");
                ui.label(format!("Convert {name} to \"{}\"?", target.label()));
                ui.label(
                    RichText::new(
                        "Renamed Powers (e.g. Evasion ↔ Dodge, Power Absorption ↔ Absorption) are mapped automatically. \
                         Powers with no equivalent are flagged so you can replace them. Costs are recalculated under the new rules.",
                    )
                    .small(),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Convert").clicked() {
                        done = Some(true);
                    }
                    if ui.button("Convert a copy").clicked() {
                        done = Some(false);
                    }
                    if ui.button("Cancel").clicked() {
                        self.convert_dialog = None;
                    }
                });
            });
            if let Some(in_place) = done {
                if in_place {
                    if let Some(d) = self.doc() {
                        let missing = d.ch.convert_to(target);
                        if !missing.is_empty() {
                            let msg = format!("No equivalent for: {}", missing.join(", "));
                            self.toast(msg, theme::WARN);
                        }
                    }
                } else if let Some(d) = self.docs.get(self.active) {
                    let mut c = d.ch.clone();
                    let missing = c.convert_to(target);
                    c.story.hero_name = format!("{} ({})", c.display_name(), target.short().to_lowercase());
                    self.docs.push(Doc::new(c, None));
                    self.active = self.docs.len() - 1;
                    if !missing.is_empty() {
                        self.toast(format!("No equivalent for: {}", missing.join(", ")), theme::WARN);
                    }
                }
                self.convert_dialog = None;
            }
        }

        // Unsaved changes.
        if let Some(p) = &self.pending {
            let (title, idx) = match p {
                Pending::Close(i) => ("Close character?", Some(*i)),
                Pending::Quit => ("Quit PP Forge?", None),
            };
            let dirty: Vec<String> = match idx {
                Some(i) => self.docs.get(i).map(|d| vec![d.ch.display_name()]).unwrap_or_default(),
                None => self.docs.iter().filter(|d| d.dirty()).map(|d| d.ch.display_name()).collect(),
            };
            let mut choice: Option<u8> = None;
            egui::Modal::new(egui::Id::unique("unsaved")).show(ctx, |ui| {
                ui.set_width(400.0);
                ui.heading(title);
                ui.label(format!("Unsaved changes: {}", dirty.join(", ")));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        choice = Some(0);
                    }
                    if ui.button("Discard").clicked() {
                        choice = Some(1);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(2);
                    }
                });
            });
            match (choice, idx) {
                (Some(0), Some(i)) => {
                    if self.save(i, false) {
                        self.close_doc(i);
                    }
                    self.pending = None;
                }
                (Some(1), Some(i)) => {
                    self.close_doc(i);
                    self.pending = None;
                }
                (Some(0), None) => {
                    let mut ok = true;
                    for i in 0..self.docs.len() {
                        if self.docs[i].dirty() {
                            ok &= self.save(i, false);
                        }
                    }
                    self.pending = None;
                    if ok {
                        self.allow_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                (Some(1), None) => {
                    self.pending = None;
                    self.allow_close = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                (Some(_), _) => self.pending = None,
                _ => {}
            }
        }

        if self.about_open {
            egui::Window::new("About PP Forge").collapsible(false).resizable(false).open(&mut self.about_open).show(ctx, |ui| {
                ui.heading("PP Forge");
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.label("A character builder and manager for Prowlers & Paragons Ultimate Edition.");
                ui.label("Supports the core rules as printed, or the core rules with the Combined Updates errata.");
                ui.add_space(6.0);
                ui.label(RichText::new("Prowlers & Paragons is a trademark of its publisher. This is an unofficial fan tool; you need the rulebook to play.").small().weak());
            });
        }
        if self.rules_open {
            egui::Window::new("Ruleset differences").default_width(560.0).open(&mut self.rules_open).show(ctx, |ui| {
                egui::ScrollArea::vertical().max_height(480.0).show(ui, |ui| {
                    ui.label(RichText::new(tabs::RULES_NOTES).size(13.0));
                });
            });
        }

        // Toasts.
        self.toasts.retain(|t| t.until > Instant::now());
        if !self.toasts.is_empty() {
            egui::Area::new(egui::Id::unique("toasts")).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0]).show(ctx, |ui| {
                for t in &self.toasts {
                    egui::Frame::new().fill(theme::palette(self.settings.dark_mode).card).stroke(egui::Stroke::new(1.5, t.color)).corner_radius(8).inner_margin(egui::Margin::symmetric(14, 8)).show(ui, |ui| {
                        ui.label(RichText::new(&t.text).color(t.color));
                    });
                    ui.add_space(4.0);
                }
            });
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }
}

impl NewDialog {
    fn default_for(r: Ruleset) -> NewDialog {
        NewDialog { name: String::new(), ruleset: r, level: PowerLevel::Standard, char_type: CharType::Hero, package: Package::None }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.theme_applied != Some((self.settings.dark_mode, self.settings.ui_scale)) {
            theme::apply(ctx, self.settings.dark_mode, self.settings.ui_scale);
            self.theme_applied = Some((self.settings.dark_mode, self.settings.ui_scale));
        }
        // Intercept window close when there are unsaved changes.
        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.allow_close && self.docs.iter().any(|d| d.dirty()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.pending = Some(Pending::Quit);
            } else {
                self.remember_open();
            }
        }
        // Dropped files.
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        for p in dropped {
            self.open_path(p);
        }
        self.shortcuts(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let dark = self.settings.dark_mode;
        let pal = theme::palette(dark);

        egui::Panel::top("menu").frame(egui::Frame::new().fill(pal.bg).inner_margin(egui::Margin::symmetric(8, 4))).show(ui, |ui| {
            self.menu_bar(ui);
        });
        egui::Panel::left("sidebar")
            .resizable(true)
            .default_size(230.0)
            .size_range(180.0..=360.0)
            .frame(egui::Frame::new().fill(pal.bg).inner_margin(egui::Margin::same(10)))
            .show(ui, |ui| self.sidebar(ui));
        if !self.docs.is_empty() && self.tab != Tab::Play {
            egui::Panel::right("summary")
                .resizable(true)
                .default_size(300.0)
                .size_range(240.0..=460.0)
                .frame(egui::Frame::new().fill(pal.panel).inner_margin(egui::Margin::same(12)))
                .show(ui, |ui| self.summary_panel(ui));
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(pal.panel).inner_margin(egui::Margin::same(14))).show(ui, |ui| {
            if self.docs.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.3);
                    ui.label(RichText::new("No character open").size(22.0).strong());
                    ui.add_space(8.0);
                    if ui.button("Create a character").clicked() {
                        self.new_dialog = Some(NewDialog::default_for(self.settings.default_ruleset));
                    }
                    if ui.button("Open a saved character").clicked() {
                        self.open_dialog();
                    }
                });
                return;
            }
            if self.active >= self.docs.len() {
                self.active = self.docs.len() - 1;
            }
            // Header with name and tab strip.
            let ruleset = self.docs[self.active].ch.ruleset;
            ui.horizontal(|ui| {
                ui.label(RichText::new(self.docs[self.active].ch.display_name()).size(24.0).strong());
                theme::badge(ui, ruleset.label(), theme::ruleset_color(ruleset));
                if self.docs[self.active].dirty() {
                    ui.label(RichText::new("unsaved").small().color(theme::WARN));
                }
            });
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for t in Tab::ALL {
                    let selected = self.tab == t;
                    let text = RichText::new(t.label()).strong().color(if selected { Color32::WHITE } else { pal.muted });
                    let b = egui::Button::new(text).fill(if selected { theme::ACCENT_DIM } else { Color32::TRANSPARENT }).corner_radius(14);
                    if ui.add(b).clicked() {
                        self.tab = t;
                    }
                }
            });
            ui.add_space(8.0);
            let tab = self.tab;
            let idx = self.active;
            let doc = &mut self.docs[idx];
            let width = ui.available_width();
            egui::ScrollArea::vertical().id_salt(("tab", idx, tab as u8)).auto_shrink([false, false]).show(ui, |ui| {
                ui.set_max_width(width - 14.0);
                tabs::show(ui, tab, doc, &mut self.ui_state, dark);
            });
            doc.track_changes();
        });

        self.dialogs(&ctx);
    }

}
