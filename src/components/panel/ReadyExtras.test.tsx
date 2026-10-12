import { screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BackendError } from '../../lib/backend';
import type { Fact, HarnessView, Settings } from '../../lib/contract';
import {
  antigravityView,
  claudeView,
  codexView,
  litellmView,
  opencodeView,
} from '../../mock/builders';
import { asStale, fact, problem, viewOf } from '../../mock/fixtures';
import { ctxOf, envOf, NOW, renderPanel, settingsWith, win } from '../../test/helpers';
import { HarnessPanel } from '../HarnessPanel';

function ready(view: HarnessView, settings?: Settings) {
  const options = settings ? { settings } : {};
  return renderPanel(<HarnessPanel view={view} ctx={ctxOf(settings)} />, options);
}

const env = envOf();
const plain = (text: string | null | undefined) => (text ?? '').replace(/\s/g, ' ');
const factValue = (label: string) =>
  plain(screen.getByText(label).closest('div')?.querySelector('dd')?.textContent);

describe('facts', () => {
  it('list a key limits with units', () => {
    ready(litellmView(env, 'ok'));
    expect(screen.getByRole('heading', { name: 'Limites da chave' })).toBeVisible();
    expect(factValue('Requisições por minuto')).toBe('60 req.');
    expect(factValue('Tokens por minuto')).toBe('200 mil tokens');
  });

  it('label the OpenCode local estimate and keep it apart from the quota', () => {
    ready(opencodeView(env, 'ok'));
    expect(screen.getByRole('heading', { name: 'Estimativa local' })).toBeVisible();
    expect(factValue('Custo local, 5 horas')).toBe('US$ 0,42');
    expect(factValue('Custo local, 7 dias')).toBe('US$ 3,18');
    expect(factValue('Tokens locais, 5 horas')).toBe('184 mil tokens');
  });

  it('show Codex credits', () => {
    ready(codexView(env, 'ok'));
    expect(factValue('Créditos adicionais')).toBe('120 créditos');
  });

  it('show an unlabelled fact as its id instead of dropping it', () => {
    ready(viewOf('litellm', { windows: [win()], facts: [fact('weird_thing', 3, 'x')] }));
    expect(factValue('weird_thing')).toBe('3 x');
  });

  it('draw nothing without facts', () => {
    ready(viewOf('codex', { windows: [win()] }));
    expect(screen.queryByRole('definition')).toBeNull();
  });

  describe('of a key that expires', () => {
    const DATE = /^\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}$/;
    const litellm = (...facts: Fact[]) => viewOf('litellm', { windows: [win()], facts });

    it('say when, as a date and never as a raw epoch', () => {
      ready(litellm(fact('expires', NOW + 30 * 86_400, 'epoch_s')));
      expect(DATE.test(factValue('Expira em'))).toBe(true);
      expect(document.body).not.toHaveTextContent(/epoch_s|expires|1\.8\d\d\.\d{3}\.\d{3}/);
    });

    it('say it in the past tense once the date has passed', () => {
      ready(litellm(fact('expires', NOW - 3600, 'epoch_s')));
      expect(DATE.test(factValue('Expirou em'))).toBe(true);
      expect(screen.queryByText('Expira em')).toBeNull();
    });

    it('keep an expiry the source did not give unknown', () => {
      ready(litellm({ id: 'expires', number: null, text: null, unit: 'epoch_s' }));
      expect(factValue('Expira em')).toBe('—');
    });
  });

  it.each([
    ['key', 'Limites da chave'],
    ['user', 'Limites do usuário'],
    ['team', 'Limites do time'],
  ])('title the LiteLLM limits after the level they belong to (%s)', (scope, title) => {
    const facts = [fact('rpm_limit', 60, 'requests')];
    ready(viewOf('litellm', { windows: [win()], facts, meta: { scope } }));
    expect(screen.getByRole('heading', { name: title })).toBeVisible();
  });

  it('title them as a key when the source did not say whose they are', () => {
    ready(viewOf('litellm', { windows: [win()], facts: [fact('rpm_limit', 60, 'requests')] }));
    expect(screen.getByRole('heading', { name: 'Limites da chave' })).toBeVisible();
  });
});

describe('old numbers', () => {
  const stale = (error?: Parameters<typeof asStale>[3]) =>
    asStale(codexView(env, 'warn'), env, 12, error);

  it('stay readable, dimmed as a block, with their age', () => {
    ready(stale());
    expect(document.querySelector('.stack')).toHaveAttribute('data-stale', 'true');
    expect(screen.getByText('Mostrando os últimos números lidos, de 12 min atrás.')).toBeVisible();
    expect(screen.getAllByRole('meter').length).toBeGreaterThan(0);
  });

  it('stop extrapolating a pace', () => {
    ready(stale());
    expect(screen.queryByText(/No ritmo atual/)).toBeNull();
    expect(screen.queryByText(/Esgotado\. Volta/)).toBeNull();
  });

  it('say why the refresh failed and offer exactly one way forward', async () => {
    const { user, backend } = ready(stale(problem('network', {}, true)));
    expect(screen.getByText('A última atualização falhou: Sem conexão com a fonte.')).toBeVisible();
    const action = screen.getByRole('button', { name: 'Tentar de novo' });
    expect(within(action.closest('.banner') as HTMLElement).getAllByRole('button')).toHaveLength(1);
    await user.click(action);
    expect(backend.calls).toContain('refresh');
  });

  it('send an authentication failure to the settings', async () => {
    const view = asStale(
      litellmView(env, 'ok'),
      env,
      18,
      problem('unauthorized', { httpStatus: '401' }),
    );
    const { user, openSettings } = ready(view);
    await user.click(screen.getByRole('button', { name: 'Abrir configurações' }));
    expect(openSettings).toHaveBeenCalledExactlyOnceWith('litellm');
  });

  it('do not invent an age', () => {
    ready({ ...stale(), readAt: null });
    expect(screen.getByText('Mostrando os últimos números lidos.')).toBeVisible();
  });

  it('keep a failed refresh visible even when the data is not old yet', () => {
    ready({ ...codexView(env, 'ok'), error: problem('timeout', {}, true) });
    expect(
      screen.getByText('A última atualização falhou: A leitura demorou demais.'),
    ).toBeVisible();
    expect(screen.queryByText(/Mostrando os últimos números/)).toBeNull();
    expect(document.querySelector('.stack')).not.toHaveAttribute('data-stale');
  });

  it('show a fresh view without any banner', () => {
    ready(codexView(env, 'ok'));
    expect(document.querySelector('.banner')).toBeNull();
  });
});

