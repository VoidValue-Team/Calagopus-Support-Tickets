import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type ReplyTicket, replyTicketSchema, ticketDetailSchema } from '../../schemas/tickets.ts';
import type { TicketScope } from './getTickets.ts';

export default async function replyTicket(scope: TicketScope, ticketUuid: string, payload: ReplyTicket) {
  const base =
    scope === 'account' ? '/api/client/support/tickets' : '/api/admin/extensions/dev.voidvalueteam.tickets/tickets';
  const { data } = await axiosInstance.post(`${base}/${ticketUuid}/reply`, serializeForApi(replyTicketSchema, payload));
  return parseFromApi(ticketDetailSchema, data.ticket);
}
