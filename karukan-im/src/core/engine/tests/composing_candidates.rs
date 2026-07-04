//! Tests for suppressing the predictive candidate window while composing.
//!
//! No kanji model is loaded, so composing-phase candidates come from the
//! rewriter path (half-width katakana, emoji shortcodes). That's enough to
//! observe whether the popup is shown or held back until Space.

use crate::core::engine::EngineConfig;

use super::*;

/// Engine with the predictive popup disabled while composing.
fn suppressed_engine() -> InputMethodEngine {
    InputMethodEngine::with_config(EngineConfig {
        show_composing_candidates: false,
        ..EngineConfig::default()
    })
}

fn has_show_candidates(result: &EngineResult) -> bool {
    result
        .actions
        .iter()
        .any(|a| matches!(a, EngineAction::ShowCandidates(_)))
}

fn has_hide_candidates(result: &EngineResult) -> bool {
    result
        .actions
        .iter()
        .any(|a| matches!(a, EngineAction::HideCandidates))
}

fn show_candidate_texts(result: &EngineResult) -> Vec<String> {
    result
        .actions
        .iter()
        .find_map(|a| match a {
            EngineAction::ShowCandidates(list) => {
                Some(list.candidates().iter().map(|c| c.text.clone()).collect())
            }
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn default_engine_shows_predictive_popup_while_composing() {
    // Baseline: with the popup enabled, typing surfaces rewriter candidates
    // (half-width katakana) via a ShowCandidates action during composing.
    let mut engine = InputMethodEngine::new();
    engine.process_key(&press('a'));
    let result = engine.process_key(&press('i'));
    assert!(
        has_show_candidates(&result),
        "default engine should show the predictive popup while composing"
    );
}

#[test]
fn suppressed_engine_hides_popup_while_composing() {
    let mut engine = suppressed_engine();
    engine.process_key(&press('a'));
    let result = engine.process_key(&press('i'));

    // Popup held back...
    assert!(
        !has_show_candidates(&result),
        "suppressed engine must not show candidates while composing"
    );
    assert!(
        has_hide_candidates(&result),
        "suppressed engine must emit HideCandidates to close any stale popup"
    );
    // ...but the preedit still updates so the user sees what they typed.
    assert_eq!(engine.preedit().unwrap().text(), "あい");
}

#[test]
fn suppressed_engine_still_updates_preedit_with_live_conversion() {
    // With live conversion on, the inline preedit must keep updating even
    // though the popup is suppressed (only the window is held back, not the
    // inline conversion).
    let mut engine = suppressed_engine();
    engine.live.enabled = true;
    engine.process_key(&press('a'));
    let result = engine.process_key(&press('i'));

    assert!(!has_show_candidates(&result));
    assert!(
        result
            .actions
            .iter()
            .any(|a| matches!(a, EngineAction::UpdatePreedit(_))),
        "preedit must still update while composing with the popup suppressed"
    );
}

#[test]
fn emoji_mode_is_exempt_from_suppression() {
    // The emoji shortcode picker is an explicit "choose from the list"
    // interaction, so it keeps showing candidates even when the predictive
    // popup is disabled.
    let mut engine = suppressed_engine();
    engine.process_key(&press(':'));
    for ch in ['s', 'm', 'i', 'l'] {
        engine.process_key(&press(ch));
    }
    let result = engine.process_key(&press('e'));

    let texts = show_candidate_texts(&result);
    assert!(
        texts.iter().any(|t| t == "😄"),
        "emoji candidates must show despite suppression, got {texts:?}"
    );
}
