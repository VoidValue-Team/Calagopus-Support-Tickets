import { axiosInstance } from '@/api/axios.ts';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';
import { ticketSummarySchema } from '../../schemas/tickets.ts';
export type TicketScope = 'account' | 'admin' | { serverUuid: string };
export default async function getTickets(scope: TicketScope, page: number, search: string) {
  const base =
    scope === 'account'
      ? '/api/client/support/tickets'
      : scope === 'admin'
        ? '/api/admin/extensions/dev.voidvalueteam.tickets/tickets'
        : `/api/client/servers/${scope.serverUuid}/support/tickets`;
  const { data } = await axiosInstance.get(base, { params: { page, search } });
  return parsePaginationFromApi(ticketSummarySchema, data.tickets);
}
