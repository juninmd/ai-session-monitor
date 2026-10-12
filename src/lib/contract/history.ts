// Mirror of crates/usage-model/src/history.rs: the past windows of one harness, for the burndown.
import { z } from 'zod';
import { HarnessIdSchema } from './view';

export const MAX_EPOCHS_PER_WINDOW = 8;
export const MAX_POINTS_PER_EPOCH = 120;

export const HistoryPointSchema = z.strictObject({
  t: z.number().int(),
  usedPct: z.number().min(0).max(100),
});
export type HistoryPoint = z.infer<typeof HistoryPointSchema>;

export const HistoryEpochSchema = z.strictObject({
  windowId: z.string(),
  resetsAt: z.number().int(),
  points: z.array(HistoryPointSchema).max(MAX_POINTS_PER_EPOCH),
});
export type HistoryEpoch = z.infer<typeof HistoryEpochSchema>;

export const HarnessHistorySchema = z.strictObject({
  harness: HarnessIdSchema,
  epochs: z.array(HistoryEpochSchema),
});
export type HarnessHistory = z.infer<typeof HarnessHistorySchema>;
