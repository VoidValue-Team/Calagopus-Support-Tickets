import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { type Department, departmentSchema } from '../../schemas/tickets.ts';
export default async function getDepartments(): Promise<Department[]> {
  const { data } = await axiosInstance.get('/api/client/support/tickets/departments');
  return data.departments.map((v: unknown) => parseFromApi(departmentSchema, v));
}
