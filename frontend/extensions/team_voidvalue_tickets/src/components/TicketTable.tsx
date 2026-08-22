import { faInbox } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Badge, Center, SegmentedControl, SimpleGrid, Stack, Text } from '@mantine/core';
import { useState } from 'react';
import { useNavigate } from 'react-router';
import Card from '@/elements/Card.tsx';
import Select from '@/elements/input/Select.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TableLink from '@/elements/TableLink.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useSearchablePaginatedTable } from '@/plugins/useSearchablePaginatedTable.ts';
import { getAdminDepartments } from '../api/departments/manageDepartments.ts';
import getStaff from '../api/tickets/getStaff.ts';
import getTickets, { type TicketScope, type TicketView } from '../api/tickets/getTickets.ts';
import { getDepartmentName } from '../lib/departments.ts';
import { useTicketsStore } from '../stores/tickets.ts';
import { useExtTranslations } from '../translations.ts';

const statusColor = (status: string) => {
  if (status === 'closed' || status === 'resolved') return 'gray';
  if (status === 'awaiting_staff') return 'orange';
  if (status === 'awaiting_customer') return 'cyan';
  if (status === 'in_progress') return 'violet';
  return 'blue';
};

export default function TicketTable({ scope, serverUuid }: { scope: TicketScope; serverUuid?: string }) {
  const { t } = useExtTranslations();
  const navigate = useNavigate();
  const [view, setView] = useState<TicketView>('open');
  const [status, setStatus] = useState<string | null>(null);
  const [priority, setPriority] = useState<string | null>(null);
  const [departmentUuid, setDepartmentUuid] = useState<string | null>(null);
  const [assignedStaffUuid, setAssignedStaffUuid] = useState<string | null>(null);
  const [serverSearch, setServerSearch] = useState('');
  const [userSearch, setUserSearch] = useState('');
  const [slaBreached, setSlaBreached] = useState<string | null>(null);
  const [createdFrom, setCreatedFrom] = useState('');
  const [createdTo, setCreatedTo] = useState('');
  const [tag, setTag] = useState('');
  const canAssign = useAdminCan('support.assign');
  const departments = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'filter-departments'],
    queryFn: getAdminDepartments,
    enabled: scope === 'admin',
    silent: true,
  });
  const staff = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'staff'],
    queryFn: getStaff,
    enabled: scope === 'admin' && canAssign,
    silent: true,
  });
  const { tickets, setTickets } = useTicketsStore();
  const { loading, error, search, setSearch, setPage } = useSearchablePaginatedTable({
    queryKey: [
      'extensions',
      'team.voidvalue.tickets',
      'tickets',
      scope,
      view,
      status,
      priority,
      departmentUuid,
      assignedStaffUuid,
      serverSearch,
      userSearch,
      slaBreached,
      createdFrom,
      createdTo,
      tag,
    ],
    fetcher: (page, term) =>
      getTickets(scope, page, term, view, serverUuid, {
        status,
        priority,
        departmentUuid,
        assignedStaffUuid,
        serverSearch: serverSearch || undefined,
        userSearch: userSearch || undefined,
        slaBreached: slaBreached === 'true' ? true : undefined,
        createdFrom: createdFrom ? `${createdFrom}T00:00:00Z` : undefined,
        createdTo: createdTo ? `${createdTo}T23:59:59Z` : undefined,
        tag: tag || undefined,
      }),
    setStoreData: setTickets,
  });
  return (
    <Stack>
      <SegmentedControl
        value={view}
        onChange={(value) => {
          setView(value as TicketView);
          setPage(1);
        }}
        data={[
          { value: 'open', label: t('categories.open', {}) },
          { value: 'closed', label: t('categories.closed', {}) },
          { value: 'all', label: t('categories.all', {}) },
        ]}
      />
      <TextInput
        value={search}
        onChange={(event) => setSearch(event.currentTarget.value)}
        placeholder={t('actions.search', {})}
      />
      {scope === 'admin' && (
        <SimpleGrid cols={{ base: 1, sm: 2, lg: 4 }}>
          <Select
            label={t('fields.status', {})}
            clearable
            data={(['open', 'awaiting_staff', 'awaiting_customer', 'in_progress', 'resolved', 'closed'] as const).map(
              (value) => ({ value, label: t(`statuses.${value}`, {}) }),
            )}
            value={status}
            onChange={setStatus}
          />
          <Select
            label={t('fields.priority', {})}
            clearable
            data={(['low', 'normal', 'high', 'urgent'] as const).map((value) => ({
              value,
              label: t(`priorities.${value}`, {}),
            }))}
            value={priority}
            onChange={setPriority}
          />
          <Select
            label={t('fields.department', {})}
            clearable
            searchable
            data={(departments.data ?? []).map((department) => ({
              value: department.uuid,
              label: getDepartmentName(department, t),
            }))}
            value={departmentUuid}
            onChange={setDepartmentUuid}
          />
          {canAssign && (
            <Select
              label={t('fields.assignedTo', {})}
              clearable
              searchable
              data={(staff.data ?? []).map((entry) => ({ value: entry.uuid, label: entry.username }))}
              value={assignedStaffUuid}
              onChange={setAssignedStaffUuid}
            />
          )}
          <TextInput
            label={t('filters.customer', {})}
            value={userSearch}
            onChange={(event) => setUserSearch(event.currentTarget.value)}
          />
          <TextInput
            label={t('filters.server', {})}
            value={serverSearch}
            onChange={(event) => setServerSearch(event.currentTarget.value)}
          />
          <Select
            label={t('filters.sla', {})}
            clearable
            data={[{ value: 'true', label: t('sla.breached', {}) }]}
            value={slaBreached}
            onChange={setSlaBreached}
          />
          <TextInput
            type='date'
            label={t('filters.createdFrom', {})}
            value={createdFrom}
            onChange={(event) => setCreatedFrom(event.currentTarget.value)}
          />
          <TextInput
            type='date'
            label={t('filters.createdTo', {})}
            value={createdTo}
            onChange={(event) => setCreatedTo(event.currentTarget.value)}
          />
          <TextInput label={t('filters.tag', {})} value={tag} onChange={(event) => setTag(event.currentTarget.value)} />
        </SimpleGrid>
      )}
      {!loading && !error && tickets.total === 0 ? (
        <Card>
          <Center py='xl'>
            <Stack align='center' c='dimmed' gap='xs'>
              <FontAwesomeIcon icon={faInbox} size='3x' />
              <Text ta='center'>{t('empty', {})}</Text>
            </Stack>
          </Center>
        </Card>
      ) : (
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
              onClick={() => navigate(ticket.uuid)}
              onKeyDown={(event) => {
                if (event.key === 'Enter' || event.key === ' ') {
                  event.preventDefault();
                  navigate(ticket.uuid);
                }
              }}
            >
              <TableData>
                <TableLink to={ticket.uuid}>{ticket.code}</TableLink>
              </TableData>
              <TableData>{ticket.subject}</TableData>
              {scope === 'admin' && <TableData>{ticket.userName}</TableData>}
              <TableData>{ticket.serverName ?? t('fields.noServer', {})}</TableData>
              <TableData>
                {getDepartmentName({ uuid: ticket.departmentUuid, name: ticket.departmentName }, t)}
              </TableData>
              <TableData>
                <Badge color={ticket.priority === 'urgent' ? 'red' : ticket.priority === 'high' ? 'orange' : 'blue'}>
                  {t(`priorities.${ticket.priority}`, {})}
                </Badge>
              </TableData>
              <TableData>
                <Badge color={statusColor(ticket.status)}>{t(`statuses.${ticket.status}`, {})}</Badge>
              </TableData>
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
      )}
    </Stack>
  );
}
