#!/usr/bin/env python3
"""Build errata_rules.json (Combined Updates) from hand-reviewed data. Summaries are paraphrased."""
import json, sys

def R_power(): return {"kind": "power"}
def R_def(): return {"kind": "default"}
def R_spec(): return {"kind": "special"}
def R_base(b, half=False):
    d = {"kind": "baseline", "base": b}
    if half: d["half"] = True
    return d
def C_rank(v): return {"kind": "per_rank", "value": v}
def C_two(): return {"kind": "per_two_ranks"}
def C_flat(v): return {"kind": "flat", "value": v}
def C_ppl(v): return {"kind": "per_power_level", "value": v}
def C_unit(v, u): return {"kind": "per_unit", "value": v, "unit": u}
def C_choice(opts, per_rank): return {"kind": "choice", "options": [{"label": l, "value": v} for l, v in opts], "per_rank": per_rank}
def C_special(n): return {"kind": "special", "note": n}
def M(name, typ, cost, summary, repeatable=False, **extra):
    d = {"name": name, "type": typ, "cost": cost, "repeatable": repeatable, "summary": summary}
    d.update(extra)
    return d
def flat(v): return {"kind": "flat", "value": v}
def prank(v): return {"kind": "per_rank", "value": v}
def ptwo(v): return {"kind": "per_two_ranks", "value": v}
def pppl(v): return {"kind": "per_power_level", "value": v}
def choice(*o, per_rank=False):
    d = {"kind": "choice", "options": list(o)}
    if per_rank: d["per_rank"] = True
    return d
def special(n): return {"kind": "special", "note": n}

powers = []
def P(name, rng_, rank, cost, summary, mods=(), resolve=True, category=None, **extra):
    d = {"name": name, "category": category, "range": rng_, "rank": rank, "cost": cost,
         "affects_resolve": resolve, "summary": summary, "mods": list(mods)}
    d.update(extra)
    powers.append(d)

ALSO_SRC = "Works on one more Source per purchase."
P("Absorption", "Touch", R_power(), C_rank(2),
  "Steal a target's Traits of one Source (knocking them out) and use them for the duration; -1d per extra target held; needs skin contact.",
  [M("Also X", "pro", prank(1), ALSO_SRC, True),
   M("Only X", "con", choice(-2, -4), "-2: only Abilities, Talents or Powers; -4: one specific Trait. Per Source.", True)])
P("Adaptation", "Self", R_def(), C_flat(12), "Body adapts instantly to survive, move and sense in hostile environments.")
P("Alternate Form", "Self", R_def(), C_ppl(4),
  "A second form built as a separate Hero at a power level no higher than yours; every form pays the total cost; one shared Resolve pool (use the strongest form).",
  [M("Independent Forms", "con", pppl(-1), "Each form has its own identity, memory and personality.")])
P("Animal Control", "Ranged", R_power(), C_rank(1), "Roll against an animal's Willpower to command it for the duration.",
  [M("Calling", "pro", flat(2), "Summon animals from Extreme Range; duration starts when they arrive."),
   M("Only X", "con", choice(-2, -4), "One category of animals; -4 if very narrow or of limited use.")])
P("Animal Empathy", "Self", R_def(), C_flat(1), "Animals are disinclined to attack you and may be dissuaded.")
P("Animal Forms", "Self", R_power(), C_rank(2),
  "Turn into animals you have seen, gaining their physical Abilities and Powers up to your rank.",
  [M("Enhanced", "pro", flat(4), "Use your rank in place of the animal's single highest Trait."),
   M("Only X", "con", choice(-2, -4), "One category of animals; -4 for a single animal or very limited group.")])
P("Animal Mimicry", "Self", R_power(), C_rank(2),
  "Gain an animal's Powers and better physical Abilities, capped at your rank.",
  [M("Enhanced", "pro", flat(4), "Use your rank in place of the animal's single highest Trait."),
   M("Selection", "pro", flat(4), "Mimic several animals at once, -1d per extra animal.")])
P("Animal Speech", "Self", R_def(), C_flat(3), "Communicate with animals.")
P("Animation", "Ranged", R_power(), C_rank(2),
  "Bring images or objects to life for the scene; one Trait at full rank, others at half; -1d per extra animated object.")
P("Armor", "Self", R_base("Toughness", True), C_rank(1), "Passive defense against physical and energy attacks.",
  [M("Activated", "con", flat(-1), "Switched on as a free action; fails if you are knocked out or paralyzed."),
   M("Hardened", "pro", flat(6), "Immune to Armor Piercing and Penetrating."),
   M("Reactive", "con", flat(-2), "A shield or bracers; unusable when surprised, immobilized or unconscious.")])
P("Astral Projection", "Self", R_def(), C_flat(12),
  "Leave your body as an astral form with Invisibility, Phasing (Spirit), Astral Sight and Willpower-rank flight.")
P("Attuned", "Self", R_def(), C_flat(3), "Sense major disturbances in an attuned area or aspect of nature; spend Resolve for details.")
P("Aura", "Self", R_power(), C_rank(2),
  "Your body deals energy damage to anyone touching or striking you, and can make close attacks; destroys mundane weapons at 6d/9d.",
  [M("Spines", "con", flat(-2), "Quills deal physical damage and give no protection from weapons.")])
