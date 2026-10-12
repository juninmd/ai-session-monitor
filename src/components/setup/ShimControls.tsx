// The Claude Code statusLine command: what is installed now, and the buttons that change it.
import { useEffect } from 'react';
import { useBackend } from '../../app/backendContext';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { ShimState, ShimStatus } from '../../lib/contract';
import { Icon, type IconName } from '../icons';
import { ActionError } from '../panel/ErrorActions';

const STATE_ICON: Record<ShimState, IconName> = {
  not_installed: 'plug',
  installed: 'check',
  foreign: 'info',
  outdated: 'warn',
  unreadable: 'warn',
};

function stateLabel(shim: ShimStatus): string {
  if (shim.state === 'installed' && shim.chained) return t('shim.state.installed_chained');
  return t(`shim.state.${shim.state}`);
}

function hintFor(state: ShimState): string | null {
  if (state === 'foreign') return t('shim.hint.foreign');
  if (state === 'unreadable') return t('shim.hint.unreadable');
  return null;
}

export function ShimControls() {
  const backend = useBackend();
  const { shim, refreshShim, installShim, uninstallShim } = useSettingsApi();
  const check = useAsyncAction(refreshShim);
  const { run: look } = check;
  const install = useAsyncAction(async () => {
    await installShim();
    await backend.refresh('claude');
  });
  const remove = useAsyncAction(async () => {
    await uninstallShim();
    await backend.refresh('claude');
  });

  // One look when the group opens, so the state shown is the file's, not a stale guess.
  useEffect(() => {
    void look();
  }, [look]);

  const failure = install.error ?? remove.error ?? check.error;
  if (shim === null && !failure) return <p className="hint">{t('shim.loading')}</p>;

  const state = shim?.state ?? null;
  const canInstall = state === 'not_installed' || state === 'foreign' || state === 'outdated';
  const canRemove = state === 'installed' || state === 'outdated';
  const hint = state === null ? null : hintFor(state);
  const busy = install.pending || remove.pending;
  return (
    <div className="shim">
      <div className="field-head">
        <span className="field-title">{t('shim.title')}</span>
        {shim && state && (
          <span className={state === 'installed' ? 'chip' : 'chip warnchip'}>
            <Icon name={STATE_ICON[state]} />
            {stateLabel(shim)}
          </span>
        )}
      </div>
      {hint && <p className="hint">{hint}</p>}
      {shim?.settingsPath && <p className="hint">{t('shim.path', { path: shim.settingsPath })}</p>}
      <div className="actions">
        {canInstall && (
          <button
            type="button"
            className="btn primary"
            disabled={busy}
            onClick={() => void install.run()}
          >
            {state === 'outdated' ? t('shim.reinstall') : t('shim.install')}
          </button>
        )}
        {canRemove && (
          <button type="button" className="btn" disabled={busy} onClick={() => void remove.run()}>
            {t('shim.uninstall')}
          </button>
        )}
        <button
          type="button"
          className="btn"
          disabled={check.pending}
          onClick={() => void check.run()}
        >
          {t('shim.recheck')}
        </button>
      </div>
      {failure && <ActionError error={failure} harness="claude" />}
    </div>
  );
}
