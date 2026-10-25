# PP Forge

A desktop character builder and manager for **Prowlers & Paragons Ultimate Edition**, written in Rust.
Each character is built under one of two rule sets, chosen per character:

- **Core Rules (UE)** — the rulebook as printed.
- **Core + Combined Errata** — the rulebook with the *Combined Updates* applied (Roles, the revised Power list and costs, reworked Pros & Cons).

It runs on Windows, macOS and Linux from the same code, with no runtime files to install: the rules data is compiled into the binary and PDF sheets use the PDF built-in fonts.

## Features

- **Multiple characters open at once**, listed in the sidebar with ruleset badge, Hero Points left and unsaved-changes marker.
- **Live rules engine**: Hero Point totals, Character Packages, baseline-rank Powers, Pros & Cons (power-specific and generic), Trait Cap checks, Edge, Health and starting Resolve (including Determination, Plot Hooks and Conditions), recalculated on every edit.
- **Validation panel** that flags overspending, Traits over the cap, missing Flaws, incompatible Pros (Armor Piercing + Penetrating), Gestalt without Alter Ego, Powers that don't exist in the chosen ruleset, and more.
- **Heroes, Villains, Foes and Extras** (NPCs skip the Hero Point budget; Foes get half Health).
- **Ruleset conversion** (Edit ▸ Convert ruleset): renamed Powers such as Evasion ↔ Dodge are mapped and anything without an equivalent is flagged. You can convert in place or make a converted copy.
- **Custom Powers, Perks and Flaws**, plus GM cost overrides on any Power (Rule Uno).
- **Advancement log** for Hero Point awards, with hard, floating and soft Trait Caps.
- **Play tab**: a clickable character sheet on the left (Edge, Health and Resolve trackers, Abilities, Talents, Powers, Flaws and Perks) and a Challenge Roll dice roller on the right. Click any Trait to load its rank into the roller. You can then pick a static Threshold (Easy to Godlike) or an opposed roll, add a bonus or penalty, spend Resolve for extra dice, take the multiple-action penalty, use half rank, apply the Role bonus (with its Resolve-paid exploding 6s), declare a Defining Moment (double rank, 6s explode), turn on "checking your swing", or take automatic successes. Results show each die, the Margin, the narrative and traditional outcomes, and the damage or special-effect duration if the roll was an attack. Rerolls cost 1 Resolve and keep the better result. There's also a short roll history, a button to copy a roll into the session log, a one-click "+1 Resolve" when a Flaw causes trouble, and a New issue reset.
- **Save/Load** as `.ppchar` (pretty-printed JSON). Saves are atomic, so a crash can't corrupt a file. Files can be opened by drag-and-drop, from the command line, or from the library list in the sidebar.
- **PDF character sheets** (US Letter, multi-page as needed) from File ▸ Export PDF, or headless: `pp_forge --pdf hero.ppchar hero.pdf`.
- Undo/redo, dark and light themes, adjustable UI scale, and a warning before closing with unsaved work.

Keyboard: `Ctrl/⌘+N` new, `O` open, `S` save, `Shift+S` save as, `P` export PDF, `W` close, `Z` undo, `Y` or `Shift+Z` redo.

## Building

