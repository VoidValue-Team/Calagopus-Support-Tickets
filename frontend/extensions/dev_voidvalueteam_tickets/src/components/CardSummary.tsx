import { Group, Text } from '@mantine/core';
import { useResource } from '@/plugins/useResource.ts';
import getStatistics from '../api/tickets/getStatistics.ts';
import { useExtTranslations } from '../translations.ts';
export default function CardSummary() {
  const { t } = useExtTranslations();
  const { data } = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'statistics'],
    queryFn: getStatistics,
    silent: true,
  });
  return (
    <Group gap='md'>
      <Text size='sm'>
        {t('statistics.open', {})}: {data?.open ?? '—'}
      </Text>
      <Text size='sm'>
        {t('statistics.urgent', {})}: {data?.urgent ?? '—'}
      </Text>
      <Text size='sm'>
        {t('statistics.sla', {})}: {data?.slaBreached ?? '—'}
      </Text>
    </Group>
  );
}
