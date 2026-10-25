// A button that opens one fixed destination; the backend maps the link name to its URL.
import { useBackend } from '../../app/backendContext';
import { useAsyncAction } from '../../app/useAsyncAction';
import type { ExternalLink, HarnessId } from '../../lib/contract';
import { Icon } from '../icons';
import { ActionError } from '../panel/ErrorActions';

export function LinkButton({
  link,
  label,
  harness,
}: {
  link: ExternalLink;
  label: string;
  harness: HarnessId;
}) {
  const backend = useBackend();
  const open = useAsyncAction(() => backend.openLink(link));
  return (
    <>
      <div className="actions">
        <button
          type="button"
          className="btn"
          disabled={open.pending}
          onClick={() => void open.run()}
        >
          {label}
          <Icon name="external" />
        </button>
      </div>
      {open.error && <ActionError error={open.error} harness={harness} />}
    </>
  );
}
