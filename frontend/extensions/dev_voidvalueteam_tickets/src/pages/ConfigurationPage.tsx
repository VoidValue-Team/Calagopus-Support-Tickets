import { Alert, SimpleGrid, Stack, Text } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import NumberInput from '@/elements/input/NumberInput.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TagsInput from '@/elements/input/TagsInput.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import TitleCard from '@/elements/TitleCard.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { getAdminDepartments } from '../api/departments.ts';
import getSettings from '../api/getSettings.ts';
import updateSettings from '../api/updateSettings.ts';
import DepartmentManager from '../components/DepartmentManager.tsx';
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
  const canManageDepartments = useAdminCan('support.manage-departments');
  const resource = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'settings'],
    queryFn: getSettings,
  });
  const departments = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'admin-departments'],
    queryFn: getAdminDepartments,
    silent: true,
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
    <TitleCard title={t('pages.settings.title', {})}>
      <Stack gap='xl'>
        <form onSubmit={form.onSubmit(save)}>
          <Stack gap='md'>
            <Card>
              <Stack>
                <div>
                  <Text fw={600}>{t('configuration.general.title', {})}</Text>
                  <Text size='sm' c='dimmed'>
                    {t('configuration.general.description', {})}
                  </Text>
                </div>
                <SimpleGrid cols={{ base: 1, md: 2 }}>
                  <Switch
                    label={t('configuration.enabled', {})}
                    {...form.getInputProps('enabled', { type: 'checkbox' })}
                  />
                  <TextInput label={t('configuration.ticketPrefix', {})} {...form.getInputProps('ticketPrefix')} />
                  <Select
                    label={t('configuration.defaultDepartment', {})}
                    description={t('configuration.defaultDepartmentDescription', {})}
                    data={(departments.data ?? []).map((department) => ({
                      value: department.uuid,
                      label: department.name,
                    }))}
                    allowDeselect
                    clearable
                    {...form.getInputProps('defaultDepartment')}
                  />
                  <Switch
                    label={t('configuration.allowUserPriority', {})}
                    {...form.getInputProps('allowUserPriority', { type: 'checkbox' })}
                  />
                </SimpleGrid>
              </Stack>
            </Card>

            <Card>
              <Stack>
                <div>
                  <Text fw={600}>{t('configuration.lifecycle.title', {})}</Text>
                  <Text size='sm' c='dimmed'>
                    {t('configuration.lifecycle.description', {})}
                  </Text>
                </div>
                <SimpleGrid cols={{ base: 1, md: 2 }}>
                  <Switch
                    label={t('configuration.allowReopen', {})}
                    {...form.getInputProps('allowReopen', { type: 'checkbox' })}
                  />
                  <NumberInput
                    label={t('configuration.reopenPeriodDays', {})}
                    min={0}
                    suffix={` ${t('units.days', {})}`}
                    {...form.getInputProps('reopenPeriodDays')}
                  />
                  <Switch
                    label={t('configuration.autoCloseEnabled', {})}
                    {...form.getInputProps('autoCloseEnabled', { type: 'checkbox' })}
                  />
                  <NumberInput
                    label={t('configuration.inactivityDays', {})}
                    min={1}
                    suffix={` ${t('units.days', {})}`}
                    {...form.getInputProps('inactivityDays')}
                  />
                  <NumberInput
                    label={t('configuration.finalCloseDelayDays', {})}
                    min={1}
                    suffix={` ${t('units.days', {})}`}
                    {...form.getInputProps('finalCloseDelayDays')}
                  />
                </SimpleGrid>
              </Stack>
            </Card>

            <Card>
              <Stack>
                <div>
                  <Text fw={600}>{t('configuration.notifications.title', {})}</Text>
                  <Text size='sm' c='dimmed'>
                    {t('configuration.notifications.description', {})}
                  </Text>
                </div>
                <SimpleGrid cols={{ base: 1, md: 2 }}>
                  <Switch
                    label={t('configuration.customerEmails', {})}
                    {...form.getInputProps('customerEmails', { type: 'checkbox' })}
                  />
                  <Switch
                    label={t('configuration.staffEmails', {})}
                    {...form.getInputProps('staffEmails', { type: 'checkbox' })}
                  />
                </SimpleGrid>
              </Stack>
            </Card>

            <Card>
              <Stack>
                <div>
                  <Text fw={600}>{t('configuration.attachments.title', {})}</Text>
                  <Text size='sm' c='dimmed'>
                    {t('configuration.attachments.description', {})}
                  </Text>
                </div>
                <Switch
                  label={t('configuration.attachmentsEnabled', {})}
                  {...form.getInputProps('attachmentsEnabled', { type: 'checkbox' })}
                />
                <SimpleGrid cols={{ base: 1, md: 2 }}>
                  <NumberInput
                    label={t('configuration.attachmentMaxBytes', {})}
                    min={1}
                    step={1048576}
                    thousandSeparator
                    suffix=' bytes'
                    {...form.getInputProps('attachmentMaxBytes')}
                  />
                  <NumberInput
                    label={t('configuration.attachmentMaxFiles', {})}
                    min={1}
                    {...form.getInputProps('attachmentMaxFiles')}
                  />
                </SimpleGrid>
                <TagsInput
                  label={t('configuration.allowedMimeTypes', {})}
                  description={t('configuration.allowedMimeTypesDescription', {})}
                  value={form.values.allowedMimeTypes}
                  onChange={(value) => form.setFieldValue('allowedMimeTypes', value)}
                  error={form.errors.allowedMimeTypes}
                  placeholder='image/png'
                />
              </Stack>
            </Card>

            <Card>
              <Stack>
                <div>
                  <Text fw={600}>{t('configuration.access.title', {})}</Text>
                  <Text size='sm' c='dimmed'>
                    {t('configuration.access.description', {})}
                  </Text>
                </div>
                <Alert color='yellow' variant='light'>
                  {t('configuration.access.warning', {})}
                </Alert>
                <SimpleGrid cols={{ base: 1, md: 2 }}>
                  <Switch
                    label={t('configuration.supportAccessEnabled', {})}
                    {...form.getInputProps('supportAccessEnabled', { type: 'checkbox' })}
                  />
                  <Switch
                    label={t('configuration.revokeAccessOnResolved', {})}
                    {...form.getInputProps('revokeAccessOnResolved', { type: 'checkbox' })}
                  />
                  <NumberInput
                    label={t('configuration.supportAccessDefaultMinutes', {})}
                    min={1}
                    suffix={` ${t('units.minutes', {})}`}
                    {...form.getInputProps('supportAccessDefaultMinutes')}
                  />
                  <NumberInput
                    label={t('configuration.supportAccessMaxMinutes', {})}
                    min={1}
                    suffix={` ${t('units.minutes', {})}`}
                    {...form.getInputProps('supportAccessMaxMinutes')}
                  />
                </SimpleGrid>
                <TagsInput
                  label={t('configuration.supportAccessPermissions', {})}
                  description={t('configuration.supportAccessPermissionsDescription', {})}
                  value={form.values.supportAccessPermissions}
                  onChange={(value) => form.setFieldValue('supportAccessPermissions', value)}
                  error={form.errors.supportAccessPermissions}
                  placeholder='control.read-console'
                />
              </Stack>
            </Card>

            <Button type='submit' loading={loading}>
              {t('actions.saveChanges', {})}
            </Button>
          </Stack>
        </form>

        {canManageDepartments && (
          <Card>
            <DepartmentManager />
          </Card>
        )}
      </Stack>
    </TitleCard>
  );
}
