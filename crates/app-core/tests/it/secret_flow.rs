//! Secrets: stored through the service, read by collectors, and never shown anywhere.

use std::collections::BTreeMap;
use std::sync::Arc;

use usage_model::{ErrorCode, HarnessId, SecretKind, SecretStatus};

use crate::doubles::{FailingStore, OneUnreadable};
use crate::rig::{rig, seconds, settle};

mod binding;

/// Fabricated; it only has to be recognisable in a dump.
const KEY: &str = "sk-fake-4f9a0c2e7b1d";
const LITELLM: HarnessId = HarnessId::Litellm;
const OPENCODE: HarnessId = HarnessId::Opencode;
const KIMI: HarnessId = HarnessId::Kimi;
const ZAI: HarnessId = HarnessId::Zai;
const OPENROUTER: HarnessId = HarnessId::Openrouter;
const DEEPSEEK: HarnessId = HarnessId::Deepseek;

fn only_field(name: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("field".to_owned(), name.to_owned())])
}

#[tokio::test(start_paused = true)]
async fn a_saved_secret_is_trimmed_stored_and_read_at_once_inside_the_refresh_gap() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Litellm, format!("  {KEY}\n")).expect("stored");
    settle().await;

    assert_eq!(status, SecretStatus { litellm: true, ..SecretStatus::default() });
    let calls = rig.fake(LITELLM).calls();
    assert_eq!(calls.len(), 2, "read again one second after the first read");
    assert_eq!(
        (calls[0].litellm_key.as_deref(), calls[1].litellm_key.as_deref()),
        (None, Some(KEY))
    );
    for id in [HarnessId::Claude, HarnessId::Codex, HarnessId::Antigravity, OPENCODE] {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read this secret");
    }
}

#[tokio::test(start_paused = true)]
async fn whitespace_of_any_script_around_a_pasted_key_is_trimmed() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    // No-break space, em space, ideographic space, next line: what a web page copy can carry.
    let pasted = format!("\u{a0}\u{2003}{KEY}\u{3000}\u{85}");
    rig.service.set_secret(SecretKind::Litellm, pasted).expect("stored");
    settle().await;

    assert_eq!(rig.fake(LITELLM).calls()[1].litellm_key.as_deref(), Some(KEY));
}

#[tokio::test(start_paused = true)]
async fn each_secret_refreshes_only_its_own_harness() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Opencode, KEY.to_owned()).expect("stored");
    settle().await;

    assert_eq!(status, SecretStatus { opencode: true, ..SecretStatus::default() });
    assert_eq!((rig.fake(OPENCODE).count(), rig.fake(LITELLM).count()), (2, 1));
    assert_eq!(rig.fake(OPENCODE).calls()[1].opencode_key.as_deref(), Some(KEY));
}

#[tokio::test(start_paused = true)]
async fn the_openrouter_and_deepseek_keys_refresh_only_their_own_harness() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.set_secret(SecretKind::Openrouter, KEY.to_owned()).expect("stored");
    settle().await;
    assert_eq!(status, SecretStatus { openrouter: true, ..SecretStatus::default() });
    assert_eq!((rig.fake(OPENROUTER).count(), rig.fake(DEEPSEEK).count()), (2, 1));
    let call = &rig.fake(OPENROUTER).calls()[1];
    assert_eq!((call.openrouter_key.as_deref(), call.deepseek_key.as_deref()), (Some(KEY), None));

    let status = rig.service.set_secret(SecretKind::Deepseek, KEY.to_owned()).expect("stored");
    settle().await;
    assert_eq!(
        status,
        SecretStatus { openrouter: true, deepseek: true, ..SecretStatus::default() }
    );
    assert_eq!((rig.fake(OPENROUTER).count(), rig.fake(DEEPSEEK).count()), (2, 2));
    assert_eq!(rig.fake(DEEPSEEK).calls()[1].deepseek_key.as_deref(), Some(KEY));
    for id in [HarnessId::Claude, HarnessId::Codex, KIMI, ZAI, LITELLM] {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read these secrets");
    }
}

