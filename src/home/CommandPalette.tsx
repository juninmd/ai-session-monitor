// Ctrl+K: one box for jumping to a provider, running an action or opening a setting. The flyout
// is opened for a few seconds at a time, so the fastest path has to be the keyboard.
//
// Not modal: it closes on Escape and on a click outside, but nothing is made inert behind it, so
// it deliberately does not claim `aria-modal`. Options are reachable with the arrow keys through
// `aria-activedescendant`, which is why they are not tab stops; Escape restores focus to whatever
// opened the palette (the stage does that on close).
import { type KeyboardEvent, useEffect, useMemo, useRef, useState } from 'react';
import { Icon } from '../components/icons';
import { filterCommands, type PaletteCommand, step } from './paletteModel';

function optionId(command: PaletteCommand): string {
  return `nx-cmd-${command.id.replace(/[^a-z0-9-]/gi, '-')}`;
}

export function CommandPalette({
  commands,
  onClose,
}: {
  commands: PaletteCommand[];
  onClose: () => void;
}) {
  const [query, setQuery] = useState('');
  const [index, setIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const results = useMemo(() => filterCommands(commands, query), [commands, query]);
  const active = results.length === 0 ? undefined : results[Math.min(index, results.length - 1)];

  useEffect(() => {
    inputRef.current?.focus();
  }, []);
  useEffect(() => {
    if (active === undefined) return;
    listRef.current
      ?.querySelector<HTMLElement>('[data-active="true"]')
      ?.scrollIntoView?.({ block: 'nearest' });
  }, [active]);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        setIndex((current) => step(current, 1, results.length));
        break;
      case 'ArrowUp':
        event.preventDefault();
        setIndex((current) => step(current, -1, results.length));
        break;
      case 'Enter':
        if (active === undefined) return;
        event.preventDefault();
        onClose();
        active.run();
        break;
      case 'Escape':
        event.preventDefault();
        onClose();
        break;
      default:
        break;
    }
  };

  return (
    <div className="nx-scrim">
      <button
        type="button"
        className="nx-scrim-backdrop"
        aria-label="Fechar o comando rápido"
        onClick={onClose}
      />
      <div className="nx-palette" role="dialog" aria-label="Comando rápido" onKeyDown={onKeyDown}>
        <div className="nx-palette-field">
          <Icon name="sliders" />
          <input
            ref={inputRef}
            type="text"
            role="combobox"
            aria-expanded="true"
            aria-controls="nx-palette-list"
            aria-activedescendant={active === undefined ? undefined : optionId(active)}
            autoComplete="off"
            aria-label="Buscar comando"
            placeholder="Ir para provider, atualizar, copiar resumo…"
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setIndex(0);
            }}
          />
          <kbd>Esc</kbd>
        </div>
        <div id="nx-palette-list" ref={listRef} className="nx-palette-list" role="listbox">
          {results.length === 0 && <p className="nx-palette-empty">Nada encontrado</p>}
          {results.map((command) => (
            <button
              key={command.id}
              type="button"
              role="option"
              id={optionId(command)}
              className="nx-cmd"
              aria-selected={command === active}
              data-active={command === active}
              tabIndex={-1}
              onMouseMove={() => setIndex(results.indexOf(command))}
              onClick={() => {
                onClose();
                command.run();
              }}
            >
              <span className="nx-cmd-label">{command.label}</span>
              <span className="nx-cmd-hint">{command.hint}</span>
              <span className="nx-cmd-group">{command.group}</span>
            </button>
          ))}
        </div>
        <p className="nx-palette-foot">
          <kbd>↑</kbd>
          <kbd>↓</kbd> navegar · <kbd>Enter</kbd> executar
        </p>
      </div>
    </div>
  );
}
