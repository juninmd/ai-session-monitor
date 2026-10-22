# ADR 0001: Architecture of the tray session monitor

Status: accepted (2026-10-10, after the user approved the interactive prototype)

## Context

A Windows tray app must show, per AI coding tool ("harness"), how much of the session is still
available: Claude Code (5 h and weekly), Codex, Antigravity, OpenCode, Kimi Code, z.ai, OpenRouter, DeepSeek, DeepInfra, Moonshot, Novita, Ollama Cloud, MiniMax and a parametrizable LiteLLM
connection. One tab per harness. Every source has a different trust level: some are documented,
some are internal files, one is a local language server. The app reads credentials and personal
usage data, so what it reads and what it sends must be explicit.

## Decisions

### D1. Tauri 2 + Vite + React + TypeScript, Windows first

Tauri keeps the resident footprint small (system WebView2, Rust core) which matters for something
that lives in the tray all day. Electron was the close alternative; it would win only if a
required native Chromium integration blocked Tauri, which is not the case.

### D2. Contract-first: Rust model, Zod mirror, golden files

`crates/usage-model` is the single vocabulary (views, windows, settings, IPC payloads, pace and
status rules). `src/lib/contract` mirrors it with strict Zod schemas and `src/lib/usage.ts` mirrors
the status rules. The two sides are pinned by files in `contract/`:
`status-vectors.json` (hand-written expectations that the Rust and TypeScript implementations must both reproduce),
`error-codes.json`, `ipc-commands.json`, and `snapshot.json`/`settings.json`/`ipc.json`, which Rust
generates (`UPDATE_GOLDEN=1`) and TypeScript parses. A shape change on one side fails the other
side's tests. Unknown is `null`, never `0`.

### D3. A window-free `app-core` crate; `src-tauri` is a thin shell

Settings, secrets, scheduling, the snapshot hub, notification planning and every IPC command's
logic live in `crates/app-core` behind three seams: `Collector` and `ClaudeShim` (injected, so tests
use fakes and app-core does not depend on the collector crates) and `SecretStore` / `EventSink`
(so tests and the browser dev mock need no Windows APIs). `src-tauri` is the composition root: it
wires the real collectors, the Credential Manager store and the Tauri event sink. Anything
that cannot be tested without a window (tray, flyout, positioning) lives in `src-tauri` and is kept
to glue plus pure helpers.

Coupling impact: the webview depends only on `Backend` (`src/lib/backend.ts`); the shell depends on
`AppService`; collectors depend on `usage-model` and `usage-support` and know nothing of Tauri.

### D4. Honest sources

Each view carries `source = { id, official }`. The UI shows an "unofficial" badge for anything
that is not a documented vendor interface (Antigravity local server, OpenCode Go usage endpoint, Kimi Code and z.ai usage endpoints,
Claude OAuth usage). Official-first where one exists: Claude's statusLine `rate_limits`
(through an installed shim) and `codex app-server`. Unofficial or optional sources are opt-in with
explicit consent and are never required for the app to work. See `docs/data-sources.md`.

### D5. Secrets only in the Windows Credential Manager; HTTP only from Rust

API keys (LiteLLM, OpenCode Go, Kimi Code, z.ai, OpenRouter, DeepSeek, DeepInfra, Moonshot, Novita, Ollama Cloud, MiniMax) are written through `set_secret` and stored with `keyring-core`
plus `windows-native-keyring-store` (keyring 4.x was split into a core and per-platform stores; the
store is opened with an explicit target and local persistence, and if the Credential Manager cannot
be opened every secret operation fails closed). They never appear in `Settings`, snapshots, logs,
error params or events; the webview can
write, clear and ask whether one exists, never read it back. The webview CSP allows no network
(`connect-src ipc: http://ipc.localhost`); every HTTP request is made by the Rust `SafeClient`
(https only except loopback, no redirects, size and time caps, optional private CA, an explicit
and host-scoped insecure mode). The app never reads the real `~/.claude`, Codex or OpenCode
credential stores; tests use temp dirs and fabricated fixtures.

