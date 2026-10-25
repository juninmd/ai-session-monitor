// Controls that save the moment they change. The saved value is the source of truth, so a
// rejected save leaves the control where it was and prints the backend's reason beside it.
import type { ReactNode } from 'react';
import { fieldOf } from '../../app/errorText';
import { fieldMessage } from '../../app/fieldErrors';
import { useSettingsApi } from '../../app/settingsContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import type { Settings } from '../../lib/contract';
import { type Option, SegmentedRow, SelectRow, SwitchRow, TimeRow } from './fields';

type Apply<V> = (settings: Settings, value: V) => Settings;

function useApply<V>(apply: Apply<V>) {
  const { save } = useSettingsApi();
  const action = useAsyncAction((value: V) => save((current) => apply(current, value)));
  const error = action.error ? fieldMessage(action.error, fieldOf(action.error)) : null;
  return { run: action.run, error };
}

interface SwitchProps {
  id: string;
  label: string;
  checked: boolean;
  apply: Apply<boolean>;
  hint?: string;
  badge?: ReactNode;
}

export function SettingSwitch({ id, label, checked, apply, hint, badge }: SwitchProps) {
  const { run, error } = useApply(apply);
  return (
    <SwitchRow
      id={id}
      label={label}
      checked={checked}
      hint={hint}
      badge={badge}
      error={error}
      onChange={(value) => void run(value)}
    />
  );
}

interface SelectProps<V extends string | number> {
  id: string;
  label: string;
  value: V;
  options: readonly Option<V>[];
  apply: Apply<V>;
  hint?: string;
}

export function SettingSelect<V extends string | number>(props: SelectProps<V>) {
  const { apply, ...rest } = props;
  const { run, error } = useApply(apply);
  return <SelectRow {...rest} error={error} onChange={(value) => void run(value)} />;
}

interface SegmentedProps<V extends string> {
  id: string;
  legend: string;
  value: V;
  options: readonly Option<V>[];
  apply: Apply<V>;
  hint?: string;
}

export function SettingSegmented<V extends string>(props: SegmentedProps<V>) {
  const { apply, id, ...rest } = props;
  const { run, error } = useApply(apply);
  return <SegmentedRow {...rest} name={id} error={error} onChange={(value) => void run(value)} />;
}

interface TimeProps {
  id: string;
  label: string;
  minutes: number;
  apply: Apply<number>;
}

export function SettingTime({ id, label, minutes, apply }: TimeProps) {
  const { run, error } = useApply(apply);
  return (
    <TimeRow
      id={id}
      label={label}
      minutes={minutes}
      error={error}
      onChange={(value) => void run(value)}
    />
  );
}
