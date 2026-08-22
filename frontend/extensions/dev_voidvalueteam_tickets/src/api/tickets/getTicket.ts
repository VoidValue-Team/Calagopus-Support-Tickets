import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { ticketDetailSchema } from '../../schemas/tickets.ts';
import type { TicketScope } from './getTickets.ts';

export default async function getTicket(scope: TicketScope, ticketUuid: string) {
  const base =
    scope === 'account' ? '/api/client/support/tickets' : '/api/admin/extensions/dev.voidvalueteam.tickets/tickets';
  const { data } = await axiosInstance.get(`${base}/${ticketUuid}`);
  return parseFromApi(ticketDetailSchema, data.ticket);
}
