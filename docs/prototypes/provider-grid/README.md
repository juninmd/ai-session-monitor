# Active provider grid prototype

Browser-only preview of a compact session monitor, based on `master` at
`1783bd9a8d3eddea5c482fa5f17d060946371313`. All readings, settings and credentials are
fabricated and held in memory. Reloading restores the demonstration.

Run `pnpm dev` and open:

<http://127.0.0.1:1420/?prototype=grid&mock=demo>

For the production build, run `pnpm build` followed by
`pnpm preview --host 127.0.0.1 --port 1421 --strictPort` and use the same query string.

## Behavior

- The main screen shows configured, enabled providers in three columns, or two on narrow
  screens. Disabled providers and providers awaiting setup or consent stay in settings.
  Configured sources with temporary errors, stale data or loading remain reachable.
- Selection follows an active source when the selected provider is disabled. Arrow keys,
  Home and End navigate the grid; Up and Down move between rows.
- Cards show up to three primary quota windows, keeping session, week and month when
  available. For grouped quotas, the most constrained live pool drives the summary:
  Antigravity normally shows Gemini, with Claude/GPT-OSS counted as extra quotas. A later
  limiting window replaces a summary row if necessary, so critical usage remains visible.
  Independent quotas are never merged. All windows stay available in the detail accordion.
  Unknown readings use a dash, expired readings say "Expirou", and stale readings say
  "Antigo", without a fresh percentage or filled bar. Unofficial sources use an inline
  information icon; hover or keyboard focus shows an explanatory tooltip, dismissed by Escape.
- Details start collapsed and open a compact table of quota, percentage and renewal time.
  Each percentage has a 3px bar; pace appears as a short line below the quota name, with
  estimates marked `~` and full projection text on hover and for screen readers. The shared
  pace rules are reused; unknown, expired and stale readings have no forecast or filled bar.
  **Histórico** expands into a 96px chart, a window selector and a brief peak summary.
  Its accessible readings table stays collapsed. History is fetched only on request;
  closing provider details stops its refreshes, and switching providers resets the disclosure.
  Plan, source, facts, notices and configuration live under **Informações adicionais**;
  warning counts remain visible on its summary. Changing providers collapses details again.
  Duplicate management links and the large preview headline
  were removed; demonstration controls live under **Controles da prévia**.
- Hover or focus on **Visão geral** shows every provider, its most constrained quota and
  status, including inactive sources. Unknown, expired and stale readings have no fresh
  percentage. The tooltip remains open while hovered and closes with Escape.
- The searchable **Todas as ferramentas** disclosure appears in settings. Choosing a
  provider opens its configuration, including inactive providers. Existing draft guards
  apply before replacing an edited form.
- At usage of 90% or higher, the main screen shows a critical notice for every affected
  active provider. **Simular 90% no Codex** demonstrates a crossing notification. The
  preview sends one notice per quota window and reset epoch, without repeating on refresh
  or stale readings. An already critical first reading shows the banner without a toast.
- Scenario links cover everyday use, critical usage, no connections, errors, stale data
  and loading. Existing settings, draft protection and failure remediation are reused.
- Codex details show a small earned-reset count and **Usar reset** only for current,
  enabled, consented app-server data with a positive whole count. Confirmation is explicit;
  cancellation preserves the balance. The prototype simulates one credit consumption and
  fabricated updated quotas, preserving unknown and expired readings. A second request
  without consumed quota reports that nothing needs resetting. This does not invoke Codex.
- **Simular oferta low-priority no Claude**, under preview controls, exposes a compact
  **Copiar /low-priority** action in Claude details. The command is copied for use in the
  interactive session; copying does not activate the mode. No real capability is detected.
- **Simular nova versão do monitor** demonstrates the existing monitor updater: a small
  blue banner above the grid says **Versão 0.2.0 disponível** and **Ver atualização** opens
  update settings. Mock download/install operations never fetch or launch anything.
  This banner concerns AI Session Monitor; provider CLI version checking is not implemented.

## Real action integration

