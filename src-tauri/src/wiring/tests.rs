use super::*;

#[cfg(not(feature = "collectors"))]
mod without_collectors {
    use super::*;

    fn ctx() -> (tempfile::TempDir, Ctx) {
        let dir = tempfile::tempdir().expect("tempdir");
        let ctx = Ctx::new(1_800_000_000, dir.path(), dir.path());
        (dir, ctx)
    }

    #[test]
    fn no_collectors_are_wired() {
        assert!(adapters(None).collectors.is_empty());
    }

    #[test]
    fn the_shim_reports_not_installed_and_refuses_to_change_anything() {
        let (_dir, ctx) = ctx();
        let shim = adapters(None).claude_shim;
        let settings = ClaudeSettings::default();
        assert_eq!(shim.status(&settings, &ctx).state, ShimState::NotInstalled);
        let err = shim.install(&settings, &ctx, Path::new("shim.exe")).expect_err("install");
        assert_eq!(err.code, ErrorCode::InstallFailed);
        let err = shim.uninstall(&settings, &ctx).expect_err("uninstall");
        assert_eq!(err.code, ErrorCode::InstallFailed);
    }
}

#[cfg(feature = "collectors")]
mod with_collectors {
    use usage_model::HarnessId;

    use super::*;

    #[test]
    fn one_collector_per_harness_in_tab_order() {
        let ids: Vec<HarnessId> = adapters(None).collectors.iter().map(|c| c.id()).collect();
        assert_eq!(ids, HarnessId::ALL);
    }
}
