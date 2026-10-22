# AGENTS.md

Windows-first system-tray app (Tauri 2 + Vite + React + TypeScript, Rust workspace) that shows how
much of each AI coding tool's session is still available: Claude Code, Codex, Cursor, GitHub Copilot, Antigravity,
OpenCode, Kimi Code, z.ai, OpenRouter, DeepSeek, DeepInfra, Moonshot, Novita, Ollama Cloud, MiniMax and a parametrizable LiteLLM connection. The UI speaks the ten most spoken languages (English by default, `general.language`); code, identifiers,
comments and commits are English. Decisions: `docs/adr/0001-architecture.md`. Wire contract:
`docs/ipc.md`. Data sources and trust levels: `docs/data-sources.md`.

## Layout and ownership

| Path | Role | Depends on |
|---|---|---|
| `crates/usage-model` | Vocabulary: views, windows, settings, IPC payloads, pace/status rules | serde only |
| `crates/usage-support` | Hardened HTTP client, atomic file writes, bounded process runner, redaction, RFC 3339 | usage-model |
| `crates/collector-{claude,codex,cursor,copilot,antigravity,litellm,opencode,kimi,zai,openrouter,deepseek,deepinfra,moonshot,novita,ollama,minimax}` | One source each: `collect(...) -> HarnessView` | usage-model, usage-support |
| `crates/statusline-shim` | Tiny binary Claude Code runs as its `statusLine`; std + serde_json only | none |
| `crates/app-core` | Window-free core: settings store, hub, scheduler, notification plan, `AppService` | usage-model, usage-support (no collectors) |
| `src-tauri` | Composition root: tray, flyout window, keyring, wiring, commands | app-core, collectors |
| `src` | React UI; talks only to `Backend` (`src/lib/backend.ts`) | `src/lib/contract` |
| `src/home` | The flyout's own surfaces — triagem, horizonte, canais de cota, the lateral detail and the Ctrl+K palette — which `App` composes | `src/app`, `src/components` |
| `contract` | Golden files pinning Rust and TypeScript to the same wire shape | - |

Rules that keep the boundaries honest:

- `app-core` never depends on a collector crate; collectors never depend on `app-core` or Tauri.
- The webview never gets a secret or a raw error string, and never makes a network request (CSP
  allows only `ipc:`). Every HTTP call goes through `usage_support::SafeClient`.
- A collector gates on settings **before any I/O**: disabled, needs consent, needs setup.
- Unknown is `null`, never `0`. A window whose `resetsAt` passed is expired: no status, no pace.

## Commands

Run from the repository root. Toolchains are pinned (`rust-toolchain.toml`, `packageManager`).

```sh
pnpm install --frozen-lockfile     # Node 22.22.2+, 24.15+ or 26+ (package.json engines)
pnpm verify                     # biome check, tsc --noEmit, vitest, vite build
pnpm dev                        # UI in a browser on 127.0.0.1:1420 with a mock backend (?mock=demo|fresh|healthy|critical|stale|errors|loading; Ctrl+K switches scenario there)
pnpm tauri dev                  # the real app (tray + flyout)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
UPDATE_GOLDEN=1 cargo test -p usage-model --test golden   # regenerate contract/{snapshot,settings,ipc}.json after an intentional shape change
node scripts/soak.mjs npm react,vite [days]               # comma-separated; newest version at least [days] old (default 1)
node scripts/soak.mjs crates tauri,keyring [days]          # same, for crates
```

`src-tauri` compiles against `../dist`, so build the frontend (`pnpm build`) before `cargo clippy
--workspace` or `cargo test --workspace` on a fresh checkout.

## Changing the contract

1. Change `crates/usage-model`, regenerate the goldens (command above), commit the JSON diff.
2. Mirror it in `src/lib/contract/*.ts` (strict Zod: an unknown field fails parsing) and, for status
   rules, in `src/lib/usage.ts`. `contract/status-vectors.json` is hand-written and shared by both.
3. A new `ErrorCode` also needs `contract/error-codes.json`, an English message in `src/i18n/locales/en` (and the other locales, see "Languages"), and an
   action mapping. A new notice code needs a message in the UI catalog (`docs/ipc.md` lists them).
4. A new IPC command needs `contract/ipc-commands.json`, an `allow-<command>` permission and an
   entry in `capabilities/default.json`; a test fails if these drift.
