import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type CreateTicket, createTicketSchema, ticketDetailSchema } from '../../schemas/tickets.ts';
import { type TicketScope, ticketBase } from './getTickets.ts';
export default async function createTicket(payload: CreateTicket, scope: TicketScope = 'account', serverUuid?: string) {
  const { data } = await axiosInstance.post(
    ticketBase(scope, serverUuid),
    serializeForApi(createTicketSchema, payload),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
