// First-run consent surfaces. Nothing is read until the user agrees; "Agora não" switches the
// source off, which the tab then shows as "Desligado" with a one-click way back.
import type { ReactNode } from 'react';
import { useBackend } from '../../app/backendContext';
import { useSettingsApi } from '../../app/settingsContext';
import { withEnabled } from '../../app/settingsPatch';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { HarnessId } from '../../lib/contract';
import { Banner } from '../panel/Banner';
import { ActionError } from '../panel/ErrorActions';
import { InfoCard } from '../panel/StateCards';
import { focusTab } from '../shell/TabStrip';

interface ConsentCardProps {
  harness: HarnessId;
  title: string;
  lead: string;
  detail: string;
  allowLabel: string;
  onAllow: () => Promise<unknown>;
  children?: ReactNode;
}

function ConsentCard(props: ConsentCardProps) {
  const { harness, title, lead, detail, allowLabel, onAllow, children } = props;
  const { save } = useSettingsApi();
  const allow = useAsyncAction(onAllow);
  const skip = useAsyncAction(() => save((s) => withEnabled(s, harness, false)));
  const failure = allow.error ?? skip.error;
  // Either action replaces this card, and a disabled or removed button drops the keyboard on
  // <body>: park it on the tab first, which stays mounted.
  const keepOnTab = (run: () => Promise<boolean>) => () => {
    focusTab(harness);
    void run();
  };
  return (
    <>
      <InfoCard icon="lock" logo={harness} title={title}>
        <p>{lead}</p>
        <details>
          <summary>{t('consent.details')}</summary>
          <p>{detail}</p>
        </details>
        <div className="actions">
          <button
            type="button"
            className="btn primary"
            disabled={allow.pending}
            onClick={keepOnTab(allow.run)}
          >
            {allowLabel}
          </button>
          <button type="button" className="btn" onClick={keepOnTab(skip.run)}>
            {t('consent.skip')}
          </button>
        </div>
        {failure && <ActionError error={failure} harness={harness} />}
      </InfoCard>
      {children}
    </>
  );
}

export function ClaudeConnect() {
  const backend = useBackend();
  const { installShim } = useSettingsApi();
  return (
    <ConsentCard
      harness="claude"
      title={t('claude.connect.title')}
      lead={t('claude.connect.lead')}
      detail={t('claude.connect.detail')}
      allowLabel={t('claude.connect.allow')}
      onAllow={async () => {
        await installShim();
        await backend.refresh('claude');
      }}
    >
      <Banner tone="info">
        <p>{t('claude.connect.tip')}</p>
      </Banner>
    </ConsentCard>
  );
}

export function CodexConnect() {
  const backend = useBackend();
  const { save } = useSettingsApi();
  return (
    <ConsentCard
      harness="codex"
      title={t('codex.connect.title')}
      lead={t('codex.connect.lead')}
      detail={t('codex.connect.detail')}
      allowLabel={t('codex.connect.allow')}
      onAllow={async () => {
        await save((s) => ({ ...s, codex: { ...s.codex, consent: true } }));
        await backend.refresh('codex');
      }}
    />
  );
}

export function AntigravityConnect() {
  const backend = useBackend();
  const { save } = useSettingsApi();
  return (
    <ConsentCard
      harness="antigravity"
      title={t('antigravity.connect.title')}
      lead={t('antigravity.connect.lead')}
      detail={t('antigravity.connect.detail')}
      allowLabel={t('antigravity.connect.allow')}
      onAllow={async () => {
        await save((s) => ({ ...s, antigravity: { ...s.antigravity, consent: true } }));
        await backend.refresh('antigravity');
      }}
    />
  );
}
