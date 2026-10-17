# Proposed interface (triagem, horizonte, comando, primeiro uso)

Browser-only preview of a reworked home screen, based on `master` at `9d24f26`. All readings,
settings and credentials are fabricated and held in memory; reloading restores the demonstration.
Nothing here reads a profile, the Credential Manager or the network.

Run `pnpm dev` and open:

<http://127.0.0.1:1420/?prototype=next&mock=demo>

The surface switch at the top selects what to review; the scenario links replace the fabricated
data. `?prototype=grid` still opens the previous prototype untouched.

## Why this exists

The current interface answers "how much is left?" well and "will it last?" badly. Concretely:

| Signal | Where it lives today | Why that is a problem |
|---|---|---|
| Share of each quota | grid card (`ProviderGrid`, three columns ≈ 7.5 rem each) | quota names are truncated, the card carries up to three bars plus a "+N" counter |
| **Will it run out before the reset?** | inside one provider's detail, 11 px, computed by `paceOf` | it is the most actionable number on the screen and the least visible |
| **When does it renew?** | column of the detail table | it is what decides urgency, and it is absent from the card |
| Which provider needs attention | fixed `TAB_ORDER` | a provider at risk sits wherever it was registered |
| Notifications | crossing of the 90 % threshold | a projection ("this runs out in 26 min") never notifies, only a percentage does |
| History | collapsed inside the detail | no trend is visible without opening a provider |
| Settings | fourteen collapsed groups | the first run has no sequence, only a list |

Everything that decides status, pace and expiry comes from `src/lib/usage.ts` (`statusOf`,
`paceOf`, `summarize`) through `src/app/windowModel.ts`. The prototype adds **no new rule**; it
changes where the existing numbers are printed and in what order.

## Surfaces

### Triagem — "do I need to act?"

- One headline, one sentence: the first provider that will not reach its renewal, and how many
  others are in the same situation. With nothing at risk it says so, and mentions the sources it
  could not read rather than implying they are fine.
- Providers are ordered by urgency, not by registration: **blocked** (quota exhausted) → **risk**
  (runs out before the renewal) → **broken** (failed or stale reading) → **watch** (at the
  threshold, no projection) → **loading** → **fine** → **unknown**.
- Inside a tier, the soonest exhaustion wins; a failure outranks an old reading.
- Four groups, each a real heading: *Precisa de atenção* (full rows, with the projection, the jump
  links from the headline, and the right to be three lines), *Lendo*, *Em dia* (one line each),
  *Sem leitura*.
- `loading` is its own tier. The first snapshot puts every enabled source in `loading`
  (`docs/ipc.md`), so treating it as "no reading" would report a gap that does not exist; the
  headline says "Lendo N fontes…" and the rows say "Lendo…" instead of a dash.
- Selecting a row opens the same detail surface the current interface uses, so nothing learned
  there is lost.

### Horizonte — "do I reach the renewal?"

- One bar per provider, from now to that window's reset. The fill is the share of that stretch the
  remaining quota covers, from `paceOf`'s runway; the rest of the track is the stretch with no
  quota, in red, with its size in time ("1 h 12 sem cota").
- The fill keeps the colour `statusOf` gives the provider, so the same quota can never read "fine"
  here and "critical" in the triage; the red belongs to the track, which is what the eye needs to
  find.
- An exhausted window fills nothing and says "esgotado". A window that covers the whole period says
  "chega ao fim". Readings with no burn rate (stale, unknown, too early) are a hatch and a word.
- The foot prints both moments that matter: "esgota às 18:40 · renova em 3 h 10".
- **Mostrar tendência** draws a trend line per row from the stored epochs the burndown already
  reads (`useHistory` + `burndownOf`). It is off by default and fetches nothing until it is
  switched on: one history request per provider is not free.

### Comando — Ctrl+K

