// Each source rendered with what its collector really emits (src/mock/emitted.ts): nothing a
// person reads or a screen reader announces may be a wire id, a catalog key or a placeholder.
import { describe, expect, it } from 'vitest';
import type { HarnessId, HarnessView } from '../../lib/contract';
import {
  antigravityEmitted,
  claudeEmitted,
  codexEmitted,
  litellmEmitted,
  opencodeEmitted,
} from '../../mock/emitted';
import { type Env, fact, notice, viewOf } from '../../mock/fixtures';
import { ctxOf, envOf, renderPanel, settingsWith, win } from '../../test/helpers';
import { readable, shownWireWords, standsAlone } from '../../test/screen';
import { rawTokens } from '../../test/vocabulary';
import { HarnessPanel } from '../HarnessPanel';

const OAUTH_ON = settingsWith((draft) => {
  draft.claude.oauthUsage = true;
});

function show(view: HarnessView) {
  renderPanel(<HarnessPanel view={view} ctx={ctxOf(OAUTH_ON)} />, { settings: OAUTH_ON });
  return readable(document.body);
}

interface Case {
  id: HarnessId;
  make: (env: Env) => HarnessView;
  /** What a person should read, in pt-BR. */
  shows: (string | RegExp)[];
}

const CASES: Case[] = [
  {
    id: 'claude',
    make: claudeEmitted,
    shows: [
      'Limite de gasto',
      'US$ 3,20 de US$ 10,00',
      'Semana por modelo',
      'Opus',
      'Sonnet',
      'Claude Design',
      'Também: API de uso, não oficial',
    ],
  },
  {
    id: 'codex',
    make: codexEmitted,
    shows: [
      'Janela de 2 dias',
      'Janela de 14 dias',
      'Créditos adicionais',
      '40 créditos',
      'O codex app-server não respondeu (Programa não encontrado).',
    ],
  },
  {
    id: 'antigravity',
    make: antigravityEmitted,
    shows: ['Grupo 3', 'Cota mensal extra', 'A cota Grupo 3 não retornou dados.'],
  },
  {
    id: 'opencode',
    make: opencodeEmitted,
    shows: [
      'Inclui estimativa local',
      'Custo local, 5 horas',
      'US$ 0,42',
      '184 mil tokens',
      'Não foi possível ler o banco local do OpenCode: o banco está em uso por outro programa.',
      'A janela de 5 horas está em 0%',
    ],
  },
  {
    id: 'litellm',
    make: litellmEmitted,
    shows: [
      'Orçamento: squad-alfa',
      'Orçamento de 1 hora',
      'Orçamento de 30 dias',
      'Orçamento por modelo',
      'Limites da chave',
      /Expira em\n\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}/,
      'Orçamento do time',
      'Não foi possível ler o orçamento do usuário (A leitura demorou demais).',
      'Ficaram de fora as menos consumidas: 3 janelas.',
    ],
  },
];

describe('a source that sends what its collector emits', () => {
  it.each(CASES)('$id reads in words, with no wire id, catalog key or placeholder', (c) => {
    const view = c.make(envOf());
    const text = show(view);
    expect({ raw: rawTokens(text), wire: shownWireWords(view, text) }).toEqual({
      raw: [],
      wire: [],
    });
    expect(text).not.toMatch(/[{}]|Aviso sem tradução/);
    for (const wanted of c.shows) {
      if (typeof wanted === 'string') expect(text).toContain(wanted);
      else expect(text).toMatch(wanted);
    }
  });

  it('keeps the OAuth hint away once the usage API is on', () => {
    expect(show(claudeEmitted(envOf()))).not.toContain('só aparecem pela API de uso da conta');
  });
});

describe('the screen check itself', () => {
  it('flags a fact and a notice nobody labelled', () => {
    const view = viewOf('litellm', {
      windows: [win()],
      facts: [fact('credit_floor', 3, 'requests')],
      notices: [notice('litellm_brand_new_code', 'info')],
    });
    const text = show(view);
    expect(rawTokens(text)).toEqual(
      expect.arrayContaining(['credit_floor', 'litellm_brand_new_code']),
    );
    expect(text).toContain('Aviso sem tradução');
  });

  it('flags a one-word id with no label, which no snake_case rule can see', () => {
    const view = viewOf('litellm', { windows: [win()], facts: [fact('burst', 3, 'requests')] });
    expect(shownWireWords(view, show(view))).toEqual(['burst']);
  });

  it.each([
    ['weekly', 'Semana, weekly-ish e Xweekly', false],
    ['team', 'Orçamento do time (team)', true],
    ['level', 'level: team', true],
    ['limit:1h', 'Orçamento de limit:1h', true],
    ['budget', 'Orçamento: budget_x', false],
  ])('reads %s in "%s" as a stand-alone word: %s', (word, text, expected) => {
    expect(standsAlone(word).test(text)).toBe(expected);
  });
});
