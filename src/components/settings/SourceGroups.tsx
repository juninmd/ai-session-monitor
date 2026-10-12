// The body of each source group in the settings view (everything below its on/off switch).
import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import { PathField } from '../forms/PathField';
import { SettingSwitch } from '../forms/SettingControls';
import { VendorConsoleLink, VendorKey } from '../setup/KeyConnect';
import { LitellmForm } from '../setup/LitellmForm';
import { OpencodeKey, OpencodeLinks, OpencodeLocalSwitch } from '../setup/OpencodeParts';
import { ClaudeGroup } from './ClaudeGroup';
import { SourceSwitch } from './SourceSwitch';

function CodexGroup() {
  const { settings } = useSettingsApi();
  return (
    <>
      <SourceSwitch harness="codex" />
      <SettingSwitch
        id="codex-consent"
        label={t('codex.consent')}
        hint={t('codex.consent_hint')}
        checked={settings.codex.consent}
        apply={(current, consent) => ({ ...current, codex: { ...current.codex, consent } })}
      />
      <PathField
        id="codex-binary"
        label={t('codex.binary')}
        hint={t('codex.binary_hint')}
        placeholder={t('codex.binary_placeholder')}
        saveLabel={t('codex.binary_save')}
        saved={settings.codex.binaryPath}
        apply={(current, binaryPath) => ({ ...current, codex: { ...current.codex, binaryPath } })}
      />
    </>
  );
}

function AntigravityGroup() {
  const { settings } = useSettingsApi();
  return (
    <>
      <SourceSwitch harness="antigravity" />
      <SettingSwitch
        id="antigravity-consent"
        label={t('antigravity.consent')}
        hint={t('antigravity.consent_hint')}
        checked={settings.antigravity.consent}
        apply={(current, consent) => ({
          ...current,
          antigravity: { ...current.antigravity, consent },
        })}
      />
    </>
  );
}

function OpencodeGroup() {
  const { settings } = useSettingsApi();
  return (
    <>
      <SourceSwitch harness="opencode" />
      <OpencodeKey />
      <OpencodeLinks showBilling />
      <OpencodeLocalSwitch />
      <PathField
        id="opencode-data-dir"
        label={t('opencode.data_dir')}
        hint={t('opencode.data_dir_hint')}
        saveLabel={t('opencode.save_settings')}
        saved={settings.opencode.dataDir}
        apply={(current, dataDir) => ({ ...current, opencode: { ...current.opencode, dataDir } })}
      />
    </>
  );
}

function KeyedGroup({ harness }: { harness: 'kimi' | 'zai' }) {
  return (
    <>
      <SourceSwitch harness={harness} />
      <VendorKey harness={harness} />
      <VendorConsoleLink harness={harness} />
    </>
  );
}

function KimiGroup() {
  return <KeyedGroup harness="kimi" />;
}

function ZaiGroup() {
  return <KeyedGroup harness="zai" />;
}

function LitellmGroup() {
  return (
    <>
      <SourceSwitch harness="litellm" />
      <LitellmForm showTitle={false} />
    </>
  );
}

export const SOURCE_GROUPS = {
  claude: ClaudeGroup,
  codex: CodexGroup,
  antigravity: AntigravityGroup,
  opencode: OpencodeGroup,
  kimi: KimiGroup,
  zai: ZaiGroup,
  litellm: LitellmGroup,
} as const;
