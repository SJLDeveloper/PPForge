#!/usr/bin/env python3
"""Generate (or verify) PP Forge's Software Bill of Materials.

Produces one CycloneDX 1.5 JSON SBOM per release platform in `sbom/`, plus a human-readable
summary in `docs/SBOM.md`. Run from the repository root:

    python3 tools/sbom.py           # regenerate sbom/ and docs/SBOM.md
    python3 tools/sbom.py --check   # exit 1 if the committed files are out of date (used by CI)

Requires `cargo-cyclonedx` (pinned below; install with
`cargo install cargo-cyclonedx --version <CYCLONEDX_VERSION> --locked`).

The raw cargo-cyclonedx output is normalized so the files are reproducible on any machine:
the per-run timestamp is removed, the random serial number is replaced by one derived from the
inputs, absolute checkout paths are replaced, and lists are sorted. The embedded rules data files
are added as `data` components, because they ship inside the binary but aren't Cargo packages.
"""

from __future__ import annotations

import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import tomllib
import uuid
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parent.parent
CYCLONEDX_VERSION = "0.5.9"
SPEC_VERSION = "1.5"

# Release platforms (must match the matrix in .github/workflows/build.yml).
TARGETS = [
    ("x86_64-pc-windows-msvc", "Windows (x86-64)"),
    ("aarch64-apple-darwin", "macOS (Apple Silicon)"),
    ("x86_64-apple-darwin", "macOS (Intel)"),
    ("x86_64-unknown-linux-gnu", "Linux (x86-64)"),
]

# Data files compiled into the binary with include_str!.
DATA_FILES = [
    (
        "data/core_rules.json",
        "Rules data: Prowlers & Paragons Ultimate Edition core rulebook",
        "Prowlers_&_Paragons_Ultimate_Edition.pdf (core rulebook, as printed)",
        "tools/build_core_rules.py",
    ),
    (
        "data/errata_rules.json",
        "Rules data: Prowlers & Paragons UE Combined Updates (errata)",
        "Combined_Updates.pdf (Combined Updates / errata document)",
        "tools/build_errata_rules.py",
    ),
]

# Why each direct dependency is here (for docs/SBOM.md).
PURPOSE = {
    "eframe": "Native window, event loop and rendering (wgpu, with OpenGL fallback)",
    "egui": "Immediate-mode GUI widgets",
    "printpdf": "PDF character sheet export",
    "rfd": "Native open/save file dialogs",
    "serde": "Serialization framework (save files, rules data)",
    "serde_json": "JSON reading/writing for `.ppchar` files and rules data",
    "directories": "Per-OS settings and Documents folder locations",
}

NOTABLE = ("GPL", "LGPL", "AGPL", "MPL", "OFL", "Ubuntu-font", "CC-BY", "EPL")


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_paths(obj, root: str):
    """Recursively replace the absolute checkout path so output doesn't depend on the machine."""
    if isinstance(obj, str):
        return obj.replace(root, "/pp_forge")
    if isinstance(obj, list):
        return [replace_paths(x, root) for x in obj]
    if isinstance(obj, dict):
        return {k: replace_paths(v, root) for k, v in obj.items()}
    return obj


def run_cyclonedx(target: str, workdir: pathlib.Path) -> dict:
    name = f"sbom-raw-{target}"
    cmd = [
        "cargo", "cyclonedx", "-q", "-f", "json", "--spec-version", SPEC_VERSION,
        "--target", target, "--override-filename", name,
    ]
    subprocess.run(cmd, cwd=workdir, check=True)
    raw = workdir / f"{name}.json"
    data = json.loads(raw.read_text())
    raw.unlink()
    return data


