# IPC and view semantics

Contract between the webview (`src/`), the shell (`src-tauri/`), the window-free core
(`crates/app-core`) and the collectors (`crates/collector-*`). Types live in
`crates/usage-model` (Rust) and `src/lib/contract` (Zod); `contract/*.json` pins both.

## Commands (webview -> shell)

Invoked through `Backend` (`src/lib/backend.ts`). Argument names are camelCase on the wire. A
failed command rejects with a serialized `HarnessError` (`{ code, retryable, params }`); the
webview never shows raw error text.

| Command | Args | Returns | Notes |
|---|---|---|---|
| `get_snapshot` | - | `Snapshot` | Current views in tab order; never blocks on a collector. |
| `refresh` | `harness?: HarnessId` | - | Non-blocking. Asks the scheduler to collect now; the result arrives as a `snapshot` event. Rate-limited per harness (>= 10 s); a refused call is not an error. |
| `get_settings` | - | `Settings` | |
| `save_settings` | `settings: Settings` | `Settings` | Normalizes (clamps ranges), persists atomically, applies (interval, enabled harnesses, autostart, tray source) and emits `settings-changed`. Returns the normalized value. |
| `secret_status` | - | `{ litellm, opencode }` | Booleans: a secret exists in the Credential Manager. Rejects with `internal` (`where = secret_store`) when the Credential Manager cannot be read; the UI keeps Settings usable and shows a notice. |
| `set_secret` | `kind: 'litellm' | 'opencode'`, `value: string` | `SecretStatus` | Trims the value; an empty one rejects with `invalid_input` (`field = value`). The value is never logged or echoed. Triggers a refresh of that harness. |
| `clear_secret` | `kind` | `SecretStatus` | Idempotent. |
| `claude_shim_status` | - | `ShimStatus` | Read-only look at Claude Code's settings.json. |
| `install_claude_shim` | - | `ShimStatus` | Requires `claude.shimConsent`. Copies the shim to a stable path under the data dir and wraps any existing statusLine. |
| `uninstall_claude_shim` | - | `ShimStatus` | Restores the previous statusLine exactly. |
| `open_link` | `link: ExternalLink` | - | Opens a fixed destination in the default browser. The webview cannot pass a URL. |
| `hide_flyout` | - | - | |
| `set_flyout_height` | `height: number` | - | Content height in CSS px; the shell clamps to the monitor work area and resizes keeping the anchor edge. |
| `quit_app` | - | - | |

## Events (shell -> webview)

| Event | Payload | When |
|---|---|---|
| `snapshot` | `Snapshot` | Whenever any view changed. |
| `settings-changed` | `Settings` | After a successful `save_settings` (also from another surface). |
| `flyout-shown` | none | The flyout became visible. The webview refocuses the selected tab and calls `refresh()`; the scheduler skips collectors that are fresh. |

## Window protocol

- The shell hides the flyout on blur (with a ~250 ms re-open guard so the tray click that
  re-opens it does not immediately close it) and on `Esc` pressed in the webview (`hide_flyout`).
- The webview owns its height: after each render it measures its content and calls
  `set_flyout_height`. Width is fixed at 432 px (27 rem). When the content is taller than the work
  area the panel body scrolls; the header and tab strip stay.
- First launch only (`general.firstRunDone = false`): the shell opens the flyout once and then
  sets the flag through `save_settings`.

## View semantics

- Tab order: `claude`, `codex`, `antigravity`, `opencode`, `litellm` (`HarnessId::ALL`).
- `state`: `disabled` (user switched it off) | `needs_consent` (nothing is read until the user
  agrees) | `needs_setup` (configuration missing: LiteLLM URL/key, OpenCode key and no local
  consent) | `loading` (first read in flight) | `ready` | `no_data` (the source works but has
  nothing yet) | `error`.
- A `ready` view with `stale = true` keeps its windows and is shown desaturated (gauges faded, text keeps its contrast) and excluded from
  the tray summary. `error` is set in the `error` state and also on a stale `ready` view whose last
  refresh failed (the windows are the previous good ones).
- `usedPct = null` is unknown and is never rendered as 0 %. Over-budget values above 100 are
  allowed and clamp to 0 % available. A window whose `resetsAt` passed is expired: shown as
  "reiniciou", no status, no pace.
- `readAt` is when the data was produced (file mtime, response time), not when we polled; the UI
  shows "lido há N min".
- Status and pace come only from the shared rules (`usage-model::status`, `src/lib/usage.ts`).
- `meta` is display-only and never holds secrets (keys below).
- `facts` are small labelled numbers (`id` is a message key; `unit` is `usd`, `tokens`,
  `requests`, `credits`, `epoch_s` (unix seconds, rendered as a date) or a free label).

### Wire vocabulary

