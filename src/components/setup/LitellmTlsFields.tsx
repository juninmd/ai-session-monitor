// TLS choice for the LiteLLM connection. "Do not verify" is per host, needs an explicit
// confirmation, and always carries a warning: the key travels over an unauthenticated channel.
import { type DraftErrors, hostOf, type LitellmDraft } from '../../app/litellmDraft';
import { t } from '../../i18n';
import type { TlsMode } from '../../lib/contract';
import { CheckField, type Option, Segmented, TextField } from '../forms/fields';
import { Banner } from '../panel/Banner';

interface TlsFieldsProps {
  draft: LitellmDraft;
  errors: DraftErrors;
  update: (patch: Partial<LitellmDraft>) => void;
}

const tlsOptions = (): readonly Option<TlsMode['mode']>[] => [
  { value: 'system', label: t('litellm.tls.system') },
  { value: 'custom_ca', label: t('litellm.tls.custom_ca') },
  { value: 'insecure', label: t('litellm.tls.insecure') },
];

export function LitellmTlsFields({ draft, errors, update }: TlsFieldsProps) {
  const pick = (mode: TlsMode['mode']) => {
    // Every switch into "do not verify" starts unconfirmed, with the server's own host prefilled.
    if (mode === 'insecure') {
      update({
        tlsMode: mode,
        insecureConfirmed: false,
        insecureHost: draft.insecureHost || hostOf(draft.baseUrl),
      });
    } else {
      update({ tlsMode: mode });
    }
  };

  return (
    <>
      <div className="field">
        <span className="field-title">{t('litellm.tls')}</span>
        <Segmented
          name="litellm-tls"
          legend={t('litellm.tls')}
          value={draft.tlsMode}
          options={tlsOptions()}
          onChange={pick}
        />
      </div>
      {draft.tlsMode === 'custom_ca' && (
        <TextField
          id="litellm-pem"
          label={t('litellm.pem')}
          placeholder={t('litellm.pem_placeholder')}
          value={draft.pemPath}
          error={errors.pemPath}
          onChange={(pemPath) => update({ pemPath })}
        />
      )}
      {draft.tlsMode === 'insecure' && (
        <>
          <Banner tone="warn">
            <p>{t('litellm.insecure_warning')}</p>
          </Banner>
          <TextField
            id="litellm-host"
            label={t('litellm.insecure_host')}
            hint={t('litellm.insecure_host_hint')}
            value={draft.insecureHost}
            error={errors.host}
            onChange={(insecureHost) => update({ insecureHost, insecureConfirmed: false })}
          />
          <CheckField
            id="litellm-confirm"
            label={t('litellm.insecure_confirm')}
            checked={draft.insecureConfirmed}
            error={errors.confirm}
            onChange={(insecureConfirmed) => update({ insecureConfirmed })}
          />
        </>
      )}
    </>
  );
}