def normalize(bom: dict, target: str, lock_hash: str) -> dict:
    bom = replace_paths(bom, str(ROOT))
    meta = bom["metadata"]
    meta.pop("timestamp", None)
    app = meta["component"]
    version = app["version"]

    # Add the embedded rules data as components the application depends on.
    data_refs = []
    for rel, desc, source, builder in DATA_FILES:
        ref = f"pp_forge-data:{rel}"
        data_refs.append(ref)
        bom["components"].append({
            "type": "data",
            "bom-ref": ref,
            "name": pathlib.Path(rel).name,
            "version": version,
            "description": desc,
            "scope": "required",
            "hashes": [{"alg": "SHA-256", "content": sha256(ROOT / rel)}],
            "properties": [
                {"name": "pp_forge:path", "value": rel},
                {"name": "pp_forge:embedded", "value": "compiled into the binary (include_str!)"},
                {"name": "pp_forge:source-document", "value": source},
                {"name": "pp_forge:generated-by", "value": builder},
                {"name": "pp_forge:content", "value": "game mechanics (names, ranges, ranks, costs, modifier values) with paraphrased summaries; no rulebook text"},
            ],
        })
    for dep in bom["dependencies"]:
        if dep["ref"] == app["bom-ref"]:
            dep["dependsOn"] = sorted(set(dep.get("dependsOn", [])) | set(data_refs))
    bom["dependencies"].extend({"ref": r, "dependsOn": []} for r in data_refs)

    # Deterministic ordering and serial number.
    bom["components"].sort(key=lambda c: c["bom-ref"])
    bom["dependencies"].sort(key=lambda d: d["ref"])
    for d in bom["dependencies"]:
        d["dependsOn"] = sorted(d.get("dependsOn", []))
    data_hash = "".join(sha256(ROOT / rel) for rel, *_ in DATA_FILES)
    seed = f"pp_forge|{version}|{target}|{lock_hash}|{data_hash}|cyclonedx-{CYCLONEDX_VERSION}"
    bom["serialNumber"] = f"urn:uuid:{uuid.uuid5(uuid.NAMESPACE_URL, seed)}"
    return bom


def license_of(c: dict) -> str:
    out = []
    for lic in c.get("licenses", []):
        out.append(lic.get("expression") or lic.get("license", {}).get("id") or lic.get("license", {}).get("name") or "?")
    return " ".join(" AND ".join(out).split()) if out else "(none declared)"


def summary_markdown(boms: dict[str, dict], lock: dict) -> str:
    any_bom = next(iter(boms.values()))
    app = any_bom["metadata"]["component"]
    root_ref = app["bom-ref"]
    lock_versions = {p["name"]: p["version"] for p in lock["package"]}

    direct_refs = set()
    for bom in boms.values():
        for d in bom["dependencies"]:
            if d["ref"] == root_ref:
                direct_refs |= set(d["dependsOn"])

    union: dict[str, dict] = {}
    per_target_counts = {}
    for target, bom in boms.items():
        libs = [c for c in bom["components"] if c["type"] == "library"]
        per_target_counts[target] = len(libs)
        for c in libs:
            union[c["purl"]] = c

    direct = sorted(
        (c for c in union.values() if c["bom-ref"] in direct_refs),
        key=lambda c: c["name"],
    )
    tally = Counter(license_of(c) for c in union.values())
    notable = sorted(
        (c for c in union.values() if any(n in license_of(c) for n in NOTABLE)),
        key=lambda c: c["name"],
    )

    lines = [
        "# Software Bill of Materials (SBOM)",
        "",
        "<!-- Generated by tools/sbom.py — do not edit by hand. Regenerate with `python3 tools/sbom.py`. -->",
        "",
        f"PP Forge **{app['version']}** ships an SBOM in [CycloneDX {SPEC_VERSION}](https://cyclonedx.org/) JSON format for each release platform.",
        "The machine-readable files live in [`sbom/`](../sbom/) and are attached to every GitHub Release.",
        "CI regenerates them on every push and fails if the committed copies are out of date, so they always match `Cargo.lock` and the embedded data.",
        "",
        "## Files",
        "",
        "| Platform | SBOM file | Third-party packages |",
        "|---|---|---|",
    ]
    for target, label in TARGETS:
        lines.append(f"| {label} | [`sbom/pp_forge-{target}.cdx.json`](../sbom/pp_forge-{target}.cdx.json) | {per_target_counts[target]} |")
    lines += [
        "",
        f"Across all platforms: **{len(union)}** distinct third-party packages, all from crates.io and each pinned by version and SHA-256 checksum in `Cargo.lock` ({len(lock['package'])} lock entries, including target-specific ones). The counts differ by platform because Rust only compiles each operating system's own windowing, graphics and file-dialog libraries.",
        "",
        "## Direct dependencies",
        "",
        "| Package | Version | License | Purpose |",
        "|---|---|---|---|",
    ]
    for c in direct:
        lines.append(f"| `{c['name']}` | {c['version']} | {license_of(c)} | {PURPOSE.get(c['name'], '')} |")
    lines += [
        "",
        "Everything else is a transitive dependency of these.",
        "",
        "## Embedded data",
        "",
        "These files are compiled into the executable and are listed in each SBOM as `data` components with SHA-256 hashes.",
        "",
        "| File | Source document | Generated by | SHA-256 |",
        "|---|---|---|---|",
    ]
    for rel, _desc, source, builder in DATA_FILES:
        lines.append(f"| `{rel}` | {source} | `{builder}` | `{sha256(ROOT / rel)[:16]}…` |")
    lines += [
        "",
        "The data holds game mechanics (names, ranges, ranks, costs, modifier values) and short paraphrased summaries, not rulebook text. See [RULES_DATA.md](RULES_DATA.md).",
        "",
        "## License summary (all platforms)",
        "",
        "| License expression | Packages |",
        "|---|---|",
    ]
    for lic, n in sorted(tally.items(), key=lambda kv: (-kv[1], kv[0])):
        lines.append(f"| `{lic}` | {n} |")
    lines += [
        "",
        "## Licenses worth noting",
        "",
        "Nearly everything is permissively licensed (MIT, Apache-2.0, BSD, Zlib, ISC, Unicode, BSL). The packages below carry other terms. None of them is strong copyleft for a distributed binary, but they come with attribution or file-level obligations:",
        "",
    ]
    if notable:
        lines += ["| Package | Version | License |", "|---|---|---|"]
        for c in notable:
            lines.append(f"| `{c['name']}` | {c['version']} | `{license_of(c)}` |")
    else:
        lines.append("None.")
    lines += [
        "",
        "Where an expression offers a choice (`A OR B`), PP Forge can use the permissive option.",
        "",
        "## PP Forge's own license",
        "",
        "PP Forge itself does not yet declare a license (`Cargo.toml` has no `license` field and there is no `LICENSE` file). Until one is chosen, the code is \"all rights reserved\" by default. See the open items in [DESIGN.md](DESIGN.md#10-open-items).",
        "",
        "## Regenerating",
        "",
        "```sh",
        f"cargo install cargo-cyclonedx --version {CYCLONEDX_VERSION} --locked   # once",
        "python3 tools/sbom.py            # after changing Cargo.toml / Cargo.lock or data/",
        "python3 tools/sbom.py --check    # what CI runs",
        "```",
        "",
    ]
    return "\n".join(lines)


