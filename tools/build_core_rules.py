#!/usr/bin/env python3
"""Build data/core_rules.json from hand-reviewed data extracted from core.txt."""
import json

# ---------- helpers ----------
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

def M(name, typ, cost, summary, repeatable=False):
    return {"name": name, "type": typ, "cost": cost, "repeatable": repeatable, "summary": summary}
def flat(v): return {"kind": "flat", "value": v}
def prank(v): return {"kind": "per_rank", "value": v}
def ptwo(v): return {"kind": "per_two_ranks", "value": v}
def pppl(v): return {"kind": "per_power_level", "value": v}
def choice(*o, per_rank=False):
    d = {"kind": "choice", "options": list(o)}
    if per_rank: d["per_rank"] = True
    return d
def rng(a, b, per_rank=False):
    d = {"kind": "range", "min": a, "max": b}
    if per_rank: d["per_rank"] = True
    return d
def special(n): return {"kind": "special", "note": n}

powers = []
def P(name, rng_, rank, cost, summary, mods=(), resolve=True, category=None):
    powers.append({"name": name, "category": category, "range": rng_, "rank": rank, "cost": cost,
                   "affects_resolve": resolve, "summary": summary, "mods": list(mods)})

# ---------- POWERS ----------
P("Adaptation", "Self", R_def(), C_flat(12),
  "Your body automatically adjusts to any environment, letting you breathe, move, perceive and survive in hostile or alien surroundings.")
P("Alternate Form", "Self", R_def(), C_ppl(4),
  "You can become a second form built as its own Hero at a power level up to yours (4 HP per level: Street 4 ... Iconic 24); every form pays the cost and they share the lowest Resolve.",
  [M("Independent Forms", "con", pppl(-1), "Each form has its own identity, memories and personality.")])
P("Animal Control", "Ranged", R_power(), C_rank(1),
  "Roll against an animal's Willpower to make it obey you for the Power's duration.",
  [M("Calling", "pro", flat(2), "Reach animals out to Extreme Range; they come to you and the duration starts when they arrive."),
   M("Only X", "con", choice(-2, -4), "Limited to one category of animals; -4 if the group is very narrow or of little use.")])
P("Animal Empathy", "Self", R_def(), C_flat(1),
  "Animals sense your kinship and rarely attack you without strong reason, and may be talked down.",
  [M("Speech", "pro", flat(2), "You can actually converse with animals.")])
P("Animal Mimicry", "Self", R_power(), C_rank(2),
  "Gain a chosen animal's Powers and better physical Abilities, each capped at your Power rank; switching animals is a free action once per page.",
  [M("Enhanced", "pro", flat(4), "Replace the mimicked animal's single highest Trait with your Power rank."),
   M("Selection", "pro", flat(4), "Mimic several animals at once using the best Traits of each, at -1d effective rank per extra animal.")])
P("Animation", "Ranged", R_power(), C_rank(2),
  "Bring images or objects to life for a scene; they get Agility, Might, Toughness and fitting Powers, one Trait at your rank and the rest at half. -1d per extra animated thing; Structure can't exceed your rank.")
P("Armor", "Self", R_base("Toughness", half=True), C_rank(1),
  "Passive defense against physical and energy attacks; you are immune to attacks whose rank is at most half your Armor rank (unless Penetrating).",
  [M("Activated", "con", flat(-1), "Must be switched on (free action) and shuts off if you are knocked out."),
   M("Reactive", "con", flat(-2), "Works as an active defense, like a shield or bracers you block with.")])
P("Astral Projection", "Self", R_def(), C_flat(12),
  "Leave your body as an astral form with Invisibility, Phasing (Intangible), Super Senses (Astral Sight) and Flight at your Willpower rank, while your body lies dormant.")
P("Attuned", "Self", R_def(), C_flat(3),
  "You sense significant disturbances, events or powerful beings within a natural force, region or the cosmos you are attuned to; spend 1 Resolve for more detail.",
  resolve=False)
P("Aura", "Self", R_power(), C_rank(2),
  "Your body deals one energy type to anyone touching or punching you (passive defenses only) and can be used for energy close attacks; mundane objects with lower Structure can't hurt you.",
  [M("Spines", "con", flat(-2), "Quills or spines deal physical damage instead, and give no protection against physical attacks.")])
P("Banish", "Ranged", R_power(), C_rank(1),
  "Roll against Willpower (or Toughness if chosen) or an active defense to send a target to another time, place or dimension until they are defeated.")
P("Blast", "Ranged", R_power(), C_rank(1),
  "A damaging ranged attack of a damage type chosen at purchase.",
  [M("Block", "pro", flat(2), "Use Blast as an active defense against ranged attacks (not area attacks); forgoing your next action starts a 3-exchange blast duel."),
   M("Melee", "pro", flat(2), "Use Blast as an active defense against close combat attacks."),
   M("Subdual", "con", flat(-1), "Deals subdual damage."),
   M("Weapons", "con", flat(-1), "Represents skill with a chosen weapon group; attack with your Blast rank only while armed, using the weapon's damage.")])
P("Blending", "Self", R_def(), C_flat(3),
  "+3d to Covert rolls to stay unseen; while motionless you can attempt to hide in plain sight at half that rank.",
  resolve=False)
P("Blind", "Ranged", R_power(), C_two(),
  "Roll against Toughness or an active defense to blind a living target: -3d to its attacks and active defenses unless it can compensate.")
P("Blind Fighting", "Self", R_def(), C_flat(3),
  "You suffer no penalties for darkness or unseen opponents in combat.")
P("Blink", "Self", R_power(), C_rank(1),
  "Flicker in and out of reality in combat: use Blink in place of Might for close attacks and Agility for active defense and movement rolls (or a higher such Trait +1d).")
P("Boost", "Self", R_base("special"), C_special("costs the same Hero Points per rank as the Trait it boosts; that Trait is its baseline"),
  "Raises one chosen Ability, Talent or Power up to the Boost rank; must be given Cons describing when and how it works.",
  [M("Rage", "con", flat(-2), "Activates when you are furious; Intellect and all Talents drop to 1d while it lasts, ending by scene's end.")])
P("Buff", "Zone", R_def(), C_flat(12),
  "Spend 1 Resolve and an action to give yourself and allies in the zone +1d on all challenge rolls for the scene; doesn't stack.",
  [M("Greater", "pro", flat(6), "Each extra Resolve spent adds +1d more, up to 3 Resolve for +3d. (Book prints the cost as '(6)' without a sign.)")])
