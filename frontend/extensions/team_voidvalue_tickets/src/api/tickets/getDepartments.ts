import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { type Department, departmentSchema } from '../../schemas/tickets.ts';
import { type TicketScope, ticketBase } from './getTickets.ts';
export default async function getDepartments(
  scope: TicketScope = 'account',
  serverUuid?: string,
): Promise<Department[]> {
  const { data } = await axiosInstance.get(`${ticketBase(scope, serverUuid)}/departments`);
  return data.departments.map((v: unknown) => parseFromApi(departmentSchema, v));
}
