import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type CreateTicket, createTicketSchema, ticketDetailSchema } from '../../schemas/tickets.ts';
export default async function createTicket(payload: CreateTicket, serverUuid?: string) {
  const url = serverUuid ? `/api/client/servers/${serverUuid}/support/tickets` : '/api/client/support/tickets';
  const { data } = await axiosInstance.post(url, serializeForApi(createTicketSchema, payload));
  return parseFromApi(ticketDetailSchema, data.ticket);
}
