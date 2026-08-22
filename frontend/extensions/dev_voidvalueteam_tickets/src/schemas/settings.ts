import { z } from 'zod';
export const settingsSchema = z
  .object({
    enabled: z.boolean(),
    ticketPrefix: z.string().trim().min(1).max(12),
    allowUserPriority: z.boolean(),
    allowReopen: z.boolean(),
    reopenPeriodDays: z.number().int().min(0),
    defaultDepartment: z.string().uuid().nullable(),
    customerEmails: z.boolean(),
    staffEmails: z.boolean(),
    attachmentsEnabled: z.boolean(),
    attachmentMaxBytes: z
      .number()
      .int()
      .positive()
      .max(60 * 1024 * 1024),
    attachmentMaxFiles: z.number().int().positive().max(20),
    allowedMimeTypes: z.array(z.string().trim().min(1)).min(1),
    supportAccessEnabled: z.boolean(),
    supportAccessPermissions: z.array(z.string().trim().min(1)),
    supportAccessDefaultMinutes: z.number().int().positive(),
    supportAccessMaxMinutes: z.number().int().positive(),
    revokeAccessOnResolved: z.boolean(),
    autoCloseEnabled: z.boolean(),
    inactivityDays: z.number().int().positive(),
    finalCloseDelayDays: z.number().int().positive(),
  })
  .refine((settings) => settings.supportAccessDefaultMinutes <= settings.supportAccessMaxMinutes, {
    path: ['supportAccessDefaultMinutes'],
    message: 'Default duration cannot exceed maximum duration.',
  });
export type Settings = z.infer<typeof settingsSchema>;

export const publicSettingsSchema = settingsSchema.pick({
  enabled: true,
  allowUserPriority: true,
  allowReopen: true,
  reopenPeriodDays: true,
  defaultDepartment: true,
  attachmentsEnabled: true,
  attachmentMaxBytes: true,
  attachmentMaxFiles: true,
  allowedMimeTypes: true,
});
export type PublicSettings = z.infer<typeof publicSettingsSchema>;
