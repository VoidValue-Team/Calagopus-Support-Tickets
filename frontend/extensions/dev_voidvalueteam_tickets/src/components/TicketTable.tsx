import { Badge, Stack } from '@mantine/core';
import { useNavigate } from 'react-router';
import TextInput from '@/elements/input/TextInput.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TableLink from '@/elements/TableLink.tsx';
import { useSearchablePaginatedTable } from '@/plugins/useSearchablePaginatedTable.ts';
import getTickets, { type TicketScope } from '../api/tickets/getTickets.ts';
import { useTicketsStore } from '../stores/tickets.ts';
import { useExtTranslations } from '../translations.ts';
export default function TicketTable({ scope }: { scope: TicketScope }) {
  const { t } = useExtTranslations();
  const navigate = useNavigate();
  const { tickets, setTickets } = useTicketsStore();
  const { loading, error, search, setSearch, setPage } = useSearchablePaginatedTable({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'tickets', scope],
    fetcher: (page, term) => getTickets(scope, page, term),
    setStoreData: setTickets,
  });
  return (
    <Stack>
      <TextInput
        value={search}
        onChange={(event) => setSearch(event.currentTarget.value)}
        placeholder={t('actions.search', {})}
      />
      <Table
        columns={[
          t('columns.code', {}),
          t('columns.subject', {}),
          ...(scope === 'admin' ? [t('columns.customer', {})] : []),
          t('columns.server', {}),
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
          <TableRow
            key={ticket.uuid}
            className='cursor-pointer'
            tabIndex={0}
            onClick={() => navigate(`${scope === 'admin' ? '/admin' : '/account'}/support/${ticket.uuid}`)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                navigate(`${scope === 'admin' ? '/admin' : '/account'}/support/${ticket.uuid}`);
              }
            }}
          >
            <TableData>
              <TableLink to={`${scope === 'admin' ? '/admin' : '/account'}/support/${ticket.uuid}`}>
                {ticket.code}
              </TableLink>
            </TableData>
            <TableData>{ticket.subject}</TableData>
            {scope === 'admin' && <TableData>{ticket.userName}</TableData>}
            <TableData>{ticket.serverName ?? t('fields.noServer', {})}</TableData>
            <TableData>{ticket.departmentName}</TableData>
            <TableData>
              <Badge color={ticket.priority === 'urgent' ? 'red' : ticket.priority === 'high' ? 'orange' : 'blue'}>
                {t(`priorities.${ticket.priority}`, {})}
              </Badge>
            </TableData>
            <TableData>{t(`statuses.${ticket.status}`, {})}</TableData>
            <TableData>{ticket.lastReplyAt.toLocaleString()}</TableData>
            <TableData>
              {ticket.firstResponseSlaBreached || ticket.resolutionSlaBreached ? (
                <Badge color='red'>{t('sla.breached', {})}</Badge>
              ) : (
                <Badge color='green'>{t('sla.ok', {})}</Badge>
              )}
            </TableData>
          </TableRow>
        ))}
      </Table>
    </Stack>
  );
}
