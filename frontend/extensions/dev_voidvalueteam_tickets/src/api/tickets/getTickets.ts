import { axiosInstance } from '@/api/axios.ts';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';
import { ticketSummarySchema } from '../../schemas/tickets.ts';
export type TicketScope = 'account' | 'admin';
export type TicketView = 'all' | 'open' | 'closed';
export default async function getTickets(scope: TicketScope, page: number, search: string, view: TicketView) {
  const base =
    scope === 'account' ? '/api/client/support/tickets' : '/api/admin/extensions/dev.voidvalueteam.tickets/tickets';
  const { data } = await axiosInstance.get(base, { params: { page, search, view: view === 'all' ? undefined : view } });
  return parsePaginationFromApi(ticketSummarySchema, data.tickets);
}
