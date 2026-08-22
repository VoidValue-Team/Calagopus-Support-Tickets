import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type TicketStatus, ticketDetailSchema, updateTicketStatusSchema } from '../../schemas/tickets.ts';
import { type TicketScope, ticketBase } from './getTickets.ts';

export default async function updateTicketStatus(
  scope: TicketScope,
  ticketUuid: string,
  status: TicketStatus,
  serverUuid?: string,
) {
  const base = ticketBase(scope, serverUuid);
  const { data } = await axiosInstance.put(
    `${base}/${ticketUuid}/status`,
    serializeForApi(updateTicketStatusSchema, { status }),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
