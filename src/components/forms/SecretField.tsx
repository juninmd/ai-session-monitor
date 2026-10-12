// Secret fields never echo a value: the input is write-only, cleared the moment a save succeeds,
// and the only thing rendered about a stored key is the saved / not-saved chip.
import { type FormEvent, type Ref, useRef, useState } from 'react';
import { fieldOf } from '../../app/errorText';
import { fieldMessage } from '../../app/fieldErrors';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import { Icon } from '../icons';
import { describedBy, ErrorLine } from './fields';

export function SecretChip({ saved }: { saved: boolean }) {
  return (
    <span className={saved ? 'chip' : 'chip off'}>
      <Icon name={saved ? 'check' : 'lock'} />
      {saved ? t('secret.saved') : t('secret.not_saved')}
    </span>
  );
}

interface SecretInputProps {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  saved: boolean;
  hint?: string;
  placeholder?: string;
  error?: string | null;
  disabled?: boolean;
  ref?: Ref<HTMLInputElement>;
}

/** Write-only password input plus the stored-or-not chip. */
export function SecretInput(props: SecretInputProps) {
  const { id, label, value, onChange, saved, hint, placeholder, error, disabled, ref } = props;
  return (
    <div className="field">
      <div className="field-head">
        <label htmlFor={id}>{label}</label>
        <SecretChip saved={saved} />
      </div>
      <input
        id={id}
        ref={ref}
        type="password"
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        placeholder={placeholder ?? t('secret.placeholder')}
        value={value}
        disabled={disabled}
        aria-invalid={error ? 'true' : undefined}
        aria-describedby={describedBy(hint && `${id}-hint`, error && `${id}-err`)}
        onChange={(event) => onChange(event.target.value)}
      />
      {hint && (
        <p id={`${id}-hint`} className="hint">
          {hint}
        </p>
      )}
      {error && <ErrorLine id={`${id}-err`}>{error}</ErrorLine>}
    </div>
  );
}

interface SecretFieldProps {
  id: string;
  label: string;
  saved: boolean;
  onSave: (value: string) => Promise<unknown>;
  onClear: () => Promise<unknown>;
}

/** A stand-alone secret form: one field, Save and Remove. */
export function SecretField({ id, label, saved, onSave, onClear }: SecretFieldProps) {
  const [value, setValue] = useState('');
  const [local, setLocal] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const save = useAsyncAction(onSave);
  const clear = useAsyncAction(onClear);

  const edit = (next: string) => {
    setValue(next);
    setLocal(null);
    setStatus(null);
    save.clear();
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const secret = value.trim();
    if (secret === '') {
      setLocal(t('secret.empty'));
      return;
    }
    if (await save.run(secret)) {
      setValue('');
      setStatus(t('secret.saved_ok'));
    }
  };

  const remove = async () => {
    setStatus(null);
    // The button goes away with the key, and a disabled one drops the keyboard: the field stays.
    inputRef.current?.focus();
    if (await clear.run()) setStatus(t('secret.cleared_ok'));
  };

  const failure = save.error ?? clear.error;
  const error = local ?? (failure ? fieldMessage(failure, fieldOf(failure)) : null);
  return (
    <form className="form" noValidate onSubmit={submit}>
      <SecretInput
        id={id}
        ref={inputRef}
        label={label}
        value={value}
        onChange={edit}
        saved={saved}
        hint={t('secret.hint')}
        placeholder={saved ? t('secret.placeholder_keep') : undefined}
        error={error}
      />
      <div className="actions">
        <button type="submit" className="btn primary" disabled={save.pending}>
          {t('secret.save')}
        </button>
        {saved && (
          <button
            type="button"
            className="btn"
            disabled={clear.pending}
            onClick={() => void remove()}
          >
            {t('secret.clear')}
          </button>
        )}
      </div>
      <output className="result">{status}</output>
    </form>
  );
}
