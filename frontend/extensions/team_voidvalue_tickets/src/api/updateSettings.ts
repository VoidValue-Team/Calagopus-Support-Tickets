import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type Settings, settingsSchema } from '../schemas/settings.ts';
export default async function updateSettings(settings: Settings) {
  const { data } = await axiosInstance.put(
    '/api/admin/extensions/team.voidvalue.tickets/settings',
    serializeForApi(settingsSchema, settings),
  );
  return parseFromApi(settingsSchema, data.settings);
}
