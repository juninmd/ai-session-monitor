// LiteLLM connection: URL, key, scope and the advanced transport options. Settings are saved
// first and the key after them, so the refresh the key triggers already sees a complete setup.
import { type FormEvent, useEffect, useRef, useState } from 'react';
import { useBackend } from '../../app/backendContext';
import { fieldOf } from '../../app/errorText';
import { fieldMessage } from '../../app/fieldErrors';
import {
  checkBaseUrl,
  type DraftErrors,
  type DraftField,
  draftFieldOf,
  fromDraft,
  type LitellmDraft,
  toDraft,
  validateDraft,
} from '../../app/litellmDraft';
import {
  intervalLabel,
  LITELLM_POLL_PRESETS,
  numericOptions,
  secondsLabel,
  TIMEOUT_PRESETS,
} from '../../app/options';
import { useSettingsApi } from '../../app/settingsContext';
import { toHarnessError } from '../../app/toHarnessError';
import { t } from '../../i18n';
import {
  LIMITS,
  LITELLM_SCOPES,
  type LitellmScope,
  type LitellmSettings,
} from '../../lib/contract';
import { HarnessLogo } from '../brand/HarnessLogo';
import { ErrorLine, type Option, Segmented, SelectRow, TextField } from '../forms/fields';
import { SecretInput } from '../forms/SecretField';
import { LitellmTlsFields } from './LitellmTlsFields';

const ADVANCED: readonly DraftField[] = ['authHeader', 'pemPath', 'host', 'confirm'];

const SCOPE_OPTIONS: readonly Option<LitellmScope>[] = LITELLM_SCOPES.map((value) => ({
  value,
  label: t(`litellm.scope.${value}`),
}));

/**
 * What decides who receives the key: server, header and certificate trust, with the spelling
 * (case, default port, trailing slash) taken out. The backend drops the key by the same rule.
 */
function destinationOf({ baseUrl, authHeader, tls }: LitellmSettings): string | null {
  if (baseUrl === null) return null;
  const trust = tls.mode === 'custom_ca' ? `ca:${tls.pemPath}` : tls.mode;
  let where = [baseUrl];
  try {
    const url = new URL(baseUrl);
    const host = url.hostname.replace(/\.+$/, '');
    where = [url.protocol, host, url.port, url.pathname.replace(/\/+$/, '')];
  } catch {
    // Not a URL yet (still being typed): only identical text counts as the same place.
  }
  return JSON.stringify([...where, authHeader.toLowerCase(), trust]);
}

/** True when saving `draft` leaves a key stored for `saved` where it was entered. */
function sameDestination(saved: LitellmSettings, draft: LitellmDraft): boolean {
  return destinationOf(saved) === destinationOf(fromDraft(saved, draft));
}

interface LitellmFormProps {
  /** The settings group already names the section; the inline editor brings its own heading. */
  showTitle?: boolean;
  onSaved?: () => void;
}