#[tokio::test(start_paused = true)]
async fn the_kimi_and_zai_keys_refresh_only_their_own_harness_and_report_in_the_status() {
    let rig = rig().start().await;
    rig.at(seconds(1)).await;

    let status = rig
        .service
        .set_secret(
            SecretKind::Kimi,
            format!(
                " {KEY}
"
            ),
        )
        .expect("stored");
    settle().await;
    assert_eq!(status, SecretStatus { kimi: true, ..SecretStatus::default() });
    assert_eq!(rig.fake(KIMI).count(), 2);
    assert_eq!(rig.fake(KIMI).calls()[1].kimi_key.as_deref(), Some(KEY));
    assert_eq!(rig.fake(KIMI).calls()[1].zai_key, None, "a key never reaches another tab's call");

    let status = rig.service.set_secret(SecretKind::Zai, KEY.to_owned()).expect("stored");
    settle().await;
    assert_eq!(status, SecretStatus { kimi: true, zai: true, ..SecretStatus::default() });
    assert_eq!((rig.fake(KIMI).count(), rig.fake(ZAI).count()), (2, 2));
    assert_eq!(rig.fake(ZAI).calls()[1].zai_key.as_deref(), Some(KEY));
    for id in [HarnessId::Claude, HarnessId::Codex, HarnessId::Antigravity, OPENCODE, LITELLM] {
        assert_eq!(rig.fake(id).count(), 1, "{id:?} does not read these secrets");
    }

    assert!(!rig.service.clear_secret(SecretKind::Kimi).expect("cleared").kimi);
    settle().await;
    assert_eq!(rig.fake(KIMI).calls()[2].kimi_key, None);
}

#[tokio::test(start_paused = true)]
async fn clearing_is_idempotent_and_reads_the_harness_without_the_secret() {
    let rig = rig().start().await;
    rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("stored");
    settle().await;

    let cleared = rig.service.clear_secret(SecretKind::Litellm).expect("cleared");
    settle().await;
    assert_eq!(cleared, SecretStatus::default());
    assert_eq!(rig.fake(LITELLM).calls().last().map(|c| c.litellm_key.clone()), Some(None));

    let again = rig.service.clear_secret(SecretKind::Litellm).expect("nothing to clear is fine");
    assert_eq!(again, SecretStatus::default());
}

#[tokio::test(start_paused = true)]
async fn the_secret_never_reaches_events_debug_output_errors_or_disk() {
    let rig = rig().start().await;
    // The form's order: a key stored before a server is dropped when one is set.
    rig.edit(|s| s.litellm.base_url = Some("https://litellm.example.test".to_owned()));
    rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("stored");
    rig.service.refresh(None);
    rig.after(seconds(900)).await;
    let seen = rig.fake(LITELLM).calls().last().and_then(|c| c.litellm_key.clone());
    assert_eq!(seen.as_deref(), Some(KEY), "control: the collector did get it");

    let blank = rig.service.set_secret(SecretKind::Litellm, "   ".to_owned()).expect_err("blank");
    let long = rig.service.set_secret(SecretKind::Opencode, "x".repeat(5000)).expect_err("long");
    let surfaces = [
        rig.sink.dump(),
        format!("{:?}", rig.service),
        format!("{:?}", rig.service.secret_status()),
        format!("{:?}", rig.service.settings()),
        format!("{:?}", rig.service.snapshot()),
        format!("{blank:?} {long:?}"),
        format!("{:?}", rig.secrets.get(SecretKind::Litellm)),
        rig.data_text(),
    ];
    for (index, surface) in surfaces.iter().enumerate() {
        assert!(!surface.contains(KEY), "the key is in surface #{index}");
        assert!(!surface.contains("4f9a0c2e7b1d"), "part of the key is in surface #{index}");
    }
    assert!(!surfaces[5].contains("xxxxxxxx"), "a rejected value is not echoed back");
}

