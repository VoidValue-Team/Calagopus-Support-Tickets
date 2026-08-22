import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { supportServerSchema } from '../../schemas/tickets.ts';

export default async function getSupportServers() {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.voidvalueteam.tickets/tickets/servers');
  return parseFromApi(z.array(supportServerSchema), data.servers);
}
