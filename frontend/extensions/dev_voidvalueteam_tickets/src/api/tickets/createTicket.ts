import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type CreateTicket, createTicketSchema, ticketDetailSchema } from '../../schemas/tickets.ts';
export default async function createTicket(payload: CreateTicket) {
  const { data } = await axiosInstance.post(
    '/api/client/support/tickets',
    serializeForApi(createTicketSchema, payload),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