#[tokio::test(start_paused = true)]
async fn a_bad_value_is_rejected_by_field_name_only_and_changes_nothing() {
    let rig = rig().start().await;
    let rejected = [
        String::new(),
        " \t\r\n ".to_owned(),
        "\u{a0}\u{2003}\u{3000}".to_owned(),
        "a".repeat(4097),
        "é".repeat(4097),
        format!(" {} ", "a".repeat(4097)),
    ];
    for value in rejected {
        let error = rig.service.set_secret(SecretKind::Litellm, value).expect_err("rejected");
        assert_eq!((error.code, error.retryable), (ErrorCode::InvalidInput, false));
        assert_eq!(error.params, only_field("value"));
    }
    settle().await;
    assert_eq!(rig.service.secret_status().expect("status"), SecretStatus::default());
    assert_eq!(rig.fake(LITELLM).count(), 1, "a rejected secret does not trigger a read");

    let accepted = ["a".repeat(4096), format!(" {} ", "a".repeat(4096)), "é".repeat(4096)];
    for value in accepted {
        assert!(
            rig.service.set_secret(SecretKind::Litellm, value).is_ok(),
            "limit is in characters"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn a_failing_secret_store_is_reported_and_triggers_nothing() {
    let rig = rig().secret_store(Arc::new(FailingStore)).start().await;
    rig.at(seconds(1)).await;

    let status = rig.service.secret_status().expect_err("the store is down");
    let set = rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect_err("not stored");
    let clear = rig.service.clear_secret(SecretKind::Litellm).expect_err("not cleared");
    settle().await;

    for error in [&status, &set, &clear] {
        assert_eq!((error.code, error.retryable), (ErrorCode::Internal, false));
        assert_eq!(error.params.get("where").map(String::as_str), Some("secret_store"));
    }
    assert!(!format!("{set:?}").contains(KEY));
    assert_eq!(rig.fake(LITELLM).count(), 1, "nothing changed, so nothing is read again");
}

#[tokio::test(start_paused = true)]
async fn a_write_that_went_through_is_not_reported_as_failed_because_the_other_secret_is_unreadable()
 {
    let store = Arc::new(OneUnreadable::new(SecretKind::Opencode));
    let rig = rig().secret_store(store).start().await;
    rig.at(seconds(1)).await;

    let set = rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("it was stored");
    settle().await;
    assert_eq!(set, SecretStatus { litellm: true, ..SecretStatus::default() });
    assert_eq!(rig.fake(LITELLM).count(), 2, "and the harness was told to read it");
    assert_eq!(rig.fake(LITELLM).calls()[1].litellm_key.as_deref(), Some(KEY));

    let cleared = rig.service.clear_secret(SecretKind::Litellm).expect("it was removed");
    assert_eq!(cleared, SecretStatus::default());

    let error = rig.service.secret_status().expect_err("asked directly, the store's state shows");
    assert_eq!(error.params.get("where").map(String::as_str), Some("secret_store"));
}

#[tokio::test(start_paused = true)]
async fn the_status_after_a_write_names_the_secret_that_was_just_written_even_if_it_cannot_be_read()
{
    let store = Arc::new(OneUnreadable::new(SecretKind::Litellm));
    let rig = rig().secret_store(store).build();

    let set = rig.service.set_secret(SecretKind::Litellm, KEY.to_owned()).expect("it was stored");
    assert_eq!(
        set,
        SecretStatus { litellm: true, ..SecretStatus::default() },
        "the write is the truth"
    );
    let other = rig.service.set_secret(SecretKind::Opencode, KEY.to_owned()).expect("stored");
    assert_eq!(other, SecretStatus { opencode: true, ..SecretStatus::default() });
    let cleared = rig.service.clear_secret(SecretKind::Opencode).expect("removed");
    assert_eq!(cleared, SecretStatus::default());
}