P("Clairvoyance", "Self", R_def(), C_flat(9),
  "See distant places (or a familiar person/object) at any range; shielding requires a Willpower roll and other dimensions need a Superhuman (6) Willpower roll.",
  resolve=False)
P("Cloud Minds", "Ranged", R_power(), C_rank(1),
  "Become undetectable to living minds within Distant Range unless they beat your rank with Willpower once you are close or noticeable; costs 1 action per page.")
P("Communications", "Self", R_def(), C_flat(3),
  "Detect, receive, store and transmit communications over available networks worldwide without needing infrastructure.",
  resolve=False)
P("Constructs", "Ranged", R_power(), C_rank(3),
  "Create solid force constructs with Structure and Might equal to your rank, usable to attack, grab or shield; -1d per extra construct.",
  [M("Devices", "pro", prank(2), "Build complex machines that mimic other Powers, like a broad Omni-Power.")])
P("Danger Sense", "Self", R_base("Perception"), C_rank(1),
  "Use this rank instead of Perception to sense danger and for Edge, and make active defenses even when surprised (at the lower rank).",
  resolve=False)
P("Darkness", "Zone", R_def(), C_flat(6),
  "Fill the zone with darkness, fog, smoke or similar; those who can't see through it take -3d to attacks and active defenses.")
P("Dazzle", "Zone", R_power(), C_rank(1),
  "Roll against Willpower or an active defense for everyone in the zone: stunned for one page, then blinded for the rest of the duration; can't be stacked.",
  [M("Deafen", "con", flat(-1), "After the stun, targets are deafened (-1d to all rolls) instead of blinded.")])
P("Deflection", "Self", R_power(), C_choice([("Physical or energy", 1), ("Both physical and energy", 2)], True),
  "Active defense against one chosen type of ranged attack (not area); a failed attack can be redirected as a free action, -2d cumulative per extra redirect per page.",
  [M("Only X", "con", flat(-4), "Deflects just one specific object or energy type.")])
P("Density", "Self", R_power(), C_rank(1),
  "Grow denser: substitute Density for Might and Toughness and gain Armor at that rank (or existing Trait +1d); weight rank equals Density from 3d up.")
P("Detection", "Ranged", R_power(), C_two(),
  "Sense and locate a chosen kind of thing out to Extreme Range, rolling only for faint or hidden traces.",
  [M("Long Range", "pro", flat(1), "Works at interplanetary distances, up to Plot Device Range.")], resolve=False)
P("Determination", "Self", R_spec(), C_unit(5, "Resolve"),
  "Start each issue with +1 Resolve per 5 Hero Points spent; has no rank, can't be affected by other Powers, Heroes only.",
  resolve=False)
P("Dimensional Travel", "Self", R_def(), C_flat(6),
  "Spend an action to travel to other dimensions; free uses per issue equal Willpower, then 1 Resolve each.",
  [M("Portal", "pro", flat(2), "Open portals others can pass through.")], resolve=False)
P("Dispel", "Ranged", R_power(), C_two(),
  "Roll against an ongoing Power's rank to end its effect on a being or object; one Source only; also an active defense against special-effect Powers from that Source.",
  [M("Also X", "pro", ptwo(1), "Each additional Source adds +1 Hero Point per 2 ranks.", repeatable=True)])
P("Drain", "Ranged", R_power(), C_two(),
  "Roll against the target's highest Trait of one Source; drained Abilities/Talents drop to 1d and Powers stop for the duration.",
  [M("Also X", "pro", ptwo(1), "Each additional Source adds +1 Hero Point per 2 ranks.", repeatable=True),
   M("Only X", "con", choice(-2, -4), "-2 to affect only Abilities, Talents or Powers; -4 to affect only one specific Trait.")])
P("Duplication", "Self", R_def(), C_flat(25),
  "Spend an action to create a copy of yourself without this Power and without its own Resolve; can be bought again for more duplicates.")
P("Elemental Control", "Ranged", R_power(), C_rank(3),
  "Create and control one element to mimic other Powers at this rank, like Constructs and/or a narrow Omni-Power.",
  [M("Matter Creation", "pro", prank(1), "Spend Resolve equal to the weight rank (min 1) to make created matter permanent."),
   M("Only Control", "con", prank(-1), "Can only manipulate existing element within range, not create it.")])
P("Emotion Control", "Ranged", R_power(), C_rank(1),
  "Roll against Willpower to impose a strong emotion or mental state on a target.",
  [M("Only X", "con", flat(-2), "Only one specific emotion or state."),
   M("Pheromones", "con", flat(-2), "Chemical: resisted by the better of Toughness or Willpower; non-breathers, life support and toxin immunity block it.")])
P("Energy Absorption", "Self", R_power(), C_choice([("One energy type", 1), ("Kinetic (GM approval)", 3)], True),
  "Absorb one energy type: immune to it up to your rank, passive defense above that; spend absorbed ranks at once to boost an Ability, gain +1d, Blast, or heal.",
  [M("Also X", "pro", flat(2), "Absorb one more non-kinetic energy type per purchase.", repeatable=True)])
P("Ensnare", "Ranged", R_power(), C_rank(1),
  "Roll against Might or an active defense to bind a target so it can't take physical actions; Aura counts as a passive defense.",
  [M("Line", "pro", flat(2), "A tether lets you reel in a snared target whose weight rank doesn't exceed your rank."),
   M("Immobilize", "con", flat(-2), "Target is stuck in place but can still act physically.")])
P("Evasion", "Self", R_base("Agility"), C_two(),
  "Use Evasion instead of Agility for active defense, movement rolls and combat stunts (not a Travel Power).")
P("Expertise", "Self", R_base("special"), C_two(),
  "A specialty under one Ability or Talent; use this rank instead of that Trait when it applies (baseline equals that Trait). Non-combat Expertise doesn't affect Resolve.",
  resolve=False)
P("Extra Limbs", "Self", R_def(), C_flat(3),
  "Extra arms, tail, tentacles or similar give +2d when grappling and possibly other tasks.")
P("Flight", "Self", R_power(), C_rank(1),
  "Travel Power for flying; effective rank is halved indoors unless the space is very large.",
  [M("Spaceflight", "pro", flat(2), "Faster-than-light travel in space."),
   M("Gliding", "con", flat(-2), "Must descend at least a story per page unless riding an updraft."),
   M("Levitation", "con", flat(-4), "Only straight up/down movement or hovering."),
   M("Slides", "con", flat(-2), "Ride self-made slides; must stay within Distant Range of ground or large solid objects."),
   M("Wings", "con", flat(-2), "Needs room and air; no flying in cramped spaces, thin atmosphere or vacuum.")], resolve=False)
