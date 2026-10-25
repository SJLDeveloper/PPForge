//! Rules data: power, perk, flaw and modifier definitions for each supported ruleset.
//!
//! The data files in `data/` are compiled into the binary, so the application has no
//! runtime file dependencies and behaves identically on Windows, macOS and Linux.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Ruleset {
    /// Prowlers & Paragons Ultimate Edition core rulebook as printed.
    Core,
    /// Core rules with the Combined Updates errata applied.
    #[default]
    Errata,
}

impl Ruleset {
    pub const ALL: [Ruleset; 2] = [Ruleset::Core, Ruleset::Errata];
    pub fn label(self) -> &'static str {
        match self {
            Ruleset::Core => "Core Rules (UE)",
            Ruleset::Errata => "Core + Combined Errata",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            Ruleset::Core => "CORE",
            Ruleset::Errata => "ERRATA",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RankSpec {
    Power,
    Default,
    Special,
    Baseline {
        base: String,
        #[serde(default)]
        half: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostOption {
    pub label: String,
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CostSpec {
    PerRank { value: i32 },
    PerTwoRanks,
    Flat { value: i32 },
    PerPowerLevel { value: i32 },
    PerUnit { value: i32, unit: String },
    Choice {
        options: Vec<CostOption>,
        #[serde(default)]
        per_rank: bool,
    },
    Special {
        #[serde(default)]
        note: String,
    },
}

impl CostSpec {
    pub fn describe(&self) -> String {
        match self {
            CostSpec::PerRank { value } => format!("{value} HP per rank"),
            CostSpec::PerTwoRanks => "1 HP per 2 ranks".into(),
            CostSpec::Flat { value } => format!("{value} HP"),
            CostSpec::PerPowerLevel { value } => format!("{value} HP per power level"),
            CostSpec::PerUnit { value, unit } => format!("{value} HP per {unit}"),
            CostSpec::Choice { options, per_rank } => {
                let opts: Vec<String> = options.iter().map(|o| format!("{} {}", o.label, o.value)).collect();
                format!("{}{}", opts.join(" / "), if *per_rank { " HP per rank" } else { " HP" })
            }
            CostSpec::Special { note } => {
                if note.is_empty() {
                    "Special".into()
                } else {
                    format!("Special: {note}")
                }
            }
        }
    }
    pub fn is_ranked(&self) -> bool {
        matches!(
            self,
            CostSpec::PerRank { .. } | CostSpec::PerTwoRanks | CostSpec::Choice { per_rank: true, .. }
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModKind {
    Pro,
    Con,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModCost {
    Flat { value: i32 },
    PerRank { value: i32 },
    PerTwoRanks { value: i32 },
    PerPowerLevel { value: i32 },
    Choice {
        options: Vec<i32>,
        #[serde(default)]
        per_rank: bool,
    },
    Range {
        min: i32,
        max: i32,
        #[serde(default)]
        per_rank: bool,
    },
    Special {
        #[serde(default)]
        note: String,
    },
}

impl ModCost {
    pub fn describe(&self) -> String {
        fn s(v: i32) -> String {
            if v > 0 { format!("+{v}") } else { format!("{v}") }
        }
        match self {
            ModCost::Flat { value } => s(*value),
            ModCost::PerRank { value } => format!("{}/rank", s(*value)),
            ModCost::PerTwoRanks { value } => format!("{}/2 ranks", s(*value)),
            ModCost::PerPowerLevel { value } => format!("{}/power level", s(*value)),
            ModCost::Choice { options, per_rank } => {
                let o: Vec<String> = options.iter().map(|v| s(*v)).collect();
                format!("{}{}", o.join(" or "), if *per_rank { "/rank" } else { "" })
            }
            ModCost::Range { min, max, per_rank } => {
                format!("{} to {}{}", s(*min), s(*max), if *per_rank { "/rank" } else { "" })
            }
            ModCost::Special { .. } => "special".into(),
        }
    }
    /// Values a player may pick for this modifier, if it offers a choice.
    pub fn choices(&self) -> Option<Vec<i32>> {
        match self {
            ModCost::Choice { options, .. } => Some(options.clone()),
            ModCost::Range { min, max, .. } => {
                let (lo, hi) = if min.abs() <= max.abs() { (*min, *max) } else { (*max, *min) };
                let step = if hi >= lo { 1 } else { -1 };
                let mut v = Vec::new();
                let mut x = lo;
                loop {
                    v.push(x);
                    if x == hi {
                        break;
                    }
                    x += step;
                }
                Some(v)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModDef {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ModKind,
    pub cost: ModCost,
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default)]
    pub summary: String,
    /// Some modifiers replace the power's base cost (e.g. Strike's Parrying in the errata).
    #[serde(default)]
    pub override_cost: Option<CostSpec>,
    /// Raises the Trait Cap for the modified Trait (errata Overkill: +3d).
    #[serde(default)]
    pub trait_cap_bonus: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PowerDef {
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    pub range: String,
    pub rank: RankSpec,
    pub cost: CostSpec,
    #[serde(default = "yes")]
    pub affects_resolve: bool,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub mods: Vec<ModDef>,
    #[serde(default)]
    pub ignores_trait_cap: bool,
}

fn yes() -> bool {
    true
}

impl PowerDef {
    /// The Power has a rank (Power Rank or Baseline Rank).
    pub fn has_rank(&self) -> bool {
        matches!(self.rank, RankSpec::Power | RankSpec::Baseline { .. })
    }
    /// Ranks are bought individually (as opposed to a flat cost with a fixed/baseline rank).
    pub fn buys_ranks(&self) -> bool {
        self.has_rank() && (self.cost.is_ranked() || matches!(self.cost, CostSpec::Special { .. }))
    }
    pub fn is_baseline(&self) -> bool {
        matches!(self.rank, RankSpec::Baseline { .. })
    }
    pub fn rank_label(&self) -> String {
        match &self.rank {
            RankSpec::Power => "Power Rank".into(),
            RankSpec::Default => "Default Rank".into(),
            RankSpec::Special => "Special".into(),
            RankSpec::Baseline { base, half } => {
                let b = if let Some(rest) = base.strip_prefix("avg:") {
                    format!("avg of {}", rest.replace('+', " & "))
                } else if let Some(rest) = base.strip_prefix("fixed:") {
                    format!("{rest}d")
                } else if let Some(rest) = base.strip_prefix("choice:") {
                    rest.replace('|', " or ")
                } else if base == "special" {
                    "chosen Trait".into()
                } else {
                    base.clone()
                };
                format!("Baseline ({}{})", if *half { "½ " } else { "" }, b)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerkDef {
    pub name: String,
    pub cost: CostSpec,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlawKind {
    Ordinary,
    PlotHook,
    Condition,
    Special,
}

impl FlawKind {
    pub fn label(self) -> &'static str {
        match self {
            FlawKind::Ordinary => "Flaw",
            FlawKind::PlotHook => "Plot Hook",
            FlawKind::Condition => "Condition",
            FlawKind::Special => "Varies",
        }
    }
    /// Whether this Flaw grants 1 Resolve at the start of every issue by default.
    pub fn grants_issue_resolve(self) -> bool {
        matches!(self, FlawKind::PlotHook | FlawKind::Condition)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlawDef {
    pub name: String,
    pub kind: FlawKind,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesData {
    pub ruleset: String,
    pub powers: Vec<PowerDef>,
    pub generic_mods: Vec<ModDef>,
    #[serde(default)]
    pub perks: Vec<PerkDef>,
    #[serde(default)]
    pub flaws: Vec<FlawDef>,
}

/// A loaded ruleset with lookup indexes.
pub struct Rules {
    pub ruleset: Ruleset,
    pub data: RulesData,
    power_idx: HashMap<String, usize>,
    perk_idx: HashMap<String, usize>,
    flaw_idx: HashMap<String, usize>,
    generic_idx: HashMap<String, usize>,
}

impl Rules {
    fn load(ruleset: Ruleset, json: &str) -> Rules {
        let data: RulesData = serde_json::from_str(json).expect("embedded rules data is valid JSON");
        let power_idx = data.powers.iter().enumerate().map(|(i, p)| (p.name.clone(), i)).collect();
        let perk_idx = data.perks.iter().enumerate().map(|(i, p)| (p.name.clone(), i)).collect();
        let flaw_idx = data.flaws.iter().enumerate().map(|(i, p)| (p.name.clone(), i)).collect();
        let generic_idx = data.generic_mods.iter().enumerate().map(|(i, p)| (p.name.clone(), i)).collect();
        Rules { ruleset, data, power_idx, perk_idx, flaw_idx, generic_idx }
    }
    pub fn power(&self, name: &str) -> Option<&PowerDef> {
        self.power_idx.get(name).map(|&i| &self.data.powers[i])
    }
    pub fn perk(&self, name: &str) -> Option<&PerkDef> {
        self.perk_idx.get(name).map(|&i| &self.data.perks[i])
    }
    pub fn flaw(&self, name: &str) -> Option<&FlawDef> {
        self.flaw_idx.get(name).map(|&i| &self.data.flaws[i])
    }
    pub fn generic_mod(&self, name: &str) -> Option<&ModDef> {
        self.generic_idx.get(name).map(|&i| &self.data.generic_mods[i])
    }
    /// Find a modifier available to a power: its own Pros/Cons first, then the generic list.
    pub fn find_mod<'a>(&'a self, power: Option<&'a PowerDef>, name: &str, generic: bool) -> Option<&'a ModDef> {
        if !generic
            && let Some(p) = power
                && let Some(m) = p.mods.iter().find(|m| m.name == name) {
                    return Some(m);
                }
        self.generic_mod(name)
    }

    pub fn config(&self) -> &'static RulesetConfig {
        RulesetConfig::of(self.ruleset)
    }
}

pub fn rules(ruleset: Ruleset) -> &'static Rules {
    static CORE: OnceLock<Rules> = OnceLock::new();
    static ERRATA: OnceLock<Rules> = OnceLock::new();
    match ruleset {
        Ruleset::Core => CORE.get_or_init(|| Rules::load(Ruleset::Core, include_str!("../data/core_rules.json"))),
        Ruleset::Errata => ERRATA.get_or_init(|| Rules::load(Ruleset::Errata, include_str!("../data/errata_rules.json"))),
    }
}

/// Rule differences between rulesets that live in prose rather than in the data tables.
pub struct RulesetConfig {
    pub power_level_names: [&'static str; 6],
    /// Roles (Blaster, Controller, ...) were introduced by the errata.
    pub has_roles: bool,
    /// Maximum Flaws without GM approval (core: 6; errata gives no maximum).
    pub max_flaws: Option<usize>,
    /// Summoning per-rank cost: core = ceil(Threat/2) HP; errata = Threat/2 HP (half points allowed).
    pub summoning_half_points: bool,
    /// Errata: Shapeshifting rank is added to effective Agility, Might and Toughness for Resolve.
    pub shapeshifting_counts_for_resolve: bool,
    /// Errata Ch.4 states Edge uses Agility or Willpower; Ch.2 says Agility or Intellect.
    pub edge_alt_label: Option<&'static str>,
    /// Power renames used when converting a character between rulesets.
    pub renames_from_other: &'static [(&'static str, &'static str)],
}

impl RulesetConfig {
    pub fn of(r: Ruleset) -> &'static RulesetConfig {
        static CORE: RulesetConfig = RulesetConfig {
            power_level_names: ["Street Level", "Low Level", "Standard", "High Level", "Legendary", "Iconic"],
            has_roles: false,
            max_flaws: Some(6),
            summoning_half_points: false,
            shapeshifting_counts_for_resolve: false,
            edge_alt_label: None,
            renames_from_other: &[
                ("Dodge", "Evasion"),
                ("Absorption", "Power Absorption"),
                ("Mimicry", "Power Mimicry"),
                ("Form, Energy", "Energy Form"),
                ("Form, Gaseous", "Gaseous Form"),
                ("Form, Liquid", "Liquid Form"),
                ("Form, Solid", "Solid Form"),
                ("Light/Sound/Effect", "Light/Effect"),
                ("Acute Sense", "Acute X"),
                ("Analytic Sense", "Analytic X"),
            ],
        };
        static ERRATA: RulesetConfig = RulesetConfig {
            power_level_names: ["Street Level", "Low Power", "Standard Power", "High Power", "Legendary", "Iconic"],
            has_roles: true,
            max_flaws: None,
            summoning_half_points: true,
            shapeshifting_counts_for_resolve: true,
            edge_alt_label: Some("Use Willpower instead of Intellect for Edge (errata Ch. 4 wording)"),
            renames_from_other: &[
                ("Evasion", "Dodge"),
                ("Power Absorption", "Absorption"),
                ("Power Mimicry", "Mimicry"),
                ("Energy Form", "Form, Energy"),
                ("Gaseous Form", "Form, Gaseous"),
                ("Liquid Form", "Form, Liquid"),
                ("Solid Form", "Form, Solid"),
                ("Light/Effect", "Light/Sound/Effect"),
                ("Acute X", "Acute Sense"),
                ("Analytic X", "Analytic Sense"),
            ],
        };
        match r {
            Ruleset::Core => &CORE,
            Ruleset::Errata => &ERRATA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_rulesets_load() {
        for r in Ruleset::ALL {
            let rules = rules(r);
            assert!(rules.data.powers.len() > 130, "{:?} has too few powers", r);
            assert!(rules.data.generic_mods.len() > 40);
            assert!(!rules.data.perks.is_empty());
            assert!(rules.data.flaws.len() > 40);
        }
    }

    #[test]
    fn rename_targets_exist() {
        for r in Ruleset::ALL {
            let rules = rules(r);
            for (_, to) in rules.config().renames_from_other {
                assert!(rules.power(to).is_some(), "{to} missing in {:?}", r);
            }
        }
    }

    #[test]
    fn key_costs_match_books() {
        let core = rules(Ruleset::Core);
        let err = rules(Ruleset::Errata);
        assert_eq!(core.power("Regeneration").unwrap().cost, CostSpec::Flat { value: 6 });
        assert_eq!(err.power("Regeneration").unwrap().cost, CostSpec::Flat { value: 12 });
        assert_eq!(core.power("Quick Change").unwrap().cost, CostSpec::Flat { value: 3 });
        assert_eq!(err.power("Quick Change").unwrap().cost, CostSpec::Flat { value: 1 });
        assert_eq!(err.power("Strike").unwrap().cost, CostSpec::PerTwoRanks);
        assert!(core.power("Evasion").is_some() && err.power("Dodge").is_some());
        assert!(err.power("Evasion").is_none());
    }
}