- One box for: jump to a provider, refresh everything or one source, copy the fleet state as text,
  open the settings of a provider, switch the demonstration scenario. The list is built from data
  (`paletteModel.ts`), so it never goes stale.
- Types match on label first, then on hint and keywords; accents and case are folded.
- Up/Down/Enter/Escape; the surface also lists every command so the content can be reviewed without
  a keyboard.
- The palette is not modal (`role="dialog"`, no `aria-modal`): nothing behind it is made inert, it
  closes on Escape and on a click outside, the options are not tab stops (the arrow keys move
  `aria-activedescendant`), and Escape puts focus back on whatever opened it.

### Primeiro uso

- Four steps: **Escolher** (what to monitor, with what already responds), **Conectar** (pasted keys
  through the existing `VendorKey`, permissions through the existing settings write),
  **Validar** (one reading, then what each source answered, in the flyout's own words),
  **Pronto**.
- The picked list derives from `settings[id].enabled` until the user touches it, so it cannot be
  frozen on a placeholder before settings load, and a source that merely has no key yet is not
  switched off by pressing Continue.
- The wizard stays mounted while a source's settings are open, so the step and the picked list
  survive the detour.
- It reuses `useSettingsApi` and the source patch the settings screen uses; it invents no new
  channel and never shows a raw error string.

### Canais de cota — the evidence list

The answer to "are there more quotas and budgets to read?" — see `channels.ts`,
`channels-read.ts`, `channels-blocked.ts` and `candidates.ts`. Each row names the endpoint and the
fields, its shape in the app's model (**window** with a percentage and a reset, or **fact** with no
limit), the confidence, and whether the app can reach it with the credential it already accepts.

Reachable today, with the same credential and the same rule:

| Source | Channel | Shape |
|---|---|---|
| Claude Code | `extra_usage` (Usage Credits monthly budget, in cents) from the OAuth usage endpoint the app already calls — as its opt-in fallback when the CLI is absent or fails; the key is in `KNOWN_KEYS` and dropped | window |
| OpenRouter | `data.budgets[]` and `data.effective_budget` from `/api/v1/key` — several money windows **with** `resets_at` | window |
| OpenRouter | `free_model_daily_requests` — a daily count window with a reset | window |
| OpenRouter | `byok_usage*`, `include_byok_in_limit` | fact |
| z.ai | `/usage/model-usage` and `/usage/tool-usage` (24 h), plus the absolute token and MCP counts the quota response already carries and drops | fact |
| LiteLLM | `soft_budget`, `temp_budget_increase`/`temp_budget_expiry` (a money window with an end date), `/model/info` prices | window, fact |
| Codex | token counts in the `token_count` rollout event that already supplies `rate_limits` | fact |

Two of those are not simply "flip a switch": the Claude credits live on the unofficial OAuth path
and stay behind its opt-in and its badge, and both z.ai routes are host:path pairs with no entry in
`docs/data-sources.md` yet, which the project requires before anything reads them. Each row says so.

Out of reach on purpose — they need an OAuth token, a cookie, a browser session or a master key,
which `docs/data-sources.md` rules out: Codex's private `wham` endpoints (per-feature limits and
banked reset expiry), OpenCode's Console billing routes, Antigravity's Cloud Code quota, Kimi's
console detail, OpenRouter `/activity` (management key), LiteLLM `/global/spend` (master key),
Cursor's dashboard calls.

Candidate new sources with an official channel and a credential that would fit: Amp (`amp usage`),
GitHub Copilot (a plan-scoped PAT), Kiro CLI, Cline, Windsurf/Devin (service key, Enterprise),
Command Code.

### Comparar

The current interface and the proposal in one document, on the same fabricated data. It is
reachable while no settings screen is open: two `SettingsView` instances in one document would
share every element id, so opening settings from either pane leaves the comparison and shows a
single settings screen. Every id the proposal owns carries the `nx-` prefix (`ids.ts`), which keeps
its rows, panel and name spans from colliding with the current interface's `tab-<id>`,
`tab-name-<id>` and `panel` while both are on screen.