P("Force Field", "Self", R_power(), C_rank(1),
  "Shaped field giving protected characters your rank as a passive defense vs physical and energy attacks, immune at half rank or less; not mental. Activate with an action, maintain free; -1d per extra field.",
  [M("Inviolate", "pro", prank(1), "Spend 1 Resolve when raising it to block all Power effects of equal or lower rank, including mental ones."),
   M("Permeable", "pro", flat(6), "Attacks can pass out through the field in one direction."),
   M("Collapsible", "con", flat(-2), "Collapses when hit by an attack above your rank and must be re-raised."),
   M("Ward", "con", flat(-1), "Blocks mental attacks and effects instead of physical ones; can hold Intangible Phasers.")])
P("Energy Form", "Self", R_power(), C_rank(3),
  "Become one type of energy: gain Aura and Flight at this rank, weightless and immune to physical and energy attacks, but can't touch things and are vulnerable to effects on that energy.",
  [M("Also X", "pro", flat(2), "One more energy type per purchase.", repeatable=True)], category="Form")
P("Gaseous Form", "Self", R_def(), C_flat(6),
  "Become a gas with 1d Flight, immune to physical and energy attacks and able to pass through gaps, but unable to affect the physical world.",
  category="Form")
P("Liquid Form", "Self", R_def(), C_flat(12),
  "Become liquid or fine particles: immune to physical (not energy) attacks and able to flow through openings while still able to act physically.",
  category="Form")
P("Solid Form", "Self", R_power(), C_rank(2),
  "Become solid matter, substituting this rank for Might and Toughness and gaining Armor at it (or existing Trait +1d).",
  category="Form")
P("Gestalt", "Self", R_def(), C_unit(3, "alter ego"),
  "Requires the Alter Ego Flaw; your powered form is the merger of two separate alter egos, with each additional alter ego costing 3 more.")
P("Growth", "Self", R_power(), C_rank(1),
  "Grow larger: substitute Growth for Might and Toughness, gain Armor at it and Running at half (or Trait +1d); weight rank equals Growth from 4d up.")
P("Hard to Kill", "Self", R_def(), C_flat(1),
  "Only extreme measures can kill you. In games where Heroes can die, the GM may charge 3 to 6 Hero Points instead.")
P("Healing", "Touch", R_power(), C_rank(1),
  "Heal touched characters, up to your rank in damage per day; may cure disease at GM discretion.",
  [M("Repair", "con", flat(-2), "Fixes machines, robots, vehicles and technology instead of living beings.")])
P("Hibernation", "Self", R_def(), C_flat(1),
  "Enter aware suspended animation indefinitely without needing air, food or water.",
  resolve=False)
P("Hyper Breath", "Special", R_power(), C_rank(1),
  "Roll against Might or weight rank to push or pull targets (all in Close Range or one at Distant); 3+ net successes knock prone and cost a turn.",
  [M("Freezing", "pro", prank(1), "Freezing breath mimics Blast (cold) or Ensnare within Close Range.")])
P("Illusions", "Zone", R_power(), C_rank(1),
  "Create real sights and sounds that fool people and devices; doubters roll Perception against your rank, and touching reveals them.",
  [M("Mental", "pro", prank(2), "Illusions exist only in minds (resisted by Willpower) and can impose Stun-like effects on single targets.")])
P("Immortality", "Self", R_def(), C_flat(3),
  "You don't stay dead and don't die of old age. In lethal games the GM may charge 6 to 12 Hero Points.",
  [M("Vulnerable", "con", flat(-1), "You can be permanently killed in some specific, difficult way.")])
P("Immunity", "Self", R_def(), C_unit(3, "immunity"),
  "Immune to chosen hazards, each bought separately: Attack X, Energy X, Environmental, Fatigue, Life Support, Mental States, Sensory, Toxins, or custom.")
P("Inanimate", "Self", R_spec(), C_flat(9),
  "Not truly alive: immune to physical attacks and Powers that only affect the living, with effective Immunity (Life Support, Toxins); still vulnerable to mental effects. Has a non-Trained Source and no rank.",
  [M("Mindless", "pro", flat(9), "Also immune to mental attacks and Powers.")])
P("Invisibility", "Self", R_def(), C_flat(9),
  "Become invisible; foes take -3d to attacks and active defenses against you, or -2d if they detect you by other means.",
  [M("Cloaking", "pro", flat(3), "Also hidden from Radar, visual Super Senses and equivalent sensors."),
   M("Jamming", "con", flat(-3), "Not invisible, but undetectable to Radar, visual Super Senses and equivalent sensors.")])
P("Irritant", "Zone", R_power(), C_rank(1),
  "Fill the zone with a hindering effect; others take -2d on non-passive-defense rolls unless Toughness (or Willpower, if chosen) exceeds your rank.")
P("Languages", "Self", R_def(), C_flat(1),
  "Speak extra languages equal to your Intellect, chosen in advance or during play.",
  [M("Hyper Translation", "pro", flat(2), "Instantly fluent in any language you can perceive and pronounce."),
   M("Polyglot", "pro", flat(1), "Learn new languages in about a week, basics in a day or two.")], resolve=False)
P("Leadership", "Self", R_def(), C_flat(6),
  "As a free action, let each ally you can see and talk to reroll one challenge roll per combat scene.")
P("Leaping", "Self", R_base("Might", half=True), C_rank(1),
  "Travel Power for huge jumps (vertical reach is rank -1d); capped at Agility indoors; also an active defense against falls.",
  resolve=False)
P("Life Drain", "Touch", R_power(), C_rank(2),
  "Roll against Toughness (or Willpower) or an active defense: 1 damage per net success, healing yourself 1 per 2 damage dealt.",
  [M("Vampirism", "pro", flat(1), "Deals physical damage resisted normally and can hit inanimate targets, healing only from living ones.")])
P("Light/Effect", "Special", R_def(), C_flat(3),
  "Freely generate harmless light (filling the zone or a beam to Distant Range), or another innocuous environmental effect (called Effect).")
P("Lightning Reflexes", "Self", R_def(), C_flat(3),
  "+6 Edge and instant weapon draws.",
  [M("Fighting/Marksmanship", "con", flat(-1), "Only applies to close combat or to ranged combat (pick one).")])
