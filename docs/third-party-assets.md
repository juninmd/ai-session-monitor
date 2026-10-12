# Third-party assets

The only third-party artwork in the app is the logo of each monitored tool, shown in the tab strip
(20 px), in the first-run cards (32 px) and next to each source in the settings (20 px). Each logo is
one sanitized SVG kept in `src/components/brand/vendor/` and one React component next to it
(`src/components/brand/*Logo.tsx`). The logos are inlined in the bundle, so no request is made for
them at run time, and no npm package was added for the icons.

Sources were researched and retrieved on 2026-10-11. This file is not legal advice.

## Logos

| Brand | Harness | Vendored file | Upstream source (pinned) | Icon-set license | Owner of the mark | SHA-256 of the upstream file | SHA-256 of the vendored file | What was changed |
|---|---|---|---|---|---|---|---|---|
| Claude Code | `claude` | `claude.svg` | LobeHub Icons, `@lobehub/icons-static-svg@1.95.1`, [`icons/claudecode-color.svg`](https://cdn.jsdelivr.net/npm/@lobehub/icons-static-svg@1.95.1/icons/claudecode-color.svg) | MIT, Copyright (c) 2023 LobeHub | Anthropic, PBC | `670b3d8d749d0815ad8f7e62a59d51cb5e2053cb19403f3541a8aed7036a877f` | `313b393f1c0d9cf744ce83875ac977fdccc55239cd0520cec1584088307ca970` | Sanitized only: root `width`, `height` and `style` removed. Everything after the root start tag is byte-identical (path data, `#D97757`). |
| Codex | `codex` | `codex.svg` | LobeHub Icons, `@lobehub/icons-static-svg@1.95.1`, [`icons/codex-color.svg`](https://cdn.jsdelivr.net/npm/@lobehub/icons-static-svg@1.95.1/icons/codex-color.svg) | MIT, Copyright (c) 2023 LobeHub | OpenAI | `4a2f43ce46b5b6e3722c95088f88d26ef91e6a8c2e598e70642a1c54367386e4` | `519238dcde20d6056bd5f8a43183a2f06937b27666cd5ff08ab65c6269e8e528` | Sanitized only: root `width`, `height` and `style` removed, root attributes reordered. Everything after the root start tag is byte-identical (paths, gradient). |
| Antigravity | `antigravity` | `antigravity.svg` | LobeHub Icons, `@lobehub/icons-static-svg@1.95.1`, [`icons/antigravity-color.svg`](https://cdn.jsdelivr.net/npm/@lobehub/icons-static-svg@1.95.1/icons/antigravity-color.svg) | MIT, Copyright (c) 2023 LobeHub | Google LLC | `652128cf55a28bb958563ee372d0bd00208d9e6fd446f523f77aafd811eef8ca` | `a407bd965b3ebf72a818296cb8de82a9efd96bf28e2bac459615cc870f5e8bc0` | Sanitized only: root `width`, `height` and `style` removed. Everything after the root start tag is byte-identical (mask, blobs, filters). |
| OpenCode | `opencode` | `opencode.mono.svg` | OpenCode identity mark, `anomalyco/opencode` tag `v1.19.0` (commit `733b2c627e5aec39cb33cc22373bdce33053ef2c`), [`packages/identity/mark.svg`](https://raw.githubusercontent.com/anomalyco/opencode/733b2c627e5aec39cb33cc22373bdce33053ef2c/packages/identity/mark.svg) | MIT, Copyright (c) 2025 opencode (the repository licence) | Anomaly Innovations, Inc. | `e29bbe33380ad1c1ada9134b52f229d30e9776d60481512c9d81f2bb6f37def9` | `6eac8cab9381fa85c0ab5c33a8627c3462878d5a3c54de93de3c815441df13b3` | Sanitized and recoloured to one ink: outer wrapper `svg` and trailing `style` removed, background tile dropped, white and `#5A5858` fills became `currentColor` (the inner block at `fill-opacity` 0.3), viewBox cropped to `64 64 384 384`, `<title>` added. Both `d` strings are untouched. |
| LiteLLM | `litellm` | `litellm.mono.svg` | LiteLLM monogram, `BerriAI/litellm` tag `v1.105.0-rc.3` (commit `c09b4d5a09e6f5c09080c0762fb200d91572f331`), [`ui/litellm-dashboard/public/assets/logos/litellm_monogram.svg`](https://raw.githubusercontent.com/BerriAI/litellm/c09b4d5a09e6f5c09080c0762fb200d91572f331/ui/litellm-dashboard/public/assets/logos/litellm_monogram.svg) | MIT, Copyright (c) 2023 Berri AI | Berri AI | `69eaed389008a423e98590466c98b80f4ab7ffec47f4bda53ee0db3890775ba0` | `937486b8fb3a449c1662ad094a8bfdada1e2e8f73846b8ccabb8754aba39f58d` | Sanitized and recoloured to one ink: XML prolog, root `id` and `data-name`, `defs` and `style` removed, `class="cls-1"` became `fill="currentColor"` (was `#0117be`), whitespace between elements removed, `<title>` added. All seven `d` strings are untouched. |

Each vendored file is a single line with no line break, so a checkout with line-ending conversion
(`core.autocrlf`) cannot change its hash. Check them with
`sha256sum src/components/brand/vendor/*.svg`.

How the files are used, and what keeps them honest:

- The components copy the path data from the vendored files; a test compares each rendered logo
  with its file element by element (`d`, fills, gradient, mask and filter geometry). A test also
  audits every vendored file and every rendered logo against a small allowlist: one `svg` root with a
  `viewBox`, no `script`, `style`, `foreignObject`, `image`, `use`, event handler, `href` or remote
  `url()`.
- Colour logos (Claude Code, Codex, Antigravity) keep their upstream colours. Mono logos
  (OpenCode, LiteLLM) paint `currentColor`; `src/styles/brand.css` sets the ink.
- LiteLLM's brand kit allows only LiteLLM Blue `#0117BE` on light surfaces and white `#FFFFFF` on
  dark ones, with no other colour, opacity or effect. The stylesheet pins exactly those two values.
- Size is the only transformation: a uniform scale (the tab strip and the settings rows nudge the
  wide Claude Code mascot up and the full-bleed Codex and LiteLLM tiles down for optical balance;
  the first-run cards draw every logo at its natural size). Proportions and colours are never
  altered.

## Trademarks

Claude Code and Anthropic belong to Anthropic, PBC; Codex and OpenAI to OpenAI; Antigravity and
Google to Google LLC; OpenCode to Anomaly Innovations, Inc.; LiteLLM to Berri AI. The logos are
trademarks of their owners. This app shows them only to identify the tool whose usage each tab
displays. It is not affiliated with, endorsed by or sponsored by any of those companies, and the MIT
licence of an icon set covers its artwork files, not any trademark right. No owner was asked for
permission.

What each owner publishes (summarized from the pages read on 2026-10-11), and where the app stands:

- **Anthropic.** The Trademark Guidelines (effective 2024-08-01) allow the marks only as
  specifically permitted and in materials Anthropic approves beforehand, forbid altering colour,
  font or proportion, and forbid implying sponsorship, endorsement or affiliation. Anthropic
  publishes no mark list or press kit and nothing specific to the Claude Code mascot. Its Agent SDK
  branding guidelines tell partner products not to use "Claude Code-branded" visual elements or to
  appear to be an Anthropic product; that rule is written for SDK partners and this app only labels
  the sessions of the tool it monitors, but it is the closest published rule. The file is LobeHub's
  trace of the mascot (no standalone Anthropic vector exists), so treat it as provisional and swap
  it for Anthropic-supplied artwork if that becomes available. The colours are unmodified.
- **OpenAI.** No Codex-specific terms were found. OpenAI's general design guidelines (read from a
  Wayback snapshot of 2026-09-21; the live page returns 403) ask for permission, say to use logos
  exactly as provided and only where they directly relate to OpenAI services, and forbid implying
  endorsement, modifying a logo or folding it into other branding. They name the OpenAI wordmark and
  Blossom, not Codex. The cloud-and-prompt mark is the only Codex-specific mark found on an official
  OpenAI page, but its currency is unconfirmed: from about July 2026 Codex lives inside the ChatGPT
  desktop app, whose icon is the Blossom, and an OpenAI forum thread (August 2026) calls the cloud
  icon "the previous Codex logo".
- **Google.** No Antigravity-specific terms were found: `antigravity.google/press` offers the
  official icon with no licence or usage text, and Antigravity is not on Google's public trademark
  list page. Google's general guidelines ask for Google-approved artwork, no alteration and no
  implied affiliation. The vendored file is LobeHub's redraw: same outline and palette as the
  official icon (largest control-point error 0.006 percent of the width) but without the official
  file's opaque `#3186FF` base arch, so it is not Google-approved artwork. The official full-colour
  icon is `https://antigravity.google/assets/image/brand/antigravity-icon__full-color.svg` (SHA-256
  `d775965ea1f8060538c15f758a3e22487317211fcb6051333b5e023f35aa726f` as fetched on 2026-10-11; the
  URL is not version-pinnable).
- **OpenCode.** No trademark or logo-usage policy was found (`opencode.ai/brand` lists PNG and SVG
  assets without rules). The repository README asks projects that use "opencode" in their name to
  say they are not built by or affiliated with the OpenCode team; this app does not use the name in
  its own and says so anyway. The app draws the mark in one ink, dropping the official dark tile and
  recolouring the two fills, so it is a derived monochrome variant of the official logo.
- **LiteLLM.** The brand kit (`litellm.ai/brand-assets` and its Logo Guidelines PDF) offers the
  monogram for "app icons, favicons, social avatars, UI elements and small digital placements", in
  LiteLLM Blue `#0117BE` or white `#FFFFFF` only, used as provided, with no stretching, rotating,
  recolouring, outlines, shadows or other effects, and with clear space of half the logo height.
  The stylesheet pins exactly those two colours; the clear-space rule was not audited. The
  monogram first appears in `v1.105.0-dev.2`, so the pin (`v1.105.0-rc.3`) is a pre-release tag.

An owner who wants a logo removed or replaced only needs the matching vendored file, component,
test fixture entry and table row deleted.

## License notices

The MIT licence requires its copyright notice and permission notice to travel with copies or
substantial portions of the artwork, which includes the vendored files and the path data compiled
into the app. No source requires on-screen attribution. The notices:

```
Copyright (c) 2023 LobeHub      (@lobehub/icons-static-svg: Claude Code, Codex, Antigravity)
Copyright (c) 2025 opencode     (anomalyco/opencode: OpenCode)
Copyright (c) 2023 Berri AI     (BerriAI/litellm: LiteLLM)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

Where each licence text was read, at the commit the file was pinned to. The permission text above is
byte-identical to the one in all three files; LiteLLM's file also opens with a preface that puts its
`enterprise/` directory under a separate licence, which does not reach the `ui/` monogram.

| Source | Licence file | SHA-256 |
|---|---|---|
| `lobehub/lobe-icons` at `49a2130df7bfa5eb1b088261bff20a37e2967789` (the commit `@lobehub/icons-static-svg@1.95.1` was published from; the npm tarball ships no LICENSE file, its `package.json` declares MIT) | [`LICENSE`](https://raw.githubusercontent.com/lobehub/lobe-icons/49a2130df7bfa5eb1b088261bff20a37e2967789/LICENSE) | `add9d7531d1b21646317a8958e38fc727506fa39d24bdecb44154d943c82753a` |
| `anomalyco/opencode` at `733b2c627e5aec39cb33cc22373bdce33053ef2c` | [`LICENSE`](https://raw.githubusercontent.com/anomalyco/opencode/733b2c627e5aec39cb33cc22373bdce33053ef2c/LICENSE) | `625f0f619133f89bbbb2abe37369613dfa1885eba1e50d02170deb62bb42cb6b` |
| `BerriAI/litellm` at `c09b4d5a09e6f5c09080c0762fb200d91572f331` (MIT applies to everything outside `enterprise/`; the monogram lives under `ui/`) | [`LICENSE`](https://raw.githubusercontent.com/BerriAI/litellm/c09b4d5a09e6f5c09080c0762fb200d91572f331/LICENSE) | `b170d6bf8e8835dd357e011681db028f4d51e2fb0ea892058f56e01fb39b8273` |

Not wired yet: a built app or installer does not carry these notices. `src-tauri/tauri.conf.json`
has no `bundle.resources` entry for this file and the UI has no About view. Ship this file with the
bundle, or show it in an About view, before distributing a build.

## Adding or replacing a logo

1. Download the file from a pinned URL, record its SHA-256, and read the licence at the same commit.
2. Sanitize it to plain vector markup (no script, style, event handler, `image`, `foreignObject` or
   remote reference), keep a `<title>`, remove the root size attributes, and save it as one line in
   `src/components/brand/vendor/`.
3. Write the component from the file (path data copied, never retyped) and register it in
   `HarnessLogo.tsx`.
4. Extend the allowlist in `src/test/svg.ts` only if the artwork truly needs a new element, then
   update the tests in `HarnessLogo.test.tsx` and the table above.
