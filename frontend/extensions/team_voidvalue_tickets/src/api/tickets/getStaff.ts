import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { type StaffOption, staffOptionSchema } from '../../schemas/tickets.ts';

export default async function getStaff(): Promise<StaffOption[]> {
  const { data } = await axiosInstance.get('/api/admin/extensions/team.voidvalue.tickets/staff');
  return data.staff.map((staff: unknown) => parseFromApi(staffOptionSchema, staff));
}
