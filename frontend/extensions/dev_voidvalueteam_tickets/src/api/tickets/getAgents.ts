import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { supportAgentSchema } from '../../schemas/tickets.ts';

export default async function getAgents() {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.voidvalueteam.tickets/tickets/agents');
  return parseFromApi(z.array(supportAgentSchema), data.agents);
}
