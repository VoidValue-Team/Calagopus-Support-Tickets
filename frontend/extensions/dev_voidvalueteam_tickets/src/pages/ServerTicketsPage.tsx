import { useState } from 'react';
import ServerContentContainer from '@/elements/containers/ServerContentContainer.tsx';
import { useServerStore } from '@/stores/server.ts';
import CreateTicketModal from '../components/CreateTicketModal.tsx';
import TicketTable from '../components/TicketTable.tsx';
import { useExtTranslations } from '../translations.ts';
export default function ServerTicketsPage() {
  const { t } = useExtTranslations();
  const server = useServerStore((state) => state.server);
  const [key, setKey] = useState(0);
  return (
    <ServerContentContainer
      title={t('pages.server.title', {})}
      subtitle={t('pages.server.subtitle', {})}
      contentRight={<CreateTicketModal serverUuid={server.uuid} onCreated={() => setKey((v) => v + 1)} />}
    >
      <TicketTable key={key} scope={{ serverUuid: server.uuid }} />
    </ServerContentContainer>
  );
}
