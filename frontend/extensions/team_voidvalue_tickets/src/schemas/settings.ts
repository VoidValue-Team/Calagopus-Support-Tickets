import { z } from 'zod';
export const settingsSchema = z
  .object({
    enabled: z.boolean(),
    ticketPrefix: z.string().min(1).max(12),
    allowUserPriority: z.boolean(),
    allowReopen: z.boolean(),
    reopenPeriodDays: z.number().int().min(0).max(3650),
    defaultDepartment: z.string().uuid().nullable(),
    customerEmails: z.boolean(),
    staffEmails: z.boolean(),
    attachmentsEnabled: z.boolean(),
    attachmentMaxBytes: z.number().int().min(1024).max(104857600),
    attachmentMaxFiles: z.number().int().min(1).max(20),
    allowedMimeTypes: z.array(z.string().min(3).max(128)).min(1).max(32),
    supportAccessEnabled: z.boolean(),
    supportAccessPermissions: z.array(z.string().min(1).max(128)).max(64),
    supportAccessDefaultMinutes: z.number().int().min(1).max(525600),
    supportAccessMaxMinutes: z.number().int().min(1).max(525600),
    revokeAccessOnResolved: z.boolean(),
    autoCloseEnabled: z.boolean(),
    inactivityDays: z.number().int().min(1).max(3650),
    finalCloseDelayDays: z.number().int().min(1).max(3650),
  })
  .refine((settings) => settings.supportAccessDefaultMinutes <= settings.supportAccessMaxMinutes, {
    path: ['supportAccessDefaultMinutes'],
    message: 'Default support access duration exceeds its maximum.',
  });
export type Settings = z.infer<typeof settingsSchema>;
