//! Event names the shell emits to the flyout webview, pinned to the shared contract by a test.

pub const SNAPSHOT: &str = "snapshot";
pub const SETTINGS_CHANGED: &str = "settings-changed";
pub const FLYOUT_SHOWN: &str = "flyout-shown";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_the_ipc_contract() {
        let contract: serde_json::Value =
            serde_json::from_str(include_str!("../../contract/ipc-commands.json")).expect("json");
        let mut declared: Vec<&str> = contract["events"]
            .as_array()
            .expect("events array")
            .iter()
            .map(|e| e.as_str().expect("string"))
            .collect();
        declared.sort_unstable();
        let mut ours = vec![SNAPSHOT, SETTINGS_CHANGED, FLYOUT_SHOWN];
        ours.sort_unstable();
        assert_eq!(declared, ours);
    }
}