P("Luck", "Self", R_power(), C_rank(2),
  "Gain Luck dice equal to your rank each issue to add to your rolls or subtract successes from rolls against you.",
  [M("Control", "pro", flat(4), "Spend Luck on anyone within Distant Range at any time as a free action."),
   M("Unbelievable", "pro", prank(1), "Spend Luck dice on lucky breaks, which can be more outlandish than Resolve-bought ones.")])
P("Machine Control", "Ranged", R_power(), C_rank(1),
  "Remotely control machines and electronics within Distant Range, using your rank to operate them; -1d per extra machine.",
  [M("Interface", "con", flat(-2), "Only computers, and anything they control.")])
P("Martial Arts", "Touch", R_base("Might"), C_rank(1),
  "Unarmed attack and defense with subdual damage; substitute for Might in grapples and stunts and for Abilities with mundane melee weapons.",
  [M("Deflect", "pro", flat(4), "Active defense against ranged physical and energy attacks (not area)."),
   M("Deflect Missiles", "pro", flat(2), "Active defense against ranged physical attacks (not area).")])
P("Master of Disguise", "Self", R_def(), C_flat(3),
  "Make convincing disguises quickly and mimic voices and mannerisms, rolling Charm vs Perception.",
  resolve=False)
P("Matter Chameleon", "Self", R_power(), C_rank(2),
  "Take on a touched object's properties for the scene, like Solid Form capped by its Structure, plus Aura from its energy capped by that energy's rank.")
P("Mind Blast", "Ranged", R_power(), C_rank(1),
  "Psychic damage attack against Willpower; no effect on inanimate objects without a psychic presence.")
P("Mind Control", "Ranged", R_power(), C_rank(2),
  "Roll against Willpower to command a living being within Distant Range, or spend 1 Resolve to plant a delayed suggestion; +2d to resist self-destructive orders.",
  [M("Hypnosis", "con", flat(-2), "Requires a calm face-to-face conversation; useless in combat."),
   M("Pheromones", "con", flat(-2), "Chemical: resisted by the better of Toughness or Willpower; non-breathers, life support and toxin immunity block it.")])
P("Nullify", "Zone", R_power(), C_rank(2),
  "Automatically suppress Traits of one Source at or below your rank for everyone in the zone; requires concentration (1 Resolve to act); targets can pay 1 Resolve per use to push through.",
  [M("Also X", "pro", prank(1), "Each additional Source adds +1 Hero Point per rank.", repeatable=True)])
P("Omni-Power", "Special", R_power(), C_choice([("Narrow", 3), ("Broad", 5)], True),
  "Mimic other Powers at this rank, -1d per additional use on the same page; default-rank Powers need rank at least their cost; certain Powers excluded. Pros cost Resolve per use unless bought on Omni-Power.")
P("Phasing", "Self", R_def(), C_flat(9),
  "Go out of phase: immune to physical and energy attacks and able to pass through solids (not force fields) but can't affect anything physical; toggle once per turn.",
  [M("Intangible", "pro", flat(3), "Ordinary force fields can't hold you; Ward fields can unless your Willpower exceeds their rank.")])
P("Plasticity", "Self", R_power(), C_rank(2),
  "Stretchy body that can mimic effects by shaping itself, serves as passive defense vs physical attacks, and substitutes for Might in grapples and close combat.")
P("Polymorph", "Ranged", R_power(), C_rank(1),
  "Roll against Toughness (or Willpower) or an active defense to turn a living being into a harmless form for the duration.")
P("Portable Storehouse", "Self", R_def(), C_flat(6),
  "Call up mundane gear, weapons, armor or vehicles from a stash; they last for the scene unless backed by Resources or Wealth.")
P("Possession", "Ranged", R_power(), C_rank(1),
  "Roll against Willpower to take over a living body within Distant Range, using its physical Traits and your mental ones; your body lies dormant.",
  [M("Meld", "pro", flat(4), "Your body merges into the host for the duration.")])
P("Power Absorption", "Touch", R_power(), C_rank(2),
  "Touch and roll against the target's highest Trait of one Source to steal those Traits for the duration, knocking them out; -1d per extra victim.",
  [M("Also X", "pro", prank(1), "Each additional Source adds +1 Hero Point per rank.", repeatable=True),
   M("Only X", "con", choice(-2, -4), "-2 to take only Abilities, Talents or Powers; -4 for one specific Trait.")])
P("Power Mimicry", "Touch", R_power(), C_rank(2),
  "Touch and roll against the target's highest Trait of one Source to copy those Traits for the duration without harming them; -1d per extra source character.",
  [M("Also X", "pro", prank(1), "Each additional Source adds +1 Hero Point per rank.", repeatable=True),
   M("Only X", "con", choice(-2, -4), "-2 to copy only Abilities, Talents or Powers; -4 for one specific Trait.")])
P("Precognition", "Self", R_def(), C_flat(12),
  "Spend escalating Resolve (1, 2, 3...) per scene to undo a character's just-finished turn and act immediately instead.")
P("Preparation", "Self", R_spec(), C_flat(6),
  "Makes Resolve-bought narrative details far more powerful when explained by prior planning (perfect item, prepared location, known info, contacts); no rank.",
  resolve=False)
P("Psi-Screen", "Self", R_base("Willpower"), C_rank(1),
  "Use instead of Willpower against mental attacks and effects, and to sense and resist mind reading and probes.",
  [M("Cloak Mind", "pro", flat(2), "Telepaths must beat your Psi-Screen rank to detect you.")])
P("Psychometry", "Touch", R_def(), C_flat(9),
  "Learn the basic history of touched objects; spend 1 Resolve for details on a connected person or event.",
  resolve=False)
P("Quick Change", "Self", R_def(), C_flat(3),
  "Change clothing and grooming instantly as an action.",
  [M("Costume", "con", flat(-1), "Only swaps between your costume and what you last wore.")], resolve=False)
P("Radar", "Self", R_def(), C_flat(3),
  "A radar sense gives a 3D picture of your surroundings, but not flat images.",
  [M("Sonar", "con", flat(-1), "Only works underwater."),
   M("Tremor Sense", "con", flat(-1), "Senses ground vibrations instead, detecting only things touching or shaking the ground.")], resolve=False)
P("Regeneration", "Self", R_def(), C_flat(6),
  "Heal 1 damage per hour and wake up as soon as you heal a point.",
  [M("Fast", "pro", flat(6), "Heal 1 damage per minute."),
   M("Instant", "pro", flat(18), "Heal 1 damage per page.")])
P("Resistance", "Self", R_base("Toughness"), C_two(),
  "Use instead of Toughness to resist radiation, temperature extremes and toxins.")
