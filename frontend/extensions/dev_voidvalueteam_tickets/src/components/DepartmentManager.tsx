import { faPen, faPlus, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Badge, Group, SimpleGrid, Stack, Text } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import NumberInput from '@/elements/input/NumberInput.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { deleteDepartment, getAdminDepartments, saveDepartment } from '../api/departments.ts';
import { type Department, type DepartmentPayload, departmentPayloadSchema } from '../schemas/tickets.ts';
import { useExtTranslations } from '../translations.ts';

const defaults: DepartmentPayload = {
  name: '',
  description: '',
  enabled: true,
  position: 0,
  defaultPriority: 'normal',
  firstResponseSlaMinutes: 720,
  resolutionSlaMinutes: 2880,
  autoresponse: '',
  allowServerAccess: false,
  notificationEnabled: true,
};

export default function DepartmentManager() {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const [opened, setOpened] = useState(false);
  const [editing, setEditing] = useState<Department | null>(null);
  const [deleting, setDeleting] = useState<Department | null>(null);
  const [loading, setLoading] = useState(false);
  const resource = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'admin-departments'],
    queryFn: getAdminDepartments,
  });
  const form = useForm<DepartmentPayload>({
    initialValues: defaults,
    validate: zod4Resolver(departmentPayloadSchema),
  });

  useEffect(() => {
    if (!opened) return;
    form.setValues(
      editing
        ? {
            name: editing.name,
            description: editing.description,
            enabled: editing.enabled,
            position: editing.position,
            defaultPriority: editing.defaultPriority,
            firstResponseSlaMinutes: editing.firstResponseSlaMinutes,
            resolutionSlaMinutes: editing.resolutionSlaMinutes,
            autoresponse: editing.autoresponse ?? '',
            allowServerAccess: editing.allowServerAccess,
            notificationEnabled: editing.notificationEnabled,
          }
        : defaults,
    );
  }, [opened, editing?.uuid]);

  const submit = async (values: DepartmentPayload) => {
    setLoading(true);
    try {
      await saveDepartment(editing?.uuid ?? null, values);
      await resource.refetch();
      addToast(t(editing ? 'notices.departmentUpdated' : 'notices.departmentCreated', {}), 'success');
      setOpened(false);
      setEditing(null);
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Stack>
      <Group justify='space-between'>
        <div>
          <Text fw={600}>{t('configuration.departments.title', {})}</Text>
          <Text size='sm' c='dimmed'>
            {t('configuration.departments.description', {})}
          </Text>
        </div>
        <Button
          leftSection={<FontAwesomeIcon icon={faPlus} />}
          onClick={() => {
            setEditing(null);
            setOpened(true);
          }}
        >
          {t('actions.createDepartment', {})}
        </Button>
      </Group>
      <SimpleGrid cols={{ base: 1, lg: 2 }}>
        {(resource.data ?? []).map((department) => (
          <Card key={department.uuid}>
            <Group justify='space-between' align='flex-start'>
              <div>
                <Group gap='xs'>
                  <Text fw={600}>{department.name}</Text>
                  <Badge color={department.enabled ? 'green' : 'gray'}>
                    {department.enabled ? t('common.enabled', {}) : t('common.disabled', {})}
                  </Badge>
                </Group>
                <Text size='sm' c='dimmed' mt='xs'>
                  {department.description || t('configuration.departments.noDescription', {})}
                </Text>
                <Text size='xs' c='dimmed' mt='sm'>
                  {t('configuration.departments.summary', {
                    priority: t(`priorities.${department.defaultPriority}`, {}),
                    first: department.firstResponseSlaMinutes,
                    resolution: department.resolutionSlaMinutes,
                  })}
                </Text>
              </div>
              <Group gap='xs'>
                <Button
                  size='compact-sm'
                  variant='default'
                  leftSection={<FontAwesomeIcon icon={faPen} />}
                  onClick={() => {
                    setEditing(department);
                    setOpened(true);
                  }}
                >
                  {t('actions.edit', {})}
                </Button>
                <Button
                  size='compact-sm'
                  color='red'
                  variant='light'
                  leftSection={<FontAwesomeIcon icon={faTrash} />}
                  onClick={() => setDeleting(department)}
                >
                  {t('actions.delete', {})}
                </Button>
              </Group>
            </Group>
          </Card>
        ))}
      </SimpleGrid>

      <Modal
        opened={opened}
        onClose={() => setOpened(false)}
        title={editing ? t('actions.editDepartment', {}) : t('actions.createDepartment', {})}
        size='lg'
      >
        <form onSubmit={form.onSubmit(submit)}>
          <Stack>
            <SimpleGrid cols={{ base: 1, sm: 2 }}>
              <TextInput label={t('fields.name', {})} {...form.getInputProps('name')} />
              <NumberInput label={t('fields.position', {})} min={0} {...form.getInputProps('position')} />
            </SimpleGrid>
            <TextArea label={t('fields.description', {})} minRows={2} {...form.getInputProps('description')} />
            <SimpleGrid cols={{ base: 1, sm: 3 }}>
              <Select
                label={t('fields.defaultPriority', {})}
                data={(['low', 'normal', 'high', 'urgent'] as const).map((priority) => ({
                  value: priority,
                  label: t(`priorities.${priority}`, {}),
                }))}
                {...form.getInputProps('defaultPriority')}
              />
              <NumberInput
                label={t('fields.firstResponseSla', {})}
                min={1}
                {...form.getInputProps('firstResponseSlaMinutes')}
              />
              <NumberInput
                label={t('fields.resolutionSla', {})}
                min={1}
                {...form.getInputProps('resolutionSlaMinutes')}
              />
            </SimpleGrid>
            <TextArea label={t('fields.autoresponse', {})} minRows={3} {...form.getInputProps('autoresponse')} />
            <SimpleGrid cols={{ base: 1, sm: 3 }}>
              <Switch label={t('fields.enabled', {})} {...form.getInputProps('enabled', { type: 'checkbox' })} />
              <Switch
                label={t('fields.allowServerAccess', {})}
                {...form.getInputProps('allowServerAccess', { type: 'checkbox' })}
              />
              <Switch
                label={t('fields.notificationEnabled', {})}
                {...form.getInputProps('notificationEnabled', { type: 'checkbox' })}
              />
            </SimpleGrid>
            <ModalFooter>
              <Button type='submit' loading={loading}>
                {t('actions.saveChanges', {})}
              </Button>
            </ModalFooter>
          </Stack>
        </form>
      </Modal>

      <ConfirmationModal
        opened={Boolean(deleting)}
        onClose={() => setDeleting(null)}
        title={t('actions.deleteDepartment', {})}
        confirm={t('actions.deletePermanently', {})}
        onConfirmed={async () => {
          if (!deleting) return;
          await deleteDepartment(deleting.uuid);
          await resource.refetch();
          addToast(t('notices.departmentDeleted', {}), 'success');
          setDeleting(null);
        }}
      >
        <Text>{t('warnings.deleteDepartment', { name: deleting?.name ?? '' })}</Text>
      </ConfirmationModal>
    </Stack>
  );
}