P("Banish", "Ranged", R_power(), C_rank(1), "Send a target to another time/world/dimension for the duration; permanent if sent home and defeated.")
P("Blast", "Ranged", R_power(), C_rank(1), "A damaging ranged attack of a chosen type.",
  [M("Block", "pro", flat(2), "Use Blast as an active defense against ranged physical/energy attacks (not area)."),
   M("Block Missiles", "pro", flat(1), "Use Blast to defend against thrown weapons and primitive projectiles."),
   M("Marksmanship", "con", flat(-1), "Superhuman skill with mundane ranged weapons; uses the weapon's damage and properties."),
   M("Melee Weapon", "pro", flat(2), "Form a melee weapon; Blast becomes an active defense against close attacks."),
   M("Subdual", "con", flat(0), "Deals subdual damage.")])
P("Blending", "Self", R_def(), C_flat(3), "+3d to Covert to stay unseen; can hide in plain sight while still at -3d.")
P("Blind", "Ranged", R_power(), C_two(), "Blind a living target (-3d attack and active defense) for the duration.",
  [M("Deafen", "con", flat(-1), "Deafens instead of blinding.")])
P("Blind Fighting", "Self", R_def(), C_flat(3), "No penalties for darkness or unseen opponents.")
P("Blink", "Touch", R_base("avg:Agility+Might"), C_rank(1),
  "Use a linked Self Power to disorient foes: close attacks and active defense; can be Strike's baseline; subdual.",
  [M("Lethal", "pro", flat(1), "Can deal ordinary damage.")])
P("Boost", "Self", R_base("special"), C_special("costs the same per rank as the boosted Trait, which is its baseline"),
  "Replaces one chosen Ability, Talent or Power with this rank while active; needs limiting Cons.",
  [M("Rage", "con", flat(-2), "Triggered by anger; Intellect and Talents drop to 1d while active.")])
P("Buff", "Zone", R_def(), C_flat(12), "Spend 1 Resolve: you and allies in range gain +1d to all rolls for the scene; doesn't stack.",
  [M("Greater", "pro", flat(6), "Spend up to 3 Resolve for up to +3d.")])
P("Clairvoyance", "Self", R_def(), C_flat(9), "See distant places or familiar subjects; Inhuman (6) Willpower roll to cross dimensions.")
P("Cloud Minds", "Ranged", R_power(), C_rank(1), "Become undetectable to living minds within range; costs an action per page.")
P("Communications", "Self", R_def(), C_flat(3), "Send, receive and store transmissions worldwide without infrastructure.")
P("Constructs", "Ranged", R_power(), C_rank(3),
  "Create solid force constructs with rank Structure and Might; attack, grapple, shield; -1d per extra construct.",
  [M("Devices", "pro", prank(2), "Constructs can work like machines (a broad Omni-Power).")])
P("Danger Sense", "Self", R_base("Perception"), C_rank(1),
  "Sense immediate danger; replaces Perception for Edge and danger detection; active defense even when surprised.")
P("Darkness", "Zone", R_def(), C_flat(6), "Fill the area with an obscuring medium; others suffer -3d attack and active defense.",
  [M("Impenetrable", "pro", flat(3), "Only Astral Sight or True Sight can see through it.")])
P("Dazzle", "Zone", R_power(), C_rank(1), "Blinding flash: targets lose their next turn, then are blinded for the duration.",
  [M("Flash-Bang", "pro", flat(2), "Also deafens; affects blind targets unless also deaf.")])
P("Deflection", "Self", R_power(), C_rank(2), "Active defense against ranged attacks; redirect them as a free action if unharmed.",
  [M("Energy/Physical", "con", prank(-1), "Works only against energy or only physical attacks."),
   M("Only X", "con", flat(-4), "Only one type of projectile or energy."),
   M("Riposte", "con", flat(0), "Works against close combat attacks instead of ranged.")])
P("Density", "Self", R_power(), C_rank(1), "While dense, use the rank for Might, Toughness and Armor; weight rank equals Density.")
P("Detection", "Ranged", R_power(), C_two(), "Detect and locate a chosen phenomenon out to Extreme Range.",
  [M("Unlimited", "pro", flat(2), "Works at any distance, even interstellar.")], resolve=False)
P("Determination", "Self", R_spec(), C_unit(5, "extra Resolve per issue"),
  "Start each issue with 1 extra Resolve per 5 Hero Points spent; Heroes only; cannot be affected by other Powers.", resolve=False)
P("Dimensional Travel", "Self", R_def(), C_flat(6), "Travel to other dimensions; Willpower uses per issue free, then 1 Resolve each.",
  [M("Portal", "pro", flat(2), "Open portals others can use.")])
P("Dispel", "Ranged", R_power(), C_two(), "End the duration of ongoing Powers of one Source; active defense against them.",
  [M("Also X", "pro", ptwo(1), ALSO_SRC, True)])
P("Dodge", "Self", R_base("Agility"), C_two(), "Use instead of Agility for active defense against ranged attacks.")
P("Doppelganger", "Self", R_def(), C_flat(6), "Become an exact copy of any roughly humanoid person you have seen.")
P("Drain", "Ranged", R_power(), C_rank(1), "Drop a target's Traits of one Source to 0d (Powers stop) for the duration.",
  [M("Also X", "pro", flat(6), ALSO_SRC, True),
   M("Only X", "con", choice(-1, -2), "-1: only Abilities, Talents or Powers; -2: one specific Trait. Per Source.", True)])
P("Duplication", "Self", R_def(), C_flat(25), "Create a duplicate of yourself sharing Resolve; vanishes when either is defeated.")
P("Elemental Control", "Ranged", R_power(), C_rank(3), "Control one element to imitate other Powers at this rank (a narrow Omni-Power).",
  [M("Non-Generative", "con", prank(-1), "Can only control element already present."),
   M("Stable Matter", "pro", prank(1), "Create lasting matter by spending Resolve equal to its weight rank.")])