P("Running", "Self", R_base("fixed:3"), C_rank(1),
  "Travel Power for fast ground movement (also wheels, skating, bouncing); 3d baseline is normal human speed.",
  [M("Hover", "pro", flat(2), "Hover just above the ground, ignoring ground hazards and triggers.")], resolve=False)
P("Separation", "Self", R_def(), C_flat(3),
  "Detach body parts that act independently with whatever abilities make sense.")
P("Shockwave", "Zone", R_power(), C_two(),
  "Roll against Might or weight rank for everyone around you: push back with -1d until your next turn; 3+ net successes also knock prone and cost a turn.")
P("Shrinking", "Self", R_power(), C_rank(2),
  "Shrink, lowering weight rank; substitute Shrinking for Agility and gain Leaping and Strike at half rank (or Trait +1d).")
P("Slay", "Touch", R_power(), C_rank(2),
  "Roll against Toughness (or Willpower) or an active defense to deal 2 damage per net success to a living target; ignores Armor.")
P("Slick", "Zone", R_power(), C_rank(1),
  "Make the ground slippery for the scene; crossers roll Agility vs your rank or fall prone, trapped with -2d until they stand; drivers roll Vehicles.")
P("Speak with Dead", "Self", R_power(), C_rank(2),
  "Ask a corpse a number of truthful questions equal to your rank.",
  resolve=False)
P("Specialty", "Self", R_spec(), C_flat(0),
  "Street/Low Level only: each Hero takes one unique Specialty for +1d to certain rolls (can exceed Trait Cap) and 1 Resolve to make 6s explode on them; no rank.",
  resolve=False)
P("Spinning", "Self", R_power(), C_rank(1),
  "Spin at super speed: use Spinning instead of Might for close combat, grapples and stunts, and instead of Agility to defend against physical attacks (or Ability +1d).")
P("Star Gate", "Self", R_def(), C_flat(3),
  "Spend an action to travel to distant worlds; free uses per issue equal Willpower, then 1 Resolve each.",
  [M("Portal", "pro", flat(2), "Open portals others, including ships, can pass through.")], resolve=False)
P("Stretching", "Self", R_def(), C_choice([("Close Range", 1), ("Distant Range", 3), ("Extreme Range", 6)], False),
  "Stretch your body to make close and grappling attacks at non-adjacent targets, out to Close, Distant or Extreme Range depending on version.",
  [M("Long Reach", "con", flat(-1), "Just long reach; you can't stretch your whole body.")])
P("Strike", "Touch", R_base("choice:Might|Martial Arts"), C_rank(1),
  "A damaging close combat attack and defense of a chosen damage type (claws, energized fists, etc.).",
  [M("Deflect", "pro", flat(4), "Active defense against ranged physical and energy attacks (not area)."),
   M("Deflect Missiles", "pro", flat(2), "Active defense against ranged physical attacks (not area)."),
   M("Reach/Throw", "pro", flat(2), "Hit opponents within Close Range via long reach or throwing."),
   M("Sweep", "pro", flat(4), "Area attack in a 90-degree arc out to 5-25 feet (50 with Reach)."),
   M("Subdual", "con", flat(-1), "Deals subdual damage."),
   M("Weapons", "con", flat(-1), "Represents skill with a chosen weapon group; attack and defend with Strike rank only while armed.")])
P("Stun", "Ranged", R_power(), C_rank(1),
  "Roll against Toughness (or Willpower) or an active defense to incapacitate a living target.",
  [M("EMP", "con", flat(-2), "Affects Tech-Source Inanimate characters and electronics instead of the living.")])
P("Summoning", "Self", R_power(), C_special("1 Hero Point per rank for every 2d of Minion Threat (round up), e.g. 5d-6d Threat = 3 HP per rank"),
  "Summon Minions per scene by rank (2 at 1d-2d up to 1,000 at 24d-26d) with a Threat chosen at purchase, capped at Trait Cap -6d (or -3d if 10 or fewer).",
  [M("Animals", "pro", choice(1, 2, per_rank=True), "Minions are animals with special traits: +1 per rank for a narrow selection, +2 per rank for a wide variety."),
   M("Automatons", "pro", prank(1), "Minions have Inanimate (Mindless)."),
   M("Duplicates", "pro", prank(2), "Minions are copies of you sharing your unique abilities."),
   M("Shooters", "pro", prank(1), "Minions can attack out to Distant Range."),
   M("Unique", "pro", rng(1, 4, per_rank=True), "Minions get special abilities: +1 per rank minor, +2 moderate, +4 major; GM approval.")])

# Super Senses: individual senses
SS = "Super Senses"
SS_NOTE = " Part of Super Senses, which counts as a single Power ranked at the higher of Perception or Acute X."
def SSP(name, cost, summary, rank=R_base("Perception")):
    P(name, "Self", rank, cost, summary + SS_NOTE, resolve=False, category=SS)
SSP("Acute X", C_two(), "One chosen sense uses this rank instead of Perception for rolls involving it (not for Edge).")
SSP("Analytic X", C_flat(1), "Use one sense to learn a target's basic physical and chemical makeup.")
SSP("Astral Sight", C_flat(1), "See the astral realm, letting you interact with spirits and astral projectors and spot Blending or Invisible living beings.")
SSP("Circular Vision", C_flat(1), "Full 360-degree field of view.")
SSP("Enhanced Hearing", C_flat(1), "Hear very faint or distant sounds with high-threshold Perception rolls, plus infrasound and ultrasound.")
SSP("Hypersensitive Touch", C_flat(1), "Read print by touch and feel vibrations through walls.")
SSP("Lie Detection", C_flat(3), "Automatically know when someone within Close Range is lying unless they are protected.")
SSP("Microscopic Vision", C_flat(1), "Magnify vision as far as needed, down to tiny scales.")
SSP("Night Vision", C_flat(3), "See in total darkness.")
SSP("Radio Hearing", C_flat(1), "Hear and intercept radio transmissions.")
SSP("Telescopic Vision", C_flat(1), "See fine detail at great distances.")
SSP("Thermal Vision", C_flat(2), "See heat, including through thin objects and in the dark; foiled by matching temperatures.")
SSP("Tracking Scent", C_flat(1), "Identify, track and read emotions by smell (threshold 4-6 rolls).")
SSP("True Sight", C_flat(6), "See through disguises, illusions, transformations, Blending and Invisibility.")
SSP("Ultra Vision", C_flat(2), "See ultraviolet and in very dim light, but not total darkness.")
SSP("X-Ray Vision", C_flat(6), "See through solid objects.")

