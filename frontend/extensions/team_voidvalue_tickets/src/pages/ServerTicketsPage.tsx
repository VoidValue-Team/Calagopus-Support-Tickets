import { useState } from 'react';
import { Route, Routes } from 'react-router';
import ServerContentContainer from '@/elements/containers/ServerContentContainer.tsx';
import { useServerStore } from '@/stores/server.ts';
import CreateTicketModal from '../components/CreateTicketModal.tsx';
import TicketTable from '../components/TicketTable.tsx';
import { useExtTranslations } from '../translations.ts';
import TicketDetailPage from './TicketDetailPage.tsx';

function ServerTicketsList() {
  const { t } = useExtTranslations();
  const serverUuid = useServerStore((state) => state.server.uuid);
  const [key, setKey] = useState(0);
  return (
    <ServerContentContainer
      title={t('pages.server.title', {})}
      contentRight={
        <CreateTicketModal scope='server' serverUuid={serverUuid} onCreated={() => setKey((value) => value + 1)} />
      }
    >
      <TicketTable key={key} scope='server' serverUuid={serverUuid} />
    </ServerContentContainer>
  );
}

export default function ServerTicketsPage() {
  const serverUuid = useServerStore((state) => state.server.uuid);
  return (
    <Routes>
      <Route path='/' element={<ServerTicketsList />} />
      <Route path='/:ticket' element={<TicketDetailPage scope='server' serverUuid={serverUuid} />} />
    </Routes>
  );
}
