import { Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Select from '@/elements/input/Select.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import editTicket from '../api/tickets/editTicket.ts';
import getDepartments from '../api/tickets/getDepartments.ts';
import getSupportServers from '../api/tickets/getSupportServers.ts';
import { type EditTicket, editTicketSchema, type TicketDetail } from '../schemas/tickets.ts';
import { useExtTranslations } from '../translations.ts';

export default function EditTicketModal({
  detail,
  opened,
  onClose,
  onUpdated,
  canEdit,
  canMoveDepartment,
  canUpdatePriority,
}: {
  detail: TicketDetail;
  opened: boolean;
  onClose: () => void;
  onUpdated: () => Promise<unknown>;
  canEdit: boolean;
  canMoveDepartment: boolean;
  canUpdatePriority: boolean;
}) {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const [loading, setLoading] = useState(false);
  const departments = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'departments'],
    queryFn: getDepartments,
  });
  const servers = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'support-servers'],
    queryFn: getSupportServers,
    enabled: canEdit,
    silent: true,
  });
  const form = useForm<EditTicket>({
    initialValues: {
      subject: detail.subject,
      departmentUuid: detail.departmentUuid,
      priority: detail.priority,
      serverUuid: detail.serverUuid,
    },
    validate: zod4Resolver(editTicketSchema),
  });

  useEffect(() => {
    if (opened) {
      form.setValues({
        subject: detail.subject,
        departmentUuid: detail.departmentUuid,
        priority: detail.priority,
        serverUuid: detail.serverUuid,
      });
    }
  }, [opened, detail.uuid, detail.updatedAt]);

  const submit = async (values: EditTicket) => {
    setLoading(true);
    try {
      await editTicket(detail.uuid, values);
      await onUpdated();
      addToast(t('notices.ticketUpdated', {}), 'success');
      onClose();
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal opened={opened} onClose={onClose} title={t('actions.editTicket', {})}>
      <form onSubmit={form.onSubmit(submit)}>
        <Stack>
          <TextInput label={t('fields.subject', {})} disabled={!canEdit} {...form.getInputProps('subject')} />
          <Select
            label={t('fields.department', {})}
            searchable
            disabled={!canMoveDepartment}
            loading={departments.loading}
            data={[
              ...(departments.data?.some((department) => department.uuid === detail.departmentUuid)
                ? []
                : [{ value: detail.departmentUuid, label: detail.departmentName }]),
              ...(departments.data ?? []).map((department) => ({ value: department.uuid, label: department.name })),
            ]}
            {...form.getInputProps('departmentUuid')}
          />
          <Select
            label={t('fields.priority', {})}
            disabled={!canUpdatePriority}
            data={(['low', 'normal', 'high', 'urgent'] as const).map((priority) => ({
              value: priority,
              label: t(`priorities.${priority}`, {}),
            }))}
            {...form.getInputProps('priority')}
          />
          <Select
            label={t('fields.server', {})}
            description={t('fields.serverDescription', {})}
            searchable
            clearable
            disabled={!canEdit}
            loading={servers.loading}
            placeholder={t('fields.noServer', {})}
            data={[
              ...(detail.serverUuid && !servers.data?.some((server) => server.uuid === detail.serverUuid)
                ? [{ value: detail.serverUuid, label: detail.serverName ?? detail.serverUuid }]
                : []),
              ...(servers.data ?? []).map((server) => ({ value: server.uuid, label: server.name })),
            ]}
            {...form.getInputProps('serverUuid')}
          />
          <ModalFooter>
            <Button type='submit' loading={loading}>
              {t('actions.saveChanges', {})}
            </Button>
          </ModalFooter>
        </Stack>
      </form>
    </Modal>
  );
}