### D6. Consent gates before reading personal state

A harness whose source touches another tool's files or processes stays in `needs_consent` until
the user agrees (installing the Claude statusLine shim, spawning `codex app-server`, reading the
Antigravity local server, opening OpenCode's database read-only). Installing the shim edits Claude
Code's `settings.json`; it is reversible, wraps an existing statusLine instead of replacing it,
and never touches a file it cannot parse.

### D7. Flyout window

A hidden, undecorated, transparent, always-on-top, skip-taskbar window with Mica, positioned from
the tray icon rectangle against the monitor work area in physical pixels (the positioner plugin
ignores the work area), hidden on blur with a short re-open guard. Single-instance plugin first.
The `unstable` Tauri feature stays off (it breaks blur, tauri#13633). Window behaviour can only
be validated on a real Windows 11 session; `docs/manual-windows-checklist.md` lists what to check.

### D7b. Autostart through a per-user Run value

Autostart writes the quoted executable path to the current user's `HKCU\...\Run` key through the
`auto-launch` crate instead of `tauri-plugin-autostart`, whose machine-wide key is used when the
app runs elevated. State is compared on startup and on settings changes so the value is not
rewritten every launch.

### D8. Supply chain

pnpm `minimumReleaseAge: 1440` (1 day) and denied install scripts; direct Rust crates are
pinned to versions that are at least 1 day old (`scripts/soak.mjs`, `cargo update --precise`);
the Rust toolchain (`rust-toolchain.toml`) and the package manager are pinned; `Cargo.lock` and
`pnpm-lock.yaml` are committed. Transitive crates are not soak-checked individually; CI runs a
RustSec audit instead.

On 2026-10-14, the owner reduced the default release-age requirement from seven days to
one day and explicitly approved an urgent exception for `rustls 0.23.46`, published at
`2026-10-14T09:54:15.075873Z`. This exact release fixes RUSTSEC-2026-0346 through
RUSTSEC-2026-0349 (see the [official release](https://github.com/rustls/rustls/releases/tag/v%2F0.23.46)).
The registry checksum matches `Cargo.lock`; compatibility and security reviews cover the
patch. The exception applies only to this version; other updates still require 24 hours.

### D9. Deviation from the default stack: plain CSS tokens, no Tailwind/shadcn

The default for web UI is Tailwind with shadcn/ui. This UI is one small, already approved,
hand-designed flyout with a custom token system (light/dark, `color-mix` tracks, tabular figures);
Tailwind and Radix would add dependencies and a build step for no reuse. Reversal condition: a
second surface (settings window, onboarding) or more than ~15 components.

### D10. Language and units

The UI is localized through message catalogs (one per locale, English as the master and the
fallback; see AGENTS.md, "Languages"): the ten most spoken languages, English by default. It began
pt-BR only. Code,
identifiers, comments and commits are English. Time to reset is the hero number (hours and
minutes; weeks as hours or days+hours by setting); pace/runway is derived (`available / burn
rate`) and shown only when the window is old enough to extrapolate.

## Consequences

- More up-front structure (seven collector crates, a contract layer) than a single binary would
  need; the payoff is that each source can fail, change or be replaced independently, and that
  Rust/TypeScript drift is a test failure rather than a runtime surprise.
- Four of seven sources are unofficial and can break without notice; the UI states that and shows
  "unavailable" instead of a made-up number.
- The tray flyout cannot be fully verified in CI; Windows behaviour has a manual checklist.

## Alternatives not taken

- Electron: larger resident footprint, no benefit here.
- One crate with everything: window-free logic would only be testable by booting Tauri.
- Parsing Claude Code JSONL for limits: it only records consumption, not the plan windows.
- Scraping vendor dashboards with browser cookies: ToS and credential risk (rejected for OpenCode and others).
