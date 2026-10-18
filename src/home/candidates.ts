// Tools with an official usage channel and a credential the app could accept as a new source.
// Candidates, not commitments: each one would need its own collector, card and data-sources entry.

export interface Candidate {
  id: string;
  name: string;
  credential: string;
  offer: string;
  confidence: 'Alta' | 'Média' | 'Baixa';
}

export const CANDIDATES: readonly Candidate[] = [
  {
    id: 'amp',
    name: 'Amp (Sourcegraph)',
    credential: 'CLI local (`amp`)',
    offer:
      '`amp usage` devolve saldo de créditos e uso incluído. Mesmo padrão de CLI fixo já usado por Claude e Antigravity.',
    confidence: 'Alta',
  },
  {
    id: 'copilot',
    name: 'GitHub Copilot',
    credential: 'PAT com escopo de leitura de plano',
    offer:
      'Requisições premium, chat e completions, com plano e data de reset; faturamento de créditos de IA pela API REST de billing.',
    confidence: 'Média',
  },
  {
    id: 'kiro',
    name: 'Kiro CLI',
    credential: 'Credencial do próprio CLI',
    offer:
      'GetUsageLimits devolve um único pool de créditos com usado, limite, percentual e reset.',
    confidence: 'Média',
  },
  {
    id: 'cline',
    name: 'Cline',
    credential: 'Chave de API',
    offer:
      '`/users/me/plan/usage-limits`: janelas de 5 h, semanal e mensal com percentual e resetsAt.',
    confidence: 'Média',
  },
  {
    id: 'windsurf',
    name: 'Windsurf / Devin',
    credential: 'Service key da equipe',
    offer: 'GetTeamCreditBalance e GetUsageConfig pela API oficial. Só no plano Enterprise.',
    confidence: 'Alta',
  },
  {
    id: 'command-code',
    name: 'Command Code',
    credential: 'CLI local (`commandcode`)',
    offer:
      '`/alpha/billing/credits` e `/subscriptions`: janelas de gasto de 5 h e semanais, plano e créditos mensais.',
    confidence: 'Média',
  },
];
