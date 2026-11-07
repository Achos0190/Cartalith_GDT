//! Running a `propose_actions` card: the rules that decide whether Apply may run at all
//! (`ASSISTANT_ACTIONS_PROPOSAL.md` s.3.5, milestone AC-3; Rulings DW, DD).
//!
//! This module only DECIDES. It holds no handle to the app and runs nothing: the shell
//! (`shell/llm_actions.gd::run_card`) reads the answer and, only when it is empty, runs each action
//! through the existing click path of its [`Route`]. Keeping the decision here is what lets the hostile
//! corpus reach it (a Godot probe cannot loop over hundreds of rows), and keeps ONE copy of the rules.
//!
//! ## The rules (numbered `C1`..`C7`, like the validators' `A`, `B` and `V` rules)
//!
//! | Rule | Refuses | Scope |
//! |---|---|---|
//! | `C1` | a card with nothing on it | card |
//! | `C2` | more than [`MAX_CARD_ACTIONS`] actions (AQ3) | card |
//! | `C3` | any Apply while a world is generating (scope V13): the engine is bound to the worker | card |
//! | `C4` | a Session-mode card that carries a gated action ([`crate::actions::ActionEntry::session_gated`]) | action |
//! | `C5` | an action with no click path ([`Route::None`]: context actions, unreviewed rows, an unknown route) or no id | action |
//! | `C6` | the same id twice (the validator's `B9`, repeated because a card dictionary crosses the Godot boundary) | action |
//! | `C7` | a setting while the world is finalized (its settings are locked) | action |
//!
//! Nothing destructive can reach here: the catalogue has no destructive entry and `propose_actions`
//! refuses anything unavailable or unreviewed. A refusal list is the WHOLE answer: Apply runs only on an
//! empty list, never a subset, so a card is applied entirely or not at all (one undo group).
//!
//! MUST NEVER: run, open, arm or write anything; treat a missing field as permission (a missing
//! `session_gated` counts as gated, an unknown route as [`Route::None`]); return `Ok` for a card with a
//! refusal in it.

use std::collections::BTreeSet;

use crate::action_tools::MAX_CARD_ACTIONS;
use crate::actions::Route;

/// One action of a card as the shell reads it back (the fields the verdict dictionary carries).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecItem {
    /// The catalogue id.
    pub id: String,
    /// Its click path.
    pub route: Route,
    /// Whether Session mode refuses it.
    pub session_gated: bool,
}

impl ExecItem {
    /// An item from the fields a card dictionary may carry. **An absent field is never permission**: no
    /// `route` or an unknown one is [`Route::None`] (C5), no `session_gated` is gated, no `id` is the empty
    /// id (shown as such). The one place the defaults are chosen, so the Godot boundary cannot pick a
    /// plausible one.
    pub fn from_wire(id: Option<&str>, route: Option<&str>, session_gated: Option<bool>) -> Self {
        ExecItem { id: id.unwrap_or_default().to_owned(), route: route.map_or(Route::None, Route::parse), session_gated: session_gated.unwrap_or(true) }
    }
}

/// The app's state at the moment of Apply, read by the shell each time (never cached on the card).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecContext {
    /// Session mode is on (`DccApp.is_session_mode()`).
    pub session_mode: bool,
    /// A world is generating (`bridge.generating`).
    pub generating: bool,
    /// The world is finalized (`bridge.is_finalized()`); its settings are locked.
    pub finalized: bool,
}

/// The execution rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecRule {
    /// Nothing on the card.
    C1,
    /// Too many actions.
    C2,
    /// A world is generating.
    C3,
    /// Session mode refuses it.
    C4,
    /// No click path.
    C5,
    /// A duplicate id.
    C6,
    /// A setting on a finalized world.
    C7,
}

impl ExecRule {
    /// `"C1"` .. `"C7"`.
    pub fn id(self) -> &'static str {
        match self {
            ExecRule::C1 => "C1",
            ExecRule::C2 => "C2",
            ExecRule::C3 => "C3",
            ExecRule::C4 => "C4",
            ExecRule::C5 => "C5",
            ExecRule::C6 => "C6",
            ExecRule::C7 => "C7",
        }
    }
}

/// Why Apply may not run. `id` is the action concerned, or `None` for a whole-card refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecRefusal {
    /// The action, when the refusal is about one.
    pub id: Option<String>,
    /// Which rule.
    pub rule: ExecRule,
    /// The reason, written for the person reading the card.
    pub why: String,
}

/// Session mode's reason, in the words the menus use (`menus.gd::SESSION_GATED_TIP`).
pub const SESSION_WHY: &str = "Session mode keeps generation out of reach at the table, so this cannot run from a card. Window > Session mode (Ctrl+Shift+M) turns it off.";