P("Emotion Control", "Ranged", R_power(), C_rank(1), "Impose an emotion or mental state for the duration (vs Willpower).",
  [M("Only X", "con", flat(-2), "One specific emotion."),
   M("Pheromones", "con", flat(-2), "Chemical; resisted with Toughness or Willpower; non-breathers immune.")])
P("Energy Absorption", "Special", R_power(), C_rank(1),
  "Absorb one energy type from a source or attack; spend it next turn to boost a physical Ability, heal, Blast, or gain +1d.",
  [M("Also X", "pro", flat(4), "One extra energy type (not kinetic).", True),
   M("Kinetic", "pro", prank(1), "Can absorb kinetic energy from direct attacks."),
   M("Reserve", "pro", flat(4), "Hold absorbed energy for later in the scene; 1 Resolve to carry over.")])
P("Ensnare", "Ranged", R_power(), C_rank(1), "Entangle a target so they cannot take physical actions for the duration.",
  [M("Immobilize", "con", flat(-2), "Only prevents movement."),
   M("Line", "pro", flat(2), "Reel in ensnared targets.")])
P("Expertise", "Self", R_base("special"), C_two(),
  "A narrow specialty that replaces its parent Ability or Talent for rolls and benchmarks.", resolve=False)
P("Extra Limbs", "Self", R_def(), C_flat(3), "Extra manipulative limbs; +1d to grappling and similar actions.")
P("Fast Healing", "Self", R_def(), C_flat(1), "+3d to Toughness rolls for natural healing; not with Regeneration.")
P("Flight", "Self", R_power(), C_rank(1), "Travel Power: fly; halved indoors.",
  [M("Gliding", "con", flat(-2), "Must descend a story per page unless riding updrafts."),
   M("Levitation", "con", flat(-4), "Only up, down or hover."),
   M("Slides", "con", flat(-2), "Ride created slides; must stay near solid structures."),
   M("Spaceflight", "pro", flat(2), "Faster-than-light travel in space."),
   M("Wings", "con", flat(-2), "No flight in confined spaces, thin air or vacuum.")])
P("Force Field", "Zone", R_power(), C_rank(1),
  "Field repelling physical/energy damage and Toughness-resisted effects; collapses when penetrated; -1d per extra field.",
  [M("Absolute", "pro", prank(1), "Also blocks psychic attacks and Willpower-resisted effects."),
   M("Barrier", "pro", flat(6), "Create walls/bubbles with Structure equal to rank."),
   M("Permeable", "pro", flat(6), "Lets attacks pass one way."),
   M("Psychic", "con", flat(-2), "Blocks psychic/Willpower attacks instead of physical."),
   M("Stable", "pro", prank(1), "Does not collapse when penetrated.")])
P("Form, Energy", "Self", R_power(), C_rank(3), "Become energy: Aura at this rank, 3d Flight, immune to physical and energy attacks.",
  [M("Also X", "pro", flat(2), "One more energy type.", True)], category="Form")
P("Form, Gaseous", "Self", R_def(), C_flat(6), "Become gas: 1d Flight, immune to physical and energy attacks.", category="Form")
P("Form, Liquid", "Self", R_def(), C_flat(12), "Become liquid or particles: immune to physical attacks, flow through openings.",
  [M("Viscous", "con", flat(-2), "Easier to trap or contain.")], category="Form")
P("Form, Solid", "Self", R_power(), C_rank(2), "Become solid matter: use rank for Might, Toughness and Armor.", category="Form")
P("Gestalt", "Self", R_def(), C_unit(3, "purchase (2 alter egos; +1 per extra)"),
  "Requires Alter Ego: two alter egos merge into your powered form, sharing Resolve; each extra alter ego costs 3 more.")
P("Growth", "Self", R_power(), C_rank(1), "Grow larger; use rank for Might, Toughness and Armor; weight rank equals rank from 4d.")
P("Hard to Kill", "Self", R_def(), C_choice([("Standard", 1), ("Lethal Damage rules", 6)], False), "Only extreme measures can kill you.")
P("Healing", "Touch", R_power(), C_rank(1), "Heal up to rank damage per day by touch; cure mundane disease for 1 Resolve.",
  [M("Repair", "con", flat(-2), "Repairs machines instead of healing living beings.")])
P("Hibernation", "Self", R_def(), C_flat(1), "Enter suspended animation at will, needing nothing and aging slowly.")
P("Hyper Breath", "Special", R_power(), C_rank(1), "Gale-force breath pulls or pushes targets; Margin 3+ knocks them down.",
  [M("Freezing", "pro", prank(1), "Freeze targets, mimicking Ensnare.")])
P("Illusions", "Zone", R_power(), C_rank(1), "Create illusory sights and sounds; see through with Perception vs Illusions.",
  [M("Mental", "pro", prank(2), "Illusions in minds; resisted with Willpower; can inflict illusory effects and harm.")])
P("Immortality", "Self", R_def(), C_choice([("Standard", 3), ("Lethal Damage rules", 9)], False), "You don't stay dead and don't age.",
  [M("Vulnerable", "con", choice(0, -2), "Can be killed in a specific way; -2 only with Lethal Damage rules.")])
