import { t } from '../../i18n';
import { InfoCard } from '../panel/StateCards';
import { CopilotKey, CopilotLinks, CopilotLocalSwitch } from './CopilotParts';

/** First run for GitHub Copilot: a pasted token or the local-token consent, either one unlocks it. */
export function CopilotConnect() {
  return (
    <InfoCard icon="key" logo="copilot" title={t('copilot.title')}>
      <p>{t('copilot.lead')}</p>
      <details>
        <summary>{t('consent.details')}</summary>
        <p>{t('copilot.detail')}</p>
      </details>
      <CopilotKey />
      <CopilotLocalSwitch />
      <CopilotLinks />
    </InfoCard>
  );
}
