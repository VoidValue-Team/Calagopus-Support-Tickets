import { FileInput, Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import getServers from '@/api/server/getServers.ts';
import Button from '@/elements/Button.tsx';
import Select from '@/elements/input/Select.tsx';
import ServerSelect from '@/elements/input/ServerSelect.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { uploadAttachments } from '../api/tickets/attachments.ts';
import createTicket from '../api/tickets/createTicket.ts';
import getDepartments from '../api/tickets/getDepartments.ts';
import { getDepartmentName } from '../lib/departments.ts';
import { type CreateTicket, createTicketSchema } from '../schemas/tickets.ts';
import { useExtTranslations } from '../translations.ts';
export default function CreateTicketModal({
  onCreated,
  scope = 'account',
  serverUuid,
}: {
  onCreated: () => void;
  scope?: 'account' | 'server';
  serverUuid?: string;
}) {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const [opened, setOpened] = useState(false);
  const [loading, setLoading] = useState(false);
  const [files, setFiles] = useState<File[]>([]);
  const canSeeAllServers = useAdminCan('servers.read');
  const departments = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'departments', scope, serverUuid],
    queryFn: () => getDepartments(scope, serverUuid),
  });
  const form = useForm<CreateTicket>({
    initialValues: {
      subject: '',
      message: '',
      departmentUuid: '',
      serverUuid: serverUuid ?? null,
      priority: 'normal',
    },
    validate: zod4Resolver(createTicketSchema),
  });
  const submit = async (values: CreateTicket) => {
    setLoading(true);
    let detail;
    try {
      detail = await createTicket(values, scope, serverUuid);
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
      setLoading(false);
      return;
    }
    let attachmentError: unknown;
    try {
      if (files.length) {
        const customerMessage = detail.messages.find((entry) => entry.messageType === 'customer');
        if (!customerMessage) throw new Error(t('errors.createdMessageUnavailable', {}));
        await uploadAttachments(scope, detail.uuid, customerMessage.uuid, files, serverUuid);
      }
    } catch (error) {
      attachmentError = error;
    } finally {
      setLoading(false);
    }
    if (attachmentError) {
      addToast(`${t('notices.ticketCreatedWithoutAttachments', {})} ${httpErrorToHuman(attachmentError)}`, 'warning');
    } else {
      addToast(t('notices.ticketCreated', {}), 'success');
    }
    setOpened(false);
    form.reset();
    setFiles([]);
    onCreated();
  };
  return (
    <>
      <Button onClick={() => setOpened(true)}>{t('actions.create', {})}</Button>
      <Modal opened={opened} onClose={() => setOpened(false)} title={t('actions.create', {})}>
        <form onSubmit={form.onSubmit(submit)}>
          <Stack>
            <Select
              label={t('fields.department', {})}
              data={(departments.data ?? []).map((d) => ({ value: d.uuid, label: getDepartmentName(d, t) }))}
              loading={departments.loading}
              {...form.getInputProps('departmentUuid')}
            />
            {scope === 'account' && (
              <ServerSelect
                queryKey={['extensions', 'team.voidvalue.tickets', 'server-select', canSeeAllServers]}
                fetcher={async (search) => {
                  const own = await getServers(1, search, false);
                  if (!canSeeAllServers) return own;
                  const others = await getServers(1, search, true);
                  const data = [...own.data, ...others.data].filter(
                    (server, index, servers) => servers.findIndex((item) => item.uuid === server.uuid) === index,
                  );
                  return { ...own, total: data.length, perPage: data.length || 1, data };
                }}
                label={t('fields.server', {})}
                description={t('fields.serverDescription', {})}
                placeholder={t('fields.noServer', {})}
                clearable
                value={form.values.serverUuid ?? null}
                onChange={(serverUuid) => form.setFieldValue('serverUuid', serverUuid)}
              />
            )}
            <Select
              label={t('fields.priority', {})}
              data={(['low', 'normal', 'high', 'urgent'] as const).map((priority) => ({
                value: priority,
                label: t(`priorities.${priority}`, {}),
              }))}
              {...form.getInputProps('priority')}
            />
            <TextInput label={t('fields.subject', {})} {...form.getInputProps('subject')} />
            <TextArea label={t('fields.message', {})} minRows={6} {...form.getInputProps('message')} />
            <FileInput
              label={t('fields.attachments', {})}
              description={t('fields.attachmentsDescription', {})}
              accept='image/png,image/jpeg,text/plain,application/pdf'
              multiple
              clearable
              value={files}
              onChange={(value) => setFiles(value ?? [])}
            />
            <ModalFooter>
              <Button type='submit' loading={loading}>
                {t('actions.submit', {})}
              </Button>
            </ModalFooter>
          </Stack>
        </form>
      </Modal>
    </>
  );
}
