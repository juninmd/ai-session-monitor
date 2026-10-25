// Guided first run: pick what to monitor, hand over the credentials, then read once and show what
// each source actually answered. Everything it writes goes through the settings API the current
// settings screen already uses.
import { useMemo, useState } from 'react';
import { useBackend } from '../app/backendContext';
import { useSettingsApi } from '../app/settingsContext';
import { withEnabled } from '../app/settingsPatch';
import { useAsyncAction } from '../app/useAsyncAction';
import type { ViewContext } from '../app/windowModel';
import { Icon } from '../components/icons';
import { ActionError } from '../components/panel/ErrorActions';
import { VendorKey } from '../components/setup/KeyConnect';
import { t, tn } from '../i18n';
import { formatPercent } from '../i18n/format';
import type { HarnessId, HarnessView } from '../lib/contract';
import { Logo } from './Bits';
import {
  candidates,
  keyedOf,
  type OnboardStep,
  outcomeOf,
  pendingOf,
  STEP_ORDER,
  stepLabel,
  withConsent,
} from './onboardingModel';

interface OnboardingProps {
  views: readonly HarnessView[];
  ctx: ViewContext;
  onOpenSettings: (section: HarnessId | 'general' | null) => void;
  onFinish: () => void;
}

export function Onboarding({ views, ctx, onOpenSettings, onFinish }: OnboardingProps) {
  const api = useSettingsApi();
  const backend = useBackend();
  const list = useMemo(() => candidates(views), [views]);
  const byId = useMemo(() => new Map(views.map((view) => [view.id, view])), [views]);
  const [step, setStep] = useState<OnboardStep>('choose');
  // Derived until the user touches it: settings can still be loading on the first render, and a
  // fixed initial value would then freeze a wrong selection for the whole session.
  const [picked, setPicked] = useState<ReadonlySet<HarnessId> | null>(null);
  const ids = useMemo(() => list.map((item) => item.id), [list]);
  const chosen = useMemo(
    () => picked ?? new Set(ids.filter((id) => api.settings[id].enabled)),
    [picked, ids, api.settings],
  );
  const toggle = (id: HarnessId) => {
    const next = new Set(chosen);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setPicked(next);
  };
  const pending = pendingOf([...chosen], views);
  const nameOf = (id: HarnessId) => list.find((item) => item.id === id)?.name ?? id;

  const apply = useAsyncAction(async () => {
    await api.save((current) => {
      let next = current;
      for (const id of ids) next = withEnabled(next, id, chosen.has(id));
      return next;
    });
    setStep(pending.keys.length + pending.consents.length > 0 ? 'connect' : 'verify');
  });
  const read = useAsyncAction(() => backend.refresh());

  const verifyList = ids.filter((id) => chosen.has(id));
  const done = verifyList.filter((id) => {
    const view = byId.get(id);
    return view !== undefined && outcomeOf(view, ctx).tone === 'ok';
  }).length;
  const consent = useAsyncAction((id: HarnessId) =>
    api.save((current) => withConsent(current, id)),
  );
  const openKeys = pending.keys.flatMap((id) => {
    const keyed = keyedOf(id);
    return keyed === null ? [] : [{ id, keyed }];
  });

  return (
    <div className="nx-wizard">
      <ol className="nx-steps">
        {STEP_ORDER.map((name, index) => (
          <li
            key={name}
            data-state={
              name === step ? 'current' : STEP_ORDER.indexOf(step) > index ? 'done' : 'todo'
            }
          >
            <span className="nx-step-n">{index + 1}</span>
            {stepLabel(name)}
          </li>
        ))}
      </ol>

      {step === 'choose' && (
        <section className="nx-section">
          <h3 className="nx-section-head">
            {t('onboarding.choose.title')}
            <span className="nx-count">{chosen.size}</span>
          </h3>
          <p className="nx-meta nx-note-line">{t('onboarding.choose.note')}</p>
          <ul className="nx-pick">
            {list.map((item) => (
              <li key={item.id}>
                <button
                  type="button"
                  className="nx-pick-item"
                  aria-pressed={chosen.has(item.id)}
                  onClick={() => toggle(item.id)}
                >
                  <Logo id={item.id} size={16} />
                  <span className="nx-name">{item.name}</span>
                  <span className="nx-tag" data-detected={item.detected}>
                    {item.detected ? t('onboarding.tag.responds') : t('onboarding.tag.silent')}
                  </span>
                  {item.need !== 'nothing' && (
                    <span className="nx-tag">
                      {item.need === 'key'
                        ? t('onboarding.tag.needs_key')
                        : t('onboarding.tag.needs_consent')}
                    </span>
                  )}
                  <Icon name={chosen.has(item.id) ? 'check' : 'plug'} />
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}

      {step === 'connect' && (
        <section className="nx-section">
          <h3 className="nx-section-head">{t('onboarding.connect.title')}</h3>
          {pending.consents.map((id) => (
            <div className="nx-connect" key={id}>
              <p className="nx-chan-head">
                <Logo id={id} size={16} />
                <strong>{nameOf(id)}</strong>
              </p>
              <p className="nx-meta">{t('onboarding.connect.consent_note')}</p>
              <div className="actions">
                <button
                  type="button"
                  className="btn primary"
                  disabled={consent.pending}
                  onClick={() => void consent.run(id)}
                >
                  {t('onboarding.connect.allow')}
                </button>
                <button type="button" className="btn" onClick={() => onOpenSettings(id)}>
                  {t('onboarding.connect.details')}
                </button>
              </div>
            </div>
          ))}
          {openKeys.map(({ id, keyed }) => (
            <div className="nx-connect" key={id}>
              <VendorKey harness={keyed} />
            </div>
          ))}
          {pending.keys
            .filter((id) => keyedOf(id) === null)
            .map((id) => (
              <div className="nx-connect" key={id}>
                <p className="nx-chan-head">
                  <Logo id={id} size={16} />
                  <strong>{nameOf(id)}</strong>
                </p>
                <p className="nx-meta">
                  {id === 'litellm'
                    ? t('onboarding.connect.litellm_note')
                    : t('onboarding.connect.multi_note')}
                </p>
                <div className="actions">
                  <button type="button" className="btn" onClick={() => onOpenSettings(id)}>
                    {t('onboarding.connect.configure')}
                  </button>
                </div>
              </div>
            ))}
          {pending.keys.length + pending.consents.length === 0 && (
            <p className="hint">{t('onboarding.connect.nothing')}</p>
          )}
        </section>
      )}

      {step === 'verify' && (
        <section className="nx-section">
          <h3 className="nx-section-head">
            {t('onboarding.verify.title')}
            <span className="nx-count">
              {done}/{verifyList.length}
            </span>
          </h3>
          <div className="actions">
            <button
              type="button"
              className="btn primary"
              disabled={read.pending}
              onClick={() => void read.run()}
            >
              <Icon name="refresh" />
              {read.pending ? t('onboarding.verify.reading') : t('onboarding.verify.read_now')}
            </button>
          </div>
          <ul className="nx-chan-list">
            {verifyList.map((id) => {
              const view = byId.get(id);
              const outcome = view === undefined ? null : outcomeOf(view, ctx);
              const first = view?.windows[0] ?? null;
              return (
                <li className="nx-chan" key={id} data-outcome={outcome?.tone ?? 'warn'}>
                  <p className="nx-chan-head">
                    <Logo id={id} size={16} />
                    <strong>{nameOf(id)}</strong>
                    <span className="nx-chan-title">{outcome?.label ?? '—'}</span>
                  </p>
                  {first !== null && (
                    <p className="nx-meta">
                      {tn('onboarding.verify.first', view?.windows.length ?? 0, {
                        pct: formatPercent(first.usedPct),
                      })}
                    </p>
                  )}
                  {view?.state === 'needs_setup' && keyedOf(id) !== null && (
                    <div className="actions">
                      <button type="button" className="btn" onClick={() => setStep('connect')}>
                        {t('onboarding.verify.back_to_key')}
                      </button>
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        </section>
      )}

      {step === 'done' && (
        <section className="nx-section">
          <h3 className="nx-section-head">
            <Icon name="check" />
            {t('onboarding.done.title', { done, total: verifyList.length })}
          </h3>
          <p>{t('onboarding.done.body')}</p>
          <div className="actions">
            <button type="button" className="btn primary" onClick={onFinish}>
              {t('onboarding.done.go')}
            </button>
            <button type="button" className="btn" onClick={() => setStep('verify')}>
              {t('onboarding.done.again')}
            </button>
          </div>
        </section>
      )}

      {apply.error && <ActionError error={apply.error} />}
      {consent.error && <ActionError error={consent.error} />}
      <div className="nx-wizard-actions">
        {step !== 'choose' && step !== 'done' && (
          <button
            type="button"
            className="btn"
            onClick={() => setStep(STEP_ORDER[STEP_ORDER.indexOf(step) - 1] ?? 'choose')}
          >
            <Icon name="back" />
            {t('onboarding.back')}
          </button>
        )}
        {step === 'choose' && (
          <button
            type="button"
            className="btn primary"
            disabled={apply.pending || chosen.size === 0}
            onClick={() => void apply.run()}
          >
            {t('onboarding.continue', { count: chosen.size })}
          </button>
        )}
        {step === 'connect' && (
          <button type="button" className="btn primary" onClick={() => setStep('verify')}>
            {t('onboarding.connected')}
          </button>
        )}
        {step === 'verify' && (
          <button type="button" className="btn primary" onClick={() => setStep('done')}>
            {t('onboarding.finish')}
          </button>
        )}
      </div>
    </div>
  );
}