P("Immunity", "Self", R_def(), C_unit(3, "immunity"), "Immune to chosen attacks, energies, hazards or effects; 3 HP each.")
P("Inanimate", "Self", R_spec(), C_flat(9), "Not alive: immune to life-only effects, Life Support and Toxins; still vulnerable to psychic attacks.",
  [M("Mindless", "pro", flat(9), "Also immune to psychic attacks and effects.")])
P("Invisibility", "Self", R_def(), C_flat(9), "Turn invisible; enemies at -3d (or -2d if they can detect you otherwise).",
  [M("Cloaking", "pro", flat(3), "Also undetectable to Radar and Super Senses."),
   M("Jamming", "con", flat(-3), "Only undetectable to Radar and Super Senses."),
   M("Silence", "con", flat(-3), "Only silent and undetectable by hearing."),
   M("Stealthy", "pro", flat(3), "Also silent.")])
P("Irritant", "Zone", R_power(), C_rank(1), "Fill the area with an irritant: -2d to others' rolls unless their Toughness or Willpower exceeds the rank.")
P("Languages", "Self", R_def(), C_unit(1, "block of (Intellect) languages"), "Speak extra languages equal to Intellect per Hero Point.",
  [M("Hyper Translation", "pro", flat(2), "Instantly fluent in any language you encounter."),
   M("Polyglot", "pro", flat(1), "Learn new languages within days.")])
P("Leadership", "Self", R_def(), C_flat(6), "Each ally who can see and hear you gets one free reroll per combat.")
P("Leaping", "Self", R_base("Might", True), C_rank(1), "Travel Power: leap great distances; halved indoors; active defense against falls.")
P("Life Drain", "Touch", R_power(), C_rank(2), "Drain life: damage by Margin, heal half the Margin.",
  [M("Psychic", "con", flat(0), "Resisted with Willpower instead of Toughness."),
   M("Vampirism", "pro", flat(2), "Deals physical damage, can harm objects.")])
P("Light/Sound/Effect", "Special", R_def(), C_flat(3), "Generate harmless light, simple sounds, or another innocuous effect.")
P("Lightning Reflexes", "Self", R_def(), C_flat(3), "+6 Edge and instant weapon draws; doesn't stack with Super Speed.",
  [M("Fighting", "con", flat(-1), "Requires a close attack on the first page."),
   M("Shooting", "con", flat(-1), "Requires a ranged attack on the first page.")])
P("Luck", "Self", R_power(), C_rank(2), "Gain Luck dice equal to rank each issue to add to either side of any roll involving you.",
  [M("Probability Control", "pro", flat(4), "Use Luck on anyone you can see within Distant Range."),
   M("Unbelievable", "pro", prank(1), "Spend Luck dice on spectacular lucky breaks.")])
P("Machine Control", "Ranged", R_power(), C_rank(1), "Remotely control machines and devices; -1d per extra machine.",
  [M("Interface", "con", flat(-2), "Computers and anything they control only.")])
P("Martial Arts", "Touch", R_base("avg:Agility+Might"), C_rank(1),
  "Unarmed attacks, active defense, grappling and stunts in close combat; subdual unless armed.",
  [M("Block", "pro", flat(2), "Active defense against ranged physical/energy attacks (not area)."),
   M("Block Missiles", "pro", flat(1), "Active defense against slow projectiles."),
   M("Lethal", "pro", flat(1), "Can deal ordinary damage."),
   M("Weaponry", "pro", flat(2), "Ignore the Gear Limit with melee weapons (total capped at Trait Cap).")])
P("Master of Disguise", "Self", R_def(), C_flat(3), "Create convincing disguises quickly and mimic voices and mannerisms.")
P("Matter Chameleon", "Self", R_power(), C_rank(2), "Take on the properties of touched objects (like Solid Form, capped by their Structure).")
P("Mimicry", "Touch", R_power(), C_rank(2), "Copy a target's Traits of one Source for the scene, capped at your rank.",
  [M("Also X", "pro", prank(1), ALSO_SRC, True),
   M("Only X", "con", choice(-2, -4), "-2: only Abilities, Talents or Powers; -4: one specific Trait. Per Source.", True),
   M("Selection", "pro", flat(4), "Mimic several targets at once, -1d per extra target.")])
P("Mind Blast", "Ranged", R_power(), C_rank(1), "Psychic damage against living targets (vs Willpower).")
P("Mind Control", "Ranged", R_power(), C_rank(2), "Command a living being for the duration; implant triggered suggestions for 1 Resolve.",
  [M("Hypnosis", "con", flat(-2), "Requires calm face-to-face conversation."),
   M("Pheromones", "con", flat(-2), "Chemical; resisted with Toughness or Willpower.")])
P("Nullify", "Zone", R_power(), C_rank(2), "Dampening field nullifies Traits of one Source up to your rank; you must concentrate.",
  [M("Also X", "pro", prank(1), ALSO_SRC, True)])
P("Object Forms", "Self", R_power(), C_rank(2), "Turn into shapes, tools, weapons, armor, or machines up to your rank.")
P("Omni-Power", "Special", R_power(), C_choice([("Narrow", 3), ("Broad", 5)], True),
  "Versatile Power imitating other Powers at its rank; -1d per extra use on the same page.")
P("Parry", "Self", R_base("Agility"), C_two(), "Use instead of Agility for active defense against close attacks.")
P("Phasing", "Self", R_def(), C_flat(9), "Become intangible, immune to physical and energy attacks.",
  [M("Spirit", "pro", flat(3), "Become a spirit; only psychic force fields hold you.")])
