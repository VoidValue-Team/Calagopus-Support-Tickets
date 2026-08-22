import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { settingsSchema } from '../schemas/settings.ts';
export default async function getSettings() {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.voidvalueteam.tickets/settings');
  return parseFromApi(settingsSchema, data.settings);
}
