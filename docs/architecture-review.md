# Architecture review (2026-11)

A pass over the whole repository asking four questions: are the boundaries the ADR promises still
true, what does the flyout pay to start, where do names and folders disagree with the rules in
`AGENTS.md`, and what is left that is worth doing. What was measurable was measured; what was
changed is listed with the reason; what was left alone is listed too, with why.

## Verdict

The Rust side holds its shape, checked against the manifests rather than taken from the ADR:
`usage-model` and `statusline-shim` depend on no workspace crate, `usage-support` only on
`usage-model`, `app-core` and all sixteen collectors only on those two, and `src-tauri` is the one
crate that names Tauri and the collectors. The contract is pinned from both sides by the golden
files. Nothing there was changed.

The web side had drifted in three places, all fixed here: the bundle shipped every language to
every user, production code imported fabricated fixtures, and `App.tsx` had grown into the place
where every piece of flyout state lived. A fourth, the flat `src/app` folder, was a matter of
finding things rather than of correctness, and is sorted here too.

## Findings and what was done

| # | Finding | Evidence | Change |
|---|---|---|---|
| 1 | All ten message catalogs were in the entry chunk: nine unused languages parsed on every start | entry JS 1 417 kB, of which about 830 kB were catalogs | English stays in the entry; each other locale is its own chunk, fetched once (`src/i18n/catalogs.ts`). `t()` is still synchronous and still falls back to English key by key |
| 2 | `src/home/paletteModel.ts` imported `src/mock/scenarios`, which pulled the fabricated fixtures into the production bundle | `mock/scenarios`, `mock/builders` and friends in the entry chunk's source map | The names moved to `src/mock/scenarioNames.ts`; `src/test/boundaries.test.ts` fails on any new static import of a fixture from production code |
| 3 | `App.tsx` sat on the 300-line limit and owned overlays, surface shortcuts and two timers besides the wiring | 300 lines, 10 `useState`/`useRef`, 4 `useEffect` | `useOverlays` (detail and palette, with focus return), `useSurface` (state and Alt+digit), `useTimedText`; `App.tsx` is 250 lines of wiring |
| 4 | Surface ids were Portuguese (`triagem`, `horizonte`, `canais`) against "identifiers are English", with a table translating them back for the message keys | `SURFACE_WORDS` in `Fly.tsx` | Ids are `triage`, `horizon`, `channels`; the table is gone |
| 5 | Two modules were kebab-case in a camelCase tree | `channels-blocked.ts`, `channels-read.ts` | Renamed to `channelsBlocked.ts`, `channelsRead.ts` |
| 6 | `tn()` built an `Intl.PluralRules` per call | one per counted message, per render | Cached per locale, as the other `Intl` objects already were |
| 7 | `src/app` held 60 files in one folder: hooks, contexts, wording and pure models side by side | `ls src/app` | Four folders by what a file is: `hooks/` (12), `context/` (4), `text/` (7), `model/` (16), tests beside their subject. A move only, in a commit of its own, so `git log --follow` reads through it |
| 8 | The trend chart reserved a fifth of its width for labels that then overlapped the projection, and capped its height so the picture letterboxed inside its box | `RIGHT = 66` of 340; `max-block-size` | Labels are placed around the dot by rule (`labelSpot`), the plot uses the width, the frame follows the viewBox |

## Benchmark

`node scripts/bench-boot.mjs before=<old dist> after=dist --runs 15 --cpu 4` — Chromium, 4× CPU
throttle, median of 15 cold loads on the mock backend, measured from navigation to the first
settled frame (`.nx-fly[data-loaded]`). "Before" is `cfae9ef`.

| Language | Build | Entry JS | Entry gzip | JS loaded | Ready | Script time | JS heap |
|---|---|--:|--:|--:|--:|--:|--:|
| English | before | 1 383.8 kB | 367.3 kB | 1 390.6 kB | 1 110 ms | 552 ms | 6.7 MB |
| English | after | 559.7 kB | 164.9 kB | 580.8 kB | 794 ms | 397 ms | 5.9 MB |
| Português | before | 1 383.8 kB | 367.3 kB | 1 390.6 kB | 1 082 ms | 559 ms | 7.2 MB |
| Português | after | 559.7 kB | 164.9 kB | 629.9 kB | 813 ms | 423 ms | 5.7 MB |

Entry chunk −60 %, JS loaded −58 % (English) and −55 % (another language), time to ready −28 % and
−25 %, script time −28 % and −24 %. Sizes are bytes / 1024; Vite prints bytes / 1000. Both builds
include the mock backend's 120 ms of simulated latency, so the absolute "ready" is an upper bound
and the difference is the bundle's. A webview on an unthrottled machine sees a smaller absolute
gain with the same ratio of work removed.

What is left in the entry chunk: `react-dom` (about 202 kB), `zod` (89 kB), the English catalog
(68 kB) and the application itself (about 195 kB).

## Left alone, on purpose

- **The legacy ready panel.** `components/panel/ReadyView` renders a ready view only in tests: the
  product routes ready views through `home/Panel`. `ErrorView` still borrows `WindowsBlock` from it,
  and about 1 800 lines of tests reach the honesty rules through it. Removing it means moving those
  assertions onto `QuotaSheet` first; that is its own change.
- **`zod` → `zod/mini`.** Would take part of zod's 89 kB off the entry chunk (not measured), at the
  price of rewriting every schema in the contract layer. The strict parse is the contract; not traded for size here.
- **Lazy-loading settings, onboarding and the palette.** About 45 kB together; `Suspense` in a
  window that reports its own height to the shell would need the height protocol reworked.
- **Rust.** Not profiled. What was read is the loop every reading goes through (`app-core`'s `hub`
  and `publish`): a snapshot is compared with the last one before it is emitted, emits are spaced by
  `EMIT_MIN_GAP`, and the per-publish cost is cloning sixteen views. The time is spent waiting on
  the collectors' I/O, not there. The release profile is already `opt-level = "s"`, LTO, one codegen
  unit, stripped.

## What a later pass should measure

- The 20 s clock tick re-renders every tile (the context carries `nowSecs`). With 16 providers it
  is well under a frame; it stops being free at a few dozen.
- Vitest creates a jsdom per file (82 files, about a third of the suite's time).