def generate(out_dir: pathlib.Path, doc_path: pathlib.Path) -> None:
    lock_path = ROOT / "Cargo.lock"
    lock_hash = sha256(lock_path)
    lock = tomllib.loads(lock_path.read_text())
    out_dir.mkdir(parents=True, exist_ok=True)
    boms = {}
    for target, _label in TARGETS:
        bom = normalize(run_cyclonedx(target, ROOT), target, lock_hash)
        boms[target] = bom
        (out_dir / f"pp_forge-{target}.cdx.json").write_text(json.dumps(bom, indent=2, ensure_ascii=False) + "\n")
    doc_path.parent.mkdir(parents=True, exist_ok=True)
    doc_path.write_text(summary_markdown(boms, lock))


def main() -> int:
    if shutil.which("cargo-cyclonedx") is None:
        print(f"cargo-cyclonedx not found. Install it with:\n  cargo install cargo-cyclonedx --version {CYCLONEDX_VERSION} --locked", file=sys.stderr)
        return 2
    os.chdir(ROOT)
    sbom_dir = ROOT / "sbom"
    doc = ROOT / "docs" / "SBOM.md"
    if "--check" in sys.argv[1:]:
        with tempfile.TemporaryDirectory() as tmp:
            tmp = pathlib.Path(tmp)
            generate(tmp / "sbom", tmp / "SBOM.md")
            stale = []
            expected = {p.name: p for p in (tmp / "sbom").glob("*.json")}
            committed = {p.name: p for p in sbom_dir.glob("*.json")} if sbom_dir.exists() else {}
            for name in sorted(set(expected) | set(committed)):
                if name not in committed or name not in expected or committed[name].read_bytes() != expected[name].read_bytes():
                    stale.append(f"sbom/{name}")
            if not doc.exists() or doc.read_bytes() != (tmp / "SBOM.md").read_bytes():
                stale.append("docs/SBOM.md")
        if stale:
            print("SBOM is out of date. Run `python3 tools/sbom.py` and commit the result.\nChanged:\n  " + "\n  ".join(stale), file=sys.stderr)
            return 1
        print("SBOM is up to date.")
        return 0
    generate(sbom_dir, doc)
    print(f"Wrote {len(TARGETS)} SBOMs to sbom/ and docs/SBOM.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
