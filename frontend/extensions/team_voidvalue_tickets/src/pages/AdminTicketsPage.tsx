import { faClock, faCog, faGaugeHigh, faInbox, faTriangleExclamation } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { SimpleGrid, Stack } from '@mantine/core';
import { Route, Routes, useNavigate } from 'react-router';
import Button from '@/elements/Button.tsx';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import StatCard from '@/elements/StatCard.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import getStatistics from '../api/tickets/getStatistics.ts';
import TicketTable from '../components/TicketTable.tsx';
import { useExtTranslations } from '../translations.ts';
import AdminManagementPage from './AdminManagementPage.tsx';
import TicketDetailPage from './TicketDetailPage.tsx';

function AdminTicketsList() {
  const { t } = useExtTranslations();
  const canViewStatistics = useAdminCan('support.view-statistics');
  const canManageDepartments = useAdminCan('support.manage-departments');
  const canManageSavedReplies = useAdminCan('support.manage-saved-replies');
  const canManage = canManageDepartments || canManageSavedReplies;
  const navigate = useNavigate();
  const statistics = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'statistics'],
    queryFn: getStatistics,
    enabled: canViewStatistics,
    silent: true,
  });
  return (
    <AdminContentContainer
      title={t('pages.admin.title', {})}
      subtitle={t('pages.admin.subtitle', {})}
      contentRight={
        canManage ? (
          <Button variant='default' leftSection={<FontAwesomeIcon icon={faCog} />} onClick={() => navigate('manage')}>
            {t('actions.manage', {})}
          </Button>
        ) : undefined
      }
    >
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
      <Route path='/manage' element={<AdminManagementPage />} />
    </Routes>
  );
}
