import { z } from 'zod';

export const UPDATE_PHASES = [
  'idle',
  'checking',
  'current',
  'available',
  'downloading',
  'ready',
  'installing',
  'error',
  'unavailable',
] as const;
export const UPDATE_FAILURES = ['check', 'download', 'signature', 'install'] as const;
export const UpdateStatusSchema = z.strictObject({
  currentVersion: z.string(),
  phase: z.enum(UPDATE_PHASES),
  availableVersion: z.string().nullable(),
  notes: z.string().nullable(),
  downloadedBytes: z.number().int().nonnegative(),
  totalBytes: z.number().int().nonnegative().nullable(),
  checkedAt: z.number().int().nullable(),
  failure: z.enum(UPDATE_FAILURES).nullable(),
  installable: z.boolean(),
});
export type UpdateStatus = z.infer<typeof UpdateStatusSchema>;