P("Plasticity", "Self", R_power(), C_rank(2), "Rubbery body: reshape, squeeze, passive defense vs physical attacks, use for Might when grappling.")
P("Polymorph", "Ranged", R_power(), C_rank(1), "Turn a living target into something harmless for the duration.",
  [M("Psychic", "con", flat(0), "Resisted with Willpower instead of Toughness.")])
P("Port", "Ranged", R_power(), C_rank(1), "Teleport a target 50 ft (1 zone) per point of Margin.")
P("Portable Storehouse", "Self", R_def(), C_flat(6), "Summon mundane gear from a hidden store; lasts the scene unless you have Resources/Wealth.")
P("Possession", "Ranged", R_power(), C_rank(1), "Take over a living body for the duration, using its physical Abilities.",
  [M("Meld", "pro", flat(4), "Your body merges into the target.")])
P("Precognition", "Self", R_def(), C_flat(12), "Spend escalating Resolve to undo the last turn and act first.")
P("Preparation", "Self", R_spec(), C_flat(6), "Make Lucky Breaks far more potent when justified by planning.", resolve=False)
P("Psi-Screen", "Self", R_base("Willpower"), C_rank(1), "Replaces Willpower against psychic attacks and telepathic intrusion.",
  [M("Cloak Mind", "pro", flat(2), "Hide your mental presence from telepaths.")])
P("Psychometry", "Touch", R_def(), C_flat(9), "Read the history of touched objects; 1 Resolve for in-depth information.")
P("Quick Change", "Self", R_def(), C_flat(1), "Change into or out of costume instantly.",
  [M("Wardrobe", "pro", flat(2), "Change any attire and grooming instantly.")])
P("Radar", "Self", R_def(), C_flat(3), "3-D sense of surroundings regardless of light; can't read flat images.",
  [M("Ping", "con", flat(-1), "Announces you to others with Radar of the same Source."),
   M("Sonar", "con", flat(-1), "Works only underwater."),
   M("Tremor Sense", "con", flat(-1), "Senses only things touching or vibrating the ground.")])
P("Regeneration", "Self", R_def(), C_flat(12), "Heal 1 damage per minute; revive at 1 Health.",
  [M("Fast", "pro", flat(6), "Heal 1 per page."),
   M("Slow", "con", flat(-6), "Heal 1 per hour.")])
P("Resistance", "Self", R_base("Toughness"), C_two(), "Replaces Toughness against mundane hazards like radiation, temperature and toxins.")
P("Running", "Self", R_base("fixed:3"), C_rank(1), "Travel Power: run fast (3d baseline is normal human speed).",
  [M("Hover", "pro", flat(2), "Glide just above the ground.")])
P("Separation", "Self", R_def(), C_flat(3), "Detach body parts that act independently.")
P("Shapeshifting", "Self", R_power(), C_rank(2),
  "Reshape your body: rank Hero Points to raise Agility/Might/Toughness and buy physical Powers; boosts effective ranks for Resolve.")
P("Shockwave", "Zone", R_power(), C_two(), "Concussive wave pushes targets and imposes -1d; Margin 3+ knocks them prone.",
  [M("Focused", "pro", flat(2), "Single-target shockwave out to Distant Range.")])
P("Shrinking", "Self", R_power(), C_rank(2), "Shrink; use rank for Agility, gain Leaping and Strike at half rank.")
P("Slay", "Touch", R_power(), C_rank(2), "Touch deals 2 damage per Margin to living targets, ignoring Armor.",
  [M("Psychic", "con", flat(0), "Resisted with Willpower instead of Toughness.")])
P("Slick", "Zone", R_power(), C_rank(1), "Make the ground slippery; Agility vs Slick to stay standing.")
P("Speak with Dead", "Self", R_power(), C_rank(2), "Ask the dead up to rank questions.")
P("Spinning", "Self", R_power(), C_rank(1), "Spin for close attacks, all active defenses and throwing; can be Strike's baseline; subdual.",
  [M("Lethal", "pro", flat(1), "Can deal ordinary damage.")])
P("Star Gate", "Self", R_def(), C_flat(3), "Travel to distant worlds; Willpower uses per issue free, then 1 Resolve each.",
  [M("Portal", "pro", flat(2), "Open portals others can use.")])
P("Stretching", "Self", R_def(), C_choice([("Close Range", 2), ("Distant Range", 4), ("Extreme Range", 6)], False),
  "Reach and attack/grapple targets within the purchased range.")
P("Strike", "Touch", R_base("choice:Might|Martial Arts|Blink|Spinning"), C_two(), "A close combat attack (claws, energy fists, weapon).",
  [M("Grappling", "pro", flat(2), "Use Strike for grabs, holds and escapes."),
   M("Parrying", "pro", flat(2), "Also an active defense vs close attacks, but Strike then costs 1 HP per rank.", override_cost=C_rank(1)),
   M("Psychic", "con", flat(0), "Deals psychic damage; only Willpower/Psi-Screen defend."),
   M("Reach/Throw", "pro", flat(2), "Attack within Close Range."),
   M("Subdual", "con", flat(0), "Always deals subdual damage."),
   M("Subdual (Optional)", "pro", flat(1), "May choose to deal subdual damage."),
   M("Sweep", "pro", flat(4), "Area attack in front of you.")])
P("Stun", "Ranged", R_power(), C_rank(1), "Incapacitate a living target for the duration.",
  [M("EMP", "con", flat(-2), "Affects Tech Inanimate beings and electronics instead."),
   M("Psychic", "con", flat(0), "Resisted with Willpower instead of Toughness.")])