What the collectors really send, so the UI can word it. `src/test/vocabulary.ts` mirrors these
lists and `src/i18n/emitted.test.ts` reads the collectors' Rust sources, so a new value that the
UI cannot word turns `pnpm test` red.

Window ids and kinds, per source:

| Source | Window `id` | `kind` |
|---|---|---|
| `claude` | `five_hour`, `weekly`; `spend_limit` (a money cap with `amount`, when the status line reports one) | `five_hour`, `weekly`; `budget` |
| `claude` (OAuth, opt-in) | `<family>:weekly` (`group` = the model family, e.g. `opus`) | `weekly` |
| `codex` | by cadence: `five_hour`, `daily`, `weekly`, `monthly`; a cadence it cannot name stays `primary` / `secondary` with `windowMinutes` set | same, else `other` |
| `antigravity` | `<pool>:five_hour`, `<pool>:weekly`, `<pool>:<slug>` (`label` = the bucket); pool is `gemini`, `claude_gpt`, a slug from the service, or `group_<n>` (0-based index) | `five_hour`, `weekly`, `other` |
| `opencode` | `rolling`, `weekly`, `monthly` | `five_hour`, `weekly`, `monthly` |
| `litellm` | `budget` (`label` = key alias); `limit:<duration>` (`label` = the raw duration); `model:<name>` (`group` and `label` = the model) | `budget` |

Fact ids and units:

| Source | Fact `id` (`unit`) |
|---|---|
| `codex` | `credits_balance` (`credits`) |
| `opencode` | `local_cost_5h`, `local_cost_7d`, `local_cost_30d` (`usd`); `local_tokens_5h`, `local_tokens_7d` (`tokens`). Local list-price estimate, not quota. |
| `litellm` | `rpm_limit`, `max_parallel_requests` (`requests`); `tpm_limit` (`tokens`); `expires` (`epoch_s`) |

`meta` keys (display only): `alias`, `host`, `scope` (`key`, `user`, `team`) and `level` (`key`,
`user`, `team`: where the budget was found) for LiteLLM; `local_estimate = true` for OpenCode when
facts are a local estimate; `extra_source = oauth_usage` for Claude when some windows come from
the opt-in OAuth source.

Error params are non-secret and drive the wording: `reason` (per code: `install_failed`,
`tls_error`, `endpoint_changed`, `not_running = no_port`, `unauthorized = no_token`), `field`
(see below), `retryAfterSecs`, `httpStatus` and `redirect`. `host`, `phase`, `rpcCode`, `code` and
`where` are diagnostics for bug reports and are never shown to the person. On a notice,
`reason` is an error code except for `opencode_db_unreadable`, which has its own list.

### Notice codes (initial catalog)

`Notice.code` is free-form; the UI maps known codes to pt-BR text and renders any unknown code
with a generic message that includes the code, so a missing translation is visible, not silent.

| Code | Severity | Meaning |
|---|---|---|
| `claude_waiting_first_session` | info | Shim installed, no Claude Code session has reported yet. |
| `claude_no_rate_limits` | warn | Claude Code reports no `rate_limits` (API-key billing or a plan without limits). |
| `claude_shim_outdated` | warn | The configured shim executable is missing; reinstall. |
| `claude_oauth_partial` | info | OAuth usage is on but failing; showing statusLine data only (`reason` param). |
| `codex_rollout_fallback` | info | `codex app-server` unavailable; reading the newest rollout (may be old). `reason` param: the snake_case error code that stopped the app-server (`binary_not_found`, `timeout`, ...). |
| `codex_no_limits` | warn | The signed-in Codex account reports no limits (API-key auth). |
| `antigravity_pool_unknown` | warn | A quota pool returned no data (`pool` param); unknown, not zero. |
| `litellm_no_budget` | info | The key/user/team has no budget cap. |
| `litellm_level_unreadable` | warn | The user or team follow-up failed, so the budget may live at a level that could not be read (`level` = `user` or `team`, `reason` = error code). `litellm_no_budget` is withheld then. |
| `litellm_windows_truncated` | info | More than 16 per-model or concurrent budget windows; the least consumed were dropped (`dropped` param = how many). |
| `opencode_window_idle` | info | 0 % with a full-length reset: the window starts at the next request (`window` param). |
| `opencode_rate_limited` | warn | A window reports `rate-limited` (`window` param). |
| `opencode_no_key` | info | Only the local estimate is shown; paste the Go API key for the real windows. |
| `opencode_local_estimate` | info | Facts are a local list-price estimate for this PC, not quota. |
| `opencode_db_unreadable` | warn | The local database could not be read or its schema changed (`reason` param: `invalid_path`, `not_found`, `permission`, `locked`, `not_a_database`, `schema`, `timeout`, `other`). |

