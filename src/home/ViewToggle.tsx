// The switch between the two ways to read the triage: cards (two per row) or one provider per line.
import type { ViewMode } from '../app/hooks/useViewMode';
import { Icon } from '../components/icons';
import { t } from '../i18n';

const MODES: readonly { mode: ViewMode; icon: 'cards' | 'list'; label: 'cards' | 'list' }[] = [
  { mode: 'list', icon: 'list', label: 'list' },
  { mode: 'cards', icon: 'cards', label: 'cards' },
];

export function ViewToggle({
  mode,
  onChange,
}: {
  mode: ViewMode;
  onChange: (mode: ViewMode) => void;
}) {
  return (
    <fieldset className="nx-view-toggle" aria-label={t('triage.view.label')}>
      {MODES.map((item) => (
        <button
          key={item.mode}
          type="button"
          aria-pressed={mode === item.mode}
          aria-label={t(`triage.view.${item.label}`)}
          title={t(`triage.view.${item.label}`)}
          onClick={() => onChange(item.mode)}
        >
          <Icon name={item.icon} />
        </button>
      ))}
    </fieldset>
  );
}
