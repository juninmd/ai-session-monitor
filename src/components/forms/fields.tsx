import type { ReactNode, Ref } from 'react';
import { Icon } from '../icons';

/** Joins the ids of the hint and error elements a control is described by. */
export function describedBy(...ids: Array<string | false | null | undefined>): string | undefined {
  const joined = ids.filter(Boolean).join(' ');
  return joined === '' ? undefined : joined;
}

/** An alert: a rejected save leaves focus where it was, so this is the only sign it failed. */
export function ErrorLine({ id, children }: { id: string; children: ReactNode }) {
  return (
    <p id={id} className="err" role="alert">
      <Icon name="warn" />
      <span>{children}</span>
    </p>
  );
}

interface TextFieldProps {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  hint?: string;
  error?: string | null;
  placeholder?: string;
  type?: 'text' | 'url';
  disabled?: boolean;
  onBlur?: () => void;
  ref?: Ref<HTMLInputElement>;
}

export function TextField(props: TextFieldProps) {
  const { id, label, value, onChange, hint, error, placeholder, type = 'text' } = props;
  const { disabled, onBlur, ref } = props;
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      <input
        id={id}
        ref={ref}
        type={type}
        inputMode={type === 'url' ? 'url' : undefined}
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        placeholder={placeholder}
        value={value}
        disabled={disabled}
        aria-invalid={error ? 'true' : undefined}
        aria-describedby={describedBy(hint && `${id}-hint`, error && `${id}-err`)}
        onChange={(event) => onChange(event.target.value)}
        onBlur={onBlur}
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

interface SwitchRowProps {
  id: string;
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  hint?: string;
  badge?: ReactNode;
  error?: string | null;
  disabled?: boolean;
}

/** A labelled on/off row. The hint describes the switch instead of lengthening its name. */
export function SwitchRow(props: SwitchRowProps) {
  const { id, label, checked, onChange, hint, badge, error, disabled } = props;
  return (
    <>
      <div className="srow">
        <div className="lbl">
          <label htmlFor={id}>{label}</label>
          {badge}
          {hint && (
            <small id={`${id}-hint`} className="hint">
              {hint}
            </small>
          )}
        </div>
        <input
          id={id}
          className="switch"
          type="checkbox"
          role="switch"
          aria-checked={checked}
          checked={checked}
          disabled={disabled}
          aria-describedby={describedBy(hint && `${id}-hint`, error && `${id}-err`)}
          onChange={(event) => onChange(event.target.checked)}
        />
      </div>
      {error && <ErrorLine id={`${id}-err`}>{error}</ErrorLine>}
    </>
  );
}

interface CheckFieldProps {
  id: string;
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  error?: string | null;
}

/** A plain checkbox for confirmations ("I understand the risk"), as opposed to an on/off setting. */
export function CheckField({ id, label, checked, onChange, error }: CheckFieldProps) {
  return (
    <div className="check">
      <div className="check-row">
        <input
          id={id}
          type="checkbox"
          checked={checked}
          aria-invalid={error ? 'true' : undefined}
          aria-describedby={describedBy(error && `${id}-err`)}
          onChange={(event) => onChange(event.target.checked)}
        />
        <label htmlFor={id}>{label}</label>
      </div>
      {error && <ErrorLine id={`${id}-err`}>{error}</ErrorLine>}
    </div>
  );
}

export interface Option<V extends string | number> {
  value: V;
  label: string;
}

interface SelectRowProps<V extends string | number> {
  id: string;
  label: string;
  value: V;
  options: readonly Option<V>[];
  onChange: (value: V) => void;
  hint?: string;
  error?: string | null;
  disabled?: boolean;
}

export function SelectRow<V extends string | number>(props: SelectRowProps<V>) {
  const { id, label, value, options, onChange, hint, error, disabled } = props;
  return (
    <>
      <div className="srow">
        <div className="lbl">
          <label htmlFor={id}>{label}</label>
          {hint && (
            <small id={`${id}-hint`} className="hint">
              {hint}
            </small>
          )}
        </div>
        <select
          id={id}
          value={String(value)}
          disabled={disabled}
          aria-describedby={describedBy(hint && `${id}-hint`, error && `${id}-err`)}
          onChange={(event) => {
            const picked = options.find((o) => String(o.value) === event.target.value);
            if (picked) onChange(picked.value);
          }}
        >
          {options.map((o) => (
            <option key={String(o.value)} value={String(o.value)}>
              {o.label}
            </option>
          ))}
        </select>
      </div>
      {error && <ErrorLine id={`${id}-err`}>{error}</ErrorLine>}
    </>
  );
}

interface SegmentedProps<V extends string> {
  name: string;
  legend: string;
  value: V;
  options: readonly Option<V>[];
  onChange: (value: V) => void;
  disabled?: boolean;
}

/** Radio group drawn as a segmented control; arrow keys move between segments natively. */
export function Segmented<V extends string>(props: SegmentedProps<V>) {
  const { name, legend, value, options, onChange, disabled } = props;
  return (
    <div className="seg" role="radiogroup" aria-label={legend}>
      {options.map((o) => (
        <label key={o.value}>
          <input
            type="radio"
            name={name}
            value={o.value}
            checked={o.value === value}
            disabled={disabled}
            onChange={() => onChange(o.value)}
          />
          <span>{o.label}</span>
        </label>
      ))}
    </div>
  );
}

type SegmentedRowProps<V extends string> = SegmentedProps<V> & {
  hint?: string;
  error?: string | null;
};

/** A settings row whose control is a segmented group (the group is named by `legend`). */
export function SegmentedRow<V extends string>(props: SegmentedRowProps<V>) {
  const { hint, error, ...group } = props;
  return (
    <>
      <div className="srow">
        <div className="lbl">
          <span aria-hidden="true">{group.legend}</span>
          {hint && <small className="hint">{hint}</small>}
        </div>
        <Segmented {...group} />
      </div>
      {error && <ErrorLine id={`${group.name}-err`}>{error}</ErrorLine>}
    </>
  );
}
