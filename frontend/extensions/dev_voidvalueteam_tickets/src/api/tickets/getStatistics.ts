import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';

export const statisticsSchema = z.object({
  open: z.number(),
  unassigned: z.number(),
  awaitingStaff: z.number(),
  awaitingCustomer: z.number(),
  urgent: z.number(),
  slaBreached: z.number(),
  resolvedToday: z.number(),
  createdToday: z.number(),
});

export default async function getStatistics() {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.voidvalueteam.tickets/statistics');
  return parseFromApi(statisticsSchema, data.statistics);
}
