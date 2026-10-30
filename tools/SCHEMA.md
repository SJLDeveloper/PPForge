# Rules data JSON schema (one file per ruleset)

Reference for `data/core_rules.json` and `data/errata_rules.json`. The Rust types that read these files are in `src/rules.rs`; if this document and the code disagree, the code is right, so fix this document. The provenance and verification of the data are described in `docs/RULES_DATA.md`.

Top level:
```json
{
  "ruleset": "core" | "errata",
  "powers": [Power, ...],
  "generic_mods": [Mod, ...],      // the generic Pros & Cons section (apply to any Power/Trait)
  "perks": [Perk, ...],
  "flaws": [Flaw, ...]
}
```

Power:
```json
{
  "name": "Armor",                      // Title Case, as listed in the book's Powers list
  "category": null,                     // e.g. "Super Senses" for individual senses, "Forms" etc. else null
  "range": "Self" | "Touch" | "Ranged" | "Zone" | "Special",
  "rank": RankSpec,
  "cost": CostSpec,
  "affects_resolve": true,              // false for powers the ruleset says don't affect Resolve (see notes)
  "summary": "One or two sentences IN YOUR OWN WORDS describing what it does mechanically. Do not copy book sentences.",
  "mods": [Mod, ...]                    // power-specific Pros and Cons
}
```

RankSpec (pick one):
- `{"kind":"power"}`                        -> "Power Rank"
- `{"kind":"default"}`                      -> "Default Rank" (no rank, flat cost)
- `{"kind":"special"}`                      -> "Special"
- `{"kind":"baseline","base":"Toughness","half":true}`   -> "Baseline Rank (½ Toughness)"
- `{"kind":"baseline","base":"Agility"}`
- `{"kind":"baseline","base":"avg:Agility+Might"}`        -> average of two traits
- `{"kind":"baseline","base":"fixed:3"}`                  -> e.g. Running "Baseline Rank (3d)"
- `{"kind":"baseline","base":"choice:Might|Martial Arts"}` -> book offers alternatives
- `{"kind":"baseline","base":"special"}`                  -> player chooses (Boost, Expertise)
Base trait names must be exact: Agility, Intellect, Might, Perception, Toughness, Willpower, or a Power name like "Martial Arts".

CostSpec (pick one):
- `{"kind":"per_rank","value":2}`                -> "2 Hero Points per rank"
- `{"kind":"per_two_ranks"}`                     -> "1 Hero Point per 2 ranks"
- `{"kind":"flat","value":12}`                   -> "12 Hero Points"
- `{"kind":"per_power_level","value":4}`         -> "4 Hero Points per power level"
- `{"kind":"per_unit","value":3,"unit":"immunity"}`  -> e.g. 3 per immunity, 5 per Resolve, 1 per language pick
- `{"kind":"choice","options":[{"label":"Narrow","value":3},{"label":"Broad","value":5}],"per_rank":true}`
   (per_rank=false for flat alternatives like Stretching 1/3/6, Hard to Kill 1 or 6)
- `{"kind":"special","note":"shares boosted trait's cost"}`

Mod:
```json
{
  "name": "Activated",
  "type": "pro" | "con",
  "cost": ModCost,
  "repeatable": false,           // true when the book says "apply per extra Source/type/etc."
  "summary": "Own-words one-liner."
}
```
ModCost (signed integers; cons are negative):
- `{"kind":"flat","value":-1}`
- `{"kind":"per_rank","value":1}`          -> "+1 Hero Point per rank" (changes cost per rank)
- `{"kind":"per_two_ranks","value":1}`     -> "+1 Hero Point per 2 ranks"
- `{"kind":"per_power_level","value":-1}`
- `{"kind":"choice","options":[-2,-4]}`     -> "-2 or -4"
- `{"kind":"range","min":-1,"max":-4}`      -> "-1 to -4"
- `{"kind":"special","note":"..."}`

Optional fields (used by the errata data):
- Power `"ignores_trait_cap": true` — the Power may be bought above the Trait Cap (errata Teleportation).
- Mod `"override_cost": CostSpec` — choosing this modifier replaces the Power's base cost (errata Strike *Parrying*: 1 per 2 ranks → 1 per rank).
- Mod `"trait_cap_bonus": 3` — the modified Trait's cap is raised by this many dice (errata *Overkill*).
- ModCost `"per_rank": true` on `choice`/`range` — the chosen value applies per rank (core Summoning *Animals*, *Unique*).
- Power `"category"`: `"Super Senses"`, `"Form"` or (core only) `"Transformation"` groups sub-powers in the picker.

Perk: `{"name":"Fame","cost":{"kind":"flat","value":3},"summary":"..."}` (per_unit allowed, e.g. Headquarters 1 per 3 Base Points, Contacts 1 each, choice for Wealth/Great Wealth 3 or 6, per_power_level for Pet/Sidekick)
Flaw: `{"name":"Enemy","kind":"plot_hook"|"condition"|"ordinary"|"special","summary":"..."}`

Rules for writers (whether a person or a script updating the data):
- Capture EVERY power and EVERY pro/con with exact names and numbers from the text. Numbers matter most.
- Summaries must be paraphrased, short, and mechanical. Never reproduce book sentences.
- Output must be valid JSON (no comments).