P("Super Speed", "Self", R_power(), C_rank(3),
  "Travel at this speed rank; Agility equals your rank, Edge is rank x3, usable for close attack and defense and grapple defense. At 9d+ gain extra benefits (2 range classes per page, run on water/walls, 2 Minions per net success, Quick Change and Vanish).")
P("Swimming", "Self", R_power(), C_two(),
  "Travel Power through water; also breathe water, move and sense freely underwater, and ignore depth pressure and cold.",
  [M("Lungs", "con", flat(-1), "Can't breathe water but can hold your breath a long time.")], resolve=False)
P("Swing Line", "Self", R_power(), C_two(),
  "Travel Power for swinging between anchor points within Distant Range; lines have Structure equal to half your rank.",
  [M("Natural", "con", flat(-2), "Can't make lines; only works where vines or similar exist.")], resolve=False)
P("Telekinesis", "Ranged", R_power(), C_rank(2),
  "Move things at up to Distant Range with your rank as Might; attack, hold, throw, or actively defend against ranged attacks (energy needs a shielding object); -1d per extra object.",
  [M("Impel", "con", flat(-4), "Only straight-line pushes or holding in place, no fine control.")])
P("Telepathy", "Ranged", R_power(), C_rank(1),
  "Sense minds, communicate mentally, read surface thoughts or probe memories within Distant Range, rolling against Willpower or Telepathy.",
  [M("Alter Memories", "pro", flat(6), "With 3+ net successes on a probe, rewrite memories for the scene, or permanently for 1 Resolve."),
   M("Cloak Others", "pro", flat(2), "Hide the minds of everyone within Close Range from telepathy."),
   M("Dream Travel", "pro", flat(2), "Enter others' dreams, where your physical Traits use Willpower or Telepathy."),
   M("Mind Link", "pro", flat(2), "Link minds equal to your rank for shared communication within Extreme Range."),
   M("Empathy", "con", flat(-2), "Limited to emotions and mental states, not words or images.")])
P("Teleportation", "Self", R_power(), C_rank(1),
  "Instantly travel distances set by rank as a move; unfamiliar destinations risk failure and a knockout; 1 Resolve for two jumps in a page.",
  [M("Portal", "pro", flat(2), "Open portals others can pass through."),
   M("Medium", "con", flat(-2), "Only travel via a medium such as power lines, shadows or reflections.")], resolve=False)
P("Time Stop", "Zone", R_power(), C_rank(3),
  "Freeze time in the zone for pages equal to your rank at escalating Resolve cost per scene; you can move things but not harm frozen targets.")
P("Time Travel", "Self", R_def(), C_flat(3),
  "Spend an action to travel forward or back in time, under GM-defined rules.",
  [M("Portal", "pro", flat(2), "Open portals others can pass through.")], resolve=False)
P("Total Recall", "Self", R_def(), C_flat(3),
  "Perfectly remember what you attend to, with Intellect rolls to recall background details.",
  [M("Playback", "pro", flat(1), "Memories can be transferred, copied and displayed.")], resolve=False)
P("Tracer", "Touch", R_def(), C_flat(3),
  "Tag touched objects to track them worldwide for the story; finding the tag needs a Daunting (3) Perception roll.",
  [M("Tracking", "pro", flat(3), "Track someone via a recently owned item or a place they visited in the last day.")])

T = "Transformation"
P("Animal Forms", "Self", R_power(), C_rank(2),
  "Turn into familiar animals, taking their form and their physical Abilities and Powers capped at your rank. Changing is a free action on your turn.",
  [M("Enhanced", "pro", flat(4), "Replace the animal's single highest Trait with your Power rank."),
   M("Only X", "con", choice(-2, -4), "Limited to a category of animals; -4 for a single animal or very small group.")], category=T)
P("Doppelganger", "Self", R_def(), C_flat(6),
  "Become a perfect copy of any roughly humanoid person you've seen, fooling biometrics, without gaining their Traits.",
  category=T)
P("Object Forms", "Self", R_power(), C_rank(2),
  "Turn into shapes, tools, weapons, machines or vehicles: exert rank force, add half rank to users' rolls, or gain machine traits up to your rank; resize as Growth/Shrinking at this rank.",
  category=T)
P("Shapeshifting", "Self", R_power(), C_rank(2),
  "Reshape your body with Hero Points equal to your rank to temporarily raise Agility, Might, Toughness or buy body-based Powers; for Resolve, rank counts as Shapeshifting plus your best physical Ability.",
  category=T)

P("Transmutation", "Touch", R_power(), C_rank(2),
  "Alter touched matter's composition (gases in Close Range automatically; solids/liquids vs Structure or weight), changing Structure or state for the scene; 1 Resolve for permanence or unique substances.")
P("Tunneling", "Self", R_power(), C_rank(1),
  "Travel Power for digging through the ground, leaving tunnels open or collapsed.",
  resolve=False)
P("Two-Dimensional", "Self", R_def(), C_flat(6),
  "Become flat and shadow-like: weightless and silent, able to slide along any surface ignoring gravity and slip through cracks.")
P("Two-Fisted", "Self", R_def(), C_flat(3),
  "Spend 1 Resolve to defeat 2 Minions per net success with a physical attack; +1d Weapon Bonus when dual-wielding mundane weapons.",
  [M("Fighting/Marksmanship", "con", flat(-1), "Only applies to close combat/melee weapons or to ranged combat/ranged weapons (pick one).")])
P("Vanish", "Self", R_def(), C_flat(3),
  "Using an action and movement, disappear and relocate up to a page's movement unseen.")
P("Variant", "Special", R_spec(), C_flat(3),
  "An alternate version of a Power you already have, worth the same Hero Points as the original after Pros/Cons; not allowed for Powers that affect other Traits.")
P("Ventriloquism", "Self", R_def(), C_flat(3),
  "Mimic sounds and voices and throw your voice, rolling Charm vs Perception.",
  resolve=False)
P("Wall-Crawling", "Self", R_power(), C_two(),
  "Cling to and move along walls and ceilings at normal speed; carrying capacity limited by rank if lower than Might.",
  resolve=False)
P("Weakness Detection", "Self", R_def(), C_flat(3),
  "Spot weak points of what you observe; spend 1 Resolve to give your physical/energy attacks Penetrating against it for the scene.")

