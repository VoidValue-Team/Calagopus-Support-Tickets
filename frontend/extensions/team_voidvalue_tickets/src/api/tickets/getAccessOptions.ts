import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';

const schema = z.object({
  enabled: z.boolean(),
  permissions: z.array(z.string()),
  defaultMinutes: z.number(),
  maxMinutes: z.number(),
});

export default async function getAccessOptions() {
  const { data } = await axiosInstance.get('/api/admin/extensions/team.voidvalue.tickets/access-options');
  return parseFromApi(schema, data);
}
