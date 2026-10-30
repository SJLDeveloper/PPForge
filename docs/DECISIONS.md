# Design decisions

A short log of significant choices: what was decided, why, and what was traded away. Add an entry when you make a choice that a future contributor might otherwise question or undo. Newest last.

---

### D1. Rust with egui/eframe for a single cross-platform GUI codebase
**Decision:** write the app in Rust with egui (immediate-mode GUI) on eframe.
**Why:** one codebase compiles to a native executable for Windows, macOS and Linux with no runtime, installer or web engine. Immediate mode keeps the state model simple: the UI is a function of the `Character`.
**Trade-offs:** egui doesn't look like a native platform UI; accessibility is via AccessKit but less mature than native toolkits; binaries are 20 MB or more.

### D2. Rules as data, compiled into the binary
**Decision:** Powers, Pros & Cons, Perks and Flaws live in JSON per ruleset (`data/*.json`), embedded with `include_str!`.
**Why:** the books are mostly tables, and data is easier to review against them than code. Embedding means there are no runtime files to lose, and every copy of a given version behaves identically.
**Trade-offs:** fixing data needs a rebuild and a release (no user-editable rule packs yet). Rules expressed in prose rather than tables still need code (`RulesetConfig`, the engine).

### D3. Two rulesets side by side, chosen per character
**Decision:** core and errata are separate, complete datasets. Each character records its ruleset, and conversion is explicit (Edit ▸ Convert ruleset), mapping renamed Powers.
**Why:** groups play with either. Characters must keep the rules they were built under, and a silent conversion would change their costs.
**Trade-offs:** some duplication between the two data files.

### D4. Characters store references by name; the engine recomputes everything
**Decision:** saved characters store what was chosen (Power name, ranks, Pros/Cons), never computed costs or stats.
**Why:** data fixes apply to existing characters automatically, and there's no stale derived state.
**Trade-offs:** renaming an entry in the data breaks old saves unless a rename mapping is added.

### D5. Costs in half Hero Points, rounded once per Power
**Decision:** the engine adds up costs in half-points and rounds up at the end of each Power.
**Why:** "1 HP per 2 ranks" bases, "+1 per 2 ranks" Pros and the errata's Threat/2 Summoning combine exactly, with no compounding rounding errors.

### D6. Inform, don't enforce (Rule Uno)
**Decision:** validation produces Errors, Warnings and Info notes, but never blocks editing or saving. GM overrides exist for cost, Edge, Health, Resolve, starting HP, Trait Cap and Resolve classification.
**Why:** the books give the GM the final say, and tables house-rule constantly.

### D7. JSON save files with lenient loading
**Decision:** `.ppchar` files are pretty-printed JSON with `format_version`, every field defaults when missing, and saves are atomic (temp file, then rename).
**Why:** files are human-readable and easy to diff or recover, old files keep working, and a crash can't corrupt a save.

### D8. PDF built-in fonts with our own text measurement
**Decision:** character sheets use the PDF built-in Helvetica fonts, with character widths hard-coded for wrapping.
**Why:** there are no font files to bundle or find, PDFs are small, and output is identical on every OS.
**Trade-offs:** only the WinAnsi character set; other characters are replaced.

### D9. wgpu first, OpenGL fallback
**Decision:** start with the wgpu renderer, and on failure restart with glow (OpenGL).
**Why:** wgpu is the modern default, and the fallback keeps the app working on old GPUs, VMs and remote desktops.

### D10. CI builds every platform; releases come from version tags
**Decision:** GitHub Actions builds, tests and lints on four runners for every push and pull request. Pushing a `v*` tag publishes a Release, but only if every platform passed and the SBOM is current.
**Why:** the maintainer develops on Linux, and CI is the only real test of the Windows and macOS builds.
**Trade-offs:** builds aren't code-signed (see Open items in DESIGN.md).

### D11. SBOM: CycloneDX per platform, normalized, committed, and checked in CI
**Decision:** `tools/sbom.py` runs `cargo-cyclonedx` (pinned) for each release target, normalizes the output (no timestamp, a serial number derived from the inputs, no absolute paths, sorted lists), adds the embedded rules data as `data` components, and writes `sbom/*.cdx.json` and `docs/SBOM.md`. CI fails if the committed files are stale, and releases attach them.
**Why:** dependencies differ by OS, so one SBOM would be wrong. Committed, reproducible files make dependency changes visible in pull request diffs. The rules data ships inside the binary, so it belongs in the bill of materials.
**Trade-offs:** any `Cargo.lock` or data change needs an SBOM regeneration in the same pull request (CI points this out).

### D12. Changes go through pull requests; commits use the noreply identity
**Decision:** work happens on branches and merges into `main` through pull requests. Commits are authored as `SJLDeveloper <66124490+SJLDeveloper@users.noreply.github.com>`.
**Why:** CI runs on every pull request before merging. GitHub rejects pushes that would publish the owner's private email address (GH007).
