import { BackendError } from '../lib/backend';
import type { HarnessError } from '../lib/contract';

/** Anything that is not a BackendError collapses to `internal`: raw error text is never shown. */
export function toHarnessError(error: unknown): HarnessError {
  return error instanceof BackendError
    ? error.error
    : { code: 'internal', retryable: false, params: {} };
}
