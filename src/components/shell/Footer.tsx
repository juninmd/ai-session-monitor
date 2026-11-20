import { useContext, useEffect, useRef, useState } from 'react';
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

/** How long the bar stays up after a fresh read, so the reload is seen without reaching for it. */
const PEEK_MS = 3000;

/** True for a moment each time a newer read arrives (not for the first one the page loads with). */
function usePeek(readAt: number | null): boolean {
  const [peek, setPeek] = useState(false);
  const last = useRef(readAt);
  useEffect(() => {
    if (readAt === last.current) return;
    const first = last.current === null;
    last.current = readAt;
    if (first) return;
    setPeek(true);
    const timer = setTimeout(() => setPeek(false), PEEK_MS);
    return () => clearTimeout(timer);
  }, [readAt]);
  return peek;
}

export function Footer({ readAt, nowSecs, onOpenUpdate }: FooterProps) {
  const version = useContext(UpdateContext)?.status?.currentVersion;
  const peek = usePeek(readAt);
  const text =
    readAt === null
      ? t('footer.never')
      : t('footer.read_ago', { age: formatAge(Math.max(0, nowSecs - readAt)) });
  return (
    <>
      <UpdateArea onOpen={onOpenUpdate} />
      {/* Auto-hide: the bar rests below the frame and rises when the pointer or the keyboard
          reaches the bottom edge, or for a moment after each fresh read. */}
      <div className="nx-foot" data-peek={peek || undefined}>
        <span className="nx-foot-edge" aria-hidden="true" />
        <footer className="fly-foot">
          <span>{text}</span>
          {version && <span className="fly-version">v{version}</span>}
        </footer>
      </div>
    </>
  );
}
