import { z } from 'zod';
export const ticketStatusSchema = z.enum([
  'open',
  'awaiting_staff',
  'awaiting_customer',
  'in_progress',
  'resolved',
  'closed',
]);
export const ticketPrioritySchema = z.enum(['low', 'normal', 'high', 'urgent']);
export const ticketSummarySchema = z.object({
  uuid: z.string().uuid(),
  number: z.number(),
  code: z.string(),
  userUuid: z.string().uuid(),
  userName: z.string(),
  serverUuid: z.string().uuid().nullable(),
  serverName: z.string().nullable(),
  departmentUuid: z.string().uuid(),
  departmentName: z.string(),
  subject: z.string(),
  status: ticketStatusSchema,
  priority: ticketPrioritySchema,
  assignedStaffUuid: z.string().uuid().nullable(),
  assignedStaffName: z.string().nullable(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date(),
  lastReplyAt: z.coerce.date(),
  firstResponseDueAt: z.coerce.date(),
  resolutionDueAt: z.coerce.date(),
  firstResponseSlaBreached: z.boolean(),
  resolutionSlaBreached: z.boolean(),
});
export const messageSchema = z.object({
  uuid: z.string().uuid(),
  ticketUuid: z.string().uuid(),
  authorUuid: z.string().uuid().nullable(),
  authorName: z.string().nullable(),
  messageType: z.enum(['customer', 'staff', 'internal_note', 'system']),
  body: z.string(),
  createdAt: z.coerce.date(),
  editedAt: z.coerce.date().nullable(),
});
export const attachmentSchema = z.object({
  uuid: z.string().uuid(),
  messageUuid: z.string().uuid(),
  uploaderUuid: z.string().uuid().nullable(),
  originalFilename: z.string(),
  mimeType: z.string(),
  sizeBytes: z.number(),
  sha256: z.string().nullable(),
  createdAt: z.coerce.date(),
});
export const accessStatusSchema = z.enum(['pending', 'approved', 'rejected', 'revoked', 'expired']);
export const accessRequestSchema = z.object({
  uuid: z.string().uuid(),
  requestedByUuid: z.string().uuid(),
  requestedByName: z.string(),
  permissions: z.array(z.string()),
  requestedDurationMinutes: z.number(),
  reason: z.string(),
  status: accessStatusSchema,
  reviewedByUuid: z.string().uuid().nullable(),
  reviewedByName: z.string().nullable(),
  createdAt: z.coerce.date(),
  reviewedAt: z.coerce.date().nullable(),
});
export const accessGrantSchema = z.object({
  uuid: z.string().uuid(),
  requestUuid: z.string().uuid().nullable(),
  serverUuid: z.string().uuid(),
  userUuid: z.string().uuid(),
  userName: z.string(),
  permissions: z.array(z.string()),
  reason: z.string(),
  grantedAt: z.coerce.date(),
  expiresAt: z.coerce.date(),
  revokedAt: z.coerce.date().nullable(),
  revokeReason: z.string().nullable(),
});
export const ticketDetailSchema = ticketSummarySchema.extend({
  messages: z.array(messageSchema),
  attachments: z.array(attachmentSchema),
  history: z.array(
    z.object({
      uuid: z.string().uuid(),
      actorUuid: z.string().uuid().nullable(),
      actorName: z.string().nullable(),
      event: z.string(),
      data: z.record(z.string(), z.unknown()),
      createdAt: z.coerce.date(),
    }),
  ),
  accessRequests: z.array(accessRequestSchema),
  accessGrants: z.array(accessGrantSchema),
  tags: z.array(z.object({ uuid: z.string().uuid(), name: z.string(), color: z.string() })),
});
export const departmentSchema = z.object({
  uuid: z.string().uuid(),
  name: z.string(),
  description: z.string(),
  enabled: z.boolean(),
  position: z.number(),
  defaultPriority: ticketPrioritySchema,
  firstResponseSlaMinutes: z.number(),
  resolutionSlaMinutes: z.number(),
  autoresponse: z.string().nullable(),
  allowServerAccess: z.boolean(),
  notificationEnabled: z.boolean(),
});
export const createTicketSchema = z.object({
  subject: z.string().min(3).max(180),
  message: z.string().min(1).max(20000),
  departmentUuid: z.string().uuid(),
  serverUuid: z.string().uuid().nullable().optional(),
  priority: ticketPrioritySchema.optional(),
});
export const replyTicketSchema = z.object({
  message: z.string().min(1).max(20000),
  internalNote: z.boolean().optional(),
});
export const departmentPayloadSchema = departmentSchema.omit({ uuid: true });
export const savedReplySchema = z.object({
  uuid: z.string().uuid(),
  title: z.string(),
  body: z.string(),
  departmentUuid: z.string().uuid().nullable(),
  enabled: z.boolean(),
  createdByUuid: z.string().uuid().nullable(),
  createdAt: z.coerce.date(),
  updatedAt: z.coerce.date(),
});
export const savedReplyPayloadSchema = savedReplySchema.pick({
  title: true,
  body: true,
  departmentUuid: true,
  enabled: true,
});
export const staffOptionSchema = z.object({ uuid: z.string().uuid(), username: z.string() });
export const updateTicketStatusSchema = z.object({ status: ticketStatusSchema });
export type TicketSummary = z.infer<typeof ticketSummarySchema>;
export type TicketDetail = z.infer<typeof ticketDetailSchema>;
export type TicketAttachment = z.infer<typeof attachmentSchema>;
export type Department = z.infer<typeof departmentSchema>;
export type CreateTicket = z.infer<typeof createTicketSchema>;
export type ReplyTicket = z.infer<typeof replyTicketSchema>;
export type TicketStatus = z.infer<typeof ticketStatusSchema>;
export type DepartmentPayload = z.infer<typeof departmentPayloadSchema>;
export type SavedReply = z.infer<typeof savedReplySchema>;
export type SavedReplyPayload = z.infer<typeof savedReplyPayloadSchema>;
export type StaffOption = z.infer<typeof staffOptionSchema>;
