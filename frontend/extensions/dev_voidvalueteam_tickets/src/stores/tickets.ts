import { create } from 'zustand';
import type { TicketSummary } from '../schemas/tickets.ts';

interface Store {
  tickets: Pagination<TicketSummary>;
  setTickets: (tickets: Pagination<TicketSummary>) => void;
}
export const useTicketsStore = create<Store>((set) => ({
  tickets: { total: 0, perPage: 25, page: 1, data: [] },
  setTickets: (tickets) => set({ tickets }),
}));
