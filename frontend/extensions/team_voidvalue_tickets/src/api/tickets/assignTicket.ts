import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { ticketDetailSchema } from '../../schemas/tickets.ts';

export default async function assignTicket(ticketUuid: string, staffUuid: string | null) {
  const { data } = await axiosInstance.put(
    `/api/admin/extensions/team.voidvalue.tickets/tickets/${ticketUuid}/assign`,
    { staff_uuid: staffUuid },
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
