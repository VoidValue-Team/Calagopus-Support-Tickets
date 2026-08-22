import { Route, Routes } from 'react-router';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import TicketTable from '../components/TicketTable.tsx';
import { useExtTranslations } from '../translations.ts';
import TicketDetailPage from './TicketDetailPage.tsx';

function AdminTicketsList() {
  const { t } = useExtTranslations();
  return (
    <AdminContentContainer title={t('pages.admin.title', {})} subtitle={t('pages.admin.subtitle', {})}>
      <TicketTable scope='admin' />
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