describe('closing notes per source', () => {
  it('point OpenCode users at the Zen billing page, not the Go console', async () => {
    const { user, backend } = ready(opencodeView(env, 'ok'));
    expect(screen.getByText(/O saldo do Zen \(pré-pago\) não tem API/)).toBeVisible();
    expect(screen.queryByRole('button', { name: /console do OpenCode/ })).toBeNull();
    await user.click(screen.getByRole('button', { name: /Ver cobrança do Zen/ }));
    expect(backend.openedLinks).toEqual(['opencode_billing']);
  });

  it('remind Antigravity users to keep the IDE open', () => {
    ready(antigravityView(env, 'ok'));
    expect(screen.getByText(/Mantenha o Antigravity aberto/)).toBeVisible();
  });

  it('let LiteLLM users edit the connection in place', async () => {
    const { user } = ready(
      litellmView(env, 'ok'),
      settingsWith(() => {}),
    );
    const edit = screen.getByRole('button', { name: 'Editar conexão' });
    expect(edit).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByLabelText('URL do servidor')).toBeNull();
    await user.click(edit);
    expect(edit).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByLabelText('URL do servidor')).toHaveValue('https://gateway.exemplo.dev');
    await user.click(edit);
    expect(screen.queryByLabelText('URL do servidor')).toBeNull();
  });

  it('add nothing for Codex', () => {
    ready(codexView(env, 'ok'));
    expect(screen.queryByRole('button', { name: /Editar|cobrança|console/ })).toBeNull();
  });
});

describe('badges', () => {
  it('say where the numbers came from and whether the source is official', () => {
    ready(claudeView(env, 'ok'));
    expect(screen.getByText('Max 5×')).toBeVisible();
    expect(screen.getByText('Lido há 4 min')).toBeVisible();
    expect(screen.getByText('Leitura oficial · barra de status')).toHaveAttribute(
      'title',
      'Interface documentada pelo fornecedor.',
    );
  });

  it('warn about an unofficial source', () => {
    ready(antigravityView(env, 'ok'));
    const chip = screen.getByText('Leitura não oficial · servidor local da IDE');
    expect(chip).toHaveAttribute(
      'title',
      expect.stringContaining('pode mudar ou parar de funcionar'),
    );
    expect(chip.closest('.warnchip')).not.toBeNull();
  });

  it('show a source id nobody named yet as it came', () => {
    ready(viewOf('codex', { windows: [win()], source: { id: 'brand_new', official: true } }));
    expect(screen.getByText('Leitura oficial · brand_new')).toBeVisible();
  });

  it('name the LiteLLM key, host and scope', () => {
    ready(litellmView(env, 'ok'));
    expect(screen.getByText('Esta chave: dev-pessoal')).toBeVisible();
    expect(screen.getByText('gateway.exemplo.dev')).toBeVisible();
  });

  it('cap the extra metadata so a chatty source cannot flood the header', () => {
    const meta = { a: '1', b: '2', c: '3', d: '4', e: '5' };
    ready(viewOf('codex', { windows: [win()], meta }));
    expect(screen.getByText('c: 3')).toBeVisible();
    expect(screen.queryByText('d: 4')).toBeNull();
  });

  it('flag a connection that skips certificate checks', () => {
    const insecure = settingsWith((s) => {
      s.litellm.tls = { mode: 'insecure', host: 'gateway.exemplo.dev' };
    });
    ready(litellmView(envOf({ settings: insecure }), 'ok'), insecure);
    expect(screen.getByText('Certificado não verificado')).toBeVisible();
  });

  it('flag the unofficial Claude usage API when it is on', () => {
    const on = settingsWith((s) => {
      s.claude.oauthUsage = true;
    });
    ready(claudeView(envOf({ settings: on }), 'ok'), on);
    expect(screen.getByText('Também: API de uso, não oficial')).toBeVisible();
  });

  it('show no age chip before anything was read', () => {
    ready(viewOf('codex', { windows: [win()] }));
    expect(screen.queryByText(/^Lido há/)).toBeNull();
  });
});

describe('a failed refresh action', () => {
  it('names the problem next to the button', async () => {
    const view = asStale(codexView(env, 'ok'), env, 5, problem('network', {}, true));
    const { user, backend } = ready(view);
    vi.spyOn(backend, 'refresh').mockRejectedValue(new BackendError(problem('timeout', {}, true)));
    await user.click(screen.getByRole('button', { name: 'Tentar de novo' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('A leitura demorou demais');
  });
});
