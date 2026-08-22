import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { ticketDetailSchema } from '../../schemas/tickets.ts';

const schema = z.object({ tags: z.array(z.string().min(1).max(40)).max(20) });

export default async function updateTags(ticketUuid: string, tags: string[]) {
  const { data } = await axiosInstance.put(
    `/api/admin/extensions/team.voidvalue.tickets/tickets/${ticketUuid}/tags`,
    serializeForApi(schema, { tags }),
  );
  return parseFromApi(ticketDetailSchema, data.ticket);
}
