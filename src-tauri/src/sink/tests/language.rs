//! The language setting reaching the tray: the menu labels and the tooltip.

use super::*;

#[test]
fn the_tray_menu_follows_the_language_and_only_when_it_changes() {
    let rig = rig();
    let languages = || rig.rec.languages.lock().expect("lock").clone();
    rig.sink.prime(&settings(None, false));
    assert_eq!(languages(), [Language::PtBr], "the first settings relabel the menu");
    rig.sink.settings_changed(&settings(Some(HarnessId::Codex), false));
    assert_eq!(languages(), [Language::PtBr], "an unrelated change leaves it alone");
    let mut french = settings(None, false);
    french.general.language = Language::Fr;
    rig.sink.settings_changed(&french);
    assert_eq!(languages(), [Language::PtBr, Language::Fr]);
}

#[test]
fn the_tooltip_changes_with_the_language() {
    let rig = rig();
    rig.sink.prime(&settings(None, false));
    rig.sink.snapshot_changed(&snapshot(vec![view(HarnessId::Claude, 28.0, 113)]));
    let mut english = settings(None, false);
    english.general.language = Language::En;
    rig.rec.take();
    rig.sink.settings_changed(&english);
    assert!(
        rig.rec.calls().contains(
            &"tooltip:Claude Code · 5 hours: 72% available · resets in 1h 53m".to_owned()
        ),
        "{:?}",
        rig.rec.calls()
    );
}
