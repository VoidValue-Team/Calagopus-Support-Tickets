import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi, serializeForApi } from '@/lib/api-transform.ts';
import {
  type SavedReply,
  type SavedReplyPayload,
  savedReplyPayloadSchema,
  savedReplySchema,
} from '../../schemas/tickets.ts';

const base = '/api/admin/extensions/team.voidvalue.tickets/saved-replies';

export async function getSavedReplies(search?: string): Promise<SavedReply[]> {
  const { data } = await axiosInstance.get(base, { params: { search: search || undefined } });
  return data.saved_replies.map((reply: unknown) => parseFromApi(savedReplySchema, reply));
}

export async function createSavedReply(payload: SavedReplyPayload): Promise<void> {
  await axiosInstance.post(base, serializeForApi(savedReplyPayloadSchema, payload));
}

export async function updateSavedReply(uuid: string, payload: SavedReplyPayload): Promise<void> {
  await axiosInstance.put(`${base}/${uuid}`, serializeForApi(savedReplyPayloadSchema, payload));
}

export async function deleteSavedReply(uuid: string): Promise<void> {
  await axiosInstance.delete(`${base}/${uuid}`);
}
