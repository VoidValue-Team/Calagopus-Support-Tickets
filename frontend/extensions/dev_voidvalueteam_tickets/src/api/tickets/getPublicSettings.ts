import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { publicSettingsSchema } from '../../schemas/settings.ts';
import type { TicketScope } from './getTickets.ts';

export default async function getPublicSettings(scope: TicketScope) {
  const base =
    scope === 'admin' ? '/api/admin/extensions/dev.voidvalueteam.tickets/tickets' : '/api/client/support/tickets';
  const { data } = await axiosInstance.get(`${base}/configuration`);
  return parseFromApi(publicSettingsSchema, data.configuration);
}
