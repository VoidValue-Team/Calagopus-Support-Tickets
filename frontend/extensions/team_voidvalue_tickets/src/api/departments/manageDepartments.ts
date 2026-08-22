import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import {
  type Department,
  type DepartmentPayload,
  departmentPayloadSchema,
  departmentSchema,
} from '../../schemas/tickets.ts';

const base = '/api/admin/extensions/team.voidvalue.tickets/departments';

export async function getAdminDepartments(): Promise<Department[]> {
  const { data } = await axiosInstance.get(base);
  return data.departments.map((department: unknown) => parseFromApi(departmentSchema, department));
}

export async function createDepartment(payload: DepartmentPayload): Promise<void> {
  await axiosInstance.post(base, serializeForApi(departmentPayloadSchema, payload));
}

export async function updateDepartment(uuid: string, payload: DepartmentPayload): Promise<void> {
  await axiosInstance.put(`${base}/${uuid}`, serializeForApi(departmentPayloadSchema, payload));
}

export async function deleteDepartment(uuid: string): Promise<void> {
  await axiosInstance.delete(`${base}/${uuid}`);
}
