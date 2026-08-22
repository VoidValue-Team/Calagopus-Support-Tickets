import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { type TicketAttachment, ticketDetailSchema } from '../../schemas/tickets.ts';
import { type TicketScope, ticketBase } from './getTickets.ts';

export async function uploadAttachments(
  scope: TicketScope,
  ticketUuid: string,
  messageUuid: string,
  files: File[],
  serverUuid?: string,
) {
  const payload = new FormData();
  payload.append('message_uuid', messageUuid);
  for (const file of files) payload.append('files', file);
  const { data } = await axiosInstance.post(`${ticketBase(scope, serverUuid)}/${ticketUuid}/attachments`, payload);
  return parseFromApi(ticketDetailSchema, data.ticket);
}

export async function downloadAttachment(
  scope: TicketScope,
  ticketUuid: string,
  attachment: TicketAttachment,
  serverUuid?: string,
) {
  const { data } = await axiosInstance.get(
    `${ticketBase(scope, serverUuid)}/${ticketUuid}/attachments/${attachment.uuid}`,
    {
      responseType: 'blob',
    },
  );
  const url = URL.createObjectURL(data as Blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = attachment.originalFilename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}
