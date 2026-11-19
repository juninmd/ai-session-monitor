# Benchmark competitivo do AI Session Monitor

Pesquisa documental em 18 de novembro de 2026. A base local é `0a2c619`. O recorte inclui monitores públicos descobertos nas buscas por cotas de Claude Code, Codex, Cursor e outros agentes, com prioridade para bandeja e widgets Windows. Projetos macOS e CLI entram quando oferecem uma referência funcional relevante. A unidade de comparação é o recurso documentado. Quantidades de catálogo, integrações ativas, cotas oficiais e leitores de logs são medidas diferentes. O levantamento cobre os projetos abaixo e pode ser ampliado conforme surgirem outros.

## Comparação

| Produto | Plataforma ou formato | Recurso que serve de referência |
| --- | --- | --- |
| AI Session Monitor | Bandeja Windows | 16 coletores/configurações incluindo LiteLLM, triagem, horizonte, histórico local de até 35 dias, CSV, 10 idiomas, proteção de captura e API loopback opcional |
| [Win-CodexBar](https://github.com/nesszer/Win-CodexBar) | Bandeja Windows | Catálogo amplo e distribuição pelo winget |
| [CodexBar-Windows](https://github.com/joemwong/CodexBar-Windows) | Bandeja Windows | Catálogo anunciado de 56 provedores e anel de cota |
| [CodexBar-Win](https://github.com/babakarto/CodexBar-Win) | Bandeja Windows | Cotas de Claude e Codex acompanhadas de custos |
| [Rate Tray](https://github.com/nowrap/rate-tray) | Bandeja Windows | Fluxo focado em Claude e Codex e consulta de atualização opt-in |
| [AIUsage.NET](https://github.com/LuigiElleBalotta/AIUsage.NET) | Bandeja Windows | Cotas e gastos no mesmo aplicativo |
| [ai-usagebar-win](https://github.com/FranzoiDev/ai-usagebar-win) | Bandeja Windows | Cotas de planos e créditos de API |
| [Codex Minibar](https://github.com/vertopolkaLF/codex-minibar) | Bandeja Windows | Contagem regressiva e notificações de cota |
| [Quota Usage Monitor](https://github.com/rozsazoltan/quota-usage-monitor) | Widget Windows | Cotas e renovações visíveis na barra de tarefas |
| [Codex Usage](https://github.com/upstream-ray/codex-usage-monitor) | Widget Windows | Cota persistente na barra de tarefas com alertas |
| [PulseBar](https://github.com/Hitbee-dev/PulseBar) | Widget Windows | Telemetria local de tokens e métricas do sistema ao lado das cotas |
| [AI Usage Overlays](https://github.com/CosmonautJones/ai-usage-overlays) | HUD Windows | Overlay persistente com tendências resumidas |
| [Waifu Usage Monitor](https://github.com/leancoderkavy/waifu-usage-monitor) | Windows e macOS | Múltiplas contas, calendário de renovações e overlay |
| [AIUsageTracker](https://github.com/SysAdminDoc/AIUsageTracker) | Windows | Alarmes por janela, modo compacto e mapa de calor |
| [AI Usage Tracker](https://github.com/medenmite/ai-usage-tracker) | Windows | Histórico, ritmo recente e atividade local de Claude/Codex |
| [AIUsageMonitor](https://github.com/SethRobinson/AIUsageMonitor) | Bandeja Windows | Contas lado a lado e troca de conta Claude |
| [AiUsageMonitor](https://github.com/marvasgit/AiUsageMonitor) | Windows | Cartões por conta com contagem regressiva |
| [AI Quota HUD](https://github.com/Pakapong26/ai-quota-hud) | Windows | Múltiplas contas, alertas e identificação de renovações antecipadas |
| [Codex Claude Usage](https://github.com/Kyuhan1230/ai-usage-monitor) | Bandeja Windows | Faixa de previsão, confiança e detecção de picos |
| [Codex Usage de Victor Zakharov](https://github.com/VictorZakharov/codex-usage) | Bandeja Windows | Histórico de até 90 dias com projeções e marcas de renovação |
| [AI Usage Bar](https://github.com/kutsaltotem/codex-usage-monitor/blob/main/README.en.md) | Windows | Tendência filtrada por provedor |
| [AI Usage Clock](https://github.com/AIUsageClock/ai-usage-clock/blob/main/CHANGELOG.md) | Desktop | Custo equivalente de API e comparação entre ciclos |
| [CodexBar](https://github.com/steipete/CodexBar) | macOS e CLI | Alcance de integrações e fontes alternativas |
| [OpenUsage](https://www.openusage.ai/) | Barra de menus macOS | 12 provedores anunciados, extensão por plugins e ordenação por esgotamento |
| [TokenBar de lwh0630](https://github.com/lwh0630/TokenBar) | Desktop e móvel | Catálogo anunciado de 50 provedores e janelas por conta |
| [TokenBar de Nanako](https://tokenbar.nyanako.com/) | Barra de menus macOS | Leituras de logs de mais de 25 agentes e análises por modelo |
| [TokenBar de wuruoye](https://github.com/wuruoye/TokenBar) | Barra de menus macOS | Custos, sessões e atividade entre dispositivos |
| [TokenBar de Arnie016](https://github.com/Arnie016/TokenBar) | Barra de menus macOS | Monitoramento de cotas e previsão de custo |
| [QuotaBar](https://github.com/productinfo/quotabar) | Barra de menus macOS | Ocultação de dados durante compartilhamento de tela |
| [AI Usage Tracker de Danielw412](https://github.com/Danielw412/AI-usage-tracker) | Dashboard web | Atribuição do consumo por sessão e por dispositivo |
| [ccusage](https://github.com/ccusage/ccusage) | CLI | Relatórios de tokens e custo a partir de logs locais |

## Resultado implementado

O Horizonte usa todas as janelas utilizáveis da fonte para escolher a cota esgotada ou com esgotamento projetado mais próximo. Ele usa o percentual restante como critério de seleção quando nenhuma janela indica interrupção, e mostra a próxima renovação conhecida mesmo quando ela pertence a uma janela diferente. A priorização mantém as mesmas regras de ritmo e status do projeto. Cotas expiradas e leituras antigas ficam fora da projeção. Uma cota esgotada com renovação desconhecida aparece como esgotada, sem duração estimada.

O calendário no Horizonte ordena as renovações futuras por horário e identifica o provedor e a cota de cada evento. Mostra horários reportados por fontes atuais mesmo quando o percentual de uso é desconhecido. O calendário usa o snapshot que já está na interface e funciona com o histórico desligado.

O Histórico compara o último ponto armazenado com a mediana estimada dos ciclos anteriores no mesmo estágio. A estimativa interpola entre duas leituras e exclui ciclos que não cobrem aquele ponto. A mediana reduz o peso de um ciclo atípico.

O Histórico também apresenta cenários de esgotamento quando o ritmo recente indica risco. A estimativa usa duas taxas observadas na última hora do mesmo ciclo, com pelo menos cinco minutos em cada intervalo, leitura final de até 15 minutos e concordância com a cota atual. Se a taxa mais lenta chegar à renovação, o texto indica essa possibilidade. As [capturas no Chromium](evidence/quota-forecast/README.md) mostram os cenários de calendário e de aceleração.

| Cenário reproduzível com dados sintéticos | Base `0a2c619` | Candidato |
| --- | --- | --- |
| Janela semanal com 5% e renovação em 30 min. Janela de cinco horas com 10%, renovação em 60 min e esgotamento projetado em 26,7 min. | Exibe a semanal como suficiente e oculta o risco da janela de cinco horas. | Exibe a janela de cinco horas como risco e informa a renovação semanal em 30 min. |
| Dois provedores em risco. Um esgota em 26,7 min, outro em 45 min, e o segundo tem uma interrupção mais longa. | Mostra primeiro o que tem a interrupção mais longa. | Mostra primeiro o que esgota em 26,7 min. |
| Cota reportada como esgotada sem horário de renovação. | Classifica a janela como desconhecida. | Sinaliza esgotada e mantém duração e renovação desconhecidas. |
| Histórico desativado nas configurações. | O controle de tendências aceita cliques sem exibir séries. | O controle fica desativado e o Horizonte não consulta o histórico. |
| Duas renovações futuras no mesmo provedor e uma leitura antiga em outro. | A projeção mostra uma janela por provedor. | O calendário lista as duas renovações ativas em ordem cronológica e omite a leitura antiga. |
| Última leitura em 62,3% do tempo do ciclo e com 90% da cota consumida. Os três ciclos anteriores encerraram com 40%, 50% e 100% de consumo. | Compara o pico atual com a média final de 63%. | Compara 90% com a mediana estimada de 31% no mesmo estágio, usando séries sintéticas lineares. |

Reprodução dos cenários de cota e histórico: `pnpm exec vitest run src/home/horizonModel.test.ts src/home/home.horizon.test.tsx src/home/historyModel.test.ts src/i18n/locales.test.ts`.

## Lacunas com evidência

1. **Cobertura e contas.** Os catálogos Windows acima são maiores e alguns monitores exibem contas lado a lado. A comparação de cobertura deve usar uma lista de provedores com coleta verificada e distinguir conta, plano e pool.
2. **Previsão.** O Horizonte mostra uma estimativa pontual pelo ritmo médio da janela. O Histórico agora exibe cenários pelo ritmo recente. A confiança estatística e a detecção de picos seguem como lacunas frente aos monitores especializados. A comparação exige reproduzir as mesmas séries de consumo e medir erro e antecedência do aviso.
3. **Custos e consumo.** O histórico atual guarda percentuais de cota. Produtos com ingestão de logs mostram tokens, custo equivalente e atribuição. Esses valores precisam ser comparados separadamente da fatura real e da cota da assinatura.
4. **Acesso e desempenho.** Widgets e overlays oferecem leitura persistente. Uma comparação de desempenho exige as versões instaladas, o mesmo hardware Windows, a mesma cadência de consulta e cargas equivalentes. Medir inicialização fria, fluidez ao abrir a bandeja e CPU/memória de toda a árvore de processos em repouso. O [benchmark de arquitetura](architecture-review.md) mede duas versões do próprio frontend.

As funcionalidades dos concorrentes foram conferidas nas fontes primárias vinculadas. A comparação numérica acima usa o modelo local e dados fabricados. Ficam pendentes medições lado a lado de desempenho, coleta com contas reais e aceitação visual no Windows. O resultado demonstra a correção nas tarefas da tabela e identifica onde a liderança geral ainda precisa ser demonstrada.
