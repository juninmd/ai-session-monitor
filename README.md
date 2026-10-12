# AI Session Monitor

Ícone na bandeja do Windows que mostra **quanto da sessão você ainda tem disponível** nas suas
ferramentas de código com IA, uma aba por ferramenta:

| Aba | O que mostra | Fonte | Confiança |
|---|---|---|---|
| **Claude Code** | sessão de 5 h e semana: % disponível, tempo até reiniciar, ritmo | `rate_limits` do statusLine (via shim instalado com seu consentimento) | oficial |
| **Codex** | janela de 5 h e semanal, plano, créditos e recargas de reset | `codex app-server` (JSON-RPC) | oficial |
| **Antigravity** | pools Gemini e Claude/GPT-OSS (5 h e semana) | servidor de linguagem local da IDE | **não oficial** |
| **OpenCode** | Go: janelas de 5 h, semana e mês; estimativa local de custo | endpoint de uso do Go + `opencode.db` (somente leitura) | **não oficial** |
| **LiteLLM** | orçamento da chave, usuário ou time (parametrizável) | API do proxy (`/key/info`, `/user/info`, `/team/info`) | oficial |

"Em horas" significa duas coisas, sempre mostradas juntas: o **tempo até reiniciar** (número
principal) e uma **estimativa de ritmo** (quanto tempo a cota dura se você continuar no ritmo
atual). Valor desconhecido aparece como desconhecido, nunca como 0 %.

## Privacidade e segurança

- Cada fonte que lê arquivos ou processos de outra ferramenta fica parada até você **consentir** na aba.
- O app **só lê**: nunca escreve em um provedor, nunca renova tokens e nunca usa cookies de navegador.
- As únicas credenciais guardadas são as chaves de API do LiteLLM e do OpenCode Go que você colar;
  ficam no **Gerenciador de Credenciais do Windows**, nunca em arquivo, log ou na interface.
- O histórico de uso (seção "Histórico" de cada aba) fica só neste computador, em `history.json`: porcentagens e horários por janela, por 35 dias, sem conta nem rótulos. Dá para desligar e apagar em Configurações.
- A interface web não faz requisições de rede; todo HTTP sai do Rust (somente https, sem
  redirecionamentos, com limites de tamanho e tempo). Ela só pode ouvir eventos do app, nunca emitir.

Detalhes por fonte, o que é enviado e como desligar: [`docs/data-sources.md`](docs/data-sources.md).

## Requisitos

- Windows 10/11 com WebView2 (já incluso no Windows 11).
- Node.js 22.22.2+, 24.15+ ou 26+ e pnpm 10 (`corepack enable`).
- Rust 1.97.1 (o `rust-toolchain.toml` instala sozinho via rustup) e MSVC Build Tools.

## Desenvolvimento

```sh
pnpm install --frozen-lockfile
pnpm dev            # interface no navegador com backend simulado: http://127.0.0.1:1420/?mock=demo
pnpm tauri dev      # o app de verdade (bandeja + flyout)
pnpm verify         # lint, tipos, testes e build da interface
cargo test --workspace
```

Cenários do backend simulado (`?mock=`): `demo`, `fresh`, `healthy`, `critical`, `stale`, `errors`, `loading`.

O app embute o `statusline-shim` como sidecar. `pnpm tauri build` o compila e prepara (o
`beforeBuildCommand` roda `node src-tauri/scripts/prepare-sidecar.mjs --release`). `pnpm tauri dev`
não: o `build.rs` deixa um placeholder, e instalar o shim do Claude falha em dev até você rodar esse
script uma vez (`node src-tauri/scripts/prepare-sidecar.mjs`).

Para rodar o app sem tocar no seu perfil real (build de debug), aponte diretórios temporários:

```powershell
$env:AISM_DATA_DIR = "$env:TEMP\aism-data"; $env:AISM_HOME_DIR = "$env:TEMP\aism-home"
$env:AISM_MEMORY_SECRETS = "1"; $env:AISM_SHOW_ON_START = "1"
pnpm tauri dev
```

Esses interruptores existem só em builds de debug (são compilados fora do release). Com qualquer um
ativo, as chaves ficam apenas em memória e o Gerenciador de Credenciais real não é tocado.

## Build

```sh
pnpm tauri build --no-bundle    # executável de release
pnpm tauri build                # instalador NSIS (por usuário)
```

## Estrutura

`crates/usage-model` (vocabulário e regras de status) · `crates/usage-support` (HTTP seguro,
processos, arquivos) · `crates/collector-*` (uma fonte cada) · `crates/app-core` (agendador, store,
notificações) · `src-tauri` (bandeja, janela, chaves) · `src` (React). Decisões:
[`docs/adr/0001-architecture.md`](docs/adr/0001-architecture.md); contrato:
[`docs/ipc.md`](docs/ipc.md); verificação manual no Windows:
[`docs/manual-windows-checklist.md`](docs/manual-windows-checklist.md).

## Limitações conhecidas

- Antigravity e o endpoint de uso do OpenCode não têm API pública: podem mudar sem aviso (o app mostra
  "indisponível" em vez de inventar um número). A leitura do Antigravity ainda não foi exercitada
  contra uma IDE real.
- O statusLine do Claude Code só atualiza enquanto o Claude Code está aberto.
- Se você desativar a inicialização com o Windows no Gerenciador de Tarefas, ela continua desativada
  até você reativá-la lá. Se o executável mudar de pasta, desligue e ligue a opção de novo para
  registrar o novo caminho.
- O cliente HTTP ignora proxies do sistema e variáveis `HTTP(S)_PROXY` de propósito (uma variável
  alheia não pode capturar suas chaves). Atrás de proxy corporativo, a conexão com LiteLLM e OpenCode
  precisa ser direta; CA privada é suportada pela configuração de certificado da aba LiteLLM.
- Comportamento da bandeja (clique, perda de foco, DPI, vários monitores) só se confirma numa sessão
  real do Windows; veja o checklist manual.
- Uma falha ao iniciar o serviço aparece como notificação (etapa e código do erro) e o app sai com
  código 1; sem o WebView2 instalado o app não consegue abrir janela nem avisar, e só escreve no stderr.

## Licença

MIT. Veja [`LICENSE`](LICENSE).

Logotipos das ferramentas monitoradas: marcas de seus respectivos donos, usadas só para
identificá-las, sem afiliação nem endosso; origem, licenças e hashes em
[`docs/third-party-assets.md`](docs/third-party-assets.md).
