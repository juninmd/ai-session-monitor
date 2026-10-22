import { useContext, useState } from 'react';
import { useBackend } from '../../app/backendContext';
import { useDraftGuard } from '../../app/draftContext';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { type UpdateApi, UpdateContext, useUpdateState } from '../../app/useUpdateState';
import { t } from '../../i18n';
import { formatMegabytes } from '../../i18n/format';
import { SettingSwitch } from '../forms/SettingControls';
import { ActionError } from '../panel/ErrorActions';

export function UpdateSettings() {
  const api = useContext(UpdateContext);
  return api ? <UpdateBody api={api} /> : <Standalone />;
}

function Standalone() {
  const api = useUpdateState(useBackend());
  return <UpdateBody api={api} />;
}

function UpdateBody({ api }: { api: UpdateApi }) {
  const settings = useSettingsApi();
  const drafts = useDraftGuard();
  const [confirm, setConfirm] = useState(false);
  const { status, action } = api;
  const install = useAsyncAction(async () => {
    if (drafts?.blocked()) return;
    drafts?.lock(true);
    try {
      await settings.save((current) => current);
      await action.run('install');
    } finally {
      drafts?.lock(false);
    }
  });
  const busy =
    action.pending ||
    install.pending ||
    status?.phase === 'checking' ||
    status?.phase === 'downloading' ||
    status?.phase === 'installing';
  const retryDownload = status?.phase === 'error' && status.availableVersion !== null;
  const download = status?.phase === 'available' || retryDownload;
  const hasDraft = drafts?.entries.some((entry) => entry.dirty || entry.pending) ?? false;
  const phaseText =
    status?.phase === 'error'
      ? t(`update.failure_${status.failure ?? 'check'}`)
      : status?.phase === 'available'
        ? t('update.available', { version: status.availableVersion ?? '—' })
        : t(`update.${status?.phase ?? 'idle'}`);
  return (
    <section className="sg update-settings">
      {status && <p className="hint">{t('update.version', { version: status.currentVersion })}</p>}
      <SettingSwitch
        id="set-auto-update"
        label={t('update.auto')}
        hint={t('update.auto_hint')}
        checked={settings.settings.general.autoUpdateCheck}
        apply={(current, value) => ({
          ...current,
          general: { ...current.general, autoUpdateCheck: value },
        })}
      />
      <p role="status" aria-live="polite">
        {phaseText}
      </p>
      {status?.phase === 'downloading' && (
        <div className="update-progress">
          <progress
            aria-label={t('update.downloading')}
            {...(status.totalBytes !== null && status.totalBytes > 0
              ? { value: status.downloadedBytes, max: status.totalBytes }
              : {})}
          />
          <span>{formatMegabytes(status.downloadedBytes)}</span>
        </div>
      )}
      {status?.notes && (
        <details className="update-notes">
          <summary>{t('update.notes')}</summary>
          <p>{status.notes}</p>
        </details>
      )}
      {status && !status.installable && <p className="hint">{t('update.dev')}</p>}
      {hasDraft && status?.phase === 'ready' && (
        <div className="hint">
          <p>{t('draft.install')}</p>
          <button
            type="button"
            className="btn"
            onClick={() => {
              const entry = drafts?.entries.find((item) => item.dirty || item.pending);
              if (entry) document.getElementById(entry.id)?.focus();
            }}
          >
            {t('draft.review')}
          </button>
        </div>
      )}
      <div className="actions">
        {status?.phase !== 'ready' && status?.phase !== 'installing' && (
          <button
            type="button"
            className="btn"
            disabled={busy || status?.phase === 'unavailable'}
            onClick={() => void action.run('check')}
          >
            {t('update.check')}
          </button>
        )}
        {download && (
          <button
            type="button"
            className="btn primary"
            disabled={busy || !status?.installable}
            onClick={() => void action.run('download')}
          >
            {t('update.download')}
          </button>
        )}
        {status?.phase === 'ready' && !confirm && (
          <button
            type="button"
            className="btn primary"
            disabled={busy || hasDraft || !status.installable}
            onClick={() => setConfirm(true)}
          >
            {t('update.install')}
          </button>
        )}
      </div>
      {confirm && status?.phase === 'ready' && (
        <fieldset className="update-confirm" aria-label={t('update.install')}>
          <p>{t('update.confirm')}</p>
          <div className="actions">
            <button
              type="button"
              className="btn primary"
              disabled={busy || hasDraft}
              onClick={() => void install.run()}
            >
              {t('update.confirm_button')}
            </button>
            <button type="button" className="btn" disabled={busy} onClick={() => setConfirm(false)}>
              {t('update.later')}
            </button>
          </div>
        </fieldset>
      )}
      {action.error && <ActionError error={action.error} />}
      {install.error && <ActionError error={install.error} />}
    </section>
  );
}
