import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type ReplyTicket, replyTicketSchema, ticketDetailSchema } from '../../schemas/tickets.ts';
import { type TicketScope, ticketBase } from './getTickets.ts';

export default async function replyTicket(
  scope: TicketScope,
  ticketUuid: string,
  payload: ReplyTicket,
  serverUuid?: string,
) {
  const base = ticketBase(scope, serverUuid);
  const { data } = await axiosInstance.post(`${base}/${ticketUuid}/reply`, serializeForApi(replyTicketSchema, payload));
  return parseFromApi(ticketDetailSchema, data.ticket);
}
