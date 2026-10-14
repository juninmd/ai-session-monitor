// Settings inside the flyout: general options first, then one collapsible group per source.
import { useEffect, useRef, useState } from 'react';
import { useBackend } from '../../app/backendContext';
import { useDraftGuard } from '../../app/draftContext';
import { describeError } from '../../app/errorText';
import { harnessName, type SettingsSection, TAB_ORDER } from '../../app/harness';
import { useSettingsApi } from '../../app/settingsContext';
import { isEnabled } from '../../app/settingsPatch';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { HarnessId } from '../../lib/contract';
import { HarnessLogo } from '../brand/HarnessLogo';
import { Icon } from '../icons';
import { Banner, Note } from '../panel/Banner';
import { ActionError } from '../panel/ErrorActions';
import { GeneralSettings } from './GeneralSettings';
import { SOURCE_GROUPS } from './SourceGroups';
import { UpdateSettings } from './UpdateSettings';

interface SettingsViewProps {
  /** The group an error action pointed at; it opens expanded. */
  section: SettingsSection | null;
  onBack: () => void;
}

/** Sources with no vendor-documented interface and no other badge in the summary. */
const UNOFFICIAL: ReadonlySet<HarnessId> = new Set(['antigravity', 'kimi', 'zai']);

function GroupSummary({ harness, onClick }: { harness: HarnessId; onClick: () => void }) {
  const { settings } = useSettingsApi();
  const on = isEnabled(settings, harness);
  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: native summary provides keyboard activation and disclosure semantics.
    <summary
      onClick={(event) => {
        event.preventDefault();
        onClick();
      }}
    >
      <HarnessLogo id={harness} size={20} />
      <span className="grp-name">{harnessName(harness)}</span>
      {UNOFFICIAL.has(harness) && (
        <span className="chip warnchip">
          <Icon name="warn" />
          {t('settings.unofficial_chip')}
        </span>
      )}
      <span className={on ? 'grp-state on' : 'grp-state'}>
        {on ? t('settings.on') : t('settings.off')}
      </span>
    </summary>
  );
}

function SourceGroup(props: {
  harness: HarnessId;
  open: boolean;
  onToggle: (open: boolean) => void;
}) {
  const { harness, open, onToggle } = props;
  const Body = SOURCE_GROUPS[harness];
  return (
    <details id={`grp-${harness}`} className="grp" open={open}>
      <GroupSummary harness={harness} onClick={() => onToggle(!open)} />
      {/* Mounted only while open: a collapsed group neither polls nor keeps a stale draft. */}
      {open && (
        <div className="grp-body">
          <Body />
        </div>
      )}
    </details>
  );
}

export function SettingsView({ section, onBack }: SettingsViewProps) {
  const guard = useDraftGuard();
  const backend = useBackend();
  const api = useSettingsApi();
  const rootRef = useRef<HTMLDivElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const sourcesRef = useRef<HTMLHeadingElement>(null);
  const help = useAsyncAction(() => backend.openLink('project_readme'));
  const [open, setOpen] = useState<ReadonlySet<HarnessId>>(
    () => new Set(section !== null && section !== 'general' ? [section] : []),
  );

  useEffect(() => {
    headingRef.current?.focus();
  }, []);

  useEffect(() => {
    if (!api.loaded || section === null || section === 'general') return;
    const group = rootRef.current?.querySelector<HTMLElement>(`#grp-${section}`);
    group?.scrollIntoView({ block: 'nearest' });
    group?.querySelector('summary')?.focus();
  }, [api.loaded, section]);

  const toggle = (harness: HarnessId, isOpen: boolean) => {
    const apply = () =>
      setOpen((current) => {
        if (current.has(harness) === isOpen) return current;
        const next = new Set(current);
        if (isOpen) next.add(harness);
        else next.delete(harness);
        return next;
      });
    if (!isOpen && guard) guard.request(apply, harness);
    else apply();
  };

  const failure = api.loadError ? describeError(api.loadError) : null;
  return (
    <div ref={rootRef} className="settings">
      <div className="set-head">
        <button
          type="button"
          className="icon-btn"
          aria-label={t('settings.back')}
          onClick={() => (guard ? guard.request(onBack) : onBack())}
        >
          <Icon name="back" />
        </button>
        <h2 id="settings-title" ref={headingRef} tabIndex={-1}>
          {t('settings.title')}
        </h2>
      </div>
      {failure ? (
        <section className="card notice" data-status="crit">
          <span className="badge">
            <Icon name="warn" />
          </span>
          <h3>{failure.title}</h3>
          <p>{failure.body}</p>
          <div className="actions">
            <button type="button" className="btn primary" onClick={api.reload}>
              {t('action.retry')}
            </button>
          </div>
        </section>
      ) : !api.loaded ? (
        <output className="hint">{t('settings.loading')}</output>
      ) : (
        <fieldset className="settings-controls" disabled={guard?.locked}>
          <GeneralSettings />
          <UpdateSettings />
          <section className="sg" aria-labelledby="sg-sources">
            <h3 id="sg-sources" ref={sourcesRef} tabIndex={-1}>
              {t('settings.sources')}
            </h3>
            {api.secretsError && (
              <Banner tone="warn">
                <p role="alert">{t('settings.secrets_unreadable')}</p>
                <div className="actions">
                  <button
                    type="button"
                    className="btn"
                    onClick={() => {
                      // A successful read removes this banner, and the focused button with it.
                      sourcesRef.current?.focus();
                      api.reload();
                    }}
                  >
                    {t('action.retry')}
                  </button>
                </div>
              </Banner>
            )}
            {TAB_ORDER.map((harness) => (
              <SourceGroup
                key={harness}
                harness={harness}
                open={open.has(harness)}
                onToggle={(isOpen) => toggle(harness, isOpen)}
              />
            ))}
          </section>
          <Note icon="lock">{t('settings.secrets_note')}</Note>
          <div className="actions">
            <button type="button" className="btn" onClick={() => void help.run()}>
              {t('settings.help')}
              <Icon name="external" />
            </button>
          </div>
          {help.error && <ActionError error={help.error} />}
        </fieldset>
      )}
    </div>
  );
}
