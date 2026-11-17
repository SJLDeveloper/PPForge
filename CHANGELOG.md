# Changelog

All notable changes to PP Forge. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses [semantic versioning](https://semver.org/) (see *Versioning* in the README).

Releases v0.1.0 to v0.1.2 were tagged while `Cargo.toml` still said 0.1.0, so those builds report "Version 0.1.0" in Help ▸ About. From v0.1.3 on, the tag and the app version always match.

## [Unreleased]

## [0.1.3] - 2026-11-17

### Added
- Design docs: `docs/DESIGN.md`, `docs/RULES_DATA.md` and `docs/DECISIONS.md`.
- A Software Bill of Materials (CycloneDX 1.5) for each platform in `sbom/`. CI checks it and releases attach it.
- `CHANGELOG.md`, and a release check that the tag matches the `Cargo.toml` version.

### Changed
- Overview ▸ At a Glance is now an even four-column grid (three or two columns on narrower windows). Long names wrap inside their column instead of running off the right edge.
- The app version is now 0.1.3, matching the release tag.

### Fixed
- The sidebar's ✕ button closes the character again. Clicking a card's text selects it, and closing a character no longer moves the selection to the wrong card.
- A new clippy lint in Rust 1.100 (`needless_range_loop`) that failed CI.

## [0.1.2] - 2026-10-29

### Added
- Pushing a `v*` tag publishes a GitHub Release with Windows, Linux and macOS (Apple Silicon and Intel) packages. v0.1.2 is the first tag with downloadable packages.

### Changed
- CI actions updated to their Node 24 versions.

## [0.1.1] - 2026-10-25

### Changed
- The Intel macOS build runs on `macos-15-intel` (`macos-13` was retired).

## [0.1.0] - 2026-10-25

### Added
- First release: a character builder for both the core rules and core + Combined Updates, with multiple open characters, save/load (`.ppchar`), PDF character sheets, and a Play tab with a clickable sheet and a dice roller.

[Unreleased]: https://github.com/SJLDeveloper/PPForge/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/SJLDeveloper/PPForge/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/SJLDeveloper/PPForge/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/SJLDeveloper/PPForge/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/SJLDeveloper/PPForge/releases/tag/v0.1.0
