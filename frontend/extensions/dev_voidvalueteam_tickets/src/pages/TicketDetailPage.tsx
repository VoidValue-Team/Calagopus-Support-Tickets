import { faArrowLeft, faDownload, faPaperclip, faPaperPlane, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Badge, FileInput, Group, SimpleGrid, Stack, Text } from '@mantine/core';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import AccountContentContainer from '@/elements/containers/AccountContentContainer.tsx';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import Spinner from '@/elements/Spinner.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import assignTicket from '../api/tickets/assignTicket.ts';
import { downloadAttachment, uploadAttachments } from '../api/tickets/attachments.ts';
import deleteTicket from '../api/tickets/deleteTicket.ts';
import getAgents from '../api/tickets/getAgents.ts';
import getTicket from '../api/tickets/getTicket.ts';
import type { TicketScope } from '../api/tickets/getTickets.ts';
import replyTicket from '../api/tickets/replyTicket.ts';
import updateTicketStatus from '../api/tickets/updateTicketStatus.ts';
import type { TicketStatus } from '../schemas/tickets.ts';
import { useExtTranslations } from '../translations.ts';

const statuses: TicketStatus[] = ['open', 'awaiting_staff', 'awaiting_customer', 'in_progress', 'resolved', 'closed'];

const statusColor = (status: TicketStatus) => {
  if (status === 'closed' || status === 'resolved') return 'gray';
  if (status === 'awaiting_staff') return 'orange';
  if (status === 'awaiting_customer') return 'cyan';
  if (status === 'in_progress') return 'violet';
  return 'blue';
};

const readableSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

