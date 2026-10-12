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
});
