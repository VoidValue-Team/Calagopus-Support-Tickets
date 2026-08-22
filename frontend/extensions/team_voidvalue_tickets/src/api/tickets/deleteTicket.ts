import { axiosInstance } from '@/api/axios.ts';

export default async function deleteTicket(ticketUuid: string) {
  await axiosInstance.delete(`/api/admin/extensions/team.voidvalue.tickets/tickets/${ticketUuid}`);
}