The [official Codex app-server documentation](https://learn.chatgpt.com/docs/app-server#8-earned-rate-limit-resets-chatgpt)
documents `account/rateLimitResetCredit/consume`, with a nonempty `idempotencyKey`, an
optional `creditId`, and outcomes `reset`, `alreadyRedeemed`, `nothingToReset`, `noCredit`.
A real backend must reuse the same UUID for retries of one redemption and re-read
`account/rateLimits/read` afterward; it must not infer which quota windows were renewed.
The current collector only reads the count. Adding a real consume action requires a new
IPC contract, permission, mapped errors, backend command and safe process lifecycle.

Claude's [open SDK issue 456](https://github.com/anthropics/claude-agent-sdk-typescript/issues/456)
reports `/low-priority` as hidden and interactive-only in CLI 2.1.263, with offers exposed
through `lowPriorityOffer` but no SDK activation. This is a user report in Anthropic's
repository, not a documented public activation API. Availability for a real account and a
later installed version has not been verified. The prototype offers only command copying.

## Prototype boundary

The normal browser route and Tauri launch keep the existing UI. `prototype=grid` only
activates outside Tauri. No profiles, real credentials, collectors or system settings
are read or changed by this preview.

The tray at the bottom of the preview and its notification are browser simulations.
Windows' native tray tooltip has a 127 UTF-16-unit limit (`src-tauri/src/tooltip.rs`),
so displaying all fourteen providers requires a custom native hover surface in a later
implementation. Native tray hover, DPI, multi-monitor behavior and desktop notification
delivery have not been validated by this browser prototype.

The Rust core already supports configurable threshold notifications. The preview uses
`alertThresholds: [90]` in its fabricated settings; the real application's existing
default of `[80, 95]` is unchanged.

## Validation and captures

`pnpm verify` passed lint, TypeScript, 1,579 Vitest tests, three release-script tests and
the production build. Twenty-seven prototype tests cover visibility, exact threshold crossings,
notification deduplication, reset epochs, inactive-provider configuration, tooltip
interaction, selection fallback, all window kinds/model pools, null/expired/stale bars,
usage display modes, source tooltip interaction, bounded grid summaries, limiting pool
selection, complete quota tables and secondary information/configuration disclosures,
compact forecast estimates and full text, neutral stale/expired/unknown details, available
usage mode, lazy history, stopped history refreshes and disabled history, reset capability
validation/rechecking, credit consumption simulation, confirmation/cancellation/focus,
conditional command copying, and opening update settings without downloading or installing.

A Playwright smoke run against the production build in headless Microsoft Edge checked
six visible providers with thirteen primary quota bars in two desktop rows and three mobile rows,
all fourteen tooltip
entries, keyboard navigation, the settings search, the simulated critical notification,
empty/error/stale states, full-width quota bars, collapsed/expanded details, no horizontal
page overflow at 360px and 320px, light and dark themes,
and the regular route with fourteen tabs. Summary cards measured at most 130.5px high
on desktop. All four Antigravity quotas remained reachable in its compact details. Source
tooltips were checked for hover, keyboard focus, Escape and viewport bounds. Compact detail
bars, a 96px history chart, history window selection and its collapsed/expanded readings
table were checked in the production browser. Expanded history also stayed within 320px;
stale details had no forecast or filled bar. The browser console had no errors.
Reset confirmation/cancellation/one-credit consumption/no-op, conditional low-priority
copying, update banner/settings navigation and narrow update/reset layouts were also
checked. The browser copy test uses an isolated clipboard stand-in.

- [Desktop dark](01-grid-dark.png)
- [Complete tooltip](02-tooltip-all-providers.png)
- [90% notification](03-critical-notification.png)
- [Settings overview](04-settings-overview.png)
- [Narrow screen](05-grid-mobile.png)
- [Desktop light](06-grid-light.png)
- [Compact provider details](07-compact-details.png)
- [Unofficial source tooltip](08-source-tooltip.png)
- [Compact history and forecasts](09-compact-history.png)
- [Compact history on a narrow screen](10-compact-history-mobile.png)
- [Compact history in the light theme](11-compact-history-light.png)
- [Codex reset confirmation](12-codex-reset-confirmation.png)
- [Claude low-priority command](13-claude-low-priority.png)
- [Monitor update banner](14-monitor-update-banner.png)
- [Monitor update on a narrow screen](15-monitor-update-mobile.png)
- [Codex reset on a narrow screen](16-codex-reset-mobile.png)