New codes are added by the collector that emits them and reported at integration so the UI
catalog is updated in the same change.

## Error codes

`ErrorCode` (23 values, `contract/error-codes.json`). The UI maps each to a message and one
action (retry, open settings, open the console, reinstall). `retryable = true` means the next tick
may succeed by itself. Params are non-secret (`httpStatus`, `host`, ...).

### Field names in rejected commands

`save_settings`, `set_secret` and `install_claude_shim` reject with `invalid_url` or `invalid_input`
and `params.field` naming the offending input (never its value). The UI shows field-level wording for
the names marked UI; any other name gets the generic message. `install_claude_shim` also rejects a
relative or NUL-bearing data or home directory as `invalid_input` with `field = configDir`; the
contract has no better name for it.

| Command | Code | `field` | UI |
|---|---|---|---|
| `save_settings` | `invalid_url` | `baseUrl` (not https, userinfo, query or fragment; plain http only to loopback) | yes |
| `save_settings` | `invalid_input` | `entityId` (user/team scope without an id) | yes |
| `save_settings` | `invalid_input` | `authHeader` (letters, digits and hyphen only, up to 128; not a header the HTTP client sets itself) | yes |
| `save_settings` | `invalid_input` | `pemPath` (custom CA path not absolute, NUL, too long, or a UNC share, device-namespace or reserved device path) | yes |
| `save_settings` | `invalid_input` | `host` (insecure TLS host empty or different from the base URL host) | yes |
| `save_settings` | `invalid_input` | `configDir`, `binaryPath`, `dataDir` (custom path not absolute, NUL, too long, or a UNC share, device-namespace or reserved device path; the extended-length form `\\?\C:\` stays valid) | yes |
| `set_secret` | `invalid_input` | `value` (empty after trimming, or longer than 4096 characters) | yes |
| `install_claude_shim` | `invalid_input` | `shimConsent` (consent not given) | yes |

OpenCode: 401/403 from the usage endpoint are reported as `unauthorized` with `ambiguous = true`
because valid keys have been rejected by the service; the UI says "could not verify the key or the
Go subscription" and offers the console link, never a certain "bad key".

## Polling

- `general.intervalSecs` (30..3600, default 60) drives local sources (Claude shim file, Codex,
  Antigravity, OpenCode local database) and also Claude when OAuth is on: the collector itself
  asks the OAuth endpoint no more than once per 300 s. LiteLLM uses `litellm.pollIntervalSecs`
  (>= 60, default 300). Scheduled OpenCode polls, which reach its Go endpoint, never run faster
  than 300 s; a manual refresh can (see the 10 s gap above).
- Failures back off exponentially per harness up to 15 minutes; a successful read resets it. A
  `retryAfterSecs` param is surfaced to the person but not honored by the scheduler. A Codex
  rollout fallback counts as a success (no error on the view), so it does not back off.
- `timing::cadence` gives each harness its pace: `general.intervalSecs` for Claude, Codex and
  Antigravity, `max(intervalSecs, 300 s)` for OpenCode, `litellm.pollIntervalSecs` for LiteLLM.
- A manual `refresh` is dropped when that harness started a collection less than 10 s ago
  (`REFRESH_MIN_GAP`).
- Snapshot events: the webview registers the `snapshot` listener first, then calls
  `get_snapshot`; whichever finishes last, the UI keeps the larger `generatedAt`.
- At each start the shell refreshes the installed Claude shim copy from the bundled build. It is a
  silent step: no status, no notice and no IPC command.
- Disabled and consent-gated harnesses are never polled.
- The first snapshot after start has every enabled harness in `loading`.

## Notifications

`general.notifyOnOut` and `general.notifyOnReset` (both off by default) switch the desktop toasts.
`app-core` plans them from snapshot changes and the shell shows them; title and body are
ready-made pt-BR text and carry no path, key or raw error.

- Window ran out: a fresh, non-expired window of a `ready` view goes from some availability to
  none. The first snapshot after start never notifies.
- Window available again: the window that was out is seen with availability, or, when its source
  stays silent (the Claude status line only moves while Claude Code runs), the clock announces it
  once at `resetsAt` ("limite disponível", without a percentage). A reading that later confirms
  that reset does not notify a second time.
- Settings recovered: `settings.json` was unreadable, was kept as `settings.corrupt-<epoch>.json`
  and defaults are in effect. Shown once per start, before the first snapshot, whatever the two
  toggles say.

## Data directory

`%LOCALAPPDATA%\dev.aisessionmonitor.tray\` holds `settings.json` (non-secret), `claude\rate_limits.json`
(written by the shim, `capturedAt` epoch seconds plus the raw `rate_limits` object), and `bin\`
(the stable copy of the shim referenced from Claude Code's settings). Secrets are never written here.
