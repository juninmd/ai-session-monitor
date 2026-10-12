use usage_model::{HarnessId, Settings};

use super::tests::store_in;

#[test]
fn tab_preferences_survive_reopening_the_store_including_an_empty_selection() {
    let dir = tempfile::tempdir().expect("tempdir");
    for tabs in [vec![HarnessId::Codex, HarnessId::Litellm], vec![]] {
        let mut settings = Settings::default();
        settings.general.visible_tabs = tabs.clone();
        settings.claude.enabled = false;
        store_in(dir.path()).save_then(&settings, || {}).expect("save");

        let loaded = store_in(dir.path()).load(1_800_000_000).expect("reopen");
        assert!(loaded.recovered.is_none());
        assert_eq!(loaded.settings.general.visible_tabs, tabs);
        assert!(!loaded.settings.claude.enabled);
        assert!(loaded.settings.codex.enabled);
    }
}
