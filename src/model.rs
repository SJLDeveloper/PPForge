//! Character data model. Everything here is serialized to the `.ppchar` JSON save format.

use crate::rules::Ruleset;
use serde::{Deserialize, Serialize};

pub const FORMAT_VERSION: u32 = 1;

pub const ABILITIES: [&str; 6] = ["Agility", "Intellect", "Might", "Perception", "Toughness", "Willpower"];
pub const TALENTS: [&str; 12] = [
    "Academics", "Charm", "Command", "Covert", "Investigation", "Medicine", "Professional", "Science", "Streetwise",
    "Survival", "Technology", "Vehicles",
];
pub const MOTIVATIONS: [&str; 12] = [
    "Adventure", "Career", "Devotion", "Idealism", "Justice", "Knowledge", "Nature", "Reason", "Redemption",
    "Responsibility", "Trouble", "Virtue",
];
pub const VILLAIN_MOTIVATIONS: [&str; 12] = [
    "Bond", "Conviction", "Desperation", "Greed", "Insanity/Instinct", "Malice", "Power", "Progress", "Rebellion",
    "Revenge", "Thrills", "Zealotry",
];

pub const AGI: usize = 0;
pub const INT: usize = 1;
pub const MIG: usize = 2;
pub const PER: usize = 3;
pub const TOU: usize = 4;
pub const WIL: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CharType {
    #[default]
    Hero,
    Villain,
    Foe,
    Extra,
}

