import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CheckField, SegmentedRow, SelectRow, SwitchRow, TextField } from './fields';

const OPTIONS = [{ value: 'a', label: 'A' }] as const;
const MESSAGE = 'Valor inválido.';
const noop = () => {};

// A rejected save leaves focus on the control that was used, so the role is what tells a screen
// reader the save failed; the polite <output> beside a form only ever announces the success.
const withError = (error: string | null) => ({
  TextField: <TextField id="f" label="Campo" value="" onChange={noop} error={error} />,
  SwitchRow: <SwitchRow id="f" label="Opção" checked={false} onChange={noop} error={error} />,
  CheckField: <CheckField id="f" label="Aceito" checked={false} onChange={noop} error={error} />,
  SelectRow: (
    <SelectRow id="f" label="Lista" value="a" options={OPTIONS} onChange={noop} error={error} />
  ),
  SegmentedRow: (
    <SegmentedRow
      name="f"
      legend="Grupo"
      value="a"
      options={OPTIONS}
      onChange={noop}
      error={error}
    />
  ),
});

describe('a field error', () => {
  it.each(Object.entries(withError(MESSAGE)))('%s is announced as an alert', (_name, ui) => {
    render(ui);
    expect(screen.getByRole('alert')).toHaveTextContent(MESSAGE);
  });

  it.each(Object.entries(withError(null)))('%s renders no alert while valid', (_name, ui) => {
    render(ui);
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('still describes the control it belongs to', () => {
    render(withError(MESSAGE).TextField);
    const alert = screen.getByRole('alert');
    expect(screen.getByLabelText('Campo')).toHaveAttribute('aria-describedby', alert.id);
  });

  // A screen reader reads the error only from the control it is linked to, so each kind of
  // control must be marked invalid and point at its error line.
  it.each([
    ['SwitchRow', () => screen.getByRole('switch')],
    ['SelectRow', () => screen.getByRole('combobox')],
    ['SegmentedRow', () => screen.getByRole('radiogroup')],
  ])('%s is marked invalid and describes its error', (name, control) => {
    render(withError(MESSAGE)[name as keyof ReturnType<typeof withError>]);
    const alert = screen.getByRole('alert');
    expect(control()).toHaveAttribute('aria-invalid', 'true');
    expect(control()).toHaveAttribute('aria-describedby', alert.id);
  });

  it('clears aria-invalid on a valid segmented group', () => {
    render(withError(null).SegmentedRow);
    const group = screen.getByRole('radiogroup');
    expect(group).not.toHaveAttribute('aria-invalid');
    expect(group).not.toHaveAttribute('aria-describedby');
  });
});
