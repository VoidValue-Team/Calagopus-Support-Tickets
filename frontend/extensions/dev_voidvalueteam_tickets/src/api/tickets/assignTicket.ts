import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { assignTicketSchema, ticketDetailSchema } from '../../schemas/tickets.ts';

export default async function assignTicket(ticketUuid: string, staffUuid: string | null) {
  const { data } = await axiosInstance.put(
    `/api/admin/extensions/dev.voidvalueteam.tickets/tickets/${ticketUuid}/assign`,
    serializeForApi(assignTicketSchema, { staffUuid }),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
