import type { ReactNode } from 'react';
import { useBackend } from '../../app/backendContext';
import { harnessName } from '../../app/harness';
import { useNavigation } from '../../app/navigation';
import { noticeKey, noticeText, uniqueNotices } from '../../app/noticeText';
import { useSettingsApi } from '../../app/settingsContext';
import { withEnabled } from '../../app/settingsPatch';
import { useAsyncAction } from '../../app/useAsyncAction';
import { hasMessage, t } from '../../i18n';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { HarnessLogo } from '../brand/HarnessLogo';
import { Icon, type IconName } from '../icons';
import { ActionError } from './ErrorActions';

interface InfoCardProps {
  icon: IconName;
  title: string;
  /** A tool's logo takes the badge; the icon (lock, key) then rides on its corner. */
  logo?: HarnessId;
  children?: ReactNode;
}

/** The notice card every state uses: round icon badge, heading, body and an action row. */
export function InfoCard({ icon, title, logo, children }: InfoCardProps) {
  return (
    <section className="card notice">
      <span className="badge" data-brand={logo}>
        {logo && <HarnessLogo id={logo} size={32} />}
        <Icon name={icon} className="badge-ico" />
      </span>
      <h2>{title}</h2>
      {children}
    </section>
  );
}

export function DisabledCard({ harness }: { harness: HarnessId }) {
  const { save } = useSettingsApi();
  const navigation = useNavigation();
  const enable = useAsyncAction(() => save((s) => withEnabled(s, harness, true)));
  return (
    <InfoCard icon="power" title={t('disabled.title', { name: harnessName(harness) })}>
      <p>{t('disabled.body')}</p>
      <div className="actions">
        <button
          type="button"
          className="btn primary"
          disabled={enable.pending}
          onClick={() => void enable.run()}
        >
          {t('disabled.enable')}
        </button>
        <button type="button" className="btn" onClick={() => navigation.openSettings(harness)}>
          {t('disabled.settings')}
        </button>
      </div>
      {enable.error && <ActionError error={enable.error} harness={harness} />}
    </InfoCard>
  );
}

export function NoDataCard({ view }: { view: HarnessView }) {
  const backend = useBackend();
  const read = useAsyncAction(() => backend.refresh(view.id));
  const key = `no_data.body.${view.id}`;
  const notices = uniqueNotices(view.notices);
  return (
    <InfoCard icon="clock" title={t('no_data.title')}>
      {notices.length > 0 ? (
        notices.map((notice) => <p key={noticeKey(notice)}>{noticeText(notice)}</p>)
      ) : (
        <p>{hasMessage(key) ? t(key) : t('no_data.body.default')}</p>
      )}
      <div className="actions">
        <button
          type="button"
          className="btn"
          disabled={read.pending}
          onClick={() => void read.run()}
        >
          {t('no_data.retry')}
        </button>
      </div>
      {read.error && <ActionError error={read.error} harness={view.id} />}
    </InfoCard>
  );
}

/** Placeholder blocks with the same footprint as the hero and a flat section. */
export function PanelSkeleton() {
  return (
    <div className="skeleton" aria-busy="true">
      <span className="vh">{t('loading.label')}</span>
      <span className="sk" style={{ inlineSize: '9rem', blockSize: '1.4rem' }} />
      <section className="card hero" aria-hidden="true">
        <span className="sk" style={{ inlineSize: '7rem', blockSize: '.9rem' }} />
        <span className="sk" style={{ inlineSize: '11rem', blockSize: '2.6rem' }} />
        <span className="sk" style={{ inlineSize: '100%', blockSize: '.625rem' }} />
        <span className="sk" style={{ inlineSize: '8rem', blockSize: '.9rem' }} />
      </section>
      <section className="flat" aria-hidden="true">
        <span className="sk" style={{ inlineSize: '6rem', blockSize: '.9rem' }} />
        <span className="sk" style={{ inlineSize: '100%', blockSize: '.625rem' }} />
      </section>
    </div>
  );
}
