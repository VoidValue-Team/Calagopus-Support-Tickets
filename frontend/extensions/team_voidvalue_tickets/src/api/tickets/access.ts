import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { ticketDetailSchema } from '../../schemas/tickets.ts';
import { type TicketScope, ticketBase } from './getTickets.ts';

const requestSchema = z.object({
  permissions: z.array(z.string()).min(1).max(64),
  durationMinutes: z.number().int().positive(),
  reason: z.string().min(3).max(500),
});

const reviewSchema = z.object({ action: z.enum(['approve', 'reject']) });

export async function requestAccess(ticketUuid: string, payload: z.infer<typeof requestSchema>) {
  const { data } = await axiosInstance.post(
    `/api/admin/extensions/team.voidvalue.tickets/tickets/${ticketUuid}/access-requests`,
    serializeForApi(requestSchema, payload),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}

export async function reviewAccess(
  scope: Exclude<TicketScope, 'admin'>,
  ticketUuid: string,
  requestUuid: string,
  action: 'approve' | 'reject',
  serverUuid?: string,
) {
  const { data } = await axiosInstance.put(
    `${ticketBase(scope, serverUuid)}/${ticketUuid}/access-requests/${requestUuid}`,
    serializeForApi(reviewSchema, { action }),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}

export async function revokeAccess(ticketUuid: string, grantUuid: string) {
  const { data } = await axiosInstance.delete(
    `/api/admin/extensions/team.voidvalue.tickets/tickets/${ticketUuid}/access-grants/${grantUuid}`,
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
