// Settings inside the flyout, in two levels: a short menu of sections, and one section at a time in
// the body. A deep link (an error action's button, the provider drawer) skips straight to a section.
import { useEffect, useRef, useState } from 'react';
import { useDraftGuard } from '../../app/draftContext';
import { describeError } from '../../app/errorText';
import type { SettingsSection } from '../../app/harness';
import { SETTINGS_TITLE_ID } from '../../app/homeIds';
import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import type { HarnessId } from '../../lib/contract';
import { Icon } from '../icons';
import { GeneralSettings } from './GeneralSettings';
import { HistorySettings } from './HistorySettings';
import { LEVEL_TITLE, levelOf, type SettingsLevel, sourceOf } from './levels';
import { SettingsMenu } from './SettingsMenu';
import { SourcesSection } from './SourcesSection';
import { UpdateSettings } from './UpdateSettings';

interface SettingsViewProps {
  /** The section an error action or the gear pointed at; the menu opens when it is null. */
  section: SettingsSection | null;
  onBack: () => void;
}

export function SettingsView({ section, onBack }: SettingsViewProps) {
  const api = useSettingsApi();
  const guard = useDraftGuard();
  const rootRef = useRef<HTMLDivElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const [level, setLevel] = useState<SettingsLevel>(() => levelOf(section));
  const [open, setOpen] = useState<ReadonlySet<HarnessId>>(() => {
    const source = sourceOf(section);
    return new Set(source === null ? [] : [source]);
  });
  // The source whose group should take the focus: the one a deep link named, until the user moves
  // the screen themselves. Opening a section from the menu focuses its heading instead.
  const [focusSource, setFocusSource] = useState<HarnessId | null>(() => sourceOf(section));
  const applied = useRef(section);

  // Where the level's focus goes: the deep-linked group, else the level's own heading. The effect is
  // idempotent, so StrictMode's second pass in development lands in the same place.
  useEffect(() => {
    const group =
      api.loaded && level === 'sources' && focusSource !== null
        ? rootRef.current?.querySelector<HTMLElement>(`#grp-${focusSource}`)
        : null;
    if (group) {
      group.scrollIntoView({ block: 'nearest' });
      group.querySelector('summary')?.focus();
      return;
    }
    headingRef.current?.focus();
  }, [api.loaded, focusSource, level]);

  // A new section arriving from outside steers the screen, even while settings are already open.
  useEffect(() => {
    if (applied.current === section) return;
    applied.current = section;
    const source = sourceOf(section);
    setLevel(levelOf(section));
    setFocusSource(source);
    if (source !== null) setOpen((current) => new Set([...current, source]));
  }, [section]);

  const openSection = (next: SettingsLevel) => {
    setFocusSource(null);
    setLevel(next);
  };

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

  // The arrow leaves the section it is in, and leaving the screen is guarded like any form.
  const back = () => {
    const leave = level === 'menu' ? onBack : () => setLevel('menu');
    if (guard) guard.request(leave);
    else leave();
  };
  const failure = api.loadError ? describeError(api.loadError) : null;
  return (
    <div ref={rootRef} className="settings">
      <div className="set-head">
        <button type="button" className="icon-btn" aria-label={t('settings.back')} onClick={back}>
          <Icon name="back" />
        </button>
        <h2 id={SETTINGS_TITLE_ID} ref={headingRef} tabIndex={-1}>
          {t(LEVEL_TITLE[level])}
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
          {level === 'menu' && <SettingsMenu onOpen={openSection} />}
          {level === 'general' && <GeneralSettings />}
          {level === 'history' && <HistorySettings enabled={api.settings.general.historyEnabled} />}
          {level === 'update' && <UpdateSettings />}
          {level === 'sources' && (
            <SourcesSection
              open={open}
              onToggle={toggle}
              focusHeading={() => headingRef.current?.focus()}
            />
          )}
        </fieldset>
      )}
    </div>
  );
}
