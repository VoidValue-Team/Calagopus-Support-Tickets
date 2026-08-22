import { faArrowLeft, faEdit, faPlus, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { ActionIcon, Badge, Group, SimpleGrid, Stack, Tabs, Text } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useState } from 'react';
import { useNavigate } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import NumberInput from '@/elements/input/NumberInput.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import {
  createDepartment,
  deleteDepartment,
  getAdminDepartments,
  updateDepartment,
} from '../api/departments/manageDepartments.ts';
import {
  createSavedReply,
  deleteSavedReply,
  getSavedReplies,
  updateSavedReply,
} from '../api/savedReplies/manageSavedReplies.ts';
import { getDepartmentDescription, getDepartmentName } from '../lib/departments.ts';
import {
  type Department,
  type DepartmentPayload,
  departmentPayloadSchema,
  type SavedReply,
  type SavedReplyPayload,
  savedReplyPayloadSchema,
} from '../schemas/tickets.ts';
import { useExtTranslations } from '../translations.ts';

const emptyDepartment: DepartmentPayload = {
  name: '',
  description: '',
  enabled: true,
  position: 0,
  defaultPriority: 'normal',
  firstResponseSlaMinutes: 720,
  resolutionSlaMinutes: 2880,
  autoresponse: null,
  allowServerAccess: false,
  notificationEnabled: true,
};

function DepartmentsManager() {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const [editing, setEditing] = useState<Department | null>(null);
  const [opened, setOpened] = useState(false);
  const [deleting, setDeleting] = useState<Department | null>(null);
  const [saving, setSaving] = useState(false);
  const departments = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'admin-departments'],
    queryFn: getAdminDepartments,
  });
  const form = useForm<DepartmentPayload>({
    initialValues: emptyDepartment,
    validate: zod4Resolver(departmentPayloadSchema),
  });
  const openEditor = (department?: Department) => {
    setEditing(department ?? null);
    form.setValues(department ? departmentPayloadSchema.parse(department) : emptyDepartment);
    setOpened(true);
  };
  const submit = async (payload: DepartmentPayload) => {
    setSaving(true);
    try {
      if (editing) await updateDepartment(editing.uuid, payload);
      else await createDepartment(payload);
      await departments.refetch();
      setOpened(false);
      addToast(t('notices.departmentSaved', {}), 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSaving(false);
    }
  };
  return (
    <Stack>
      <Group justify='space-between'>
        <Text fw={600}>{t('management.departments.title', {})}</Text>
        <Button leftSection={<FontAwesomeIcon icon={faPlus} />} onClick={() => openEditor()}>
          {t('management.departments.create', {})}
        </Button>
      </Group>
      <SimpleGrid cols={{ base: 1, md: 2 }}>
        {(departments.data ?? []).map((department) => (
          <Card key={department.uuid}>
            <Group justify='space-between' align='flex-start'>
              <div>
                <Group gap='xs'>
                  <Text fw={600}>{getDepartmentName(department, t)}</Text>
                  <Badge color={department.enabled ? 'green' : 'gray'}>
                    {t(department.enabled ? 'common.enabled' : 'common.disabled', {})}
                  </Badge>
                </Group>
                <Text size='sm' c='dimmed' mt='xs'>
                  {getDepartmentDescription(department, t) || t('common.noDescription', {})}
                </Text>
              </div>
              <Group gap='xs'>
                <ActionIcon variant='default' onClick={() => openEditor(department)} aria-label={t('actions.edit', {})}>
                  <FontAwesomeIcon icon={faEdit} />
                </ActionIcon>
                <ActionIcon
                  color='red'
                  variant='light'
                  onClick={() => setDeleting(department)}
                  aria-label={t('actions.delete', {})}
                >
                  <FontAwesomeIcon icon={faTrash} />
                </ActionIcon>
              </Group>
            </Group>
          </Card>
        ))}
      </SimpleGrid>
      <Modal opened={opened} onClose={() => setOpened(false)} title={t('management.departments.editor', {})} size='lg'>
        <form onSubmit={form.onSubmit(submit)}>
          <Stack>
            <TextInput label={t('fields.name', {})} {...form.getInputProps('name')} />
            <TextArea label={t('fields.description', {})} {...form.getInputProps('description')} />
            <SimpleGrid cols={{ base: 1, sm: 2 }}>
              <NumberInput label={t('fields.position', {})} min={0} {...form.getInputProps('position')} />
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
            <TextArea label={t('fields.autoresponse', {})} {...form.getInputProps('autoresponse')} />
            <Switch label={t('common.enabled', {})} {...form.getInputProps('enabled', { type: 'checkbox' })} />
            <Switch
              label={t('fields.allowServerAccess', {})}
              {...form.getInputProps('allowServerAccess', { type: 'checkbox' })}
            />
            <Switch
              label={t('fields.notifications', {})}
              {...form.getInputProps('notificationEnabled', { type: 'checkbox' })}
            />
            <ModalFooter>
              <Button type='submit' loading={saving}>
                {t('actions.save', {})}
              </Button>
            </ModalFooter>
          </Stack>
        </form>
      </Modal>
      <ConfirmationModal
        opened={Boolean(deleting)}
        onClose={() => setDeleting(null)}
        title={t('management.departments.delete', {})}
        confirm={t('actions.delete', {})}
        onConfirmed={async () => {
          if (!deleting) return;
          try {
            await deleteDepartment(deleting.uuid);
            await departments.refetch();
            setDeleting(null);
            addToast(t('notices.departmentDeleted', {}), 'success');
          } catch (error) {
            addToast(httpErrorToHuman(error), 'error');
          }
        }}
      >
        <Text>{t('management.departments.deleteWarning', { name: deleting?.name ?? '' })}</Text>
      </ConfirmationModal>
    </Stack>
  );
}

