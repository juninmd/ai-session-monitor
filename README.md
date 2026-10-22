# 🚀 AI Session Monitor

<div align="center">

[![Windows 10 | 11](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/juninmd/ai-session-monitor)
[![Tauri 2](https://img.shields.io/badge/Tauri-v2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Rust 2024](https://img.shields.io/badge/Rust-2024%20(1.97.1)-DEA584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React 19](https://img.shields.io/badge/React-19-61DAFB?style=for-the-badge&logo=react&logoColor=black)](https://react.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)
[![Auto Updates](https://img.shields.io/badge/Updates-Minisign%20Signed-informational?style=for-the-badge&logo=github)](https://github.com/juninmd/ai-session-monitor/releases)

**Monitore a cota restante, o tempo de reset e o ritmo de consumo das suas sessões de IA direto da bandeja do Windows.**

[Recursos](#-principais-recursos) • [Provedores Suportados](#-provedores-suportados) • [Instalação](#-instalação-e-download) • [Segurança](#-privacidade-e-segurança) • [Desenvolvimento](#-desenvolvimento-e-build) • [Arquitetura](#-arquitetura)

</div>

---

## 💡 O que é o AI Session Monitor?

O **AI Session Monitor** é um aplicativo nativo e ultraleve para a área de notificação (System Tray) do Windows. Desenvolvido com **Tauri 2, Rust e React 19**, ele consolida em um único painel elegante o status das suas sessões de codificação e créditos em **14 ferramentas e provedores de IA**.

Nunca mais seja pego de surpresa pelo fim da janela de 5 horas do Claude, pela cota semanal do Codex ou pelo saldo zerado de uma API no meio de uma tarefa complexa.

---

## ✨ Principais Recursos

- ⏱️ **Janelas de Sessão em Tempo Real:** Acompanhe a porcentagem restante e o tempo exato até o próximo reset em janelas de 5 horas, semanais e mensais.
- ⚡ **Estimativa de Ritmo (Burn Rate):** Sabe em quanto tempo sua cota vai acabar se continuar codando no ritmo atual.
- 🛡️ **Zero-Trust & Privacidade Total:** Suas chaves ficam guardadas no **Windows Credential Manager**. O app é estritamente **read-only**, não grava em servidores remotos e o webview não tem acesso à internet externa.
- 📊 **Histórico Local de 35 Dias:** Visualize tendências de consumo diretamente pelo app. Os dados ficam salvos apenas na sua máquina em `history.json` e podem ser desativados a qualquer momento.
- 🎛️ **Navegação entre Provedores:** Busque uma ferramenta e veja o estado de todas no quadro “Todas as ferramentas”. Ative ou desative fontes nas Configurações.
- 🔄 **Releases Assinadas:** O pipeline do GitHub gera instaladores com **Minisign** e verifica o manifesto, os bytes baixados e as assinaturas antes da publicação. A integração da consulta e instalação no aplicativo está em desenvolvimento.
- 🔔 **Alertas Inteligentes:** Notificações discretas na bandeja do Windows quando uma sessão atingir níveis de alerta ou esgotamento.

---

## 🔌 Provedores Suportados

| Provedor | O que é monitorado | Método de Coleta | Tipo de Integração |
|---|---|---|:---:|
| **Claude Code** | Sessão de 5 h e semana (% disponível, tempo até reiniciar, ritmo) e cotas por modelo (Opus, Sonnet, Haiku/Fable) | `rate_limits` via shim do `statusLine` ou `claude -p "/usage"` | Oficial *(Por modelo: Comunitário)* |
| **Codex** | Janela de 5 h e semana, plano atual, créditos e recargas de reset | `codex app-server` (JSON-RPC local) | Oficial |
| **Cursor** | Uso incluso do ciclo (% usado, reinício) e gasto sob demanda | `cursor.com/api/usage-summary` com o token de login lido do banco local do Cursor (opt-in) | Comunitário |
| **GitHub Copilot** | Requisições premium do mês (% usado, reinício estimado), chat e completions ilimitados | `api.github.com/copilot_internal/user` com token colado ou lido do arquivo local da extensão (opt-in) | Comunitário |
| **Antigravity** | Pools Gemini e Claude/GPT-OSS (5 h e semanal) | CLI `agy` (`/usage`) ou servidor local da IDE | Comunitário |
| **OpenCode** | Janelas de 5 h, semana e mês no Go; estimativa de custo local | Endpoint de uso do Go + `opencode.db` (leitura local) | Comunitário |
| **Kimi Code** | Janelas de 5 h, semana e pool mensal | Endpoint de uso Kimi Code via API Key | Comunitário |
| **z.ai** | Plano GLM Coding: janelas de 5 h, semana e cota de ferramentas MCP | Endpoint de cota z.ai via API Key | Comunitário |
| **OpenRouter** | Limite da chave, consumo do dia/semana/mês e saldo de créditos | API oficial do OpenRouter (`/key`, `/credits`) | Oficial |
| **DeepSeek** | Saldo da conta por moeda (CNY e USD, nunca somados) | API de saldo do DeepSeek via API Key | Oficial |
| **DeepInfra** | Saldo em conta, gasto do mês e janela percentual de limite | API de cobrança DeepInfra via API Key | Oficial |
| **Moonshot** | Saldo da conta, vouchers promocionais e dinheiro (USD) | API oficial da Moonshot (`api.moonshot.ai`) | Oficial |
| **Novita** | Saldo da conta, limite de crédito e cobranças pendentes | API de saldo da Novita via API Key | Oficial |
| **Ollama Cloud** | Créditos mensais incluídos (ou janelas antigas) e saldo comprado | API de saldo do Ollama Cloud via API Key | Oficial |
| **MiniMax** | Cota do Token Plan por modelo: intervalo ativo e semana | Rota de cota do Token Plan via API Key | Comunitário |
| **LiteLLM** | Orçamento parametrizável de chave, usuário ou equipe (suporte a CA própria) | API do LiteLLM Proxy (`/key/info`, `/user/info`, `/team/info`) | Oficial |

> [!NOTE]
> **O que significa o indicador "Em horas"?**
> Dois valores são apresentados em conjunto: o **tempo até reiniciar** (contagem regressiva até o reset da janela) e a **estimativa de ritmo** (quanto tempo a cota durará no ritmo de requisições recente). Quando o consumo não puder ser determinado com exatidão, o valor é exibido como *desconhecido*, nunca como 0 %.

---

## 🔒 Privacidade e Segurança

Projetado sob o princípio do **privilégio mínimo**:

1. **Armazenamento Seguro de Credenciais:** As chaves de API informadas são mantidas no **Gerenciador de Credenciais do Windows** (`keyring` do SO). Nenhuma chave é salva em texto puro, logs, arquivos de configuração ou trafegada para a interface gráfica.
2. **Somente Leitura (Read-Only):** O monitor nunca executa comandos de escrita nas suas contas, não faz dispatch de tarefas nos LLMs e não renova tokens por conta própria.
3. **Isolamento de Rede na Interface:** A janela do app roda com **Content Security Policy (CSP)** estrita (`connect-src ipc:`). O frontend React não faz chamadas HTTP diretas; todas as requisições partem do núcleo em Rust através de um cliente seguro (`SafeClient`) com limites rigorosos de tempo e tamanho.
4. **Proteção Contra Vazamento em Proxies:** O cliente HTTP em Rust ignora deliberadamente variáveis de ambiente `HTTP(S)_PROXY` do sistema, evitando que proxies de terceiros interceptem credenciais.
5. **Consentimento Explícito:** Fontes que leem arquivos locais ou inspecionam processos (como Claude, OpenCode e Antigravity) permanecem pausadas até você autorizar expressamente na aba.

---

## 📥 Instalação e Download

### Requisitos de Sistema
- **Sistema Operacional:** Windows 10 (Build 19041+) ou Windows 11 (64-bit).
- **WebView2 Runtime:** Já instalado por padrão no Windows 11 e na maioria dos sistemas Windows 10 atualizados.

### Baixar Instalador
Acesse a página de **[Releases](https://github.com/juninmd/ai-session-monitor/releases)** e baixe a versão mais recente do instalador:
- `AI.Session.Monitor_<versao>_x64-setup.exe` (Instalador NSIS por usuário, não requer elevação de administrador).

O aplicativo cria o ícone na bandeja do sistema. O fluxo de atualização implementado consulta o GitHub automaticamente a cada 6 horas (configurável), mostra a nova versão e permite baixar e instalar após confirmação. O instalador é verificado por assinatura antes da execução. A primeira publicação com esse fluxo e a prova de upgrade entre versões assinadas ainda estão pendentes; veja [o progresso e as validações](docs/ui-updater-progress.md).

---

## 🛠️ Desenvolvimento e Build

### Pré-requisitos
- [Node.js](https://nodejs.org/) `22.22.2+`, `24.15+` ou `>=26.0.0`
- [pnpm](https://pnpm.io/) `10+` (`corepack enable`)
- [Rust](https://www.rust-lang.org/) `1.97.1` (definido em `rust-toolchain.toml`) e Visual Studio C++ Build Tools

### Comandos Rápidos

```sh
# Instalação das dependências
pnpm install --frozen-lockfile

# Desenvolvimento com backend mock no navegador (ideal para estilização de UI)
pnpm dev

# App nativo completo em modo desenvolvimento (Bandeja + Flyout Tauri)
pnpm tauri dev

# Suíte completa de verificação (Biome lint + formatação, TypeScript check, Vitest e Vite build)
pnpm verify

# Testes automatizados do workspace Rust
cargo test --workspace
```

### Cenários do Mock Backend
Para testar estados de borda da interface no navegador (`http://127.0.0.1:1420/?mock=<cenario>`):
- `?mock=demo` — Dados realistas de demonstração com múltiplos provedores ativos.
- `?mock=fresh` — Estado limpo recém-instalado, aguardando consentimentos e chaves.
- `?mock=healthy` — Todas as cotas cheias e sem avisos pendentes.
- `?mock=critical` — Sessões em esgotamento iminente e cotas zeradas.
- `?mock=stale` — Conexões expiradas ou serviços fora do ar.
- `?mock=errors` — Erros de autenticação e falhas de rede simuladas.
- `?mock=loading` — Estado perpétuo de carregamento.

### Isolamento Seguro em Builds de Debug
Para executar o aplicativo sem tocar no seu Gerenciador de Credenciais real ou arquivos do seu usuário:

```powershell
$env:AISM_DATA_DIR = "$env:TEMP\aism-data"
$env:AISM_HOME_DIR = "$env:TEMP\aism-home"
$env:AISM_MEMORY_SECRETS = "1"
$env:AISM_SHOW_ON_START = "1"
pnpm tauri dev
```

> [!TIP]
> Essas variáveis de ambiente só existem em compilações de `debug` e são totalmente removidas em builds de `release`.

### Gerando o Build de Produção

```sh
# Executável avulso de release
pnpm tauri build --no-bundle

# Instalador NSIS final
pnpm tauri build
```

---

## 🏛️ Arquitetura

O projeto é organizado como um workspace modular garantindo separação rígida de responsabilidades:

```
ai-session-monitor/
├── crates/
│   ├── usage-model/        # Modelos de domínio, janelas, cálculo de status e ritmo (sem I/O)
│   ├── usage-support/      # HTTP cliente blindado, escrita atômica, runners de processo
│   ├── collector-*/        # Um crate independente para cada fonte (sem depender de Tauri)
│   ├── statusline-shim/    # Binário ultra-leve injetado no statusLine do Claude Code
│   └── app-core/           # Hub central, agendamento de coletas e store de configurações
├── src-tauri/              # Inicialização nativa do Windows, System Tray, atalhos e IPC
├── src/                    # Interface React 19 + TypeScript + Tailwind/CSS limpo
├── docs/                   # ADRs, contratos IPC e especificações de segurança
└── contract/               # Arquivos Golden pinning da paridade Rust <-> TypeScript
```

Documentação de aprofundamento:
- [Decisões de Arquitetura (ADR 0001)](docs/adr/0001-architecture.md)
- [Fontes de Dados e Níveis de Confiança](docs/data-sources.md)
- [Contrato IPC & Mensagens](docs/ipc.md)
- [Checklist de Homologação Manual no Windows](docs/manual-windows-checklist.md)
- [Referências e Links Oficiais](REFERENCES.md)

---

## ⚠️ Limitações Conhecidas

- **Integrações Comunitárias:** Os endpoints de Cursor, GitHub Copilot, Antigravity, OpenCode, Kimi Code, z.ai e MiniMax não são APIs públicas documentadas e podem sofrer alterações pelos provedores. O app trata quebras sinalizando status de "indisponível" em vez de inventar dados.
- **Claude Code statusLine:** O statusLine só emite métricas enquanto o processo do Claude Code estiver em execução ativa.
- **Inicialização com o Windows:** Caso desativada manualmente pelo Gerenciador de Tarefas do Windows, deve ser reativada por lá. Se o executável for movido de diretório, desligue e religue a opção nas configurações do app.
- **Proxies Corporativos:** Devido à política estrita anti-interceptação de chaves, conexões através de proxy local podem necessitar de configuração direta ou CA corporativa na aba do LiteLLM.

---

## 📄 Licença

Distribuído sob a licença **MIT**. Consulte [`LICENSE`](LICENSE) para mais detalhes.

Logotipos e marcas das ferramentas monitoradas pertencem aos seus respectivos proprietários e são utilizados estritamente para identificação dos provedores. Detalhes de ativos de terceiros em [`docs/third-party-assets.md`](docs/third-party-assets.md).
