// Small per-source additions to a ready view that the generic window layout cannot express.
import { useState } from 'react';
import { useNavigation } from '../../app/navigation';
import { t } from '../../i18n';
import type { HarnessId } from '../../lib/contract';
import { Icon } from '../icons';
import { LitellmForm } from '../setup/LitellmForm';
import { OpencodeLinks } from '../setup/OpencodeParts';
import { Note } from './Banner';

/** Opus, Sonnet and Fable need the opt-in `claude -p "/usage"` or usage API source; while it is off, say so instead of a gap. */
export function ClaudeLocked() {
  const navigation = useNavigation();
  return (
    <section className="flat">
      <h3 className="win-title">{t('claude.locked.title')}</h3>
      <Note icon="lock">{t('claude.locked.body')}</Note>
      <div className="actions">
        <button type="button" className="btn" onClick={() => navigation.openSettings('claude')}>
          {t('action.settings')}
        </button>
      </div>
    </section>
  );
}

function LitellmEditor() {
  const [editing, setEditing] = useState(false);
  return (
    <>
      <Note>{t('litellm.delay_note')}</Note>
      <div className="actions">
        <button
          type="button"
          className="btn"
          aria-expanded={editing}
          aria-controls="litellm-edit"
          onClick={() => setEditing((current) => !current)}
        >
          <Icon name="sliders" />
          {t('litellm.edit')}
        </button>
      </div>
      {editing && (
        <div id="litellm-edit">
          <LitellmForm onSaved={() => setEditing(false)} />
        </div>
      )}
    </>
  );
}

/** What closes a ready view for each source. */
export function HarnessFooter({ harness }: { harness: HarnessId }) {
  switch (harness) {
    case 'antigravity':
      return <Note>{t('antigravity.note')}</Note>;
    case 'opencode':
      return (
        <>
          <Note>{t('opencode.zen_note')}</Note>
          <OpencodeLinks showConsole={false} showBilling />
        </>
      );
    case 'kimi':
      return <Note>{t('kimi.note')}</Note>;
    case 'zai':
      return <Note>{t('zai.note')}</Note>;
    case 'litellm':
      return <LitellmEditor />;
    case 'claude':
    case 'codex':
      return null;
  }
}