const emptySavedReply: SavedReplyPayload = { title: '', body: '', departmentUuid: null, enabled: true };

function SavedRepliesManager() {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const [editing, setEditing] = useState<SavedReply | null>(null);
  const [opened, setOpened] = useState(false);
  const [deleting, setDeleting] = useState<SavedReply | null>(null);
  const [saving, setSaving] = useState(false);
  const replies = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'saved-replies'],
    queryFn: () => getSavedReplies(),
  });
  const departments = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'admin-departments'],
    queryFn: getAdminDepartments,
  });
  const form = useForm<SavedReplyPayload>({
    initialValues: emptySavedReply,
    validate: zod4Resolver(savedReplyPayloadSchema),
  });
  const openEditor = (reply?: SavedReply) => {
    setEditing(reply ?? null);
    form.setValues(reply ? savedReplyPayloadSchema.parse(reply) : emptySavedReply);
    setOpened(true);
  };
  const submit = async (payload: SavedReplyPayload) => {
    setSaving(true);
    try {
      if (editing) await updateSavedReply(editing.uuid, payload);
      else await createSavedReply(payload);
      await replies.refetch();
      setOpened(false);
      addToast(t('notices.savedReplySaved', {}), 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSaving(false);
    }
  };
  return (
    <Stack>
      <Group justify='space-between'>
        <Text fw={600}>{t('management.savedReplies.title', {})}</Text>
        <Button leftSection={<FontAwesomeIcon icon={faPlus} />} onClick={() => openEditor()}>
          {t('management.savedReplies.create', {})}
        </Button>
      </Group>
      {(replies.data ?? []).map((reply) => (
        <Card key={reply.uuid}>
          <Group justify='space-between' align='flex-start'>
            <div>
              <Group gap='xs'>
                <Text fw={600}>{reply.title}</Text>
                {!reply.enabled && <Badge color='gray'>{t('common.disabled', {})}</Badge>}
              </Group>
              <Text size='sm' c='dimmed' mt='xs' lineClamp={2}>
                {reply.body}
              </Text>
            </div>
            <Group gap='xs'>
              <ActionIcon variant='default' onClick={() => openEditor(reply)} aria-label={t('actions.edit', {})}>
                <FontAwesomeIcon icon={faEdit} />
              </ActionIcon>
              <ActionIcon
                color='red'
                variant='light'
                onClick={() => setDeleting(reply)}
                aria-label={t('actions.delete', {})}
              >
                <FontAwesomeIcon icon={faTrash} />
              </ActionIcon>
            </Group>
          </Group>
        </Card>
      ))}
      <Modal opened={opened} onClose={() => setOpened(false)} title={t('management.savedReplies.editor', {})} size='lg'>
        <form onSubmit={form.onSubmit(submit)}>
          <Stack>
            <TextInput label={t('fields.title', {})} {...form.getInputProps('title')} />
            <Select
              label={t('fields.department', {})}
              clearable
              data={(departments.data ?? []).map((department) => ({
                value: department.uuid,
                label: getDepartmentName(department, t),
              }))}
              {...form.getInputProps('departmentUuid')}
            />
            <TextArea label={t('fields.response', {})} minRows={8} {...form.getInputProps('body')} />
            <Switch label={t('common.enabled', {})} {...form.getInputProps('enabled', { type: 'checkbox' })} />
            <ModalFooter>
              <Button type='submit' loading={saving}>
                {t('actions.save', {})}
              </Button>
            </ModalFooter>
          </Stack>
        </form>
      </Modal>
      <ConfirmationModal
        opened={Boolean(deleting)}
        onClose={() => setDeleting(null)}
        title={t('management.savedReplies.delete', {})}
        confirm={t('actions.delete', {})}
        onConfirmed={async () => {
          if (!deleting) return;
          try {
            await deleteSavedReply(deleting.uuid);
            await replies.refetch();
            setDeleting(null);
            addToast(t('notices.savedReplyDeleted', {}), 'success');
          } catch (error) {
            addToast(httpErrorToHuman(error), 'error');
          }
        }}
      >
        <Text>{t('management.savedReplies.deleteWarning', { title: deleting?.title ?? '' })}</Text>
      </ConfirmationModal>
    </Stack>
  );
}

export default function AdminManagementPage() {
  const { t } = useExtTranslations();
  const navigate = useNavigate();
  const canDepartments = useAdminCan('support.manage-departments');
  const canSavedReplies = useAdminCan('support.manage-saved-replies');
  return (
    <AdminContentContainer
      title={t('management.title', {})}
      subtitle={t('management.subtitle', {})}
      contentRight={
        <Button variant='default' leftSection={<FontAwesomeIcon icon={faArrowLeft} />} onClick={() => navigate('..')}>
          {t('actions.backToTickets', {})}
        </Button>
      }
    >
      <Tabs defaultValue={canDepartments ? 'departments' : 'saved-replies'}>
        <Tabs.List>
          {canDepartments && <Tabs.Tab value='departments'>{t('management.departments.title', {})}</Tabs.Tab>}
          {canSavedReplies && <Tabs.Tab value='saved-replies'>{t('management.savedReplies.title', {})}</Tabs.Tab>}
        </Tabs.List>
        {canDepartments && (
          <Tabs.Panel value='departments' pt='lg'>
            <DepartmentsManager />
          </Tabs.Panel>
        )}
        {canSavedReplies && (
          <Tabs.Panel value='saved-replies' pt='lg'>
            <SavedRepliesManager />
          </Tabs.Panel>
        )}
      </Tabs>
    </AdminContentContainer>
  );
}
