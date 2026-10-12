# AGENTS.md

Windows-first system-tray app (Tauri 2 + Vite + React + TypeScript, Rust workspace) that shows how
much of each AI coding tool's session is still available: Claude Code, Codex, Antigravity,
OpenCode, Kimi Code, z.ai, OpenRouter, DeepSeek, DeepInfra, Moonshot, Novita, Ollama Cloud, MiniMax and a parametrizable LiteLLM connection. UI language is pt-BR; code, identifiers,
comments and commits are English. Decisions: `docs/adr/0001-architecture.md`. Wire contract:
`docs/ipc.md`. Data sources and trust levels: `docs/data-sources.md`.

## Layout and ownership

| Path | Role | Depends on |
|---|---|---|
| `crates/usage-model` | Vocabulary: views, windows, settings, IPC payloads, pace/status rules | serde only |
| `crates/usage-support` | Hardened HTTP client, atomic file writes, bounded process runner, redaction, RFC 3339 | usage-model |
| `crates/collector-{claude,codex,antigravity,litellm,opencode,kimi,zai,openrouter,deepseek,deepinfra,moonshot,novita,ollama,minimax}` | One source each: `collect(...) -> HarnessView` | usage-model, usage-support |
| `crates/statusline-shim` | Tiny binary Claude Code runs as its `statusLine`; std + serde_json only | none |
| `crates/app-core` | Window-free core: settings store, hub, scheduler, notification plan, `AppService` | usage-model, usage-support (no collectors) |
| `src-tauri` | Composition root: tray, flyout window, keyring, wiring, commands | app-core, collectors |
| `src` | React UI; talks only to `Backend` (`src/lib/backend.ts`) | `src/lib/contract` |
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
pnpm dev                        # UI in a browser on 127.0.0.1:1420 with a mock backend (?mock=demo|fresh|healthy|critical|stale|errors|loading)
pnpm tauri dev                  # the real app (tray + flyout)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
UPDATE_GOLDEN=1 cargo test -p usage-model --test golden   # regenerate contract/{snapshot,settings,ipc}.json after an intentional shape change
node scripts/soak.mjs npm react,vite [days]               # comma-separated; newest version at least [days] old (default 7)
node scripts/soak.mjs crates tauri,keyring [days]          # same, for crates
```

`src-tauri` compiles against `../dist`, so build the frontend (`pnpm build`) before `cargo clippy
--workspace` or `cargo test --workspace` on a fresh checkout.

## Changing the contract

1. Change `crates/usage-model`, regenerate the goldens (command above), commit the JSON diff.
2. Mirror it in `src/lib/contract/*.ts` (strict Zod: an unknown field fails parsing) and, for status
   rules, in `src/lib/usage.ts`. `contract/status-vectors.json` is hand-written and shared by both.
3. A new `ErrorCode` also needs `contract/error-codes.json`, a pt-BR message in `src/i18n`, and an
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
- Unofficial sources (Antigravity, OpenCode, Kimi Code, z.ai, Claude OAuth) stay opt-in and badged. Do not add
  another unofficial source without an entry in `docs/data-sources.md`.
- Dependencies: direct crates and npm packages must be at least 7 days old (soak script);
  pnpm denies install scripts (`onlyBuiltDependencies: []`). Do not hand-edit lockfiles.
- No `unsafe` (workspace-wide `forbid`). `unwrap` is denied outside tests; `expect` and `print!`
  warn. Files stay under 300 lines.

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
