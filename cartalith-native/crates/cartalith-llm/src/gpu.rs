//! The local-provider GPU rule (Ruling DF, `LLM_ASSISTANT_SCOPE.md` s.5a), as
//! pure decisions.
//!
//! WHY: when the language model runs on the same machine it competes with
//! Cartalith for the dedicated GPU. The owner asked for a warning always, and
//! for Cartalith to scale back to the lowest-power GPU when the server is on
//! the same machine, with a user override. This module decides *what the
//! rule says*; applying it (pointing the compute path at another adapter and
//! restoring the previous choice) is the shell's job, because the preference
//! lives in `cartalith-gpu` and this crate depends on no other crate.
//!
//! WHAT IT CAN AND CANNOT SWITCH (scope s.5a, verified 2026-10-25): only the
//! wgpu compute path, by device key, no restart. The Godot display adapter is
//! not switchable from inside the running app under `gl_compatibility`, so for
//! it the rule is the warning and the "detected as" line, and
//! [`GpuPlan::effect`] says so rather than implying more.
//!
//! MUST NEVER: pick a discrete adapter as the "low-power" one, resolve a name
//! to decide the class, or report "will switch" when no integrated adapter
//! exists.

use crate::host::HostClass;

/// The user's override (Ruling DF rule 3). Default [`GpuOverride::Automatic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuOverride {
    /// Follow the detected class: switch for the same machine only.
    Automatic,
    /// Switch whatever the detected class.
    AlwaysLowPower,
    /// Never touch the GPU choice.
    NeverChange,
}

impl GpuOverride {
    /// The id stored in settings (never renamed once shipped).
    pub const fn id(self) -> &'static str {
        match self {
            GpuOverride::Automatic => "automatic",
            GpuOverride::AlwaysLowPower => "always",
            GpuOverride::NeverChange => "never",
        }
    }

    /// Parse a stored id. An unknown or empty id is `None`: the caller shows
    /// the default and does not pretend the stored text meant something.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "automatic" => Some(GpuOverride::Automatic),
            "always" => Some(GpuOverride::AlwaysLowPower),
            "never" => Some(GpuOverride::NeverChange),
            _ => None,
        }
    }

    /// The words the picker shows.
    pub const fn label(self) -> &'static str {
        match self {
            GpuOverride::Automatic => "Automatic",
            GpuOverride::AlwaysLowPower => "Always low-power GPU",
            GpuOverride::NeverChange => "Never change",
        }
    }
}

/// One enumerated adapter, as `WorldGen.gpu_enumerate_devices` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adapter {
    /// The stable device key `gpu_set_selected_devices` takes.
    pub key: String,
    /// `"discrete"`, `"integrated"`, `"virtual"`, `"software"` or `"other"`.
    pub kind: String,
}

/// The adapter to scale back to: the first **integrated** one. `None` when the
/// machine has none (only a discrete card, a software rasteriser, a virtual
/// device). Labelled judgement: a software adapter is not chosen, because the
/// Windows Basic Render Driver would run the whole pipeline at rasteriser
/// speed, a worse outcome than leaving the choice alone; the scope's phrase
/// "else the software path" is read as "else nothing", and the effect line
/// says so. The owner may overrule.
pub fn pick_low_power(adapters: &[Adapter]) -> Option<&Adapter> {
    adapters.iter().find(|a| a.kind == "integrated")
}

/// What the settings screen shows and what the shell does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPlan {
    /// Show the warning at all (a local provider is selected).
    pub warn: bool,
    /// The warning's headline.
    pub headline: String,
    /// The "detected as" line: always names the class (Ruling DF rule 4).
    pub detected: String,
    /// What will happen, in words.
    pub effect: String,
    /// The shell should point the compute path at [`GpuPlan::target`].
    pub switch: bool,
    /// The device key to select when `switch` is true. Absent when there is
    /// nothing to switch to; never an empty string standing for "none".
    pub target: Option<String>,
}

