//! Rules engine: Hero Point costs, derived characteristics and validation.
//!
//! Costs are tracked internally in half-points so that "1 Hero Point per 2 ranks" Powers and
//! "+1 per 2 ranks" modifiers combine exactly; totals are rounded up to whole Hero Points.

use crate::model::*;
use crate::rules::{self, CostSpec, ModCost, ModDef, PowerDef, RankSpec, Rules, Ruleset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct PowerCost {
    pub cost: i32,
    /// Final rank including baseline, if the Power has a rank.
    pub rank: Option<i32>,
    /// Baseline portion of the rank, if any.
    pub baseline: Option<i32>,
    /// Human-readable cost explanation.
    pub detail: String,
    pub floor_applied: bool,
    pub unknown: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub hp_available: i32,
    pub hp_starting: i32,
    pub hp_earned: i32,
    pub cost_package: i32,
    pub cost_abilities: i32,
    pub cost_talents: i32,
    pub cost_powers: i32,
    pub cost_perks: i32,
    pub cost_flaws: i32,
    pub total_spent: i32,
    pub trait_cap: i32,
    pub edge: i32,
    pub health: i32,
    pub resolve: Option<i32>,
    pub resolve_base: i32,
    pub resolve_bonus: i32,
    pub highest_rank: i32,
    pub highest_from: String,
    pub ability_costs: [i32; 6],
    pub talent_costs: [i32; 12],
    pub power_costs: Vec<PowerCost>,
    pub perk_costs: Vec<i32>,
    pub issues: Vec<Issue>,
}

impl Summary {
    pub fn remaining(&self) -> i32 {
        self.hp_available - self.total_spent
    }
}

/// Resolve the PowerDef for an entry (built-in or custom).
pub fn power_def<'a>(rules: &'a Rules, p: &'a PowerEntry) -> Option<&'a PowerDef> {
    if let Some(c) = &p.custom {
        return Some(c);
    }
    rules.power(&p.def)
}

/// Look up a modifier picked on a power.
pub fn mod_def<'a>(rules: &'a Rules, power: Option<&'a PowerDef>, m: &ModPick) -> Option<&'a ModDef> {
    rules.find_mod(power, &m.name, m.generic)
}

fn div_ceil2(halves: i32) -> i32 {
    if halves <= 0 { halves / 2 } else { (halves + 1) / 2 }
}

/// Value of a trait by name, for baselines.
pub fn trait_rank(c: &Character, rules: &Rules, name: &str, depth: u8) -> Option<i32> {
    if let Some(i) = ABILITIES.iter().position(|a| *a == name) {
        return Some(c.abilities[i].rank);
    }
    if let Some(i) = TALENTS.iter().position(|a| *a == name) {
        return Some(c.talents[i].rank);
    }
    if depth > 3 {
        return None;
    }
    // Another Power (e.g. Martial Arts as Strike's baseline): use its final rank.
    c.powers.iter().find(|p| p.def == name).and_then(|p| power_cost(c, rules, p, depth + 1).rank)
}

fn baseline_value(c: &Character, rules: &Rules, p: &PowerEntry, base: &str, half: bool, depth: u8) -> i32 {
    let v = if let Some(rest) = base.strip_prefix("avg:") {
        let parts: Vec<i32> = rest.split('+').filter_map(|n| trait_rank(c, rules, n, depth)).collect();
        if parts.is_empty() { 0 } else { (parts.iter().sum::<i32>() as f32 / parts.len() as f32).ceil() as i32 }
    } else if let Some(rest) = base.strip_prefix("fixed:") {
        rest.parse().unwrap_or(0)
    } else if let Some(rest) = base.strip_prefix("choice:") {
        let first = rest.split('|').next().unwrap_or("Might");
        let chosen = if p.base_trait.is_empty() { first } else { p.base_trait.as_str() };
        trait_rank(c, rules, chosen, depth).unwrap_or(0)
    } else if base == "special" {
        if p.base_trait.is_empty() { 0 } else { trait_rank(c, rules, &p.base_trait, depth).unwrap_or(0) }
    } else {
        trait_rank(c, rules, base, depth).unwrap_or(0)
    };
    if half { (v + 1) / 2 } else { v }
}

struct Accum {
    halves_per_rank: i32,
    flat: i32,
    ranked: bool,
    two_rank_base: bool,
    notes: Vec<String>,
}

fn apply_mods(acc: &mut Accum, rules: &Rules, def: Option<&PowerDef>, mods: &[ModPick], pl: i32) {
    for m in mods {
        let Some(md) = mod_def(rules, def, m) else {
            acc.notes.push(format!("unknown modifier '{}' ignored", m.name));
            continue;
        };
        let t = m.times();
        let pick = |opts: &[i32]| -> i32 { m.value.filter(|v| opts.contains(v)).unwrap_or(opts[0]) };
        match &md.cost {
            ModCost::Flat { value } => acc.flat += value * t,
            ModCost::PerRank { value } => {
                if acc.ranked {
                    acc.halves_per_rank += 2 * value * t
                } else {
                    acc.flat += value * t
                }
            }
            ModCost::PerTwoRanks { value } => {
                if acc.ranked {
                    acc.halves_per_rank += value * t
                } else {
                    acc.flat += value * t
                }
            }
            ModCost::PerPowerLevel { value } => acc.flat += value * pl * t,
            ModCost::Choice { options, per_rank } => {
                let v = pick(options);
                if *per_rank && acc.ranked { acc.halves_per_rank += 2 * v * t } else { acc.flat += v * t }
            }
            ModCost::Range { per_rank, .. } => {
                let opts = md.cost.choices().unwrap_or_default();
                let v = if opts.is_empty() { 0 } else { pick(&opts) };
                if *per_rank && acc.ranked { acc.halves_per_rank += 2 * v * t } else { acc.flat += v * t }
            }
            ModCost::Special { .. } => {
                if matches!(md.name.as_str(), "Overkill" | "Weak") {
                    acc.halves_per_rank -= 2;
                } else {
                    acc.flat += m.value.unwrap_or(0) * t;
                }
            }
        }
    }
}

/// Cost and rank of a Power entry.
pub fn power_cost(c: &Character, rules: &Rules, p: &PowerEntry, depth: u8) -> PowerCost {
    let Some(def) = power_def(rules, p) else {
        return PowerCost {
            cost: p.cost_override.unwrap_or(0),
            unknown: true,
            detail: format!("'{}' is not part of the {} rules", p.def, rules.ruleset.label()),
            ..Default::default()
        };
    };
    let pl = p.units.clamp(1, 6);
    // A selected modifier may replace the base cost (errata Strike: Parrying).
    let mut base_cost = def.cost.clone();
    for m in &p.mods {
        if let Some(md) = mod_def(rules, Some(def), m)
            && let Some(oc) = &md.override_cost {
                base_cost = oc.clone();
            }
    }

    let mut acc = Accum { halves_per_rank: 0, flat: 0, ranked: false, two_rank_base: false, notes: Vec::new() };
    match &base_cost {
        CostSpec::PerRank { value } => {
            acc.ranked = true;
            acc.halves_per_rank = 2 * value;
        }
        CostSpec::PerTwoRanks => {
            acc.ranked = true;
            acc.two_rank_base = true;
            acc.halves_per_rank = 1;
        }
        CostSpec::Flat { value } => acc.flat = *value,
        CostSpec::PerPowerLevel { value } => acc.flat = value * pl,
        CostSpec::PerUnit { value, .. } => acc.flat = value * p.units.max(0),
        CostSpec::Choice { options, per_rank } => {
            let v = options.get(p.option).or(options.first()).map(|o| o.value).unwrap_or(0);
            if *per_rank {
                acc.ranked = true;
                acc.halves_per_rank = 2 * v;
            } else {
                acc.flat = v;
            }
        }
        CostSpec::Special { .. } => match def.name.as_str() {
            "Boost" => {
                acc.ranked = true;
                acc.halves_per_rank = 2 * p.boost_cost.max(0);
                if p.boost_cost == 0 {
                    acc.two_rank_base = true;
                    acc.halves_per_rank = 1;
                }
            }
            "Summoning" => {
                acc.ranked = true;
                let t = p.threat.max(1);
                acc.halves_per_rank = if rules.config().summoning_half_points { t } else { 2 * ((t + 1) / 2) };
            }
            _ => acc.notes.push("special cost: set a GM-approved cost override".into()),
        },
    }
    let base_flat_is_zero = matches!(base_cost, CostSpec::Flat { value: 0 });
    apply_mods(&mut acc, rules, Some(def), &p.mods, pl);

    // Rank.
    let (rank, baseline) = match &def.rank {
        RankSpec::Power => (Some(p.ranks.max(0)), None),
        RankSpec::Baseline { base, half } => {
            let b = baseline_value(c, rules, p, base, *half, depth);
            let bought = if def.buys_ranks() { p.ranks.max(0) } else { 0 };
            (Some(b + bought), Some(b))
        }
        RankSpec::Default | RankSpec::Special => (None, None),
    };
    let purchased = match &def.rank {
        RankSpec::Power => p.ranks.max(0),
        RankSpec::Baseline { .. } if def.buys_ranks() => p.ranks.max(0),
        _ => 0,
    };

    let mut floor_applied = false;
    let mut cost = if acc.ranked {
        let hpr = acc.halves_per_rank.max(1);
        if hpr != acc.halves_per_rank {
            floor_applied = true;
        }
        div_ceil2(hpr * purchased) + acc.flat
    } else {
        acc.flat
    };
    // No Trait can cost less than 1 HP (or 1 HP per 2 ranks).
    if !base_flat_is_zero {
        let floor = if acc.ranked && acc.two_rank_base { div_ceil2(purchased).max(1) } else { 1 };
        if cost < floor {
            cost = floor;
            floor_applied = true;
        }
    }
    let rate = if acc.ranked {
        let h = acc.halves_per_rank.max(1);
        if h % 2 == 0 { format!("{} HP/rank", h / 2) } else { format!("{}/2 HP per rank", h) }
    } else {
        String::new()
    };
    let mut detail = if acc.ranked {
        format!("{rate} × {purchased} rank(s){}", if acc.flat != 0 { format!(" {:+} flat", acc.flat) } else { String::new() })
    } else {
        base_cost.describe()
    };
    if floor_applied {
        detail.push_str(" (minimum cost applied)");
    }
    for n in &acc.notes {
        detail.push_str(&format!("; {n}"));
    }
    if let Some(o) = p.cost_override {
        cost = o;
        detail = format!("GM override ({o} HP)");
    }
    PowerCost { cost, rank, baseline, detail, floor_applied, unknown: false }
}

/// Cost of an Ability or Talent entry given the package baseline.
pub fn trait_cost(rules: &Rules, t: &TraitEntry, package_base: i32) -> i32 {
    let purchased = (t.rank - package_base).max(0);
    let mut acc = Accum { halves_per_rank: 2, flat: 0, ranked: true, two_rank_base: false, notes: Vec::new() };
    apply_mods(&mut acc, rules, None, &t.mods, 1);
    div_ceil2(acc.halves_per_rank.max(1) * purchased) + acc.flat
}

pub fn perk_cost(rules: &Rules, p: &PerkEntry) -> i32 {
    if let Some(o) = p.cost_override {
        return o;
    }
    let Some(def) = rules.perk(&p.def) else { return 0 };
    match &def.cost {
        CostSpec::Flat { value } => *value,
        CostSpec::PerUnit { value, .. } => value * p.units.max(0),
        CostSpec::PerPowerLevel { value } => value * p.units.clamp(1, 6),
        CostSpec::Choice { options, .. } => options.get(p.option).or(options.first()).map(|o| o.value).unwrap_or(0),
        CostSpec::PerRank { value } => value * p.units.max(0),
        CostSpec::PerTwoRanks => div_ceil2(p.units.max(0)),
        CostSpec::Special { .. } => 0,
    }
}

fn mod_cap_bonus(rules: &Rules, def: Option<&PowerDef>, mods: &[ModPick]) -> i32 {
    mods.iter().filter_map(|m| mod_def(rules, def, m)).map(|m| m.trait_cap_bonus).max().unwrap_or(0)
}

pub fn flaw_grants_issue_resolve(rules: &Rules, f: &FlawEntry) -> bool {
    f.issue_resolve
        .unwrap_or_else(|| rules.flaw(&f.def).map(|d| d.kind.grants_issue_resolve()).unwrap_or(false))
}

/// Compute everything about a character.
pub fn evaluate(c: &Character) -> Summary {
    let rules = rules::rules(c.ruleset);
    let cfg = rules.config();
    let mut s = Summary::default();
    let mut issues: Vec<Issue> = Vec::new();
    let mut err = |sev: Severity, m: String| issues.push(Issue { severity: sev, message: m });

    // Budget.
    s.hp_starting = c.options.starting_hp_override.unwrap_or(c.power_level.hero_points());
    s.hp_earned = c.earned_hp();
    s.hp_available = s.hp_starting + s.hp_earned;

    // Trait cap.
    let mut cap = c.power_level.trait_cap();
    match c.options.cap_mode {
        CapMode::Hard => {}
        CapMode::Floating => cap += s.hp_earned.max(0) / 10,
        CapMode::Soft => cap += c.options.cap_bonus,
    }
    s.trait_cap = cap;

    // Abilities & Talents.
    s.cost_package = c.package.cost();
    let ab = c.package.ability_base();
    let tb = c.package.talent_base();
    for (i, a) in c.abilities.iter().enumerate() {
        s.ability_costs[i] = trait_cost(rules, a, ab);
        let tcap = cap + mod_cap_bonus(rules, None, &a.mods);
        if a.rank > tcap {
            err(Severity::Error, format!("{} {}d exceeds the Trait Cap of {}d", ABILITIES[i], a.rank, tcap));
        }
        if a.rank < ab {
            err(Severity::Error, format!("{} is below the package minimum of {}d", ABILITIES[i], ab));
        }
        if a.rank < 1 && c.char_type == CharType::Hero {
            err(Severity::Warning, format!("{} is 0d; Heroes normally have at least 1d", ABILITIES[i]));
        } else if a.rank == 1 && c.char_type == CharType::Hero {
            err(Severity::Info, format!("{} 1d indicates a significant impairment", ABILITIES[i]));
        }
    }
    for (i, t) in c.talents.iter().enumerate() {
        s.talent_costs[i] = trait_cost(rules, t, tb);
        let tcap = cap + mod_cap_bonus(rules, None, &t.mods);
        if t.rank > tcap {
            err(Severity::Error, format!("{} {}d exceeds the Trait Cap of {}d", TALENTS[i], t.rank, tcap));
        }
        if t.rank < tb {
            err(Severity::Error, format!("{} is below the package minimum of {}d", TALENTS[i], tb));
        }
    }
    s.cost_abilities = s.ability_costs.iter().sum();
    s.cost_talents = s.talent_costs.iter().sum();

    // Powers.
    let names: Vec<&str> = c.powers.iter().map(|p| p.def.as_str()).collect();
    let has = |n: &str| names.contains(&n);
    for p in &c.powers {
        let pc = power_cost(c, rules, p, 0);
        let def = power_def(rules, p);
        let label = power_label(p);
        if pc.unknown {
            err(Severity::Error, format!("{label}: not available in {}; replace it or set a cost override", c.ruleset.label()));
        }
        if let (Some(r), Some(d)) = (pc.rank, def) {
            let tcap = cap + mod_cap_bonus(rules, Some(d), &p.mods);
            if r > tcap && !d.ignores_trait_cap && c.char_type == CharType::Hero {
                err(Severity::Error, format!("{label} {r}d exceeds the Trait Cap of {tcap}d"));
            }
            if d.buys_ranks() && !d.is_baseline() && p.ranks <= 0 {
                err(Severity::Warning, format!("{label} has no ranks"));
            }
        }
        if let Some(d) = def {
            if matches!(d.cost, CostSpec::Special { .. }) && p.cost_override.is_none() && !matches!(d.name.as_str(), "Boost" | "Summoning") {
                err(Severity::Warning, format!("{label} has a special cost; enter a GM-approved cost override"));
            }
            for m in &p.mods {
                if mod_def(rules, Some(d), m).is_none() {
                    err(Severity::Warning, format!("{label}: modifier '{}' is not in these rules", m.name));
                }
            }
            let mods: Vec<&str> = p.mods.iter().map(|m| m.name.as_str()).collect();
            if mods.contains(&"Armor Piercing") && (mods.contains(&"Penetrating") || mods.contains(&"Precise")) {
                err(Severity::Error, format!("{label}: Armor Piercing can't be combined with Penetrating or Precise"));
            }
            if mods.contains(&"Penetrating") && mods.contains(&"Precise") {
                err(Severity::Error, format!("{label}: Penetrating can't be combined with Precise"));
            }
            if d.name == "Summoning" {
                let max_t = cap - if cfg.summoning_half_points { 4 } else { 6 };
                if p.threat > max_t {
                    err(Severity::Warning, format!("Summoning Threat {}d is above the usual maximum of {}d", p.threat, max_t));
                }
            }
        }
        s.power_costs.push(pc);
    }
    s.cost_powers = s.power_costs.iter().map(|p| p.cost).sum();

    if has("Fast Healing") && has("Regeneration") {
        err(Severity::Error, "Fast Healing and Regeneration can't be taken together".into());
    }
    if has("Gestalt") && !c.flaws.iter().any(|f| f.def == "Alter Ego") {
        err(Severity::Error, "Gestalt requires the Alter Ego Flaw".into());
    }
    if has("Specialty") && !matches!(c.power_level, PowerLevel::Street | PowerLevel::Low) {
        err(Severity::Warning, "Specialty is intended for Street Level and Low Level games".into());
    }
    if has("Lightning Reflexes") && has("Super Speed") {
        err(Severity::Info, "Lightning Reflexes doesn't stack with Super Speed".into());
    }

    // Perks.
    for p in &c.perks {
        let v = perk_cost(rules, p);
        if rules.perk(&p.def).is_none() && p.cost_override.is_none() {
            err(Severity::Warning, format!("Perk '{}' is not in these rules; set a cost", p.def));
        }
        s.perk_costs.push(v);
    }
    s.cost_perks = s.perk_costs.iter().sum();

    // Flaws: the first 3 are free; each extra costs 3 Hero Points.
    let nflaws = c.flaws.len();
    s.cost_flaws = if nflaws > 3 { (nflaws as i32 - 3) * 3 } else { 0 };
    if c.char_type == CharType::Hero {
        if nflaws == 0 {
            err(Severity::Error, "Heroes must have at least one Flaw".into());
        }
        if nflaws > 3 && !c.options.creation_complete {
            err(Severity::Warning, "New Heroes may start with at most 3 Flaws; extra Flaws are bought later for 3 HP each".into());
        }
        if let Some(max) = cfg.max_flaws
            && nflaws > max {
                err(Severity::Warning, format!("More than {max} Flaws requires GM approval"));
            }
    }
    let fl: Vec<&str> = c.flaws.iter().map(|f| f.def.as_str()).collect();
    if c.ruleset == Ruleset::Errata && fl.contains(&"Unusual Looks") && fl.contains(&"Frightening") {
        err(Severity::Error, "Unusual Looks and Frightening can't both be taken".into());
    }
    for f in &c.flaws {
        if rules.flaw(&f.def).is_none() {
            err(Severity::Info, format!("Flaw '{}' is custom (not in these rules)", f.def));
        }
    }
    if cfg.has_roles && c.char_type == CharType::Hero && c.role == Role::None {
        err(Severity::Info, "No Role selected (Roles are optional; skip if your GM doesn't use them)".into());
    }

    s.total_spent = s.cost_package + s.cost_abilities + s.cost_talents + s.cost_powers + s.cost_perks + s.cost_flaws;
    if c.char_type.uses_budget() {
        if s.total_spent > s.hp_available {
            err(Severity::Error, format!("Overspent by {} Hero Points", s.total_spent - s.hp_available));
        } else if s.total_spent < s.hp_available {
            err(Severity::Info, format!("{} Hero Points unspent", s.hp_available - s.total_spent));
        }
    }

    // Derived: Edge.
    let a = |i: usize| c.abilities[i].rank;
    let rank_of = |name: &str| -> Option<i32> {
        c.powers.iter().zip(&s.power_costs).filter(|(p, _)| p.def == name).filter_map(|(_, pc)| pc.rank).max()
    };
    let perception = a(PER).max(rank_of("Danger Sense").unwrap_or(0));
    let second = if c.options.edge_uses_willpower { a(AGI).max(a(WIL)) } else { a(AGI).max(a(INT)) };
    let mut edge = perception + second;
    if has("Lightning Reflexes") {
        edge += 6;
    }
    if let Some(ss) = rank_of("Super Speed") {
        edge = edge.max(ss * 3);
    }
    s.edge = c.options.edge_override.unwrap_or(edge);

    // Health.
    let best = a(MIG).max(a(WIL));
    let mut health = if c.options.more_health { a(TOU) + best } else { (a(TOU) + best + 1) / 2 };
    if c.char_type == CharType::Foe {
        health = (health + 1) / 2;
    }
    s.health = c.options.health_override.unwrap_or(health);

    // Resolve (Heroes only).
    let mut highest = 0;
    let mut from = String::new();
    for (i, t) in c.abilities.iter().enumerate() {
        if t.rank > highest {
            highest = t.rank;
            from = ABILITIES[i].to_string();
        }
    }
    let shapeshift = if cfg.shapeshifting_counts_for_resolve { rank_of("Shapeshifting").unwrap_or(0) } else { 0 };
    if shapeshift > 0 {
        for i in [AGI, MIG, TOU] {
            let v = (a(i) + shapeshift).min(cap);
            if v > highest {
                highest = v;
                from = format!("{} + Shapeshifting", ABILITIES[i]);
            }
        }
    }
    for (p, pc) in c.powers.iter().zip(&s.power_costs) {
        let def = power_def(rules, p);
        let counts = p.resolve_override.unwrap_or_else(|| def.map(|d| d.affects_resolve).unwrap_or(true));
        if !counts {
            continue;
        }
        if let Some(r) = pc.rank
            && r > highest {
                highest = r;
                from = power_label(p);
            }
    }
    s.highest_rank = highest;
    s.highest_from = from;
    s.resolve_base = (2 * (cap - highest)).max(0);
    let determination: i32 = c
        .powers
        .iter()
        .filter(|p| p.def == "Determination")
        .map(|p| p.units.max(0))
        .sum();
    let flaw_bonus = c.flaws.iter().filter(|f| flaw_grants_issue_resolve(rules, f)).count() as i32;
    s.resolve_bonus = determination + flaw_bonus;
    s.resolve = if c.char_type == CharType::Hero {
        Some(c.options.resolve_override.unwrap_or(s.resolve_base + s.resolve_bonus))
    } else {
        None
    };

    issues.sort_by_key(|i| std::cmp::Reverse(i.severity));
    s.issues = issues;
    s
}

pub fn power_label(p: &PowerEntry) -> String {
    let name = p.custom.as_ref().map(|c| c.name.clone()).unwrap_or_else(|| p.def.clone());
    if p.descriptor.trim().is_empty() { name } else { format!("{} ({})", name, p.descriptor.trim()) }
}

/// Short description of the mods on an entry, e.g. "Area, Charges (-2)".
pub fn mods_label(rules: &Rules, def: Option<&PowerDef>, mods: &[ModPick]) -> String {
    mods.iter()
        .map(|m| {
            let md = mod_def(rules, def, m);
            let mut s = m.name.clone();
            let mut extras = Vec::new();
            if let (Some(md), Some(v)) = (md, m.value)
                && (md.cost.choices().is_some() || matches!(md.cost, ModCost::Special { .. })) {
                    extras.push(format!("{v:+}"));
                }
            if m.count > 1 {
                extras.push(format!("×{}", m.count));
            }
            if !m.note.trim().is_empty() {
                extras.push(m.note.trim().to_string());
            }
            if !extras.is_empty() {
                s.push_str(&format!(": {}", extras.join(", ")));
            }
            s
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pw(def: &str, ranks: i32) -> PowerEntry {
        PowerEntry { def: def.into(), ranks, ..Default::default() }
    }

    /// Core rulebook example: Darlene's Low Level hero (book walkthrough in Chapter 2).
    #[test]
    fn darlene_example_costs_100() {
        let mut c = Character::new(Ruleset::Core);
        c.power_level = PowerLevel::Low;
        c.apply_package(Package::Superhero);
        c.abilities[AGI].rank = 6;
        c.abilities[MIG].rank = 8;
        c.abilities[PER].rank = 6;
        c.abilities[TOU].rank = 6;
        let t = |n: &str| TALENTS.iter().position(|x| *x == n).unwrap();
        c.talents[t("Command")].rank = 6;
        c.talents[t("Covert")].rank = 6;
        c.talents[t("Investigation")].rank = 10;
        c.talents[t("Streetwise")].rank = 6;
        c.add_power(pw("Armor", 5));
        c.add_power(PowerEntry { def: "Determination".into(), units: 1, ..Default::default() });
        c.add_power(pw("Regeneration", 0));
        c.perks.push(PerkEntry { def: "Contacts".into(), units: 4, ..Default::default() });
        c.flaws.push(FlawEntry { def: "Compulsion/Severe Compulsion".into(), ..Default::default() });
        c.flaws.push(FlawEntry { def: "Quirk".into(), ..Default::default() });
        c.flaws.push(FlawEntry { def: "Relationship".into(), ..Default::default() });
        let s = evaluate(&c);
        assert_eq!(s.cost_abilities, 14);
        assert_eq!(s.cost_talents, 16);
        assert_eq!(s.power_costs[0].rank, Some(8), "Armor = half Toughness (3) + 5");
        assert_eq!(s.cost_powers, 16);
        assert_eq!(s.cost_perks, 4);
        assert_eq!(s.total_spent, 100);
        assert_eq!(s.remaining(), 0);
    }

    #[test]
    fn per_two_rank_powers_round_up() {
        let mut c = Character::new(Ruleset::Errata);
        c.add_power(pw("Blind", 10));
        c.add_power(pw("Detection", 3));
        let s = evaluate(&c);
        assert_eq!(s.power_costs[0].cost, 5);
        assert_eq!(s.power_costs[1].cost, 2);
    }

    #[test]
    fn baseline_power_rank_and_cost() {
        let mut c = Character::new(Ruleset::Errata);
        c.abilities[AGI].rank = 6;
        c.add_power(pw("Parry", 4)); // 1 HP per 2 ranks, baseline Agility
        let s = evaluate(&c);
        assert_eq!(s.power_costs[0].rank, Some(10));
        assert_eq!(s.power_costs[0].cost, 2);
    }

    #[test]
    fn errata_parrying_changes_strike_cost() {
        let mut c = Character::new(Ruleset::Errata);
        c.abilities[MIG].rank = 4;
        let mut p = pw("Strike", 6);
        let base = power_cost(&c, rules::rules(Ruleset::Errata), &p, 0);
        assert_eq!(base.cost, 3);
        p.mods.push(ModPick::new("Parrying", false));
        let with = power_cost(&c, rules::rules(Ruleset::Errata), &p, 0);
        assert_eq!(with.cost, 6 + 2);
        assert_eq!(with.rank, Some(10));
    }

    #[test]
    fn flat_cons_and_minimum_cost() {
        let mut c = Character::new(Ruleset::Errata);
        let mut p = pw("Blast", 3);
        p.mods.push(ModPick { value: Some(-6), ..ModPick::new("Limited", true) });
        c.add_power(p);
        let s = evaluate(&c);
        assert_eq!(s.power_costs[0].cost, 1, "never below 1 HP");
        assert!(s.power_costs[0].floor_applied);
    }

    #[test]
    fn overkill_halves_might() {
        let rules = rules::rules(Ruleset::Errata);
        let t = TraitEntry { rank: 10, mods: vec![ModPick::new("Overkill", true)], ..Default::default() };
        assert_eq!(trait_cost(rules, &t, 0), 5);
        let core = rules::rules(Ruleset::Core);
        assert_eq!(trait_cost(core, &t, 0), 5);
    }

    #[test]
    fn overkill_raises_errata_trait_cap() {
        let mut c = Character::new(Ruleset::Errata);
        c.abilities[MIG] = TraitEntry { rank: 14, mods: vec![ModPick::new("Overkill", true)], ..Default::default() };
        let s = evaluate(&c);
        assert!(!s.issues.iter().any(|i| i.message.contains("Might") && i.severity == Severity::Error));
        c.abilities[MIG].rank = 16;
        let s = evaluate(&c);
        assert!(s.issues.iter().any(|i| i.message.contains("Might 16d exceeds")));
    }

    #[test]
    fn summoning_costs_differ_by_ruleset() {
        let mut p = pw("Summoning", 4);
        p.threat = 5;
        let c = Character::new(Ruleset::Core);
        assert_eq!(power_cost(&c, rules::rules(Ruleset::Core), &p, 0).cost, 12); // ceil(5/2)=3 per rank
        let c = Character::new(Ruleset::Errata);
        assert_eq!(power_cost(&c, rules::rules(Ruleset::Errata), &p, 0).cost, 10); // 2.5 per rank
    }

    #[test]
    fn derived_values() {
        // Psi Lance sample hero (Standard, 125 HP): Willpower 12d, Toughness 3d => Health 8, Resolve 0 + 2 Plot Hooks.
        let mut c = Character::new(Ruleset::Core);
        c.abilities = std::array::from_fn(|_| TraitEntry { rank: 3, ..Default::default() });
        c.abilities[INT].rank = 6;
        c.abilities[WIL].rank = 12;
        c.flaws.push(FlawEntry { def: "Obligation".into(), ..Default::default() });
        c.flaws.push(FlawEntry { def: "Relationship".into(), ..Default::default() });
        c.flaws.push(FlawEntry { def: "Secret Identity".into(), ..Default::default() });
        let s = evaluate(&c);
        assert_eq!(s.health, 8);
        assert_eq!(s.edge, 3 + 6);
        assert_eq!(s.resolve, Some(2));
    }

    #[test]
    fn resolve_scales_with_highest_rank() {
        let mut c = Character::new(Ruleset::Errata);
        c.abilities[AGI].rank = 9; // cap 12 -> 3 below -> 6 Resolve
        let s = evaluate(&c);
        assert_eq!(s.resolve_base, 6);
        c.add_power(pw("Detection", 12)); // ignored for Resolve in errata
        assert_eq!(evaluate(&c).resolve_base, 6);
        c.add_power(pw("Blast", 11));
        assert_eq!(evaluate(&c).resolve_base, 2);
    }

    #[test]
    fn every_power_costs_without_panicking() {
        for r in Ruleset::ALL {
            let rules = rules::rules(r);
            let c = Character::new(r);
            for d in &rules.data.powers {
                let mut p = pw(&d.name, 5);
                for m in &d.mods {
                    p.mods.push(ModPick::new(&m.name, false));
                }
                let pc = power_cost(&c, rules, &p, 0);
                assert!(!pc.unknown);
                assert!(pc.cost >= 0 || d.mods.iter().any(|m| m.kind == rules::ModKind::Con), "{}", d.name);
            }
        }
    }
}
