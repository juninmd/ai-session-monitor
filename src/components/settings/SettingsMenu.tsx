// The settings menu: one row per section with its current state, then the secrets note and help.
import { useContext } from 'react';
import { useBackend } from '../../app/backendContext';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { UpdateContext } from '../../app/useUpdateState';
import { t } from '../../i18n';
import { Icon } from '../icons';
import { Note } from '../panel/Banner';
import { ActionError } from '../panel/ErrorActions';
import { LEVEL_TITLE, type SettingsLevel } from './levels';
import { historySummary, monitoringSummary, sourcesSummary, updateSummary } from './summaries';

const ROWS = [
  'general',
  'history',
  'update',
  'sources',
] as const satisfies readonly SettingsLevel[];

/** Every row a section can be opened from, in the order the screen reads them. */
export function SettingsMenu({ onOpen }: { onOpen: (level: SettingsLevel) => void }) {
  const api = useSettingsApi();
  const backend = useBackend();
  const update = useContext(UpdateContext);
  const help = useAsyncAction(() => backend.openLink('project_readme'));
  const summary: Record<(typeof ROWS)[number], string> = {
    general: monitoringSummary(api.settings),
    history: historySummary(api.settings),
    update: updateSummary(api.settings.general.autoUpdateCheck, update?.status ?? null),
    sources: sourcesSummary(),
  };
  return (
    <div className="set-menu">
      <ul className="set-rows">
        {ROWS.map((level) => (
          <li key={level}>
            <button type="button" className="set-row" onClick={() => onOpen(level)}>
              <span className="set-row-name">{t(LEVEL_TITLE[level])}</span>
              <span className="set-row-sum">{summary[level]}</span>
              <Icon name="next" />
            </button>
          </li>
        ))}
      </ul>
      <Note icon="lock">{t('settings.secrets_note')}</Note>
      <div className="actions">
        <button type="button" className="btn" onClick={() => void help.run()}>
          {t('settings.help')}
          <Icon name="external" />
        </button>
      </div>
      {help.error && <ActionError error={help.error} />}
    </div>
  );
}
