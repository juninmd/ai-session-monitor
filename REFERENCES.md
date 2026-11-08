# References

Official and community documentation, specifications, and upstream repositories referenced by **AI Session Monitor**.

## Core Frameworks & Tooling

- [Tauri v2 Documentation](https://v2.tauri.app/) - Desktop application framework (Rust + WebView2).
- [React 19](https://react.dev/) - UI library.
- [Vite](https://vite.dev/) - Frontend build tool and dev server.
- [Biome](https://biomejs.dev/) - Toolchain for formatting and linting TypeScript/JavaScript.
- [Minisign](https://jedisct1.github.io/minisign/) - Cryptographic signature verification for releases and auto-updates.

## Supported AI Providers & APIs

### Official APIs
- [Claude Code statusLine](https://code.claude.com/docs/en/statusline) - `rate_limits` (5 h, 7 d, `spend_limit`) and the status line JSON. Claude Code setup: https://code.claude.com/docs/en/setup.
- [OpenAI Codex](https://github.com/openai/codex) - `codex app-server` JSON-RPC protocol (docs: https://developers.openai.com/codex/app-server).
- [OpenRouter API](https://openrouter.ai/docs/api/reference/limits) - Key spending limits (`/api/v1/key`) and credit balances (`/api/v1/credits`).
- [DeepSeek Open Platform](https://api-docs.deepseek.com/api/get-user-balance) - Account balance API (`/user/balance`).
- [DeepInfra Billing API](https://docs.deepinfra.com/api-reference/billing/get-checklist) - Checklist, usage and spending limits.
- [Moonshot AI Platform](https://platform.kimi.ai/docs/api/balance) - Balance API on `api.moonshot.ai` (the `.cn` and `kimi.com` platforms use separate keys).
- [Novita AI Documentation](https://novita.ai/docs) - Account balance and credit limits.
- [Ollama Cloud](https://ollama.com/) - Cloud credits and quota endpoints.
- [LiteLLM Proxy](https://docs.litellm.ai/) - Spend management and budgets (`/key/info`, `/user/info`, `/team/info`).

### Unofficial & Local Integrations
- [Google Antigravity](https://antigravity.google/) - CLI `agy` (`/usage`) and local IDE server telemetry.
- [OpenCode](https://opencode.ai/) - Go endpoint and local `opencode.db` cache.
- [Cursor](https://cursor.com/) - Usage summary endpoint and the sign-in token in Cursor's local store (community-documented, see `docs/data-sources.md`).
- [GitHub Copilot](https://github.com/features/copilot) - `copilot_internal/user` quota endpoint and the OAuth token the Copilot extension keeps (community-documented).
- [Kimi Code](https://github.com/MoonshotAI/kimi-cli) - Coding usages endpoint `api.kimi.com/coding/v1/usages`, as read from the vendor's CLI source.
- [z.ai](https://docs.z.ai/) - GLM Coding Plan quota endpoint (`zai-org/ZCode` document).
- [MiniMax](https://platform.minimax.io/) - Token Plan `token_plan/remains` route on `api.minimax.io` (community-documented; not published by MiniMax).

### Related Projects (comparison)
- [CodexBar](https://github.com/steipete/CodexBar) - macOS menu bar app for the same kind of limits.
- [ai-usagebar](https://github.com/akitaonrails/ai-usagebar) - Rust CLI/TUI/tray for many providers, including a Windows tray build.
- [Usage Monitor for Claude](https://github.com/jens-duttke/usage-monitor-for-claude) - Windows and Linux tray app for Claude with event commands.
- [ClaudeBar](https://github.com/tddworks/ClaudeBar) - macOS menu bar app with a long provider list.
- [ccusage](https://github.com/ryoppippi/ccusage) - Local token and cost history from Claude Code transcripts (CLI).
