## What and why

<!-- What does this change, and why? For rules-data changes, cite the book and section. -->

## Checklist

- [ ] `cargo test` and `cargo clippy --all-targets -- -D warnings` pass
- [ ] If `Cargo.toml`, `Cargo.lock` or `data/` changed: ran `python3 tools/sbom.py` and committed `sbom/` and `docs/SBOM.md` (CI fails otherwise)
- [ ] If the architecture, save format, rules engine, UI structure or build changed: updated `docs/DESIGN.md`
- [ ] If rules data or its generators changed: updated `docs/RULES_DATA.md` (and `tools/SCHEMA.md` if the format changed)
- [ ] If this is a significant design choice: added an entry to `docs/DECISIONS.md`
- [ ] Bumped `.ppchar` `format_version` if old files would no longer load correctly
