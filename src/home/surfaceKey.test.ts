import { describe, expect, it } from 'vitest';
import { type SurfaceKeyEvent, surfaceFromKey } from './surfaceKey';

const SURFACES = ['triage', 'horizon', 'channels'] as const;

function key(over: Partial<SurfaceKeyEvent> & { altgr?: boolean } = {}): SurfaceKeyEvent {
  const { altgr, ...rest } = over;
  return {
    key: '1',
    code: 'Digit1',
    altKey: true,
    ctrlKey: false,
    metaKey: false,
    repeat: false,
    isComposing: false,
    target: document.createElement('button'),
    getModifierState: (name) => name === 'AltGraph' && altgr === true,
    ...rest,
  };
}

describe('surfaceFromKey', () => {
  it('maps Alt+digit on a button to the surface at that position', () => {
    expect(surfaceFromKey(key(), SURFACES)).toBe('triage');
    expect(surfaceFromKey(key({ key: '3', code: 'Digit3' }), SURFACES)).toBe('channels');
  });

  it('ignores a digit past the last surface', () => {
    expect(surfaceFromKey(key({ key: '4', code: 'Digit4' }), SURFACES)).toBeUndefined();
  });

  it('leaves text fields alone so a draft is not interrupted', () => {
    expect(
      surfaceFromKey(key({ target: document.createElement('input') }), SURFACES),
    ).toBeUndefined();
    expect(
      surfaceFromKey(key({ target: document.createElement('textarea') }), SURFACES),
    ).toBeUndefined();
    expect(
      surfaceFromKey(key({ target: document.createElement('select') }), SURFACES),
    ).toBeUndefined();
  });

  it('ignores Alt+numpad character codes, AltGr, repeats and IME composition', () => {
    expect(surfaceFromKey(key({ code: 'Numpad1' }), SURFACES)).toBeUndefined();
    expect(surfaceFromKey(key({ altgr: true }), SURFACES)).toBeUndefined();
    expect(surfaceFromKey(key({ repeat: true }), SURFACES)).toBeUndefined();
    expect(surfaceFromKey(key({ isComposing: true }), SURFACES)).toBeUndefined();
  });

  it('needs Alt alone', () => {
    expect(surfaceFromKey(key({ altKey: false }), SURFACES)).toBeUndefined();
    expect(surfaceFromKey(key({ ctrlKey: true }), SURFACES)).toBeUndefined();
    expect(surfaceFromKey(key({ metaKey: true }), SURFACES)).toBeUndefined();
  });
});
