import { useState } from 'react';
import { tq } from '../../app/quota';
import { t } from '../../i18n';
import { formatPercent } from '../../i18n/format';
import type { Status } from '../../lib/usage';
import { Icon } from '../icons';
import { TrayRing } from './TrayRing';

interface HeaderProps {
  /** `pct` is already the printed share (available, or used under `showUsed`). */
  tray: { pct: number; status: Status; showUsed: boolean } | null;
  settingsOpen: boolean;
  /** Sources whose last read failed: the refresh icon turns into the retry for them. */
  errorCount: number;
  onRefresh: () => void;
  onToggleSettings: () => void;
  onQuit: () => void;
}

export function Header({
  tray,
  settingsOpen,
  errorCount,
  onRefresh,
  onToggleSettings,
  onQuit,
}: HeaderProps) {
  // A new key restarts the spin on every press, even while the previous turn is still running.
  const [spins, setSpins] = useState(0);
  const trayLabel = tray
    ? tq('header.tray', tray.showUsed, { pct: formatPercent(tray.pct) })
    : t('header.tray_none');
  const refreshLabel =
    errorCount > 0 ? t('header.retry', { count: errorCount }) : t('header.refresh');
  return (
    <header className="fly-head">
      <h1 className="fly-title" title={trayLabel}>
        <TrayRing pct={tray?.pct ?? null} status={tray?.status ?? null} />
        <span>{t('app.title')}</span>
      </h1>
      <div className="fly-actions">
        <button
          type="button"
          className={['icon-btn', spins > 0 && 'spin', errorCount > 0 && 'warn']
            .filter(Boolean)
            .join(' ')}
          aria-label={refreshLabel}
          title={refreshLabel}
          onClick={() => {
            setSpins((count) => count + 1);
            onRefresh();
          }}
        >
          <Icon key={spins} name="refresh" />
        </button>
        <button
          type="button"
          className="icon-btn"
          aria-label={t('header.settings')}
          title={t('header.settings')}
          aria-pressed={settingsOpen}
          onClick={onToggleSettings}
        >
          <Icon name="sliders" />
        </button>
        <button
          type="button"
          className="icon-btn"
          aria-label={t('header.quit')}
          title={t('header.quit')}
          onClick={onQuit}
        >
          <Icon name="power" />
        </button>
      </div>
    </header>
  );
}