P("Summoning", "Special", R_power(), C_special("errata: half the Minions' Threat Hero Points per rank"),
  "Summon Minions per scene by rank with a Threat chosen at purchase (max Trait Cap -4d).",
  [M("Animals", "pro", flat(6), "Minions are animals."),
   M("Automatons", "pro", flat(9), "Minions have Inanimate (Mindless)."),
   M("Duplicates", "pro", flat(9), "Minions are duplicates sharing your abilities."),
   M("Lieutenants", "pro", flat(4), "Trade numbers for higher Threat."),
   M("Shooters", "pro", flat(6), "Minions can attack at range."),
   M("Unique", "pro", choice(3, 6, 9), "Minions have minor/significant/major special abilities.")])
SS = "Super Senses"
P("Acute Sense", "Self", R_base("Perception"), C_two(), "One sense uses this rank instead of Perception.", resolve=False, category=SS)
P("Analytic Sense", "Self", R_def(), C_flat(1), "Determine a subject's composition by examining it.", category=SS)
P("Astral Sight", "Self", R_def(), C_flat(2), "See the astral realm, spirits and hidden living beings.", category=SS)
P("Circular Vision", "Self", R_def(), C_flat(1), "360-degree field of view.", category=SS)
P("Enhanced Hearing", "Self", R_def(), C_flat(1), "Hear infrasound and ultrasound.", category=SS)
P("Hypersensitive Touch", "Self", R_def(), C_flat(1), "Read print by touch and feel vibrations.", category=SS)
P("Lie Detection", "Self", R_def(), C_flat(3), "Detect lies within Close Range.", category=SS)
P("Microscopic Vision", "Self", R_def(), C_flat(1), "See microscopic detail with focus.", category=SS)
P("Night Vision", "Self", R_def(), C_flat(3), "See in total darkness.", category=SS)
P("Radio Hearing", "Self", R_def(), C_flat(1), "Hear radio transmissions.", category=SS)
P("Telescopic Vision", "Self", R_def(), C_flat(1), "See fine detail at great distance.", category=SS)
P("Thermal Vision", "Self", R_def(), C_flat(3), "See heat; see in the dark and track residual heat.", category=SS)
P("Tracking Scent", "Self", R_def(), C_flat(1), "Track and identify by scent.", category=SS)
P("True Sight", "Self", R_def(), C_flat(6), "See a being's true form through disguises and illusions.", category=SS)
P("Ultra Vision", "Self", R_def(), C_flat(2), "Low-light and ultraviolet vision.", category=SS)
P("X-Ray Vision", "Self", R_def(), C_flat(6), "See through solid objects with focus.", category=SS)
P("Super Speed", "Self", R_power(), C_rank(3),
  "Travel Power: superhuman speed; substitutes for Agility; Edge equals rank x3; big benefits at 9d+.")
P("Swimming", "Self", R_power(), C_two(), "Travel Power: swim fast, breathe water, resist ocean pressure.",
  [M("Lungs", "con", flat(-1), "Hold breath for a long time instead of breathing water.")])
P("Swing Line", "Self", R_base("fixed:3"), C_rank(1), "Travel Power: swing between anchor points outdoors.",
  [M("Natural", "con", flat(-2), "Only works where natural lines (vines) exist.")])
P("Telekinesis", "Ranged", R_power(), C_rank(2), "Move objects with your mind as rank Might; active defense vs ranged attacks.",
  [M("Gravity Flux", "con", flat(-6), "Only vertical movement or holding in place."),
   M("Impel Objects", "con", flat(-4), "Only straight-line movement or holding in place.")])
P("Telepathy", "Ranged", R_power(), C_rank(1), "Read surface thoughts, send thoughts, sense minds, probe memories.",
  [M("Alter Memories", "pro", flat(6), "Alter memories on a Margin 3+ probe."),
   M("Cloak Others", "pro", flat(2), "Hide the minds of everyone near you."),
   M("Dream Travel", "pro", flat(2), "Enter others' dreams."),
   M("Empathy", "con", flat(-2), "Emotions only."),
   M("Mind Link", "pro", flat(2), "Link rank minds for group telepathy.")])
P("Teleportation", "Self", R_power(), C_rank(1), "Teleport as a move or farther as an action; may exceed the Trait Cap.",
  [M("Medium", "con", flat(-2), "Only along or through a medium."),
   M("Portal", "pro", flat(2), "Open portals others can use.")], ignores_trait_cap=True)
P("Time Stop", "Special", R_power(), C_rank(3), "Stop time for rank pages at escalating Resolve cost.",
  [M("Companions", "pro", prank(1), "Up to Willpower allies are unaffected.")])
P("Time Travel", "Self", R_def(), C_flat(3), "Travel through time; Willpower uses per issue free.",
  [M("Portal", "pro", flat(2), "Open portals others can use.")])
P("Total Recall", "Self", R_def(), C_flat(3), "Perfect memory of anything you attend to.",
  [M("Playback", "pro", flat(1), "Memories can be transferred and displayed.")])
P("Tracer", "Touch", R_def(), C_flat(3), "Tag beings or objects and follow them for the story.",
  [M("Tracking", "pro", flat(3), "Track untagged subjects from belongings or recent locations.")])
