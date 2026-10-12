import { useBackend } from '../../app/backendContext';
import { describeError, type ErrorAction } from '../../app/errorText';
import { useNavigation } from '../../app/navigation';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { HarnessError, HarnessId } from '../../lib/contract';
import { Icon } from '../icons';

/** Runs the one action an error offers: retry, open settings, open the console, reinstall. */
export function useErrorAction(action: ErrorAction, harness: HarnessId) {
  const backend = useBackend();
  const navigation = useNavigation();
  const { installShim } = useSettingsApi();
  const task = useAsyncAction(async () => {
    switch (action) {
      case 'retry':
        await backend.refresh(harness);
        return;
      case 'settings':
        navigation.openSettings(harness);
        return;
      case 'console':
        await backend.openLink('opencode_go_console');
        return;
      case 'reinstall':
        await installShim();
        await backend.refresh(harness);
        return;
    }
  });
  return { ...task, label: t(`action.${action}`) };
}

/** A failure of the action itself (the link did not open, the install failed again). */
export function ActionError({ error, harness }: { error: HarnessError; harness?: HarnessId }) {
  const text = describeError(error, harness);
  return (
    <p className="err" role="alert">
      <Icon name="warn" />
      <span>{`${text.title}. ${text.body}`}</span>
    </p>
  );
}