# ---------- GENERIC MODS ----------
gm = [
  M("Affect Inanimate", "pro", flat(1), "A living-only Power can also affect Inanimate beings of one Source.", repeatable=True),
  M("Area of Effect (Area)", "pro", flat(2), "Ranged Power may target a 10-50 ft area or a single target, sized per use."),
  M("Area of Effect (Burst)", "pro", flat(1), "Ranged Power always hits an area of fixed size (10-50 ft)."),
  M("Area of Effect (Zone)", "pro", choice(2, 4), "May target the area 5-25 ft around you or a single target; +2 on Ranged Powers, +4 on Touch Powers."),
  M("Area of Effect (Nova)", "pro", choice(1, 2), "Always hits a fixed area 5-25 ft around you; +1 on Ranged Powers, +2 on Touch Powers."),
  M("Armor Piercing", "pro", flat(2), "Halves passive defenses from mundane armor; incompatible with Weak."),
  M("Build-Up", "con", flat(-2), "Usable only from the third page of action onward."),
  M("Burnout", "con", flat(-4), "Roll 2 dice each use; 6 or less burns it out for the scene."),
  M("Carrier Attack", "pro", flat(6), "On a hit, immediately make a follow-up attack with a separately bought carrier Power (passive defenses only)."),
  M("Charges", "con", rng(-1, -4), "Limited uses per scene: -1 for 6, -2 for 3, -4 for 1."),
  M("Close", "pro", flat(2), "Touch Power can reach targets within Close Range."),
  M("Close", "con", flat(-2), "Ranged Power is limited to Close Range."),
  M("Concentration", "con", flat(-4), "Using it prevents other actions, movement and defense until your next turn; ongoing effects need continued focus."),
  M("Conditional", "con", rng(-1, -4), "Only works under certain conditions: -1 often, -2 occasionally, -4 rarely."),
  M("Constant", "con", flat(-2), "Always on and can't be turned off."),
  M("Contagious", "pro", flat(4), "Anyone touching an affected target can be attacked too and becomes contagious."),
  M("Costly", "con", flat(-4), "Each use costs 1 Resolve."),
  M("Degrades", "con", flat(-2), "Loses 1d per use for the scene."),
  M("Delay", "con", flat(-4), "Takes effect after a fixed delay too long to matter in combat."),
  M("Exclusive", "con", flat(-1), "Can't be used alongside other Exclusive Powers; must be on at least 2 Powers."),
  M("Expansive", "pro", rng(6, 18), "Zone Power: spend 1 Resolve to enlarge the area for the scene: +6 Close, +12 Distant, +18 Extreme Range."),
  M("Fuse", "pro", flat(2), "Set the Power to trigger after a chosen delay up to Willpower hours; can cancel within Extreme Range."),
  M("Harmful", "con", rng(-1, -4), "Hurts you each use: -1 for 1 damage, -2 for 1-3, -4 for 1-6; 1 Resolve avoids it."),
  M("Imbue", "pro", flat(4), "A self-only Power can be granted by touch to up to Willpower subjects."),
  M("Independent", "pro", flat(6), "The Power acts as a separate character on pages you let it."),
  M("Item", "con", flat(-1), "Comes from a carried or worn item, mundane or unbreakable; on Abilities/Talents the rank drops to 2d without it."),
  M("Limited", "con", rng(-1, -4), "Catch-all reduction in usefulness: -1 somewhat, -2 significantly, -4 severely."),
  M("Line of Sight", "pro", rng(2, 6), "Reach Extreme Range: +2 from Distant, +4 from Close, +6 from Touch (+4 for Strike (Throw))."),
  M("Ongoing", "pro", flat(6), "Damage continues at 1 per page for pages equal to the initial damage."),
  M("Only Inanimate", "con", rng(-1, -4), "Living-only Power affects Inanimate beings instead: -1 any Source, -2 two Sources, -4 one Source."),
  M("Overkill", "con", special("reduces base cost by 1 Hero Point per rank (1/rank becomes 1 per 2 ranks)"), "Targets double successes on active defenses against it."),
  M("Overload", "pro", flat(2), "Boost +3d for one page (lose it for the scene) or +6d (lose it for the issue)."),
  M("Penetrating", "pro", flat(4), "Halves passive defenses except Force Field; incompatible with Weak."),
  M("Phase Shift", "pro", flat(4), "Affects incorporeal targets as if solid, and lets phased users affect solid ones."),
  M("Ranged", "pro", choice(2, 4), "Reach Distant Range: +2 from Close, +4 from Touch (+2 for Strike (Throw))."),
  M("Readied", "con", flat(-2), "Must spend a page (or an action and 1 Resolve) readying between uses."),
  M("Recharge", "con", flat(-4), "Usable only once every 3 pages."),
  M("Resisted", "pro", flat(6), "Special-effect Power can't be resisted with active defenses; GM approval."),
  M("Ricochet", "pro", flat(2), "Bank shots around cover without penalty; 1 Resolve to defeat 2 Minions per net success."),
  M("Selective", "pro", flat(4), "Doesn't affect anyone you don't want; 1 Resolve to pick targets within an area attack."),
  M("Shutdown", "con", rng(-1, -4), "Something blocks or switches it off: -1 often works, -2 occasionally, -4 rarely."),
  M("Side Effect", "con", rng(-1, -4), "Detrimental side effect: -1 annoying, -2 harmful, -4 catastrophic."),
  M("Signature", "con", flat(-1), "Leaves a traceable signature."),
  M("Subtle", "pro", flat(2), "No noise, light or other easily noticed signs at a distance."),
  M("Undetectable", "pro", flat(6), "Can't be detected or traced by ordinary senses; -3d to active defenses against such attacks."),
  M("Sustained", "con", flat(-2), "Costs 1 action per page to maintain."),
  M("Touch", "con", choice(-2, -4), "Requires contact: -2 from Close Range, -4 from Distant Range."),
  M("Toxin", "con", flat(-2), "A disease, drug or poison; toxin immunity blocks it and Resistance defends."),
  M("Trap", "pro", flat(2), "Set the Power as a trap on a small area or object; up to Willpower traps at once."),
  M("Two-Handed", "con", flat(-1), "Needs both hands free."),
  M("Uncontrolled", "con", flat(-2), "Activates on its own in certain situations."),
  M("Unreliable", "con", flat(-2), "Roll 2 dice each use; 6 or less fails and wastes the action."),
  M("Weak", "con", special("reduces base cost by 1 Hero Point per rank (1/rank becomes 1 per 2 ranks)"), "Targets double successes on passive defenses; incompatible with Armor Piercing and Penetrating."),
]

