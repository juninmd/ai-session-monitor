import { useContext } from 'react';
import { UpdateContext } from '../../app/useUpdateState';
import { t } from '../../i18n';
import { Icon } from '../icons';

export function UpdateBanner({ onOpen }: { onOpen: () => void }) {
  const status = useContext(UpdateContext)?.status;
  if (!status || (status.phase !== 'available' && status.phase !== 'ready')) return null;
  return (
    <div className="update-banner">
      <div className="update-banner-lead">
        <Icon name="info" className="update-banner-icon" />
        <span>{t('update.available', { version: status.availableVersion ?? '—' })}</span>
      </div>
      <button className="btn" type="button" onClick={onOpen}>
        {t('update.manage')}
      </button>
    </div>
  );
}
