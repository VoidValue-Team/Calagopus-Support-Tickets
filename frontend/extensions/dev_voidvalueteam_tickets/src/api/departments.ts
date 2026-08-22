import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import { type DepartmentPayload, departmentPayloadSchema, departmentSchema } from '../schemas/tickets.ts';

const base = '/api/admin/extensions/dev.voidvalueteam.tickets/departments';

export async function getAdminDepartments() {
  const { data } = await axiosInstance.get(base);
  return parseFromApi(z.array(departmentSchema), data.departments);
}

export async function saveDepartment(uuid: string | null, payload: DepartmentPayload) {
  const serialized = serializeForApi(departmentPayloadSchema, payload);
  const { data } = uuid
    ? await axiosInstance.put(`${base}/${uuid}`, serialized)
    : await axiosInstance.post(base, serialized);
  return parseFromApi(z.array(departmentSchema), data.departments);
}

export async function deleteDepartment(uuid: string) {
  const { data } = await axiosInstance.delete(`${base}/${uuid}`);
  return parseFromApi(z.array(departmentSchema), data.departments);
}
