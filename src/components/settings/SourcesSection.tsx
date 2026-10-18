// The Fontes de dados section: the secrets banner and one collapsible group per source.
import { harnessName, TAB_ORDER } from '../../app/harness';
import { useSettingsApi } from '../../app/settingsContext';
import { isEnabled } from '../../app/settingsPatch';
import { t } from '../../i18n';
import type { HarnessId } from '../../lib/contract';
import { HarnessLogo } from '../brand/HarnessLogo';
import { Icon } from '../icons';
import { Banner } from '../panel/Banner';
import { SOURCE_GROUPS } from './SourceGroups';
import { UNOFFICIAL } from './summaries';

interface SourcesSectionProps {
  open: ReadonlySet<HarnessId>;
  onToggle: (harness: HarnessId, open: boolean) => void;
  /** Moves focus back to the screen heading when a retry removes the banner's button. */
  focusHeading: () => void;
}

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

export function SourcesSection({ open, onToggle, focusHeading }: SourcesSectionProps) {
  const api = useSettingsApi();
  return (
    <div className="sg">
      {api.secretsError && (
        <Banner tone="warn">
          <p role="alert">{t('settings.secrets_unreadable')}</p>
          <div className="actions">
            <button
              type="button"
              className="btn"
              onClick={() => {
                // A successful read removes this banner, and the focused button with it.
                focusHeading();
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
          onToggle={(isOpen) => onToggle(harness, isOpen)}
        />
      ))}
    </div>
  );
}