/// Decide the rule.
///
/// * `local_selected`: a local provider is selected and the feature is on
///   (`Preset::is_local`, or a custom endpoint classed same-machine or LAN).
/// * `class`: the base-URL host's class, or `None` when the URL did not parse
///   (then nothing is detected and nothing switches).
/// * `ov`: the user's override.
/// * `adapters`: this machine's adapters (empty is normal on a headless box).
pub fn plan(local_selected: bool, class: Option<HostClass>, ov: GpuOverride, adapters: &[Adapter]) -> GpuPlan {
    if !local_selected {
        return GpuPlan {
            warn: false,
            headline: String::new(),
            detected: String::new(),
            effect: String::new(),
            switch: false,
            target: None,
        };
    }
    let headline = "Cartalith and the language model may compete for the GPU. If the model runs on this \
                    computer, give it the dedicated card and let Cartalith use the integrated one."
        .to_owned();
    let detected = match class {
        Some(c) => format!("Detected as: {}.", c.label()),
        None => "Detected as: unknown (the address could not be read).".to_owned(),
    };
    let want = match ov {
        GpuOverride::NeverChange => false,
        GpuOverride::AlwaysLowPower => true,
        GpuOverride::Automatic => class == Some(HostClass::SameMachine),
    };
    let display_note = " The display adapter is not changed from inside the app; use Windows Settings, \
                        Graphics, to choose the GPU for Cartalith itself.";
    if !want {
        let why = match (ov, class) {
            (GpuOverride::NeverChange, _) => "Override is Never change: the GPU choice is left alone.".to_owned(),
            (_, Some(HostClass::LocalNetwork)) => {
                "The model is on another computer on your network, so nothing is switched.".to_owned()
            }
            (_, Some(HostClass::Remote)) => "The model is remote, so nothing is switched.".to_owned(),
            _ => "The address does not say it is this computer, so nothing is switched; set the override to \
                  Always low-power GPU if it is."
                .to_owned(),
        };
        return GpuPlan { warn: true, headline, detected, effect: format!("{why}{display_note}"), switch: false, target: None };
    }
    match pick_low_power(adapters) {
        Some(a) => GpuPlan {
            warn: true,
            headline,
            detected,
            effect: format!(
                "While this provider is active, generation uses the integrated GPU instead of the dedicated \
                 one, from the next generate. Your own GPU choice returns when you turn this off.{display_note}"
            ),
            switch: true,
            target: Some(a.key.clone()),
        },
        None => GpuPlan {
            warn: true,
            headline,
            detected,
            effect: format!("No integrated GPU was found on this computer, so nothing is switched.{display_note}"),
            switch: false,
            target: None,
        },
    }
}

