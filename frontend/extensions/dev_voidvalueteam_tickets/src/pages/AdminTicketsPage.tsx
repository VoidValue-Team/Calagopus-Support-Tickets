import { faClock, faGaugeHigh, faInbox, faTriangleExclamation } from '@fortawesome/free-solid-svg-icons';
import { SimpleGrid, Stack } from '@mantine/core';
import { Route, Routes } from 'react-router';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import StatCard from '@/elements/StatCard.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import getStatistics from '../api/tickets/getStatistics.ts';
import TicketTable from '../components/TicketTable.tsx';
import { useExtTranslations } from '../translations.ts';
import TicketDetailPage from './TicketDetailPage.tsx';

function AdminTicketsList() {
  const { t } = useExtTranslations();
  const canViewStatistics = useAdminCan('support.view-statistics');
  const statistics = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'statistics'],
    queryFn: getStatistics,
    enabled: canViewStatistics,
    silent: true,
  });
  return (
    <AdminContentContainer title={t('pages.admin.title', {})} subtitle={t('pages.admin.subtitle', {})}>
      <Stack>
        {canViewStatistics && (
          <SimpleGrid cols={{ base: 1, sm: 2, xl: 4 }}>
            <StatCard icon={faInbox} label={t('statistics.open', {})} value={String(statistics.data?.open ?? '—')} />
            <StatCard
              icon={faClock}
              label={t('statistics.awaitingStaff', {})}
              value={String(statistics.data?.awaitingStaff ?? '—')}
            />
            <StatCard
              icon={faTriangleExclamation}
              label={t('statistics.urgent', {})}
              value={String(statistics.data?.urgent ?? '—')}
            />
            <StatCard
              icon={faGaugeHigh}
              label={t('statistics.sla', {})}
              value={String(statistics.data?.slaBreached ?? '—')}
            />
          </SimpleGrid>
        )}
        <TicketTable scope='admin' />
      </Stack>
    </AdminContentContainer>
  );
}

export default function AdminTicketsPage() {
  return (
    <Routes>
      <Route path='/' element={<AdminTicketsList />} />
      <Route path='/:ticket' element={<TicketDetailPage scope='admin' />} />
    </Routes>
  );
}
