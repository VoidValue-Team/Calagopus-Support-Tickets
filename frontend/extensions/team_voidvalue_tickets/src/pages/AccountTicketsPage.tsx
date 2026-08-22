import { useState } from 'react';
import { Route, Routes } from 'react-router';
import AccountContentContainer from '@/elements/containers/AccountContentContainer.tsx';
import CreateTicketModal from '../components/CreateTicketModal.tsx';
import TicketTable from '../components/TicketTable.tsx';
import { useExtTranslations } from '../translations.ts';
import TicketDetailPage from './TicketDetailPage.tsx';

function AccountTicketsList() {
  const { t } = useExtTranslations();
  const [key, setKey] = useState(0);
  return (
    <AccountContentContainer
      title={t('pages.account.title', {})}
      subtitle={t('pages.account.subtitle', {})}
      contentRight={<CreateTicketModal onCreated={() => setKey((v) => v + 1)} />}
    >
      <TicketTable key={key} scope='account' />
    </AccountContentContainer>
  );
}

export default function AccountTicketsPage() {
  return (
    <Routes>
      <Route path='/' element={<AccountTicketsList />} />
      <Route path='/:ticket' element={<TicketDetailPage scope='account' />} />
    </Routes>
  );
}