## Boundary and what is not proven

- `?prototype=next` only activates outside Tauri; the Tauri launch and the plain browser route
  keep the current interface. The backend is the in-memory mock, with 60 ms of fabricated latency.
- No collector, contract or Rust file was changed. The proposal needs no wire change: every new
  channel is either a `UsageWindow` or a `Fact`, both of which already exist in
  `crates/usage-model`.
- The prototype's JavaScript ships as a code-split chunk of the release bundle and its CSS reaches
  the main stylesheet, exactly like the `grid` prototype before it. Every selector added to
  `src/styles/index.css` is scoped to `html[data-prototype="next"]` or prefixed `nx-`, so no rule
  in the production flyout changes; the previous draft had one unscoped `kbd` rule and it is now
  scoped.
- Reading a key's query string decides the scenario, exactly as the previous prototype does.
- Not verified here: whether the mock's shapes match a live vendor answer for the channels that are
  not read yet. And the surfaces were reviewed by reading and by tests, not by a human in a browser
  or with a screen reader: the visual density of the triage list on a real 27 rem flyout, and the
  feel of the roving tab stop, still need eyes.

## Captures

Taken from the production build (`pnpm build` + `pnpm preview`) in headless Chrome over the
DevTools protocol, dark theme unless noted, 1440 px wide. The flyout keeps the application's real
frame: **432 px wide** (27 rem) and capped at `calc(100vh - 5.25rem)`, so what is measured here is
the density the app would actually have. On `?prototype=next&mock=demo` at 1440×1000 the flyout is
432 × 785 px, and the triage row order measures name line → bar → verdict → quota, 18 / 3 / 17 / 15
px, with no overlap.

| File | What it shows |
|---|---|
| [Triagem, dark](01-triagem-dark.png) | `mock=demo`: the headline naming the provider at risk, one card in *Precisa de atenção*, three one-line rows in *Em dia*, one in *Sem leitura*, and the shared detail panel |
| [Triagem, crítico](02-triagem-critical.png) | `mock=critical`: four exhausted providers with their return time, four jump links, and Codex at risk |
| [Horizonte](03-horizonte.png) | One runway bar per provider, the red uncovered remainder on Codex, "chega ao fim" fills, hatched rows where nothing can be projected |
| [Comando](04-comando.png) | The command surface with the whole list built from data |
| [Comando, paleta](04b-paleta.png) | The same page with Ctrl+K open |
| [Primeiro uso](05-primeiro-uso.png) | Step 1, with each source marked "já responde", "sem leitura" or "precisa de chave" |
| [Canais de cota](06-canais.png) | The whole evidence list, reachable and forbidden |
| [Comparar](07-comparar.png) | Both interfaces at 432 px, side by side on a 58 rem stage |
| [Triagem, carregando](08-triagem-loading.png) | `mock=loading`: "Lendo 5 fontes…", never "sem leitura" |
| [Triagem, claro](09-triagem-light.png) | The same surface in the light theme |

No capture shows interaction: hover, focus-visible, the palette's arrow keys, the wizard's steps
after the first, and any transition still need a human at a keyboard.

## Tests

`src/prototype/next/*.test.ts(x)` — 64 tests. The models: urgency order and wording, the covered
share and the sort of the horizon, the command list, its filtering and what the copied summary may
contain, and every pure helper of the wizard. The rendering: tier order in the DOM, the headline
tone and its jump links, the shared detail panel, the loading state, the empty state, arrow-key
navigation with a single tab stop, Ctrl+K opening, filtering, running and closing with focus
restored, the history staying unrequested until the trend is on, the wizard walking from a missing
key to a first reading and surviving a detour into settings, the evidence list, and both panes of
the comparison including an assertion that no element id repeats.

```sh
pnpm exec vitest run src/prototype/next
```
