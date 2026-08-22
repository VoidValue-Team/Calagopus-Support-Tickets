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
export const supportAgentSchema = z.object({
  uuid: z.string().uuid(),
  username: z.string(),
});
export const ticketDetailSchema = ticketSummarySchema.extend({
  messages: z.array(messageSchema),
  attachments: z.array(attachmentSchema),
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
export const updateTicketStatusSchema = z.object({ status: ticketStatusSchema });
export const assignTicketSchema = z.object({ staffUuid: z.string().uuid().nullable() });
export type TicketSummary = z.infer<typeof ticketSummarySchema>;
export type TicketDetail = z.infer<typeof ticketDetailSchema>;
export type TicketAttachment = z.infer<typeof attachmentSchema>;
export type SupportAgent = z.infer<typeof supportAgentSchema>;
export type Department = z.infer<typeof departmentSchema>;
export type CreateTicket = z.infer<typeof createTicketSchema>;
export type ReplyTicket = z.infer<typeof replyTicketSchema>;
export type TicketStatus = z.infer<typeof ticketStatusSchema>;
