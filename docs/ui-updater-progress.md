# UI and GitHub updater acceptance

## Implemented locally

- Searchable overview of all 14 tools, current statuses, keyboard focus restoration,
  Escape handling and responsive light/dark layouts. Tab navigation follows rendered tools.
- Backend-owned updater state, four IPC commands, progress event, strict Rust/TypeScript
  contract, permissions, golden fixtures and deterministic browser mock.
- Automatic GitHub checks at startup and every six hours, persisted opt-out, manual checks,
  release notes, available-version banner, bounded download with progress, safe retry,
  deferred installation and a separate installation confirmation in pt-BR.
- SafeClient release transport with verified TLS, no credentials/proxies, restricted GitHub
  redirects, complete-body checks, 1 MiB manifest and 128 MiB installer caps and deadlines.
  Collector transports continue refusing all redirects.
- Stable-version and downgrade rules; both Minisign signatures and authenticated version
  must validate before an installer can run. Raw errors, URLs and signatures stay in Rust.
- Windows x64 production adapter stages the verified NSIS executable, drains the settings
  save queue and scheduler, flushes history, opens the installer with `/P /UPDATE /R`, then
  exits. Launch failure resumes polling; staging failure leaves it running. Unsaved form
  drafts block installation until saved or explicitly discarded; inputs freeze during
  installation. Edited groups/navigation/quit require explicit discard, pending form writes
  cannot be discarded, and external settings preserve local edits. Debug builds cannot install.
- Isolated debug profiles use a separate single-instance identity, preventing smoke commands
  from reaching the owner's already running app. They disable updater networking entirely.
- GitHub publication verifies actual assets downloaded from the uploaded draft, their URLs,
  sizes, SHA-256 digests, signature files, both signatures and signed version. Failures leave
  the release unpublished. New manifests use public download URLs.
- Draft preparation checks out the exact candidate commit, refuses existing published or
  unknown release states, and stops on create/resume or identity lookup failures before
  handing a release id to the build job.

## Evidence on 2026-10-14

- `pnpm verify`: Biome, TypeScript, Vitest, three Node release-verifier tests and production
  Vite build passed. The latest run passed 1,552 Vitest tests across 62 files plus the three
  Node tests; see `test-results/draft-review/final-verify.log`.
- `cargo test --workspace --locked`: 2,069 tests passed, two explicitly ignored live tests.
- Workspace Clippy and production Windows Clippy/check passed with warnings denied.
- Soak verification: all 27 direct Rust dependencies were at least seven days old.
- Production Edge smoke: six data scenarios in both themes, tool search/selection and 320 px
  layout; updater check/download/confirm/defer in both themes. No page errors or overflow.
- Live public-channel acceptance through SafeClient: downloaded the actual GitHub installer,
  verified its signature and authenticated version, rejected modified bytes. No installation.
  Reproduce with `cargo test -p usage-support --test live_release -- --ignored`.
- Real Windows debug tray/flyout smoke with a disposable profile: real updater/settings IPC,
  settings screen and unavailable/install gates passed. The smoke exited through `quit_app`;
  the owner's existing app remained running. WebView2 logged one class-unregistration message
  during shutdown; no UI page errors were observed.
- Draft protection passed in production Edge in both themes and at 320 px, and in the real
  Windows WebView2 with a disposable profile. Tab stays in the discard dialog; Escape keeps
  edits and restores focus. Browser checks also cover installation blocking and explicit
  discard; regression tests cover pending saves and externally updated LiteLLM settings.
  Both smoke runs exited cleanly; the owner's existing app remained running. This was a
  bounded self-review, not an independent review or installed-upgrade acceptance.

Ignored evidence artifacts: `test-results/frontend-verify.log`, `cargo-tests.log`,
`ui-smoke.json`, `update-ui-smoke.json`, `native-smoke.json` and screenshots including
`update-dark-confirm.png` and `native-update-settings.png`.
Draft evidence is under `test-results/draft-review/`, including `browser.json`, `native.json`,
`native-dialog.png`, the frozen candidate hashes and the review record.

## Latest GitHub preflight

- Read-only checks confirmed `v0.1.0` is the current published stable release and both
  signing-secret names are configured. No private key value was read.
- The latest completed CI run, [38487503838](https://github.com/juninmd/ai-session-monitor/actions/runs/38487503838),
  failed in `powershell_runs_the_chain_when_git_bash_is_not_available`; the prior CI run
  failed in the same test. The fallback returned empty output on both hosted runners.
  The isolated test and all 16 chain tests passed locally during this preflight. Runner
  startup delay is a hypothesis, not a proven cause; this CI failure remains unresolved.
- Eight fabricated cases executed the actual draft preparation PowerShell block with `gh`
  mocked: published/unknown states, create/resume failures, failed/malformed identity, and
  successful creation/resume. All passed after the fix. The original block reproduced unsafe
  continuation. No GitHub mutation occurred. Evidence: `test-results/release-prepare-review/`.

## Pending external acceptance

No commit, push, workflow dispatch or release publication has been performed. The modified
workflow has not run on GitHub. Repository instructions require owner confirmation before
commit/push. After authorization, inspect CI and a newly signed release, then demonstrate
an upgrade between signed versions on an isolated Windows target. Native blur, tray clicking,
DPI, accessibility with Narrator and multiple monitors still need the corresponding checks
in `manual-windows-checklist.md`. Local compilation and simulated installation tests do not
prove an installed production upgrade or those manual behaviors.
