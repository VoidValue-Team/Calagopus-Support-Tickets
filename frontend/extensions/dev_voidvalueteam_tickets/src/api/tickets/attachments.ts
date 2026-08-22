import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { type TicketAttachment, ticketDetailSchema } from '../../schemas/tickets.ts';
import type { TicketScope } from './getTickets.ts';

const ticketBase = (scope: TicketScope) =>
  scope === 'account' ? '/api/client/support/tickets' : '/api/admin/extensions/dev.voidvalueteam.tickets/tickets';

export async function uploadAttachments(scope: TicketScope, ticketUuid: string, messageUuid: string, files: File[]) {
  const payload = new FormData();
  payload.append('message_uuid', messageUuid);
  for (const file of files) payload.append('files', file);
  const { data } = await axiosInstance.post(`${ticketBase(scope)}/${ticketUuid}/attachments`, payload, {
    headers: { 'Content-Type': 'multipart/form-data' },
  });
  return parseFromApi(ticketDetailSchema, data.ticket);
}

export async function downloadAttachment(scope: TicketScope, ticketUuid: string, attachment: TicketAttachment) {
  const { data } = await axiosInstance.get(`${ticketBase(scope)}/${ticketUuid}/attachments/${attachment.uuid}`, {
    responseType: 'blob',
  });
  const url = URL.createObjectURL(data as Blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = attachment.originalFilename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}
