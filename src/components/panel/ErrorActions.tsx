import { useBackend } from '../../app/backendContext';
import { describeError, type ErrorAction } from '../../app/errorText';
import { consoleLink } from '../../app/harness';
import { useNavigation } from '../../app/navigation';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { hasMessage, type MessageKey, t } from '../../i18n';
import type { HarnessError, HarnessId } from '../../lib/contract';
import { Icon } from '../icons';

/** The console button names the vendor; a harness without its own wording gets OpenCode's. */
function actionKey(action: ErrorAction, harness: HarnessId): MessageKey {
  if (action !== 'console') return `action.${action}` as MessageKey;
  const own = `action.console.${harness}`;
  return hasMessage(own) ? own : 'action.console.opencode';
}

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
        await backend.openLink(consoleLink(harness));
        return;
      case 'reinstall':
        await installShim();
        await backend.refresh(harness);
        return;
    }
  });
  return { ...task, label: t(actionKey(action, harness)) };
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
