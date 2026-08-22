import { Badge, Stack } from '@mantine/core';
import TextInput from '@/elements/input/TextInput.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import { useSearchablePaginatedTable } from '@/plugins/useSearchablePaginatedTable.ts';
import getTickets, { type TicketScope } from '../api/tickets/getTickets.ts';
import { useTicketsStore } from '../stores/tickets.ts';
import { useExtTranslations } from '../translations.ts';
export default function TicketTable({ scope }: { scope: TicketScope }) {
  const { t } = useExtTranslations();
  const { tickets, setTickets } = useTicketsStore();
  const { loading, error, search, setSearch, setPage } = useSearchablePaginatedTable({
    queryKey: [
      'extensions',
      'dev.voidvalueteam.tickets',
      'tickets',
      typeof scope === 'string' ? scope : scope.serverUuid,
    ],
    fetcher: (page, term) => getTickets(scope, page, term),
    setStoreData: setTickets,
  });
  return (
    <Stack>
      <TextInput
        value={search}
        onChange={(event) => setSearch(event.currentTarget.value)}
        placeholder='Search tickets'
      />
      <Table
        columns={[
          t('columns.code', {}),
          t('columns.subject', {}),
          t('columns.department', {}),
          t('columns.priority', {}),
          t('columns.status', {}),
          t('columns.updated', {}),
          t('columns.sla', {}),
        ]}
        loading={loading}
        error={error ? String(error) : null}
        pagination={tickets}
        onPageSelect={setPage}
      >
        {tickets.data.map((ticket) => (
          <TableRow key={ticket.uuid}>
            <TableData>{ticket.code}</TableData>
            <TableData>{ticket.subject}</TableData>
            <TableData>{ticket.departmentName}</TableData>
            <TableData>
              <Badge color={ticket.priority === 'urgent' ? 'red' : ticket.priority === 'high' ? 'orange' : 'blue'}>
                {ticket.priority}
              </Badge>
            </TableData>
            <TableData>{ticket.status}</TableData>
            <TableData>{ticket.lastReplyAt.toLocaleString()}</TableData>
            <TableData>
              {ticket.firstResponseSlaBreached || ticket.resolutionSlaBreached ? (
                <Badge color='red'>Breached</Badge>
              ) : (
                <Badge color='green'>OK</Badge>
              )}
            </TableData>
          </TableRow>
        ))}
      </Table>
    </Stack>
  );
}
