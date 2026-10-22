import { t } from '../i18n';
import type { UpdateStatus } from '../lib/contract';

/** Deterministic demonstration; never downloads or launches an installer. */
export class MockUpdater {
  status: UpdateStatus = {
    currentVersion: '0.1.0',
    phase: 'idle',
    availableVersion: null,
    notes: null,
    downloadedBytes: 0,
    totalBytes: null,
    checkedAt: null,
    failure: null,
    installable: true,
  };
  readonly listeners = new Set<(status: UpdateStatus) => void>();

  private emit(patch: Partial<UpdateStatus>) {
    this.status = { ...this.status, ...patch };
    for (const listener of this.listeners) listener(structuredClone(this.status));
    return structuredClone(this.status);
  }

  check(now: number) {
    return this.emit({
      phase: 'available',
      availableVersion: '0.2.0',
      checkedAt: now,
      notes: t('mock.update_notes'),
      failure: null,
    });
  }

  download() {
    return this.emit({ phase: 'ready', downloadedBytes: 8_388_608, totalBytes: 8_388_608 });
  }

  install() {
    return this.emit({ phase: 'installing' });
  }
}
