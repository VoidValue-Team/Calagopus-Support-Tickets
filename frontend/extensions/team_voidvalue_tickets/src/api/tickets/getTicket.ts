import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { ticketDetailSchema } from '../../schemas/tickets.ts';
import { type TicketScope, ticketBase } from './getTickets.ts';

export default async function getTicket(scope: TicketScope, ticketUuid: string, serverUuid?: string) {
  const base = ticketBase(scope, serverUuid);
  const { data } = await axiosInstance.get(`${base}/${ticketUuid}`);
  return parseFromApi(ticketDetailSchema, data.ticket);
}
