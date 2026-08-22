import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type EditTicket, editTicketSchema, ticketDetailSchema } from '../../schemas/tickets.ts';

export default async function editTicket(ticketUuid: string, payload: EditTicket) {
  const { data } = await axiosInstance.put(
    `/api/admin/extensions/dev.voidvalueteam.tickets/tickets/${ticketUuid}/properties`,
    serializeForApi(editTicketSchema, payload),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