You need Rust 1.95 or newer (install it from https://rustup.rs).

```sh
cargo build --release
./target/release/pp_forge          # Windows: target\release\pp_forge.exe
```

Platform notes:

- **Windows / macOS**: nothing else is needed.
- **Linux**: builds with no extra -dev packages. At runtime the app needs an X11 or Wayland session. File dialogs use the XDG desktop portal (`xdg-desktop-portal` with a GTK or KDE backend, which standard desktops already have). Without a portal, Save writes to the library folder and Open works from the sidebar library list.
- The app tries the wgpu renderer first and falls back to OpenGL automatically, so it also runs on older GPUs and in VMs.

### Building a Windows .exe on Linux

1. Install the MinGW-w64 cross-linker:
   - Debian/Ubuntu: `sudo apt install gcc-mingw-w64-x86-64`
   - Fedora: `sudo dnf install mingw64-gcc`
   - Arch: `sudo pacman -S mingw-w64-gcc`
2. Add the Windows target: `rustup target add x86_64-pc-windows-gnu`
3. Build: `cargo build --release --target x86_64-pc-windows-gnu`

The result is `target/x86_64-pc-windows-gnu/release/pp_forge.exe`. It's a single file: copy it to any 64-bit Windows 10 or 11 PC and run it. `.cargo/config.toml` already points Cargo at the MinGW linker.

`.github/workflows/build.yml` builds, tests and lints on Linux, Windows, macOS (Apple Silicon) and macOS (Intel), and uploads each binary as an artifact.

### Where files go

- Character library: `Documents/PP Forge/` by default (View ▸ Change library folder).
- Settings: the OS config directory (`%APPDATA%\PPForge\PP Forge\config` on Windows, `~/Library/Application Support/org.PPForge.PP-Forge` on macOS, `~/.config/ppforge` on Linux).

## Project layout

```
src/
  main.rs      entry point, renderer fallback, --pdf CLI
  rules.rs     rules data types, embedded JSON, per-ruleset config (renames, Roles, cap rules)
  model.rs     Character and save format (serde)
  engine.rs    costs, derived stats, validation (unit-tested against book examples)
  storage.rs   save/load, settings, library listing
  pdf.rs       PDF sheet renderer
  app.rs       window, sidebar, summary panel, menus, dialogs
  ui/tabs.rs   Overview, Abilities & Talents, Powers, Perks & Flaws, Finishing Touches, Advancement
  ui/play.rs   Play tab: clickable sheet + Challenge Roll dice roller (unit-tested)
  ui/theme.rs  visual theme
data/
  core_rules.json     Powers, Pros & Cons, Perks and Flaws from the core book
  errata_rules.json   the same for the Combined Updates
tools/
  build_core_rules.py / build_errata_rules.py   regenerate the data files (run from the repo root)
  SCHEMA.md                                     data format reference
```

Run `cargo test` to check the engine. The tests include the core book's Chapter 2 walkthrough (a Low Level Hero that totals exactly 100 Hero Points) and a Psi Lance-style sample whose Health, Edge and Resolve should match the book.

## Rules interpretations

Where the books are ambiguous, the app makes the following calls. Each one can be overridden per character.

- **Minimum cost**: a Power never costs less than 1 HP. A Power priced at "1 HP per 2 ranks" never costs less than 1 HP per 2 ranks, so flat Cons can't reduce it further. Powers the book lists at 0 HP (such as Specialty) stay at 0. When the minimum applies, the Power shows a **min** tag.
- **Half points**: costs are tracked in half Hero Points and rounded up once per Power. For example, errata Summoning at 5d Threat costs 2.5 HP per rank.
- **Health** is rounded up: (Toughness + the higher of Might or Willpower) ÷ 2. This matches the book's sample characters.
- **Edge**: the errata's Chapter 2 says Perception plus the higher of Agility or Intellect, while its Chapter 4 says Agility or Willpower. The app uses Chapter 2 by default; a per-character option switches to the Chapter 4 wording. Danger Sense replaces Perception, Lightning Reflexes adds 6, and Super Speed gives an Edge of at least its rank × 3.
- **Resolve** counts the highest Ability or Power rank, skipping Powers each ruleset says to ignore. Each Power has a "counts toward Resolve" override for GM rulings.
- **Severe Flaws** (Severe Compulsion, Very Heavy, and so on) are a toggle on each Flaw: tick "+1 Resolve/issue" when the severe, Condition version applies.

## Data and copyright

The JSON data holds game mechanics: names, ranges, ranks, costs and modifier values. The descriptions are short paraphrases, not rulebook text. You still need the books to play. This is an unofficial fan tool.
