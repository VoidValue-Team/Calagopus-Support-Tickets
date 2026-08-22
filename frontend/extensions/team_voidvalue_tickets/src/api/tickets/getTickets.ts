import { axiosInstance } from '@/api/axios.ts';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';
import { ticketSummarySchema } from '../../schemas/tickets.ts';
export type TicketScope = 'account' | 'admin' | 'server';
export type TicketView = 'all' | 'open' | 'closed';
export interface TicketFilters {
  status?: string | null;
  priority?: string | null;
  departmentUuid?: string | null;
  assignedStaffUuid?: string | null;
  serverSearch?: string;
  userSearch?: string;
  slaBreached?: boolean;
  createdFrom?: string;
  createdTo?: string;
  tag?: string;
}
export const ticketBase = (scope: TicketScope, serverUuid?: string) => {
  if (scope === 'account') return '/api/client/support/tickets';
  if (scope === 'server') {
    if (!serverUuid) throw new Error('Server UUID is required for server tickets.');
    return `/api/client/servers/${serverUuid}/extensions/team.voidvalue.tickets`;
  }
  return '/api/admin/extensions/team.voidvalue.tickets/tickets';
};
export default async function getTickets(
  scope: TicketScope,
  page: number,
  search: string,
  view: TicketView,
  serverUuid?: string,
  filters: TicketFilters = {},
) {
  const base = ticketBase(scope, serverUuid);
  const { data } = await axiosInstance.get(base, {
    params: { page, search, view: view === 'all' ? undefined : view, ...filters },
  });
  return parsePaginationFromApi(ticketSummarySchema, data.tickets);
}
