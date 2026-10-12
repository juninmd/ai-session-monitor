import { t } from '../../i18n';
import { InfoCard } from '../panel/StateCards';
import { OpencodeKey, OpencodeLinks, OpencodeLocalSwitch } from './OpencodeParts';

/** First run for OpenCode Go: the key unlocks the real windows, the local estimate needs none. */
export function OpencodeConnect() {
  return (
    <InfoCard icon="key" logo="opencode" title={t('opencode.title')}>
      <p>{t('opencode.lead')}</p>
      <OpencodeKey />
      <OpencodeLinks />
      <OpencodeLocalSwitch />
    </InfoCard>
  );
}
