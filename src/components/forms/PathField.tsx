// A saved-on-demand text setting (a file or folder path). Blank means "use the default" (null).
import { type FormEvent, useState } from 'react';
import { usePendingDraft } from '../../app/draftContext';
import { fieldOf } from '../../app/errorText';
import { fieldMessage } from '../../app/fieldErrors';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import { t } from '../../i18n';
import type { Settings } from '../../lib/contract';
import { TextField } from './fields';

interface PathFieldProps {
  id: string;
  label: string;
  hint: string;
  saveLabel: string;
  saved: string | null;
  apply: (settings: Settings, value: string | null) => Settings;
  placeholder?: string;
}

export function PathField({
  id,
  label,
  hint,
  saveLabel,
  saved,
  apply,
  placeholder,
}: PathFieldProps) {
  const { save } = useSettingsApi();
  const [draft, setDraft] = useState(saved ?? '');
  const [seen, setSeen] = useState(saved);
  const [status, setStatus] = useState<string | null>(null);
  const action = useAsyncAction((value: string | null) => save((s) => apply(s, value)));
  usePendingDraft(id, draft.trim() !== (saved ?? ''), action.pending);

  // Follow stored changes only while clean; an external event must not erase an edited path.
  if (seen !== saved) {
    if (draft.trim() === (seen ?? '')) setDraft(saved ?? '');
    setSeen(saved);
  }

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const trimmed = draft.trim();
    setStatus(null);
    if (await action.run(trimmed === '' ? null : trimmed)) {
      setDraft((current) => (current.trim() === trimmed ? trimmed : current));
      setStatus(t('settings.saved'));
    }
  };

  const error = action.error ? fieldMessage(action.error, fieldOf(action.error)) : null;
  return (
    <form className="form" noValidate onSubmit={submit}>
      <TextField
        id={id}
        label={label}
        value={draft}
        disabled={action.pending}
        placeholder={placeholder}
        hint={hint}
        error={error}
        onChange={(value) => {
          setDraft(value);
          setStatus(null);
          action.clear();
        }}
      />
      <div className="actions">
        <button type="submit" className="btn" disabled={action.pending}>
          {saveLabel}
        </button>
      </div>
      <output className="result">{status}</output>
    </form>
  );
}
