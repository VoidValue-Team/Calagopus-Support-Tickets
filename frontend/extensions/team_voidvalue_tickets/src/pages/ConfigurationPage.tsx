import { Group, SimpleGrid, Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import NumberInput from '@/elements/input/NumberInput.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TagsInput from '@/elements/input/TagsInput.tsx';
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
    queryKey: ['extensions', 'team.voidvalue.tickets', 'settings'],
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
        addToast(t('notices.settingsSaved', {}), 'success');
      })
      .catch((error) => addToast(httpErrorToHuman(error), 'error'))
      .finally(() => setLoading(false));
  };
  return (
    <form onSubmit={form.onSubmit(save)}>
      <Stack>
        <TitleCard title={t('settings.sections.general', {})}>
          <SimpleGrid cols={{ base: 1, md: 2 }}>
            <Switch label={t('settings.enabled', {})} {...form.getInputProps('enabled', { type: 'checkbox' })} />
            <TextInput label={t('settings.ticketPrefix', {})} {...form.getInputProps('ticketPrefix')} />
            <Switch
              label={t('settings.allowUserPriority', {})}
              {...form.getInputProps('allowUserPriority', { type: 'checkbox' })}
            />
            <Switch
              label={t('settings.allowReopen', {})}
              {...form.getInputProps('allowReopen', { type: 'checkbox' })}
            />
            <NumberInput
              label={t('settings.reopenPeriodDays', {})}
              min={0}
              max={3650}
              allowDecimal={false}
              disabled={!form.values.allowReopen}
              {...form.getInputProps('reopenPeriodDays')}
            />
          </SimpleGrid>
        </TitleCard>
        <TitleCard title={t('settings.sections.email', {})}>
          <SimpleGrid cols={{ base: 1, md: 2 }}>
            <Switch
              label={t('settings.customerEmails', {})}
              {...form.getInputProps('customerEmails', { type: 'checkbox' })}
            />
            <Switch
              label={t('settings.staffEmails', {})}
              {...form.getInputProps('staffEmails', { type: 'checkbox' })}
            />
          </SimpleGrid>
        </TitleCard>
        <TitleCard title={t('settings.sections.attachments', {})}>
          <Stack>
            <SimpleGrid cols={{ base: 1, md: 3 }}>
              <Switch
                label={t('settings.attachmentsEnabled', {})}
                {...form.getInputProps('attachmentsEnabled', { type: 'checkbox' })}
              />
              <NumberInput
                label={t('settings.attachmentMaxSizeMb', {})}
                min={1}
                max={100}
                allowDecimal={false}
                disabled={!form.values.attachmentsEnabled}
                value={Math.round(form.values.attachmentMaxBytes / 1048576)}
                error={form.errors.attachmentMaxBytes}
                onChange={(value) =>
                  form.setFieldValue('attachmentMaxBytes', typeof value === 'number' ? value * 1048576 : 0)
                }
              />
              <NumberInput
                label={t('settings.attachmentMaxFiles', {})}
                min={1}
                max={20}
                allowDecimal={false}
                disabled={!form.values.attachmentsEnabled}
                {...form.getInputProps('attachmentMaxFiles')}
              />
            </SimpleGrid>
            <TagsInput
              label={t('settings.allowedMimeTypes', {})}
              placeholder={t('settings.allowedMimeTypesPlaceholder', {})}
              allowReordering={false}
              value={form.values.allowedMimeTypes}
              error={form.errors.allowedMimeTypes}
              onChange={(value) => form.setFieldValue('allowedMimeTypes', value)}
            />
          </Stack>
        </TitleCard>
        <TitleCard title={t('settings.sections.supportAccess', {})}>
          <Stack>
            <SimpleGrid cols={{ base: 1, md: 2 }}>
              <Switch
                label={t('settings.supportAccessEnabled', {})}
                {...form.getInputProps('supportAccessEnabled', { type: 'checkbox' })}
              />
              <Switch
                label={t('settings.revokeAccessOnResolved', {})}
                disabled={!form.values.supportAccessEnabled}
                {...form.getInputProps('revokeAccessOnResolved', { type: 'checkbox' })}
              />
              <NumberInput
                label={t('settings.supportAccessDefaultMinutes', {})}
                min={1}
                max={525600}
                allowDecimal={false}
                disabled={!form.values.supportAccessEnabled}
                {...form.getInputProps('supportAccessDefaultMinutes')}
              />
              <NumberInput
                label={t('settings.supportAccessMaxMinutes', {})}
                min={1}
                max={525600}
                allowDecimal={false}
                disabled={!form.values.supportAccessEnabled}
                {...form.getInputProps('supportAccessMaxMinutes')}
              />
            </SimpleGrid>
            <TagsInput
              label={t('settings.supportAccessPermissions', {})}
              placeholder={t('settings.supportAccessPermissionsPlaceholder', {})}
              allowReordering={false}
              value={form.values.supportAccessPermissions}
              error={form.errors.supportAccessPermissions}
              onChange={(value) => form.setFieldValue('supportAccessPermissions', value)}
            />
          </Stack>
        </TitleCard>
        <TitleCard title={t('settings.sections.automation', {})}>
          <SimpleGrid cols={{ base: 1, md: 3 }}>
            <Switch
              label={t('settings.autoCloseEnabled', {})}
              {...form.getInputProps('autoCloseEnabled', { type: 'checkbox' })}
            />
            <NumberInput
              label={t('settings.inactivityDays', {})}
              min={1}
              max={3650}
              allowDecimal={false}
              disabled={!form.values.autoCloseEnabled}
              {...form.getInputProps('inactivityDays')}
            />
            <NumberInput
              label={t('settings.finalCloseDelayDays', {})}
              min={1}
              max={3650}
              allowDecimal={false}
              disabled={!form.values.autoCloseEnabled}
              {...form.getInputProps('finalCloseDelayDays')}
            />
          </SimpleGrid>
        </TitleCard>
        <Group justify='flex-end'>
          <Button type='submit' loading={loading}>
            {t('actions.save', {})}
          </Button>
        </Group>
      </Stack>
    </form>
  );
}