export function LitellmForm({ showTitle = true, onSaved }: LitellmFormProps) {
  const backend = useBackend();
  const { settings, secrets, save, setSecret } = useSettingsApi();
  const saved = settings.litellm;
  const connected = saved.baseUrl !== null;

  const [draft, setDraft] = useState(() => toDraft(saved));
  // A stored key is only good for the connection it was entered for; a move drops it.
  const keyFits = secrets.litellm && sameDestination(saved, draft);
  const [seen, setSeen] = useState(() => JSON.stringify(saved));
  const [secret, setSecretText] = useState('');
  const [errors, setErrors] = useState<DraftErrors>({});
  const [status, setStatus] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [advanced, setAdvanced] = useState(() => toDraft(saved).tlsMode !== 'system');
  const [focusTick, setFocusTick] = useState(0);
  const formRef = useRef<HTMLFormElement>(null);

  // Saved settings that changed elsewhere replace the draft; our own save lands here too.
  const savedJson = JSON.stringify(saved);
  if (seen !== savedJson) {
    setSeen(savedJson);
    setDraft(toDraft(saved));
    // Settings arrive after the first render; a non-default transport must not stay hidden.
    if (saved.tls.mode !== 'system') setAdvanced(true);
  }

  useEffect(() => {
    if (focusTick === 0) return;
    formRef.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
  }, [focusTick]);

  const update = (patch: Partial<LitellmDraft>) => {
    setDraft((current) => ({ ...current, ...patch }));
    setStatus(null);
  };

  const show = (found: DraftErrors) => {
    setErrors(found);
    if (ADVANCED.some((field) => found[field])) setAdvanced(true);
    setFocusTick((tick) => tick + 1);
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (pending) return;
    const typed = secret.trim() !== '';
    const found = validateDraft(draft, { saved: keyFits, typed });
    setStatus(null);
    if (Object.keys(found).length > 0) {
      show(found);
      return;
    }
    setErrors({});
    setPending(true);
    try {
      await save((current) => ({ ...current, litellm: fromDraft(current.litellm, draft) }));
      if (typed) await setSecret('litellm', secret.trim());
      setSecretText('');
      await backend.refresh('litellm');
      setStatus(t('litellm.saved'));
      onSaved?.();
    } catch (failure) {
      const error = toHarnessError(failure);
      const field = fieldOf(error);
      show({ [draftFieldOf(field)]: fieldMessage(error, field) });
    } finally {
      setPending(false);
    }
  };

  const checkUrlNow = () => {
    if (draft.baseUrl.trim() === '') return;
    const problem = checkBaseUrl(draft.baseUrl);
    setErrors((current) => ({ ...current, baseUrl: problem ?? undefined }));
  };

  const entityLabel = draft.scope === 'team' ? t('litellm.entity.team') : t('litellm.entity.user');
  return (
    <form ref={formRef} className="card form" noValidate onSubmit={submit}>
      {showTitle && (
        <h3 className="brand-title">
          <HarnessLogo id="litellm" size={32} />
          {connected ? t('litellm.title_edit') : t('litellm.title')}
        </h3>
      )}
      <TextField
        id="litellm-url"
        type="url"
        label={t('litellm.url')}
        placeholder={t('litellm.url_placeholder')}
        hint={t('litellm.url_hint')}
        value={draft.baseUrl}
        error={errors.baseUrl}
        onChange={(baseUrl) => update({ baseUrl })}
        onBlur={checkUrlNow}
      />
      <SecretInput
        id="litellm-key"
        label={t('litellm.key')}
        value={secret}
        saved={secrets.litellm}
        placeholder={keyFits ? t('secret.placeholder_keep') : undefined}
        hint={secrets.litellm && !keyFits ? t('litellm.key_rebind') : t('secret.hint')}
        error={errors.key}
        onChange={(value) => {
          setSecretText(value);
          setStatus(null);
        }}
      />
      <div className="field">
        <span className="field-title">{t('litellm.scope')}</span>
        <Segmented
          name="litellm-scope"
          legend={t('litellm.scope')}
          value={draft.scope}
          options={SCOPE_OPTIONS}
          onChange={(scope) => update({ scope })}
        />
      </div>
      {draft.scope !== 'key' && (
        <TextField
          id="litellm-entity"
          label={entityLabel}
          value={draft.entityId}
          error={errors.entityId}
          onChange={(entityId) => update({ entityId })}
        />
      )}
      <details
        className="adv"
        open={advanced}
        onToggle={(event) => setAdvanced(event.currentTarget.open)}
      >
        <summary>{t('litellm.advanced')}</summary>
        <TextField
          id="litellm-header"
          label={t('litellm.auth_header')}
          hint={t('litellm.auth_header_hint')}
          value={draft.authHeader}
          error={errors.authHeader}
          onChange={(authHeader) => update({ authHeader })}
        />
        <SelectRow
          id="litellm-poll"
          label={t('litellm.poll')}
          value={draft.pollIntervalSecs}
          options={numericOptions(
            LITELLM_POLL_PRESETS,
            LIMITS.litellmPollSecs,
            draft.pollIntervalSecs,
            intervalLabel,
          )}
          onChange={(pollIntervalSecs) => update({ pollIntervalSecs })}
        />
        <SelectRow
          id="litellm-timeout"
          label={t('litellm.timeout')}
          value={draft.timeoutSecs}
          options={numericOptions(
            TIMEOUT_PRESETS,
            LIMITS.litellmTimeoutSecs,
            draft.timeoutSecs,
            secondsLabel,
          )}
          onChange={(timeoutSecs) => update({ timeoutSecs })}
        />
        <LitellmTlsFields draft={draft} errors={errors} update={update} />
      </details>
      <div className="actions">
        <button type="submit" className="btn primary" disabled={pending}>
          {pending ? t('litellm.saving') : t('litellm.save')}
        </button>
      </div>
      {errors.form && <ErrorLine id="litellm-form-err">{errors.form}</ErrorLine>}
      <output className="result">{status}</output>
    </form>
  );
}
