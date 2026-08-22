import { Group, Text } from '@mantine/core';
import { axiosInstance } from '@/api/axios.ts';
import { useResource } from '@/plugins/useResource.ts';

interface Stats {
  open: number;
  urgent: number;
  sla_breached: number;
}
async function getStats() {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.voidvalueteam.tickets/statistics');
  return data.statistics as Stats;
}
export default function CardSummary() {
  const { data } = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'statistics'],
    queryFn: getStats,
    silent: true,
  });
  return (
    <Group gap='md'>
      <Text size='sm'>Open: {data?.open ?? '—'}</Text>
      <Text size='sm'>Urgent: {data?.urgent ?? '—'}</Text>
      <Text size='sm'>SLA: {data?.sla_breached ?? '—'}</Text>
    </Group>
  );
}
