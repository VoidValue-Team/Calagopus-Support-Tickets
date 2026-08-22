import { SimpleGrid, Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import TitleCard from '@/elements/TitleCard.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import getSettings from '../api/getSettings.ts';
import updateSettings from '../api/updateSettings.ts';
import { type Settings, settingsSchema } from '../schemas/settings.ts';
import { useExtTranslations } from '../translations.ts';

const defaults: Settings = {
  enabled: true,
  ticketPrefix: 'TCK',
  allowUserPriority: true,
  allowReopen: true,
  reopenPeriodDays: 14,
  defaultDepartment: null,
  customerEmails: true,
  staffEmails: true,
  attachmentsEnabled: true,
  attachmentMaxBytes: 10485760,
  attachmentMaxFiles: 5,
  allowedMimeTypes: ['image/png', 'image/jpeg', 'text/plain', 'application/pdf'],
  supportAccessEnabled: false,
  supportAccessPermissions: ['control.read-console', 'files.read-content'],
  supportAccessDefaultMinutes: 240,
  supportAccessMaxMinutes: 43200,
  revokeAccessOnResolved: true,
  autoCloseEnabled: true,
  inactivityDays: 5,
  finalCloseDelayDays: 2,
};
export default function ConfigurationPage() {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const [loading, setLoading] = useState(false);
  const resource = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'settings'],
    queryFn: getSettings,
  });
  const form = useForm<Settings>({ initialValues: defaults, validate: zod4Resolver(settingsSchema) });
  useEffect(() => {
    if (resource.data) form.setValues(resource.data);
  }, [resource.data]);
  const save = (values: Settings) => {
    setLoading(true);
    updateSettings(values)
      .then((saved) => {
        form.setValues(saved);
        addToast('Settings saved.', 'success');
      })
      .catch((error) => addToast(httpErrorToHuman(error), 'error'))
      .finally(() => setLoading(false));
  };
  return (
    <TitleCard title={t('pages.settings.title', {})}>
      <form onSubmit={form.onSubmit(save)}>
        <Stack>
          <SimpleGrid cols={{ base: 1, md: 2 }}>
            <Switch label='Enable support tickets' {...form.getInputProps('enabled', { type: 'checkbox' })} />
            <TextInput label='Ticket prefix' {...form.getInputProps('ticketPrefix')} />
            <Switch
              label='Allow customers to select priority'
              {...form.getInputProps('allowUserPriority', { type: 'checkbox' })}
            />
            <Switch label='Allow reopening tickets' {...form.getInputProps('allowReopen', { type: 'checkbox' })} />
            <Switch
              label='Customer email notifications'
              {...form.getInputProps('customerEmails', { type: 'checkbox' })}
            />
            <Switch label='Staff email notifications' {...form.getInputProps('staffEmails', { type: 'checkbox' })} />
            <Switch label='Enable attachments' {...form.getInputProps('attachmentsEnabled', { type: 'checkbox' })} />
            <Switch label='Enable auto-close' {...form.getInputProps('autoCloseEnabled', { type: 'checkbox' })} />
          </SimpleGrid>
          <Button type='submit' loading={loading}>
            Save
          </Button>
        </Stack>
      </form>
    </TitleCard>
  );
}