5. `src/lib/contract/wire.test.ts` stays red until the Zod enum lists mirror the Rust enums.
6. A changed or new status threshold needs a status vector on each side of it in
   `contract/status-vectors.json`.

## Security and testing rules

- Never read or write the real `~/.claude`, `~/.codex`, the OpenCode data dir, `%APPDATA%` or
  Windows Credential Manager from tests or manual runs. Use temp dirs and fabricated fixtures.
  Debug builds honour `AISM_DATA_DIR`, `AISM_HOME_DIR`, `AISM_MEMORY_SECRETS=1` and
  `AISM_SHOW_ON_START=1` so smoke runs never touch your profile (compiled out of release builds).
- Never log, serialize or place a secret in an error param, notice, fact or `meta`. Tests that
  touch secrets assert absence in the serialized output.
- Unofficial sources (Cursor, GitHub Copilot, Antigravity, OpenCode, Kimi Code, z.ai, Claude OAuth) stay opt-in and badged. Do not add
  another unofficial source without an entry in `docs/data-sources.md`.
- Dependencies: direct crates and npm packages must be at least 1 day old (soak script);
  pnpm denies install scripts (`onlyBuiltDependencies: []`). Do not hand-edit lockfiles.
- No `unsafe` (workspace-wide `forbid`). `unwrap` is denied outside tests; `expect` and `print!`
  warn. Files stay under 300 lines.

## Languages

- Ten locales: `en` (default and fallback), `zh-CN`, `hi`, `es`, `fr`, `ar`, `bn`, `pt-BR`, `ru`, `ur`;
  `ar` and `ur` are RTL (`dir` is set on the document, so CSS stays logical-property based).
  The choice is `settings.general.language` (Rust `Language`, Zod `LanguageSchema`) and is mirrored
  into `src/i18n/locale.ts`; `t()` falls back to English key by key.
- A message lives in `src/i18n/locales/<tag>/<partial>.ts`, same partials and keys in every locale
  (English is the master: its keys are `MessageKey`). One whole sentence per key with `{name}`
  placeholders, never concatenated fragments; counts go through `tn()` with `key.one|few|many|other`
  (`key.other` always). No `t()` at module scope: the language changes at runtime (App re-keys the
  home on a language change). Numerals stay Latin in every language (`-u-nu-latn`).
- `src/i18n/locales.test.ts` checks every locale against English (keys, placeholders, plural
  forms). The Vitest setup pins pt-BR (`pinLocale`), so the older tests keep asserting Portuguese;
  a test of another language calls `releaseLocale()`/`setLocale()`.
- Rust text (notifications, tray tooltip and menu, startup toasts) is in
  `crates/app-core/src/i18n/lang_<tag>.rs`, one `Messages` struct per language; a test checks that
  every language keeps the placeholders of every field. The tray text is relabelled when the
  language changes. A startup failure happens before settings load, so that toast is English.
- The optional local API (`general.localApi`, `127.0.0.1:6737`, read-only JSON, off by default) is the
  only place the app listens on a port: loopback only, `Host` check, no `Origin`, no CORS. Capture
  protection (`general.hideFromCapture`) is `set_content_protected` on every window.
- The theme (skin) is a separate, webview-only choice (`src/styles/skins.css`).

## Gotchas

- `pnpm exec biome migrate --write` rewrites `"recommended": true` to `"preset": "none"` and
  silently disables the linter; keep `"preset": "recommended"` and check a deliberate violation.
- Do not enable Tauri's `unstable` feature (window blur stops firing, tauri#13633). Do not add the
  window-state, shell, http or stronghold plugins.
- A runnable doctest fails under `cargo test --workspace` (rustdoc inherits the Tauri shell's stub
  `msvcrt.lib` link dir, `__CxxFrameHandler3` unresolved) while `-p <crate> --doc` passes: mark
  doctests `no_run` or `compile_fail`, or write a unit test instead.
- Vitest runs with a 20 s timeout because the boot and form tests re-import the whole app and a
  CPU busy with a cargo build stretches them past the 5 s default; keep new tests deterministic.
- Tray, blur, DPI and multi-monitor behaviour can only be checked on a real Windows session:
  `docs/manual-windows-checklist.md`.
- Commits and pushes need explicit confirmation from the repository owner; never auto-merge.
