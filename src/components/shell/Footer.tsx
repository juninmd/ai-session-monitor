import { useContext } from 'react';
import { UpdateContext } from '../../app/hooks/useUpdateState';
import { t } from '../../i18n';
import { formatAge } from '../../i18n/format';
import { Icon } from '../icons';

interface FooterProps {
  /** Epoch seconds of the newest read, null before the first one. */
  readAt: number | null;
  nowSecs: number;
  /** Opens the update section of the settings, where the download is chosen. */
  onOpenUpdate: () => void;
}

const SHOWN = ['available', 'downloading', 'ready', 'installing'];

/** The update area above the footer line: what is pending and the one way in, nothing else. */
function UpdateArea({ onOpen }: { onOpen: () => void }) {
  const status = useContext(UpdateContext)?.status;
  if (!status || !SHOWN.includes(status.phase)) return null;
  const downloading = status.phase === 'downloading';
  const text =
    status.phase === 'downloading'
      ? t('update.downloading')
      : status.phase === 'ready'
        ? t('update.ready')
        : status.phase === 'installing'
          ? t('update.installing')
          : t('update.available', { version: status.availableVersion ?? '—' });
  const pct =
    downloading && status.totalBytes
      ? Math.min(100, Math.round((status.downloadedBytes / status.totalBytes) * 100))
      : null;
  return (
    <div className="update-banner update-area" role="status" data-phase={status.phase}>
      <div className="update-banner-lead">
        <Icon name="info" className="update-banner-icon" />
        <span>{text}</span>
      </div>
      {downloading ? (
        <progress
          className="update-area-bar"
          aria-label={t('update.downloading')}
          value={pct ?? undefined}
          max={100}
        />
      ) : (
        <button className="btn" type="button" onClick={onOpen}>
          {t('update.manage')}
        </button>
      )}
    </div>
  );
}

export function Footer({ readAt, nowSecs, onOpenUpdate }: FooterProps) {
  const version = useContext(UpdateContext)?.status?.currentVersion;
  const text =
    readAt === null
      ? t('footer.never')
      : t('footer.read_ago', { age: formatAge(Math.max(0, nowSecs - readAt)) });
  return (
    <>
      <UpdateArea onOpen={onOpenUpdate} />
      <footer className="fly-foot">
        <span>{text}</span>
        {version && <span className="fly-version">v{version}</span>}
      </footer>
    </>
  );
}