/// Every reason Apply may not run `items` now; empty means it may. Card-level refusals (C1-C3) come first
/// and stop the check: when the whole card is refused there is no point listing each action too.
pub fn check_card(items: &[ExecItem], ctx: &ExecContext) -> Vec<ExecRefusal> {
    let card = |rule, why: String| vec![ExecRefusal { id: None, rule, why }];
    if items.is_empty() {
        return card(ExecRule::C1, "There is nothing on this card to apply.".into());
    }
    if items.len() > MAX_CARD_ACTIONS {
        return card(ExecRule::C2, format!("{} actions is more than the {MAX_CARD_ACTIONS} one card may carry.", items.len()));
    }
    if ctx.generating {
        return card(ExecRule::C3, "A world is generating; nothing can be applied until it finishes.".into());
    }
    let mut out = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for it in items {
        let mut refuse = |rule, why: String| out.push(ExecRefusal { id: Some(it.id.clone()), rule, why });
        if it.id.is_empty() {
            refuse(ExecRule::C5, "An action on this card has no id, so there is nothing to run.".into());
            continue;
        }
        if !seen.insert(it.id.as_str()) {
            refuse(ExecRule::C6, "The same action appears twice on one card.".into());
            continue;
        }
        if it.route == Route::None {
            refuse(ExecRule::C5, "The assistant has no way to run this from a card; run it yourself from its menu or the map.".into());
            continue;
        }
        if ctx.session_mode && it.session_gated {
            refuse(ExecRule::C4, SESSION_WHY.into());
            continue;
        }
        if ctx.finalized && it.route == Route::Param {
            refuse(ExecRule::C7, "This world is finalized; its settings are locked (Unfinalize it first).".into());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, route: Route, gated: bool) -> ExecItem {
        ExecItem { id: id.into(), route, session_gated: gated }
    }

    fn idle() -> ExecContext {
        ExecContext { session_mode: false, generating: false, finalized: false }
    }

    fn rules(v: &[ExecRefusal]) -> Vec<&'static str> {
        v.iter().map(|r| r.rule.id()).collect()
    }

    /// Protects "a runnable card is not refused": a mixed card of one setting, one view verb, one window
    /// and one selection passes in an idle app. MUTATION: make `check_card` return a C5 for
    /// `Route::Window` and this goes red.
    #[test]
    fn a_mixed_card_passes_when_the_app_is_idle() {
        let card = [item("param:relief.peak_height", Route::Param, true), item("view.layer.rivers", Route::View, false), item("window.factions", Route::Window, false), item("select.settlement", Route::Select, false)];
        assert_eq!(check_card(&card, &idle()), vec![]);
    }

    /// Protects the card-level rules with literals: empty is C1, six is C2 (five is not), generating is C3.
    /// MUTATION: change `>` to `>=` in the C2 test and the five-action card goes red.
    #[test]
    fn card_level_refusals_use_the_bar_of_five() {
        assert_eq!(rules(&check_card(&[], &idle())), ["C1"]);
        let five: Vec<ExecItem> = (0..5).map(|i| item(&format!("view.layer.l{i}"), Route::View, false)).collect();
        assert_eq!(check_card(&five, &idle()), vec![], "five is the bar, not over it");
        let six: Vec<ExecItem> = (0..6).map(|i| item(&format!("view.layer.l{i}"), Route::View, false)).collect();
        assert_eq!(rules(&check_card(&six, &idle())), ["C2"]);
        let gen_ctx = ExecContext { generating: true, ..idle() };
        assert_eq!(rules(&check_card(&five, &gen_ctx)), ["C3"], "generating refuses the whole card, once");
    }

    /// Protects Session mode on the card: a gated action is refused with its id and the reason, an ungated
    /// one on the same card is not, and with the mode off nothing is. MUTATION: ignore `session_gated` and
    /// the first assertion goes red; ignore `session_mode` and the last does.
    #[test]
    fn session_mode_refuses_only_the_gated_actions() {
        let card = [item("param:erosion.strength", Route::Param, true), item("window.factions", Route::Window, false), item("tool.sculpt", Route::Tool, true)];
        let on = ExecContext { session_mode: true, ..idle() };
        let got = check_card(&card, &on);
        assert_eq!(got.iter().map(|r| (r.id.as_deref(), r.rule.id())).collect::<Vec<_>>(), [(Some("param:erosion.strength"), "C4"), (Some("tool.sculpt"), "C4")]);
        assert!(got[0].why.contains("Session mode"));
        assert_eq!(check_card(&card, &idle()), vec![]);
    }

    /// Protects "no value is not permission": an action whose route is `None` is C5 whatever else is true,
    /// a duplicate is C6, and a setting on a finalized world is C7 but a view verb is not.
    #[test]
    fn a_missing_route_a_duplicate_and_a_locked_world_are_refused() {
        let ctx = ExecContext { finalized: true, ..idle() };
        let card = [item("civ.start_way", Route::None, false), item("view.layer.rivers", Route::View, false), item("view.layer.rivers", Route::View, false), item("param:a", Route::Param, false)];
        let got = check_card(&card, &ctx);
        assert_eq!(got.iter().map(|r| (r.id.as_deref().unwrap(), r.rule.id())).collect::<Vec<_>>(), [("civ.start_way", "C5"), ("view.layer.rivers", "C6"), ("param:a", "C7")]);
    }

    /// Protects `from_wire`: every absent field takes the refusing value. MUTATION: default `session_gated`
    /// to `false` and the second assertion goes red.
    #[test]
    fn an_absent_field_is_never_permission() {
        let it = ExecItem::from_wire(None, None, None);
        assert_eq!((it.id.as_str(), it.route, it.session_gated), ("", Route::None, true));
        let on = ExecContext { session_mode: true, ..idle() };
        assert_eq!(rules(&check_card(&[ExecItem::from_wire(Some("window.vault"), Some("window"), None)], &on)), ["C4"]);
        assert_eq!(rules(&check_card(&[ExecItem::from_wire(Some("window.vault"), None, Some(false))], &idle())), ["C5"]);
    }

    /// Protects the wire names of the routes the shell switches on: a rename here without the shell goes
    /// red. Literals on purpose.
    #[test]
    fn route_names_are_the_shells_words() {
        let all = [(Route::Param, "param"), (Route::View, "view"), (Route::Window, "window"), (Route::Menu, "menu"), (Route::Year, "year"), (Route::Select, "select"), (Route::Tool, "tool"), (Route::None, "none")];
        for (r, s) in all {
            assert_eq!(r.as_str(), s);
            assert_eq!(Route::parse(s), r);
        }
        assert_eq!(Route::parse("nonsense"), Route::None, "an unknown route is the refused one");
        assert_eq!(Route::parse(""), Route::None);
    }
}