# ---------- PERKS ----------
def K(name, cost, summary): return {"name": name, "cost": cost, "summary": summary}
perks = [
  K("Authority", C_flat(3), "Special legal or social powers such as clearances, immunity or arrest authority."),
  K("Contacts", C_unit(1, "type of contact"), "People in a chosen field who provide information and favors."),
  K("Fame", C_flat(3), "You are famous and recognized favorably (civilian or heroic identity)."),
  K("Fourth Wall", C_flat(6), "Earn 1 Resolve once per scene when a fourth-wall joke gets the table laughing."),
  K("Headquarters", C_unit(1, "3 Base Points"), "A base of operations; each Hero Point buys 3 Base Points."),
  K("Infamy", C_flat(3), "Ordinary criminals fear you."),
  K("Patron", C_flat(3), "A backer supplies information, gear, funding and support."),
  K("Pet/Sidekick", C_unit(1, "10 Hero Points of companion"), "A companion built with 10 Hero Points per Hero Point spent, up to half yours."),
  K("Reputation", C_flat(3), "Credibility and respect in a particular field."),
  K("Resources", C_flat(1), "Access to a supply of specialized equipment, without being rich."),
  K("Unique Vehicle", C_unit(1, "25 Vehicle Points"), "Custom vehicles; each Hero Point buys 25 Vehicle Points."),
  K("Wealth/Great Wealth", C_choice([("Wealth", 3), ("Great Wealth", 6)], False), "Truly rich (3) or among the world's richest, with a once-per-issue absurd purchase for 1 Resolve (6)."),
]

# ---------- FLAWS ----------
def F(name, kind, summary): return {"name": name, "kind": kind, "summary": summary}
flaws = [
  F("Absentminded", "ordinary", "You forget things, sometimes important ones."),
  F("Alter Ego", "ordinary", "A mundane alter ego built on 50 Hero Points that earns 1 per 5 you earn."),
  F("Amnesia", "special", "No memory of your past; counts as both Plot Hook and Condition (+1 Resolve per issue)."),
  F("Aversion/Fear", "ordinary", "You avoid something and react badly when you can't."),
  F("Beast", "ordinary", "Animal-level intellect that struggles with complex reasoning."),
  F("Blind/Deaf", "condition", "Fully blind or deaf; may need a compensating Power."),
  F("Broke", "ordinary", "No money, job or possibly home."),
  F("Clumsy", "ordinary", "You tend to break things."),
  F("Code", "ordinary", "A strict code of conduct beyond ordinary heroism."),
  F("Color Blind", "ordinary", "You can't see or distinguish certain colors."),
  F("Compulsion/Severe Compulsion", "special", "An urge you struggle to resist; a truly irresistible one is a Severe Compulsion (Condition)."),
  F("Creepy", "ordinary", "You unsettle children, psychics and animals."),
  F("Curse", "ordinary", "A supernatural burden of your design, with GM approval."),
  F("Decorum", "ordinary", "You insist on proper manners and social rules."),
  F("Disabled", "condition", "A physical disability; may need a compensating Power."),
  F("Emotionless", "ordinary", "You lack empathy and come across coldly."),
  F("Enemy", "plot_hook", "Someone or some group wants you ruined."),
  F("Finite Power", "ordinary", "A Power relies on a resource that can run out."),
  F("Flashbacks/Guilt", "ordinary", "Triggers leave you helpless for a page or at -2d for 3 pages."),
  F("Frenzy", "ordinary", "You rush in recklessly and may go berserk."),
  F("Frightening", "ordinary", "Your appearance terrifies most people."),
  F("Heavy/Very Heavy", "special", "Weight 5d-6d (Heavy, ordinary) or 7d+ (Very Heavy, Condition)."),
  F("Hidden Agenda", "ordinary", "Secret goals you won't share with allies."),
  F("Illiterate", "ordinary", "You can't read or write a known Earth language."),
  F("Impaired Sense", "ordinary", "Poor hearing, smell or vision; you may choose to fail Perception rolls."),
  F("Insane", "ordinary", "Persistent delusions, breaks with reality or psychotic episodes."),
  F("Light Sensitive", "ordinary", "Bright light disorients or incapacitates you."),
  F("Mute", "ordinary", "Your Hero can't, won't or can't be understood when speaking."),
  F("Night Blind", "ordinary", "Dim light is nearly darkness to you."),
  F("Nocturnal", "ordinary", "Sluggish by day or prone to dozing off."),
  F("Notoriety", "ordinary", "A bad reputation makes people react poorly."),
  F("Obligation", "plot_hook", "A duty such as school or a job constrains you."),
  F("Outsider", "ordinary", "From another time, world or dimension and unfamiliar with local norms."),
  F("Power Limits", "ordinary", "A self-controlled Power limitation that earns Resolve instead of reducing cost."),
  F("Quirk", "ordinary", "An unfortunate personality trait that flares at bad times."),
  F("Reaction/Severe Reaction", "special", "Adverse reactions to a trigger; an irresistible one is a Severe Reaction (Condition)."),
  F("Relationship", "plot_hook", "Friends, family or pets who need your attention and rescue."),
  F("Repair", "condition", "You don't heal naturally and must be repaired."),
  F("Requirement/Severe Requirement", "special", "Something you must periodically satisfy; a desperate, immediate need is a Severe Requirement (Condition)."),
  F("Restriction", "ordinary", "Fewer legal rights than an average person."),
  F("Secret", "ordinary", "A secret you must protect at all costs."),
  F("Secret Identity", "ordinary", "The classic superhero version of Secret."),
  F("Slow", "ordinary", "You move slowly; you may choose to fail speed-related Agility rolls."),
  F("Unlucky/Jinx", "condition", "Once per issue the GM can inflict a free misfortune on you (Unlucky) or a nearby ally (Jinx)."),
  F("Unusual Looks", "ordinary", "An off-putting appearance; redundant with Frightening."),
  F("Unusual Shape", "ordinary", "A non-human body shape with practical drawbacks."),
  F("Vulnerability", "condition", "Defenses are halved against one attack, effect or weapon."),
  F("Wanted", "plot_hook", "Authorities are actively hunting you."),
]

out = {"ruleset": "core", "powers": powers, "generic_mods": gm, "perks": perks, "flaws": flaws}
with open("data/core_rules.json", "w", encoding="utf-8") as f:
    json.dump(out, f, indent=2, ensure_ascii=False)
print("wrote", len(powers), len(gm), len(perks), len(flaws))
