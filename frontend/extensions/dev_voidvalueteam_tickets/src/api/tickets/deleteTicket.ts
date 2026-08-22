import { axiosInstance } from '@/api/axios.ts';

export default async function deleteTicket(ticketUuid: string) {
  await axiosInstance.delete(`/api/admin/extensions/dev.voidvalueteam.tickets/tickets/${ticketUuid}`);
}