/// The rule on a phone (LLM-10; Ruling DF read for a handset). A phone has one
/// GPU, so there is nothing to scale back to and nothing is ever switched,
/// whatever the override says; the override picker is not shown there. The
/// warning still names the detected class (Ruling DF rule 4), and the text
/// never gives desktop advice: no "dedicated card", no "low-power GPU", no
/// Windows settings.
///
/// * The model on another computer on the LAN (the expected phone set-up, an
///   LM Studio or Ollama server on the user's PC): nothing is shared.
/// * The model on the phone itself (a server app on `localhost`): it shares
///   the one GPU and the memory, which is worth saying, and nothing switches.
pub fn plan_phone(local_selected: bool, class: Option<HostClass>) -> GpuPlan {
    if !local_selected {
        return GpuPlan {
            warn: false,
            headline: String::new(),
            detected: String::new(),
            effect: String::new(),
            switch: false,
            target: None,
        };
    }
    let headline =
        "A language model running on this phone shares its one GPU and its memory with Cartalith.".to_owned();
    let detected = match class {
        Some(c) => format!("Detected as: {}.", c.label()),
        None => "Detected as: unknown (the address could not be read).".to_owned(),
    };
    let effect = match class {
        Some(HostClass::SameMachine) => {
            "The model is on this phone, so generating and the model will slow each other down. A phone has one \
             GPU, so nothing is switched."
        }
        Some(HostClass::LocalNetwork) => {
            "The model is on another computer on your network, so this phone's GPU is not shared and nothing is \
             switched."
        }
        Some(HostClass::Remote) => "The model is remote, so this phone's GPU is not shared and nothing is switched.",
        _ => "The address does not say where the model runs. A phone has one GPU, so nothing is switched either way.",
    }
    .to_owned();
    GpuPlan { warn: true, headline, detected, effect, switch: false, target: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Protects the LLM-10 phone form: it never switches, always names the
    /// class, and never carries desktop advice. MUTATION: return `switch:
    /// true` for SameMachine, or reuse the desktop headline or display note,
    /// and a row goes red.
    #[test]
    fn the_phone_plan_switches_nothing_and_gives_no_desktop_advice() {
        for c in [HostClass::SameMachine, HostClass::LocalNetwork, HostClass::Remote, HostClass::Unknown] {
            let p = plan_phone(true, Some(c));
            assert!(p.warn && !p.switch && p.target.is_none(), "{c:?}");
            assert_eq!(p.detected, format!("Detected as: {}.", c.label()));
            let all = format!("{} {} {}", p.headline, p.detected, p.effect).to_lowercase();
            for banned in ["low-power", "integrated", "dedicated", "windows", "computer's gpu", "display adapter"] {
                assert!(!all.contains(banned), "{c:?}: {banned:?} in {all:?}");
            }
            assert!(!p.effect.contains("  ") && !p.headline.contains("  "), "double space");
        }
        assert_eq!(
            plan_phone(true, Some(HostClass::LocalNetwork)).effect,
            "The model is on another computer on your network, so this phone's GPU is not shared and nothing is \
             switched."
        );
        assert!(plan_phone(true, Some(HostClass::SameMachine)).effect.contains("on this phone"));
        let p = plan_phone(true, None);
        assert!(p.detected.contains("unknown") && !p.switch);
        let off = plan_phone(false, Some(HostClass::SameMachine));
        assert!(!off.warn && off.headline.is_empty() && off.detected.is_empty() && off.effect.is_empty());
    }

    fn rig() -> Vec<Adapter> {
        vec![
            Adapter { key: "10de:2684:RTX".into(), kind: "discrete".into() },
            Adapter { key: "1002:164e:Radeon Graphics".into(), kind: "integrated".into() },
            Adapter { key: "1414:008c:Basic".into(), kind: "software".into() },
        ]
    }

    /// Protects: the override table of s.5a, every class against every value.
    /// MUTATION: make Automatic switch for `LocalNetwork` and the LAN row
    /// goes red; make Never switch and the Never row goes red.
    #[test]
    fn override_table() {
        use GpuOverride::*;
        use HostClass::*;
        let want = |c, o| plan(true, Some(c), o, &rig()).switch;
        for (c, auto) in [(SameMachine, true), (LocalNetwork, false), (Remote, false), (Unknown, false)] {
            assert_eq!(want(c, Automatic), auto, "automatic {c:?}");
            assert!(want(c, AlwaysLowPower), "always {c:?}");
            assert!(!want(c, NeverChange), "never {c:?}");
        }
    }

    /// Protects: a non-local provider shows no warning and switches nothing,
    /// whatever the override says.
    #[test]
    fn no_local_provider_no_warning() {
        for o in [GpuOverride::Automatic, GpuOverride::AlwaysLowPower, GpuOverride::NeverChange] {
            let p = plan(false, Some(HostClass::SameMachine), o, &rig());
            assert!(!p.warn && !p.switch && p.target.is_none() && p.detected.is_empty(), "{o:?}");
        }
    }

    /// Protects: the warning always states the detected class, in the ruling's
    /// words, for every class and every override.
    #[test]
    fn detected_line_always_names_the_class() {
        for c in [HostClass::SameMachine, HostClass::LocalNetwork, HostClass::Remote, HostClass::Unknown] {
            for o in [GpuOverride::Automatic, GpuOverride::AlwaysLowPower, GpuOverride::NeverChange] {
                let p = plan(true, Some(c), o, &rig());
                assert!(p.warn);
                assert_eq!(p.detected, format!("Detected as: {}.", c.label()));
                assert!(!p.effect.is_empty() && !p.headline.is_empty());
                assert!(!p.effect.contains("  ") && !p.headline.contains("  "), "double space in plan text");
            }
        }
        let p = plan(true, None, GpuOverride::Automatic, &rig());
        assert!(p.detected.contains("unknown") && !p.switch);
    }

    /// Protects: the target is the integrated adapter, never the discrete one
    /// or the software one, and absent when there is none.
    #[test]
    fn target_is_the_integrated_adapter_only() {
        let p = plan(true, Some(HostClass::SameMachine), GpuOverride::Automatic, &rig());
        assert_eq!(p.target.as_deref(), Some("1002:164e:Radeon Graphics"));
        let only_discrete = vec![Adapter { key: "d".into(), kind: "discrete".into() }];
        let p = plan(true, Some(HostClass::SameMachine), GpuOverride::AlwaysLowPower, &only_discrete);
        assert!(!p.switch && p.target.is_none() && p.effect.contains("No integrated GPU"));
        let sw = vec![Adapter { key: "s".into(), kind: "software".into() }];
        assert!(pick_low_power(&sw).is_none());
        assert!(pick_low_power(&[]).is_none());
        // Honest wording: a switch claim only when a switch will happen.
        let p = plan(true, Some(HostClass::SameMachine), GpuOverride::Automatic, &[]);
        assert!(!p.switch && !p.effect.contains("uses the integrated"));
    }

    /// Protects: the display adapter is never claimed switchable.
    #[test]
    fn the_display_adapter_is_never_promised() {
        for o in [GpuOverride::Automatic, GpuOverride::AlwaysLowPower, GpuOverride::NeverChange] {
            let p = plan(true, Some(HostClass::SameMachine), o, &rig());
            assert!(p.effect.contains("display adapter is not changed"), "{o:?}");
        }
    }

    /// Protects: override ids round-trip and an unknown id is not mapped to a
    /// plausible value.
    #[test]
    fn override_ids_round_trip() {
        for o in [GpuOverride::Automatic, GpuOverride::AlwaysLowPower, GpuOverride::NeverChange] {
            assert_eq!(GpuOverride::from_id(o.id()), Some(o));
        }
        assert_eq!(GpuOverride::from_id(""), None);
        assert_eq!(GpuOverride::from_id("Automatic"), None);
        assert_eq!(GpuOverride::Automatic.label(), "Automatic");
        assert_eq!(GpuOverride::AlwaysLowPower.label(), "Always low-power GPU");
        assert_eq!(GpuOverride::NeverChange.label(), "Never change");
    }
}