P("Transmutation", "Touch", R_power(), C_rank(2), "Change the composition of inanimate matter; Resolve to make permanent.")
P("Tunneling", "Self", R_power(), C_rank(1), "Travel Power: dig through the ground.")
P("Two-Dimensional", "Self", R_def(), C_flat(6), "Become flat like a shadow: weightless, silent, slide along surfaces.")
P("Two-Fisted", "Self", R_def(), C_flat(3), "1 Resolve to defeat double Minions; +1d Weapon Bonus when dual wielding mundane weapons.",
  [M("Fighting", "con", flat(-1), "Close combat only."),
   M("Shooting", "con", flat(-1), "Ranged combat only.")])
P("Vanish", "Special", R_def(), C_flat(3), "Spend an action to disappear before moving away unseen.")
P("Variant", "Special", R_spec(), C_flat(3), "Another version of a Power you have, worth no more than the original.")
P("Ventriloquism", "Self", R_def(), C_flat(3), "Mimic sounds and throw your voice (Charm vs Perception).")
P("Wall-Crawling", "Self", R_power(), C_two(), "Cling to and move along walls and ceilings.")
P("Weakness Detection", "Self", R_def(), C_flat(3), "Find weak points; 1 Resolve to gain Penetrating against a target for the scene.")

# ---------- GENERIC PROS AND CONS ----------
G = []
def g(*a, **k): G.append(M(*a, **k))
g("Area", "pro", flat(4), "Affect one target or an area up to 50 ft (1 zone), sized as desired.")
g("Static", "con", flat(-2), "Area/Zone always targets a fixed-size area.")
g("Zone", "pro", flat(4), "Affect one target or the area around you (you are unaffected).")
g("Armor Piercing", "pro", flat(2), "Targets take -2d on passive defense (not vs Force Field).")
g("Build-Up", "con", flat(-2), "Unusable until the third page of a scene.")
g("Burnout", "con", flat(-4), "Roll 2 dice after each use; 6 or less burns it out for the scene.")
g("Carrier Attack", "pro", flat(6), "Triggers a linked secondary attack when it hits.")
g("Charges/Uses", "con", choice(-1, -2, -4, -6), "-1: 6/scene or 12/day; -2: 3/scene or 6/day; -4: 1/scene or 3/day; -6: 1/day.")
g("Close (Pro)", "pro", flat(2), "Touch Power reaches Close Range.")
g("Close (Con)", "con", flat(-2), "Ranged Power limited to Close Range.")
g("Concentration", "con", flat(-4), "Using it takes your full attention until your next turn.")
g("Conditional", "con", choice(-1, -2, -4, -6), "Works only under conditions: usually / more often than not / occasionally / rarely.")
g("Constant", "con", flat(-2), "Always on; can't be switched off.")
g("Contagious", "pro", flat(4), "Spreads to anyone who touches the target.")
g("Costly", "con", flat(-4), "Costs 1 Resolve per use.")
g("Degrades", "con", flat(-2), "-1d effective rank per use for the scene.")
g("Delay", "con", flat(-4), "Takes effect after an hour or more.")
g("Exclusive", "con", flat(-1), "Can't be used together with linked Powers.")
g("Expansive", "pro", choice(6, 12), "Zone can expand to Distant (+6) or Extreme (+12) Range for Resolve.")
g("Fuse", "pro", flat(2), "Set to trigger after a delay up to 24 hours.")
g("Harmful", "con", choice(-2, -4, -6), "Damages you per use: 1 / 1-3 / 1-6 points (half value with Regeneration).")
g("Imbue", "pro", flat(4), "Use a Self Power on others you touch (Resolve for big Powers).")
g("Inanimate", "con", choice(-4, -3, -2, -1, 0), "Living-only Power affects Inanimate beings instead (-4 for one Source, less for more).")
g("Inanimate Plus", "pro", flat(1), "Also affects Inanimate beings of one Source.", True)
g("Independent", "pro", flat(6), "The Power acts on its own like a separate character.")
g("Item", "con", flat(-1), "Comes from a carried or worn item.")
g("Limited", "con", choice(-1, -2, -4, -6), "Mild / moderate / severe / nearly useless limitation.")
g("Line of Sight", "pro", choice(2, 4, 6), "Reach Extreme Range: +2 from Distant, +4 from Close, +6 from Touch.")
g("Ongoing", "pro", flat(6), "Damage continues 1 per page after the hit.")
g("Overkill", "con", prank(-1), "Targets double active defense successes; -1 HP per rank (1/rank becomes 1 per 2 ranks); that Trait's cap is +3d.", trait_cap_bonus=3)
g("Overload", "pro", flat(2), "Boost +3d or +6d for one page, then lose the Power.")
g("Penetrating", "pro", flat(6), "Halves passive defense ranks (max -6d), not vs Force Field.")
g("Phase Shift", "pro", flat(4), "Affects incorporeal targets as if solid.")
g("Precise", "con", flat(-2), "Targets get +2d passive defense.")
g("Ranged", "pro", choice(2, 4), "Reach Distant Range: +2 from Close, +4 from Touch.")
g("Ready/Reload", "con", choice(-1, -2), "-1: reload every 3 uses; -2: ready after every use.")
g("Recharge", "con", flat(-4), "Usable once every 3 pages.")
g("Resisted", "pro", flat(6), "Special effects can't be actively defended against (GM approval).")
g("Ricochet", "pro", flat(2), "Bank shots; 1 Resolve to defeat double Minions.")
g("Selective", "pro", flat(4), "Only affects whom you choose.")
g("Shutdown", "con", choice(-1, -2, -4, -6), "Something can stop it: usually works / more often than not / occasionally / rarely.")
g("Side Effect", "con", choice(-1, -2, -4), "Annoying / harmful / catastrophic side effect.")
g("Signature", "con", flat(-1), "Obvious or leaves a traceable signature.")
g("Speech", "con", flat(-2), "Target must hear and understand your words.")
g("Spray", "con", flat(0), "+1d at Touch/Close, -2d at Distant.")
g("Subtle/Undetectable", "pro", choice(2, 6), "Subtle (+2) or completely undetectable (+6).")
g("Sustained", "con", flat(-2), "Costs an action per page to maintain.")
g("Touch", "con", choice(-2, -4), "Requires contact: -2 from Close, -4 from Distant.")
g("Toxin", "con", flat(-1), "A disease/drug/poison; Resistance defends.")
g("Trap", "pro", flat(2), "Set the Power as a trap.")
g("Two-Handed", "con", flat(-1), "Needs both hands free.")
g("Uncontrolled", "con", flat(-2), "Activates or deactivates on its own in certain situations.")
g("Unreliable", "con", flat(-2), "Roll 2 dice per use; 6 or less fails.")
g("Words", "con", flat(-1), "Must speak aloud.")

