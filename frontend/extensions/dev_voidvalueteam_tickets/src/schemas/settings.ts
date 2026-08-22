import { z } from 'zod';
export const settingsSchema = z.object({
  enabled: z.boolean(),
  ticketPrefix: z.string().min(1).max(12),
  allowUserPriority: z.boolean(),
  allowReopen: z.boolean(),
  reopenPeriodDays: z.number().int().min(0),
  defaultDepartment: z.string().uuid().nullable(),
  customerEmails: z.boolean(),
  staffEmails: z.boolean(),
  attachmentsEnabled: z.boolean(),
  attachmentMaxBytes: z.number().int().positive(),
  attachmentMaxFiles: z.number().int().positive(),
  allowedMimeTypes: z.array(z.string()),
  supportAccessEnabled: z.boolean(),
  supportAccessPermissions: z.array(z.string()),
  supportAccessDefaultMinutes: z.number().int().positive(),
  supportAccessMaxMinutes: z.number().int().positive(),
  revokeAccessOnResolved: z.boolean(),
  autoCloseEnabled: z.boolean(),
  inactivityDays: z.number().int().positive(),
  finalCloseDelayDays: z.number().int().positive(),
});
export type Settings = z.infer<typeof settingsSchema>;
