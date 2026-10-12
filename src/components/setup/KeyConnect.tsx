// The key-only sources share one shape: a key unlocks the reading, and a button opens the vendor's key page.
// Pieces shared by the first-run card and the settings group.
import { useBackend } from '../../app/backendContext';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import { SecretField } from '../forms/SecretField';
import { Icon } from '../icons';
import { ActionError } from '../panel/ErrorActions';
import { InfoCard } from '../panel/StateCards';

export type KeyedHarness =
  | 'kimi'
  | 'zai'
  | 'openrouter'
  | 'deepseek'
  | 'deepinfra'
  | 'moonshot'
  | 'novita'
  | 'ollama'
  | 'minimax';

export function VendorKey({ harness }: { harness: KeyedHarness }) {
  const { secrets, setSecret, clearSecret } = useSettingsApi();
  return (
    <SecretField
      id={`${harness}-key`}
      label={t(`${harness}.key_label`)}
      saved={secrets[harness]}
      onSave={(value) => setSecret(harness, value)}
      onClear={() => clearSecret(harness)}
    />
  );
}

/** Opens a fixed destination; the backend maps the name to its URL. */
export function VendorConsoleLink({ harness }: { harness: KeyedHarness }) {
  const backend = useBackend();
  const open = useAsyncAction(() => backend.openLink(`${harness}_console`));
  return (
    <>
      <div className="actions">
        <button type="button" className="btn" onClick={() => void open.run()}>
          {t(`${harness}.console`)}
          <Icon name="external" />
        </button>
      </div>
      {open.error && <ActionError error={open.error} harness={harness} />}
    </>
  );
}

/** First run: the pasted key is the consent, nothing is read before it is saved. */
export function KeyConnect({ harness }: { harness: KeyedHarness }) {
  return (
    <InfoCard icon="key" logo={harness} title={t(`${harness}.title`)}>
      <p>{t(`${harness}.lead`)}</p>
      <details>
        <summary>{t('consent.details')}</summary>
        <p>{t(`${harness}.detail`)}</p>
      </details>
      <VendorKey harness={harness} />
      <VendorConsoleLink harness={harness} />
    </InfoCard>
  );
}