PERKS = [
 {"name": "Authority", "cost": C_flat(3), "summary": "Special rights or authority: clearances, immunity, arrest powers, nobility."},
 {"name": "Contact", "cost": C_unit(1, "contact"), "summary": "A person who provides information and small favors."},
 {"name": "Fame", "cost": C_flat(3), "summary": "Famous and favorably recognized (one identity per purchase)."},
 {"name": "Fourth Wall", "cost": C_flat(6), "summary": "Earn 1 Resolve once per scene for a fourth-wall joke that lands."},
 {"name": "Headquarters", "cost": C_unit(1, "3 Base Points"), "summary": "A base of operations; Heroes can pool Base Points."},
 {"name": "Infamy", "cost": C_flat(3), "summary": "Feared by the criminal underworld."},
 {"name": "Patron", "cost": C_flat(3), "summary": "An organization or backer that supplies support."},
 {"name": "Pet/Sidekick", "cost": C_ppl(6), "summary": "A companion at least one power level below you; 6 HP per power level."},
 {"name": "Reputation", "cost": C_flat(3), "summary": "Credibility and respect in a particular arena."},
 {"name": "Resources", "cost": C_flat(1), "summary": "Access to a supply of equipment (not wealth)."},
 {"name": "Unique Vehicle", "cost": C_unit(1, "25 Vehicle Points"), "summary": "One or more unique vehicles, outside the Gear Limit."},
 {"name": "Wealth/Great Wealth", "cost": C_choice([("Wealth", 3), ("Great Wealth", 6)], False), "summary": "Comic-book rich; Great Wealth allows an absurd purchase per issue for 1 Resolve."},
]
o, ph, co, sp = "ordinary", "plot_hook", "condition", "special"
FL = [("Absentminded", o), ("Alter Ego", o), ("Amnesia", sp), ("Aversion/Fear", o), ("Beast", o), ("Blind/Deaf", co),
      ("Broke", o), ("Clumsy", o), ("Code", o), ("Color Blind", o), ("Compulsion/Severe Compulsion", sp), ("Creepy", o),
      ("Curse", o), ("Decorum", o), ("Emotionless", o), ("Enemy", ph), ("Finite Power", o), ("Flashbacks/Guilt", o),
      ("Frenzy", o), ("Frightening", o), ("Heavy/Very Heavy", sp), ("Hidden Agenda", o), ("Illiterate", o),
      ("Impaired Sense", o), ("Impairment", co), ("Insane", o), ("Light Sensitive", o), ("Mute", o), ("Night Blind", o),
      ("Nocturnal", o), ("Notoriety", o), ("Obligation", ph), ("Outsider", o), ("Power Limits", o), ("Quirk", o),
      ("Reaction/Severe Reaction", sp), ("Relationship", ph), ("Repair", co), ("Requirement/Severe Requirement", sp),
      ("Restriction", o), ("Secret", o), ("Secret Identity", o), ("Slow", o), ("Unlucky/Jinx", co), ("Unusual Looks", o),
      ("Unusual Shape", o), ("Vulnerability", sp), ("Wanted", ph)]
FLAW_SUM = {
 "Amnesia": "No memory of your past; both a Plot Hook and a Condition (1 Resolve per issue).",
 "Compulsion/Severe Compulsion": "An urge you struggle to resist; the Severe version is a Condition.",
 "Heavy/Very Heavy": "Heavy is 5d-6d weight rank; Very Heavy (7d+) is a Condition.",
 "Reaction/Severe Reaction": "Adverse reaction to something; Severe is a Condition unless the trigger is rare.",
 "Requirement/Severe Requirement": "Must meet a need periodically; Severe (desperate and immediate) is a Condition.",
 "Vulnerability": "Defenses halved against a specific attack; Resolve only when it comes up (max 1 per scene).",
}
flaws = []
for n, k in FL:
    s = FLAW_SUM.get(n) or {ph: "Plot Hook: 1 extra Resolve at the start of every issue.",
                            co: "Condition: always in effect; 1 extra Resolve at the start of every issue.",
                            o: "Earn 1 Resolve when it causes trouble."}[k]
    flaws.append({"name": n, "kind": k, "summary": s})

out = {"ruleset": "errata", "powers": powers, "generic_mods": G, "perks": PERKS, "flaws": flaws}
json.dump(out, open(sys.argv[1] if len(sys.argv) > 1 else "data/errata_rules.json", "w"), indent=1, ensure_ascii=False)
print(len(powers), len(G), len(PERKS), len(flaws))
