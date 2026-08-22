import { Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Select from '@/elements/input/Select.tsx';
import ServerSelect from '@/elements/input/ServerSelect.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import createTicket from '../api/tickets/createTicket.ts';
import getDepartments from '../api/tickets/getDepartments.ts';
import { type CreateTicket, createTicketSchema } from '../schemas/tickets.ts';
import { useExtTranslations } from '../translations.ts';
export default function CreateTicketModal({ onCreated }: { onCreated: () => void }) {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const [opened, setOpened] = useState(false);
  const [loading, setLoading] = useState(false);
  const departments = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'departments'],
    queryFn: getDepartments,
  });
  const form = useForm<CreateTicket>({
    initialValues: {
      subject: '',
      message: '',
      departmentUuid: '',
      serverUuid: null,
      priority: 'normal',
    },
    validate: zod4Resolver(createTicketSchema),
  });
  const submit = (values: CreateTicket) => {
    setLoading(true);
    createTicket(values)
      .then(() => {
        addToast(t('notices.ticketCreated', {}), 'success');
        setOpened(false);
        form.reset();
        onCreated();
      })
      .catch((error) => addToast(httpErrorToHuman(error), 'error'))
      .finally(() => setLoading(false));
  };
  return (
    <>
      <Button onClick={() => setOpened(true)}>{t('actions.create', {})}</Button>
      <Modal opened={opened} onClose={() => setOpened(false)} title={t('actions.create', {})}>
        <form onSubmit={form.onSubmit(submit)}>
          <Stack>
            <Select
              label={t('fields.department', {})}
              data={(departments.data ?? []).map((d) => ({ value: d.uuid, label: d.name }))}
              loading={departments.loading}
              {...form.getInputProps('departmentUuid')}
            />
            <ServerSelect
              label={t('fields.server', {})}
              description={t('fields.serverDescription', {})}
              placeholder={t('fields.noServer', {})}
              clearable
              value={form.values.serverUuid ?? null}
              onChange={(serverUuid) => form.setFieldValue('serverUuid', serverUuid)}
            />
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
