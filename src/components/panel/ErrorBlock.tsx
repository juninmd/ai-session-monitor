import { describeError } from '../../app/errorText';
import type { HarnessError, HarnessId } from '../../lib/contract';
import { Icon } from '../icons';
import { ActionError, useErrorAction } from './ErrorActions';

/** A failed read: what happened in pt-BR and exactly one way forward. */
export function ErrorBlock({ error, harness }: { error: HarnessError; harness: HarnessId }) {
  const text = describeError(error, harness);
  const task = useErrorAction(text.action, harness);
  return (
    <section className="card notice" data-status="crit" role="status">
      <span className="badge">
        <Icon name="warn" />
      </span>
      <h2>{text.title}</h2>
      <p>{text.body}</p>
      <div className="actions">
        <button
          type="button"
          className="btn primary"
          disabled={task.pending}
          onClick={() => void task.run()}
        >
          {task.label}
        </button>
      </div>
      {task.error && <ActionError error={task.error} harness={harness} />}
    </section>
  );
}
