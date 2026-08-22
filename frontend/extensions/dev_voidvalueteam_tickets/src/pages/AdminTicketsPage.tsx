import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import TicketTable from '../components/TicketTable.tsx';
import { useExtTranslations } from '../translations.ts';
export default function AdminTicketsPage() {
  const { t } = useExtTranslations();
  return (
    <AdminContentContainer title={t('pages.admin.title', {})} subtitle={t('pages.admin.subtitle', {})}>
      <TicketTable scope='admin' />
    </AdminContentContainer>
  );
}