impl CharType {
    pub const ALL: [CharType; 4] = [CharType::Hero, CharType::Villain, CharType::Foe, CharType::Extra];
    pub fn label(self) -> &'static str {
        match self {
            CharType::Hero => "Hero",
            CharType::Villain => "Villain",
            CharType::Foe => "Foe",
            CharType::Extra => "Extra",
        }
    }
    pub fn uses_budget(self) -> bool {
        self == CharType::Hero
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PowerLevel {
    Street,
    Low,
    #[default]
    Standard,
    High,
    Legendary,
    Iconic,
}

impl PowerLevel {
    pub const ALL: [PowerLevel; 6] =
        [PowerLevel::Street, PowerLevel::Low, PowerLevel::Standard, PowerLevel::High, PowerLevel::Legendary, PowerLevel::Iconic];
    pub fn index(self) -> usize {
        self as usize
    }
    /// 1-based level used by "per power level" costs.
    pub fn ordinal(self) -> i32 {
        self as i32 + 1
    }
    pub fn from_ordinal(n: i32) -> PowerLevel {
        PowerLevel::ALL[(n.clamp(1, 6) - 1) as usize]
    }
    pub fn hero_points(self) -> i32 {
        [75, 100, 125, 150, 175, 200][self.index()]
    }
    pub fn trait_cap(self) -> i32 {
        [8, 10, 12, 16, 20, 24][self.index()]
    }
    pub fn name(self, ruleset: Ruleset) -> &'static str {
        crate::rules::RulesetConfig::of(ruleset).power_level_names[self.index()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Package {
    #[default]
    None,
    Civilian,
    Hero,
    Superhero,
}

impl Package {
    pub const ALL: [Package; 4] = [Package::None, Package::Civilian, Package::Hero, Package::Superhero];
    pub fn label(self) -> &'static str {
        match self {
            Package::None => "No package (buy every rank)",
            Package::Civilian => "Civilian Package (35 HP)",
            Package::Hero => "Hero Package (40 HP)",
            Package::Superhero => "Superhero Package (50 HP)",
        }
    }
    pub fn cost(self) -> i32 {
        match self {
            Package::None => 0,
            Package::Civilian => 35,
            Package::Hero => 40,
            Package::Superhero => 50,
        }
    }
    pub fn ability_base(self) -> i32 {
        match self {
            Package::None => 0,
            Package::Civilian => 2,
            Package::Hero | Package::Superhero => 3,
        }
    }
    pub fn talent_base(self) -> i32 {
        match self {
            Package::None => 0,
            Package::Civilian | Package::Hero => 2,
            Package::Superhero => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[allow(clippy::enum_variant_names)]
pub enum Source {
    #[default]
    Innate,
    Magic,
    Psychic,
    Super,
    Tech,
    Trained,
    /// For Powers like Determination that have no Source.
    NoSource,
}

impl Source {
    pub const ALL: [Source; 7] =
        [Source::Innate, Source::Magic, Source::Psychic, Source::Super, Source::Tech, Source::Trained, Source::NoSource];
    pub fn label(self) -> &'static str {
        match self {
            Source::Innate => "Innate",
            Source::Magic => "Magic",
            Source::Psychic => "Psychic",
            Source::Super => "Super",
            Source::Tech => "Tech",
            Source::Trained => "Trained",
            Source::NoSource => "—",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Role {
    #[default]
    None,
    Blaster,
    Controller,
    Defender,
    Fighter,
    Scout,
    Tank,
}

impl Role {
    pub const ALL: [Role; 7] =
        [Role::None, Role::Blaster, Role::Controller, Role::Defender, Role::Fighter, Role::Scout, Role::Tank];
    pub fn label(self) -> &'static str {
        match self {
            Role::None => "No Role",
            Role::Blaster => "Blaster",
            Role::Controller => "Controller",
            Role::Defender => "Defender",
            Role::Fighter => "Fighter",
            Role::Scout => "Scout",
            Role::Tank => "Tank",
        }
    }
    /// Short paraphrase of what the +1d Role bonus applies to.
    pub fn applies_to(self) -> &'static str {
        match self {
            Role::None => "",
            Role::Blaster => "ranged Attack Rolls",
            Role::Controller => "Attack Rolls with special-effect Powers",
            Role::Defender => "defenses vs special effects; rolls to aid, defend or protect others",
            Role::Fighter => "close combat attacks and active defense vs close attacks",
            Role::Scout => "active defense vs physical/energy attacks; combat stunts",
            Role::Tank => "passive defense vs damaging attacks",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CapMode {
    #[default]
    Hard,
    /// +1d per 10 Hero Points earned through advancement.
    Floating,
    /// GM raises the cap manually (see `cap_bonus`).
    Soft,
}

impl CapMode {
    pub const ALL: [CapMode; 3] = [CapMode::Hard, CapMode::Floating, CapMode::Soft];
    pub fn label(self) -> &'static str {
        match self {
            CapMode::Hard => "Hard",
            CapMode::Floating => "Floating (+1d / 10 HP earned)",
            CapMode::Soft => "Soft (GM sets)",
        }
    }
}

/// A Pro or Con applied to a Trait.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct ModPick {
    pub name: String,
    /// Comes from the generic Pros & Cons list rather than the Power's own list.
    pub generic: bool,
    /// Chosen value for choice/range/special modifiers.
    pub value: Option<i32>,
    /// Number of applications for repeatable modifiers (min 1).
    pub count: u32,
    /// Player's description (e.g. the Limited condition).
    pub note: String,
}

impl ModPick {
    pub fn new(name: &str, generic: bool) -> Self {
        ModPick { name: name.to_string(), generic, value: None, count: 1, note: String::new() }
    }
    pub fn times(&self) -> i32 {
        self.count.max(1) as i32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TraitEntry {
    pub rank: i32,
    pub source: Source,
    pub mods: Vec<ModPick>,
    pub note: String,
}

impl Default for TraitEntry {
    fn default() -> Self {
        TraitEntry { rank: 2, source: Source::Innate, mods: Vec::new(), note: String::new() }
    }
}

/// A Power the character has bought.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PowerEntry {
    pub uid: u64,
    /// Name of the PowerDef in the ruleset, or "Custom Power".
    pub def: String,
    /// Optional descriptor, e.g. "Fire" for Blast (Fire).
    pub descriptor: String,
    pub source: Source,
    /// Purchased ranks. For Power Rank powers this is the rank; for Baseline Rank powers it's added to the baseline.
    pub ranks: i32,
    /// Index into a Choice cost (e.g. Omni-Power Narrow/Broad).
    pub option: usize,
    /// Count for per-unit and per-power-level costs (power level is 1..6).
    pub units: i32,
    /// Minion Threat for Summoning.
    pub threat: i32,
    /// Chosen baseline trait for "choice:" and "special" baselines (Boost, Expertise, Strike).
    pub base_trait: String,
    /// Boost: Hero Points per rank of the boosted Trait.
    pub boost_cost: i32,
    pub mods: Vec<ModPick>,
    pub notes: String,
    /// GM-approved manual cost (Rule Uno).
    pub cost_override: Option<i32>,
    /// Force whether this Power counts toward the Resolve calculation.
    pub resolve_override: Option<bool>,
    /// For "Custom Power" entries.
    pub custom: Option<crate::rules::PowerDef>,
}

impl Default for PowerEntry {
    fn default() -> Self {
        PowerEntry {
            uid: 0,
            def: String::new(),
            descriptor: String::new(),
            source: Source::Super,
            ranks: 0,
            option: 0,
            units: 1,
            threat: 4,
            base_trait: String::new(),
            boost_cost: 1,
            mods: Vec::new(),
            notes: String::new(),
            cost_override: None,
            resolve_override: None,
            custom: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PerkEntry {
    pub def: String,
    pub note: String,
    pub option: usize,
    pub units: i32,
    pub cost_override: Option<i32>,
}

impl Default for PerkEntry {
    fn default() -> Self {
        PerkEntry { def: String::new(), note: String::new(), option: 0, units: 1, cost_override: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct FlawEntry {
    pub def: String,
    pub note: String,
    /// Override: does this Flaw grant 1 Resolve at the start of every issue (Plot Hook/Condition)?
    pub issue_resolve: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Award {
    pub date: String,
    pub amount: i32,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Connection {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Story {
    pub hero_name: String,
    pub real_name: String,
    pub player: String,
    pub concept: String,
    pub motivation: String,
    pub costume: [String; 3],
    pub civilian: [String; 3],
    pub quote: String,
    pub connections: Vec<Connection>,
    pub details: [String; 3],
    pub origin: String,
    pub gear: String,
    pub notes: String,
}

impl Default for Story {
    fn default() -> Self {
        Story {
            hero_name: "New Hero".into(),
            real_name: String::new(),
            player: String::new(),
            concept: String::new(),
            motivation: String::new(),
            costume: Default::default(),
            civilian: Default::default(),
            quote: String::new(),
            connections: vec![Connection::default(), Connection::default(), Connection::default()],
            details: Default::default(),
            origin: String::new(),
            gear: String::new(),
            notes: String::new(),
        }
    }
}

/// Table-level options the GM may switch on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct Options {
    /// Optional "More Health" rule: Health is the sum of the two Abilities rather than their average.
    pub more_health: bool,
    /// Lethal Damage rules in play (affects Hard to Kill / Immortality default costs).
    pub lethal_damage: bool,
    /// Errata Ch. 4 Edge wording (Agility or Willpower).
    pub edge_uses_willpower: bool,
    /// Replace the power level's starting Hero Points.
    pub starting_hp_override: Option<i32>,
    pub cap_mode: CapMode,
    /// Extra Trait Cap from a soft cap.
    pub cap_bonus: i32,
    /// Manual overrides for derived values.
    pub edge_override: Option<i32>,
    pub health_override: Option<i32>,
    pub resolve_override: Option<i32>,
    /// Character is past creation (enables >3 Flaws at 3 HP each without a warning).
    pub creation_complete: bool,
}

/// In-play status tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct PlayState {
    pub current_health: Option<i32>,
    pub current_resolve: Option<i32>,
    pub defining_moments: u32,
    pub issues_played: u32,
    pub session_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Character {
    pub format_version: u32,
    pub ruleset: Ruleset,
    pub char_type: CharType,
    pub power_level: PowerLevel,
    pub role: Role,
    pub package: Package,
    pub abilities: [TraitEntry; 6],
    pub talents: [TraitEntry; 12],
    pub powers: Vec<PowerEntry>,
    pub perks: Vec<PerkEntry>,
    pub flaws: Vec<FlawEntry>,
    pub awards: Vec<Award>,
    pub story: Story,
    pub options: Options,
    pub play: PlayState,
    pub next_uid: u64,
}

impl Default for Character {
    fn default() -> Self {
        let abilities: [TraitEntry; 6] = Default::default();
        let talents: [TraitEntry; 12] =
            std::array::from_fn(|_| TraitEntry { source: Source::Trained, ..Default::default() });
        Character {
            format_version: FORMAT_VERSION,
            ruleset: Ruleset::Errata,
            char_type: CharType::Hero,
            power_level: PowerLevel::Standard,
            role: Role::None,
            package: Package::None,
            abilities,
            talents,
            powers: Vec::new(),
            perks: Vec::new(),
            flaws: Vec::new(),
            awards: Vec::new(),
            story: Story::default(),
            options: Options::default(),
            play: PlayState::default(),
            next_uid: 1,
        }
    }
}

impl Character {
    pub fn new(ruleset: Ruleset) -> Self {
        Character { ruleset, ..Default::default() }
    }

    pub fn display_name(&self) -> String {
        let n = self.story.hero_name.trim();
        if n.is_empty() { "Unnamed".into() } else { n.to_string() }
    }

    pub fn add_power(&mut self, mut p: PowerEntry) -> u64 {
        p.uid = self.next_uid;
        self.next_uid += 1;
        let uid = p.uid;
        self.powers.push(p);
        uid
    }

    pub fn earned_hp(&self) -> i32 {
        self.awards.iter().map(|a| a.amount).sum()
    }

    /// Apply a package: raise traits to the package baseline.
    pub fn apply_package(&mut self, p: Package) {
        self.package = p;
        for a in self.abilities.iter_mut() {
            a.rank = a.rank.max(p.ability_base());
        }
        for t in self.talents.iter_mut() {
            t.rank = t.rank.max(p.talent_base());
        }
    }

    /// Switch rulesets, renaming Powers whose names differ. Returns names that have no equivalent.
    pub fn convert_to(&mut self, to: Ruleset) -> Vec<String> {
        if to == self.ruleset {
            return Vec::new();
        }
        let rules = crate::rules::rules(to);
        let renames = rules.config().renames_from_other;
        let mut missing = Vec::new();
        for p in self.powers.iter_mut() {
            if let Some((_, new)) = renames.iter().find(|(old, _)| *old == p.def) {
                p.def = new.to_string();
            }
            if p.custom.is_none() && rules.power(&p.def).is_none() {
                missing.push(p.def.clone());
            }
        }
        if to == Ruleset::Core {
            self.role = Role::None;
        }
        self.ruleset = to;
        missing
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> serde_json::Result<Character> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_json() {
        let mut c = Character::new(Ruleset::Core);
        c.story.hero_name = "Test".into();
        c.add_power(PowerEntry { def: "Blast".into(), ranks: 8, ..Default::default() });
        let s = c.to_json().unwrap();
        let back = Character::from_json(&s).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn convert_renames_powers() {
        let mut c = Character::new(Ruleset::Core);
        c.add_power(PowerEntry { def: "Evasion".into(), ranks: 2, ..Default::default() });
        c.add_power(PowerEntry { def: "Specialty".into(), ..Default::default() });
        let missing = c.convert_to(Ruleset::Errata);
        assert_eq!(c.powers[0].def, "Dodge");
        assert_eq!(missing, vec!["Specialty".to_string()]);
    }

    #[test]
    fn old_files_with_missing_fields_load() {
        let c = Character::from_json(r#"{"ruleset":"Core","story":{"hero_name":"X"}}"#).unwrap();
        assert_eq!(c.story.hero_name, "X");
        assert_eq!(c.abilities.len(), 6);
    }
}