export default function TicketDetailPage({ scope }: { scope: TicketScope }) {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const navigate = useNavigate();
  const { ticket = '' } = useParams();
  const [message, setMessage] = useState('');
  const [internalNote, setInternalNote] = useState(false);
  const [files, setFiles] = useState<File[]>([]);
  const [deleteOpened, setDeleteOpened] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<TicketStatus>('open');
  const [submitting, setSubmitting] = useState(false);
  const [mutationError, setMutationError] = useState<string | null>(null);
  const canReply = useAdminCan('support.reply');
  const canInternalNote = useAdminCan('support.internal-note');
  const canUpdateStatus = useAdminCan('support.update-status');
  const canAssign = useAdminCan('support.assign');
  const canAttachments = useAdminCan('support.attachments');
  const canDelete = useAdminCan('support.delete');
  const resource = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'ticket', scope, ticket],
    queryFn: () => getTicket(scope, ticket),
    enabled: Boolean(ticket),
  });
  const agents = useResource({
    queryKey: ['extensions', 'dev.voidvalueteam.tickets', 'agents'],
    queryFn: getAgents,
    enabled: scope === 'admin' && canAssign,
    silent: true,
  });
  const detail = resource.data;

  useEffect(() => {
    if (detail) setSelectedStatus(detail.status);
  }, [detail?.status]);

  const mutate = async (action: () => Promise<unknown>, success: string) => {
    setSubmitting(true);
    setMutationError(null);
    try {
      await action();
      await resource.refetch();
      addToast(success, 'success');
      return true;
    } catch (error) {
      setMutationError(httpErrorToHuman(error));
      return false;
    } finally {
      setSubmitting(false);
    }
  };

  const submitReply = async () => {
    if (!message.trim()) return;
    setSubmitting(true);
    setMutationError(null);
    let reply;
    try {
      reply = await replyTicket(scope, ticket, {
        message: message.trim(),
        internalNote: scope === 'admin' && internalNote,
      });
    } catch (error) {
      setMutationError(httpErrorToHuman(error));
      setSubmitting(false);
      return;
    }
    let attachmentError: unknown;
    try {
      if (files.length) {
        const expectedType = scope === 'admin' ? (internalNote ? 'internal_note' : 'staff') : 'customer';
        const createdMessage = [...reply.messages]
          .reverse()
          .find((entry) => entry.messageType === expectedType && entry.body === message.trim());
        if (!createdMessage) throw new Error(t('errors.createdMessageUnavailable', {}));
        await uploadAttachments(scope, ticket, createdMessage.uuid, files);
      }
    } catch (error) {
      attachmentError = error;
    }
    try {
      await resource.refetch();
    } catch (error) {
      setMutationError(httpErrorToHuman(error));
    }
    setSubmitting(false);
    setMessage('');
    setInternalNote(false);
    setFiles([]);
    if (attachmentError) {
      const error = `${t('notices.replySentWithoutAttachments', {})} ${httpErrorToHuman(attachmentError)}`;
      setMutationError(error);
      addToast(error, 'warning');
    } else {
      addToast(t('notices.replySent', {}), 'success');
    }
  };

  const content =
    resource.loading && !detail ? (
      <Spinner.Centered />
    ) : !detail ? (
      <Card>
        <Text c='red'>{t('errors.ticketUnavailable', {})}</Text>
      </Card>
    ) : (
      <Stack>
        <SimpleGrid cols={{ base: 1, sm: 2, lg: 3, xl: 6 }}>
          <Card>
            <Text size='xs' c='dimmed'>
              {t('fields.status', {})}
            </Text>
            <Badge mt='xs' color={statusColor(detail.status)}>
              {t(`statuses.${detail.status}`, {})}
            </Badge>
          </Card>
          <Card>
            <Text size='xs' c='dimmed'>
              {t('fields.department', {})}
            </Text>
            <Text mt='xs'>{detail.departmentName}</Text>
          </Card>
          <Card>
            <Text size='xs' c='dimmed'>
              {t('fields.customer', {})}
            </Text>
            <Text mt='xs'>{detail.userName}</Text>
          </Card>
          <Card>
            <Text size='xs' c='dimmed'>
              {t('fields.server', {})}
            </Text>
            <Text mt='xs'>{detail.serverName ?? t('fields.noServer', {})}</Text>
          </Card>
          <Card>
            <Text size='xs' c='dimmed'>
              {t('fields.priority', {})}
            </Text>
            <Badge
              mt='xs'
              color={detail.priority === 'urgent' ? 'red' : detail.priority === 'high' ? 'orange' : 'blue'}
            >
              {t(`priorities.${detail.priority}`, {})}
            </Badge>
          </Card>
          <Card>
            <Text size='xs' c='dimmed'>
              {t('fields.assignedTo', {})}
            </Text>
            {scope === 'admin' && canAssign ? (
              <Select
                mt='xs'
                searchable
                clearable
                placeholder={t('fields.unassigned', {})}
                data={[
                  ...(detail.assignedStaffUuid && !agents.data?.some((agent) => agent.uuid === detail.assignedStaffUuid)
                    ? [{ value: detail.assignedStaffUuid, label: detail.assignedStaffName ?? detail.assignedStaffUuid }]
                    : []),
                  ...(agents.data ?? []).map((agent) => ({ value: agent.uuid, label: agent.username })),
                ]}
                value={detail.assignedStaffUuid}
                loading={agents.loading}
                disabled={submitting}
                onChange={async (staffUuid) => {
                  await mutate(() => assignTicket(ticket, staffUuid), t('notices.assignmentUpdated', {}));
                }}
              />
            ) : (
              <Text mt='xs'>{detail.assignedStaffName ?? t('fields.unassigned', {})}</Text>
            )}
          </Card>
        </SimpleGrid>

        <Card>
          <Text fw={600}>{detail.subject}</Text>
          <Text size='xs' c='dimmed' mt={4}>
            {t('fields.createdAt', {})}: {detail.createdAt.toLocaleString()}
          </Text>
        </Card>

        <Stack gap='sm'>
          {detail.messages.map((entry) => (
            <Card
              key={entry.uuid}
              leftStripeClassName={
                entry.messageType === 'internal_note'
                  ? 'bg-yellow-500'
                  : entry.messageType === 'staff'
                    ? 'bg-blue-500'
                    : entry.messageType === 'system'
                      ? 'bg-gray-500'
                      : 'bg-green-500'
              }
            >
              <Group justify='space-between' align='flex-start'>
                <div>
                  <Text fw={600}>{entry.authorName ?? t(`messageTypes.${entry.messageType}`, {})}</Text>
                  <Badge mt={4} size='xs' color={entry.messageType === 'internal_note' ? 'yellow' : 'gray'}>
                    {t(`messageTypes.${entry.messageType}`, {})}
                  </Badge>
                </div>
                <Text size='xs' c='dimmed'>
                  {entry.createdAt.toLocaleString()}
                </Text>
              </Group>
              <Text mt='md' style={{ whiteSpace: 'pre-wrap' }}>
                {entry.body}
              </Text>
              {detail.attachments.some((attachment) => attachment.messageUuid === entry.uuid) && (
                <Stack gap='xs' mt='md'>
                  <Text size='xs' c='dimmed' fw={600}>
                    <FontAwesomeIcon icon={faPaperclip} className='mr-1' />
                    {t('fields.attachments', {})}
                  </Text>
                  <Group gap='xs'>
                    {detail.attachments
                      .filter((attachment) => attachment.messageUuid === entry.uuid)
                      .map((attachment) => (
                        <Button
                          key={attachment.uuid}
                          variant='default'
                          size='compact-sm'
                          leftSection={<FontAwesomeIcon icon={faDownload} />}
                          onClick={async () => {
                            try {
                              await downloadAttachment(scope, ticket, attachment);
                            } catch (error) {
                              addToast(httpErrorToHuman(error), 'error');
                            }
                          }}
                        >
                          {attachment.originalFilename} · {readableSize(attachment.sizeBytes)}
                        </Button>
                      ))}
                  </Group>
                </Stack>
              )}
            </Card>
          ))}
        </Stack>

        {mutationError && (
          <Card>
            <Text c='red'>{mutationError}</Text>
          </Card>
        )}

        {(scope === 'account' || canReply || canInternalNote) && detail.status !== 'closed' && (
          <Card>
            <Stack>
              <TextArea
                label={internalNote ? t('actions.internalNote', {}) : t('actions.reply', {})}
                minRows={5}
                value={message}
                onChange={(event) => setMessage(event.currentTarget.value)}
              />
              {scope === 'admin' && canInternalNote && (
                <Switch
                  label={t('actions.internalNote', {})}
                  description={t('actions.internalNoteDescription', {})}
                  checked={internalNote}
                  onChange={(event) => setInternalNote(event.currentTarget.checked)}
                />
              )}
              {(scope === 'account' || canAttachments) && (
                <FileInput
                  label={t('fields.attachments', {})}
                  description={t('fields.attachmentsDescription', {})}
                  accept='image/png,image/jpeg,text/plain,application/pdf'
                  multiple
                  clearable
                  value={files}
                  onChange={(value) => setFiles(value ?? [])}
                />
              )}
              <Group justify='flex-end'>
                <Button
                  loading={submitting}
                  disabled={!message.trim() || (scope === 'admin' && !internalNote && !canReply)}
                  leftSection={<FontAwesomeIcon icon={faPaperPlane} />}
                  onClick={submitReply}
                >
                  {internalNote ? t('actions.addInternalNote', {}) : t('actions.sendReply', {})}
                </Button>
              </Group>
            </Stack>
          </Card>
        )}

        {scope === 'admin' && canUpdateStatus ? (
          <Card>
            <Group align='end'>
              <Select
                label={t('fields.status', {})}
                data={statuses.map((status) => ({ value: status, label: t(`statuses.${status}`, {}) }))}
                value={selectedStatus}
                onChange={(value) => value && setSelectedStatus(value as TicketStatus)}
                flex={1}
              />
              <Button
                variant='default'
                loading={submitting}
                disabled={selectedStatus === detail.status}
                onClick={async () => {
                  await mutate(() => updateTicketStatus(scope, ticket, selectedStatus), t('notices.statusUpdated', {}));
                }}
              >
                {t('actions.updateStatus', {})}
              </Button>
            </Group>
          </Card>
        ) : scope === 'account' ? (
          <Group justify='flex-end'>
            <Button
              variant='default'
              loading={submitting}
              onClick={async () => {
                await mutate(
                  () =>
                    updateTicketStatus(
                      scope,
                      ticket,
                      detail.status === 'closed' || detail.status === 'resolved' ? 'open' : 'closed',
                    ),
                  t('notices.statusUpdated', {}),
                );
              }}
            >
              {detail.status === 'closed' || detail.status === 'resolved'
                ? t('actions.reopen', {})
                : t('actions.close', {})}
            </Button>
          </Group>
        ) : null}
      </Stack>
    );

  const title = detail ? `${detail.code} · ${detail.subject}` : t('pages.detail.title', {});
  const contentRight = (
    <Group>
      {scope === 'admin' && canDelete && detail && (
        <Button
          color='red'
          variant='light'
          leftSection={<FontAwesomeIcon icon={faTrash} />}
          onClick={() => setDeleteOpened(true)}
        >
          {t('actions.delete', {})}
        </Button>
      )}
      <Button
        variant='default'
        leftSection={<FontAwesomeIcon icon={faArrowLeft} />}
        onClick={() => navigate(`${scope === 'admin' ? '/admin' : '/account'}/support`)}
      >
        {t('actions.backToTickets', {})}
      </Button>
    </Group>
  );

  const page =
    scope === 'admin' ? (
      <AdminContentContainer title={title} subtitle={t('pages.detail.subtitle', {})} contentRight={contentRight}>
        {content}
      </AdminContentContainer>
    ) : (
      <AccountContentContainer title={title} subtitle={t('pages.detail.subtitle', {})} contentRight={contentRight}>
        {content}
      </AccountContentContainer>
    );

  return (
    <>
      {page}
      <ConfirmationModal
        opened={deleteOpened}
        onClose={() => setDeleteOpened(false)}
        title={t('actions.deleteTicket', {})}
        confirm={t('actions.deletePermanently', {})}
        onConfirmed={async () => {
          await deleteTicket(ticket);
          addToast(t('notices.ticketDeleted', {}), 'success');
          setDeleteOpened(false);
          navigate('/admin/support');
        }}
      >
        <Text>{t('warnings.deleteTicket', { code: detail?.code ?? '' })}</Text>
      </ConfirmationModal>
    </>
  );
}
