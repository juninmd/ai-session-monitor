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
- [Anthropic Claude Code](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview) - Session limits, statusLine specification, and `claude -p "/usage"`.
- [OpenAI Codex](https://github.com/openai/codex) - `codex app-server` JSON-RPC protocol.
- [OpenRouter API](https://openrouter.ai/docs) - Key spending limits (`/key`) and credit balances (`/credits`).
- [DeepSeek Open Platform](https://api-docs.deepseek.com/) - Account balance API (`/user/balance`).
- [DeepInfra Billing API](https://deepinfra.com/docs) - Balance, usage, and spending limits.
- [Moonshot AI Platform](https://platform.moonshot.cn/docs) - Balance, vouchers, and cash endpoints.
- [Novita AI Documentation](https://novita.ai/docs) - Account balance and credit limits.
- [Ollama Cloud](https://ollama.com/) - Cloud credits and quota endpoints.
- [LiteLLM Proxy](https://docs.litellm.ai/) - Spend management and budgets (`/key/info`, `/user/info`, `/team/info`).

### Unofficial & Local Integrations
- [Google Antigravity](https://antigravity.google/) - CLI `agy` (`/usage`) and local IDE server telemetry.
- [OpenCode](https://opencode.ai/) - Go endpoint and local `opencode.db` cache.
- [Kimi Code](https://kimi.moonshot.cn/) - Kimi Code quota endpoint.
- [z.ai](https://z.ai/) - GLM Coding plan quota endpoint.
- [MiniMax](https://www.minimaxi.com/) - Token Plan quota endpoint.
