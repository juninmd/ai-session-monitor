# Competitors and related projects

A snapshot of tools that show AI coding-plan limits, what they cover, and where AI Session Monitor
(ASM) differs. Checked on 2026-11-08 from each project's README or search summary; features and
provider counts change quickly, so re-check before quoting them.

## Landscape

| Project | Platform | Scope | Notes |
|---|---|---|---|
| [CodexBar](https://github.com/steipete/CodexBar) | macOS 14+ | Very broad (its README claims 113 providers) | Reuses browser cookies and OAuth where the provider needs them. Burndown history, status-page badges, cost scans, a bundled CLI, WidgetKit. |
| [ClaudeBar](https://github.com/tddworks/ClaudeBar) | macOS | ~20 providers (Claude, Codex, Gemini, Antigravity, Copilot, Cursor, Kiro, Windsurf, JetBrains, Mistral, OpenCode Go, z.ai...) | Notifications before a quota runs out. |
| [ai-usagebar](https://github.com/akitaonrails/ai-usagebar) | Linux, macOS, Windows (tray) | Broad, incl. Mistral, Droid, Cline, Devin, Grok | Rust. Waybar, TUI and tray. Reads CLI logins. Opt-in providers. |
| [Usage Monitor for Claude](https://github.com/jens-duttke/usage-monitor-for-claude) | Windows, Linux | Claude only | Windows tray. Pace-aware alerts, working-hours pacing, forecast chart, reset notifications, event commands (run a shell command on reset, on threshold, or from the tray). Shows the Claude Code version in each IDE. |
| [ClaudeTracker](https://github.com/TobiiNT/ClaudeTracker) | Windows | Claude | WPF tray. Alerts at 75/90/95 %. Reads the OAuth token from Credential Manager. |
| [TokenWatcher](https://github.com/gehariharan/tokenwatcher) | Windows | Claude, Codex | Electron. Explicitly avoids browser-cookie scraping and admin rights. |
| [ccusage](https://github.com/ryoppippi/ccusage) | CLI | Claude Code and other agent CLIs | Historical tokens and cost per day, project and session, from local transcripts. Not a quota view. |

## What the others do that ASM does not

1. **Event commands and automation.** Usage Monitor runs a user command on reset or threshold.
   ASM has no hook for that.
2. **Pace alarms around working hours.** Usage Monitor ignores non-working time when judging weekly pace.
   ASM's pace is continuous.
3. **Reset notifications.** Usage Monitor and CodexBar say when a nearly exhausted quota refills.
   ASM alerts on risk only.
4. **Local cost history for Claude and Codex.** ccusage and CodexBar scan transcripts for per-project
   spend. ASM keeps a local cost estimate only for OpenCode.
5. **More providers.** Gemini CLI quota, Windsurf, Kiro, JetBrains AI, Mistral (official budget API),
   Factory/Droid, Amp, Augment and Grok are covered by at least one competitor and not by ASM.
6. **Burndown history.** CodexBar shows a quota burndown from recorded snapshots. ASM records
   history (`history.json`) and shows a burndown too, so this is not a gap.

## What ASM does that most of them do not

- **Consent before any local read.** Sources that touch another tool's files or processes wait for
  opt-in and do no I/O before it (`docs/data-sources.md`). Several competitors read browser cookies or
  credential stores by default.
- **Secrets in Windows Credential Manager**, never in settings; the webview never sees a secret or a
  raw error and makes no network request.
- **Unknown is `null`, not `0`**, and expired windows show as reset, not as a status.
- **Fenced Claude data path.** The statusLine shim records only `rate_limits` and chains the user's
  existing status line; the CLI fallback never reads the access token.
- **Official vs unofficial badge** on each source.
- **Windows-first tray** with a flyout, ten languages, a local read-only API (off by default), and a
  capture-protection setting.

## Candidates for the next provider

Ranked by how official the interface is and how much it adds:

1. **Mistral** (official budget API, requires a key with `see_org_inference_usage`).
2. **Gemini CLI** (Code Assist quota, via the CLI's own OAuth login; needs consent).
3. **Windsurf** and **Kiro** (community-documented, local stores; needs consent and a `docs/data-sources.md` entry).
4. **JetBrains AI** (local IDE config only; read-only, opt-in).

Any new unofficial source needs a `docs/data-sources.md` entry before it ships (see `AGENTS.md`).
