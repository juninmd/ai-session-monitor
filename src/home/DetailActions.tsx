// The three controls every provider detail carries, in the drawer bar and in the inline detail:
// read again, pin to the top, open the provider's settings.
import { useNavigation } from '../app/context/navigation';
import { useSettingsApi } from '../app/context/settingsContext';
import { useAsyncAction } from '../app/hooks/useAsyncAction';
import { harnessName } from '../app/model/harness';
import type { ViewContext } from '../app/model/windowModel';
import { Icon } from '../components/icons';
import { ActionError } from '../components/panel/ErrorActions';
import { t } from '../i18n';
import type { HarnessView } from '../lib/contract';

export function DetailActions({
  view,
  ctx,
  onRefresh,
}: {
  view: HarnessView;
  ctx: ViewContext;
  onRefresh: () => void;
}) {
  const navigation = useNavigation();
  const name = harnessName(view.id);
  const { save } = useSettingsApi();
  const pinned = ctx.settings.general.pinned.includes(view.id);
  const togglePin = useAsyncAction(() =>
    save((current) => {
      const rest = current.general.pinned.filter((id) => id !== view.id);
      const pinned = rest.length === current.general.pinned.length ? [...rest, view.id] : rest;
      return { ...current, general: { ...current.general, pinned } };
    }),
  );
  return (
    <>
      <button
        type="button"
        className="icon-btn"
        aria-label={t('drawer.refresh', { name })}
        onClick={onRefresh}
      >
        <Icon name="refresh" />
      </button>
      <button
        type="button"
        className="icon-btn"
        aria-label={t('drawer.pin', { name })}
        title={pinned ? t('drawer.unpin', { name }) : t('drawer.pin', { name })}
        aria-pressed={pinned}
        disabled={togglePin.pending}
        onClick={() => void togglePin.run()}
      >
        <Icon name="pin" />
      </button>
      <button
        type="button"
        className="icon-btn"
        aria-label={t('drawer.settings', { name })}
        onClick={() => navigation.openSettings(view.id)}
      >
        <Icon name="sliders" />
      </button>
      {togglePin.error && <ActionError error={togglePin.error} />}
    </>
  );
}
