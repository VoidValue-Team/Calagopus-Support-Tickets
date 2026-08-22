import { faArrowLeft, faDownload, faKey, faPaperclip, faPaperPlane, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Badge, FileInput, Group, SimpleGrid, Stack, Text } from '@mantine/core';
import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import AccountContentContainer from '@/elements/containers/AccountContentContainer.tsx';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import ServerContentContainer from '@/elements/containers/ServerContentContainer.tsx';
import MultiSelect from '@/elements/input/MultiSelect.tsx';
import NumberInput from '@/elements/input/NumberInput.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TagsInput from '@/elements/input/TagsInput.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import Spinner from '@/elements/Spinner.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { getSavedReplies } from '../api/savedReplies/manageSavedReplies.ts';
import { requestAccess, reviewAccess, revokeAccess } from '../api/tickets/access.ts';
import assignTicket from '../api/tickets/assignTicket.ts';
import { downloadAttachment, uploadAttachments } from '../api/tickets/attachments.ts';
import deleteTicket from '../api/tickets/deleteTicket.ts';
import getAccessOptions from '../api/tickets/getAccessOptions.ts';
import getStaff from '../api/tickets/getStaff.ts';
import getTicket from '../api/tickets/getTicket.ts';
import type { TicketScope } from '../api/tickets/getTickets.ts';
import replyTicket from '../api/tickets/replyTicket.ts';
import updateTags from '../api/tickets/updateTags.ts';
import updateTicketStatus from '../api/tickets/updateTicketStatus.ts';
import { getDepartmentName } from '../lib/departments.ts';
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

const historyEventKey = (event: string) => {
  switch (event) {
    case 'ticket.created':
      return 'history.events.ticket.created' as const;
    case 'ticket.replied':
      return 'history.events.ticket.replied' as const;
    case 'ticket.status.updated':
      return 'history.events.ticket.status.updated' as const;
    case 'ticket.assignment.updated':
      return 'history.events.ticket.assignment.updated' as const;
    case 'ticket.attachments.added':
      return 'history.events.ticket.attachments.added' as const;
    case 'ticket.sla.breached':
      return 'history.events.ticket.sla.breached' as const;
    case 'ticket.tags.updated':
      return 'history.events.ticket.tags.updated' as const;
    default:
      return null;
  }
};

export default function TicketDetailPage({ scope, serverUuid }: { scope: TicketScope; serverUuid?: string }) {
  const { t } = useExtTranslations();
  const { addToast } = useToast();
  const navigate = useNavigate();
  const { ticket = '' } = useParams();
  const [message, setMessage] = useState('');
  const [internalNote, setInternalNote] = useState(false);
  const [files, setFiles] = useState<File[]>([]);
  const [deleteOpened, setDeleteOpened] = useState(false);
  const [accessOpened, setAccessOpened] = useState(false);
  const [accessPermissions, setAccessPermissions] = useState<string[]>([]);
  const [accessDuration, setAccessDuration] = useState(240);
  const [accessReason, setAccessReason] = useState('');
  const [selectedStatus, setSelectedStatus] = useState<TicketStatus>('open');
  const [selectedStaff, setSelectedStaff] = useState<string | null>(null);
  const [ticketTags, setTicketTags] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);
  const [mutationError, setMutationError] = useState<string | null>(null);
  const canReply = useAdminCan('support.reply');
  const canInternalNote = useAdminCan('support.internal-note');
  const canUpdateStatus = useAdminCan('support.update-status');
  const canAttachments = useAdminCan('support.attachments');
  const canAssign = useAdminCan('support.assign');
  const canRequestAccess = useAdminCan('support.request-access');
  const canManageTags = useAdminCan('support.manage-tags');
  const canDelete = useAdminCan('support.delete');
  const resource = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'ticket', scope, ticket],
    queryFn: () => getTicket(scope, ticket, serverUuid),
    enabled: Boolean(ticket),
  });
  const staff = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'staff'],
    queryFn: getStaff,
    enabled: scope === 'admin' && canAssign,
    silent: true,
  });
  const savedReplies = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'saved-replies', 'enabled'],
    queryFn: () => getSavedReplies(),
    enabled: scope === 'admin' && (canReply || canInternalNote),
    silent: true,
  });
  const accessOptions = useResource({
    queryKey: ['extensions', 'team.voidvalue.tickets', 'access-options'],
    queryFn: getAccessOptions,
    enabled: scope === 'admin' && canRequestAccess,
    silent: true,
  });
  const detail = resource.data;

  useEffect(() => {
    if (detail) {
      setSelectedStatus(detail.status);
      setSelectedStaff(detail.assignedStaffUuid);
      setTicketTags(detail.tags.map((tag) => tag.name));
    }
  }, [detail?.status]);

  useEffect(() => {
    if (!accessOptions.data) return;
    setAccessPermissions(accessOptions.data.permissions);
    setAccessDuration(accessOptions.data.defaultMinutes);
  }, [accessOptions.data]);

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
      reply = await replyTicket(
        scope,
        ticket,
        {
          message: message.trim(),
          internalNote: scope === 'admin' && internalNote,
        },
        serverUuid,
      );
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
        await uploadAttachments(scope, ticket, createdMessage.uuid, files, serverUuid);
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
            <Text mt='xs'>{getDepartmentName({ uuid: detail.departmentUuid, name: detail.departmentName }, t)}</Text>
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
            <Text mt='xs'>{detail.assignedStaffName ?? t('fields.unassigned', {})}</Text>
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
                              await downloadAttachment(scope, ticket, attachment, serverUuid);
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

        {scope === 'admin' && detail.history.length > 0 && (
          <Card>
            <Text fw={600} mb='md'>
              {t('history.title', {})}
            </Text>
            <Stack gap='sm'>
              {[...detail.history].reverse().map((entry) => (
                <Group key={entry.uuid} justify='space-between' align='flex-start' wrap='nowrap'>
                  <div>
                    <Text size='sm'>
                      {historyEventKey(entry.event) ? t(historyEventKey(entry.event)!, {}) : entry.event}
                    </Text>
                    <Text size='xs' c='dimmed'>
                      {entry.actorName ?? t('messageTypes.system', {})}
                    </Text>
                  </div>
                  <Text size='xs' c='dimmed'>
                    {entry.createdAt.toLocaleString()}
                  </Text>
                </Group>
              ))}
            </Stack>
          </Card>
        )}

        {(detail.accessRequests.length > 0 ||
          detail.accessGrants.length > 0 ||
          (scope === 'admin' && canRequestAccess)) && (
          <Card>
            <Group justify='space-between' align='flex-start'>
              <div>
                <Text fw={600}>{t('access.title', {})}</Text>
                <Text size='sm' c='dimmed'>
                  {t('access.description', {})}
                </Text>
              </div>
              {scope === 'admin' && canRequestAccess && accessOptions.data?.enabled && detail.serverUuid && (
                <Button
                  variant='default'
                  leftSection={<FontAwesomeIcon icon={faKey} />}
                  disabled={detail.accessRequests.some((request) => request.status === 'pending')}
                  onClick={() => setAccessOpened(true)}
                >
                  {t('access.request', {})}
                </Button>
              )}
            </Group>
            <Stack mt='md' gap='sm'>
              {detail.accessRequests.map((request) => (
                <Card key={request.uuid}>
                  <Group justify='space-between' align='flex-start'>
                    <div>
                      <Group gap='xs'>
                        <Text size='sm' fw={600}>
                          {request.requestedByName}
                        </Text>
                        <Badge
                          color={
                            request.status === 'pending' ? 'yellow' : request.status === 'approved' ? 'green' : 'gray'
                          }
                        >
                          {t(`access.statuses.${request.status}`, {})}
                        </Badge>
                      </Group>
                      <Text size='sm' mt='xs'>
                        {request.reason}
                      </Text>
                      <Text size='xs' c='dimmed' mt='xs'>
                        {request.requestedDurationMinutes} {t('access.minutes', {})} · {request.permissions.join(', ')}
                      </Text>
                    </div>
                    {scope !== 'admin' && request.status === 'pending' && (
                      <Group gap='xs'>
                        <Button
                          color='green'
                          size='compact-sm'
                          loading={submitting}
                          onClick={async () => {
                            await mutate(
                              () => reviewAccess(scope, ticket, request.uuid, 'approve', serverUuid),
                              t('notices.accessApproved', {}),
                            );
                          }}
                        >
                          {t('access.approve', {})}
                        </Button>
                        <Button
                          color='red'
                          variant='light'
                          size='compact-sm'
                          loading={submitting}
                          onClick={async () => {
                            await mutate(
                              () => reviewAccess(scope, ticket, request.uuid, 'reject', serverUuid),
                              t('notices.accessRejected', {}),
                            );
                          }}
                        >
                          {t('access.reject', {})}
                        </Button>
                      </Group>
                    )}
                  </Group>
                </Card>
              ))}
              {detail.accessGrants
                .filter((grant) => !grant.revokedAt)
                .map((grant) => (
                  <Card key={grant.uuid} leftStripeClassName='bg-green-500'>
                    <Group justify='space-between' align='flex-start'>
                      <div>
                        <Text size='sm' fw={600}>
                          {t('access.activeFor', { user: grant.userName })}
                        </Text>
                        <Text size='xs' c='dimmed'>
                          {t('access.expires', { date: grant.expiresAt.toLocaleString() })}
                        </Text>
                      </div>
                      {scope === 'admin' && canRequestAccess && (
                        <Button
                          color='red'
                          variant='light'
                          size='compact-sm'
                          loading={submitting}
                          onClick={async () => {
                            await mutate(() => revokeAccess(ticket, grant.uuid), t('notices.accessRevoked', {}));
                          }}
                        >
                          {t('access.revoke', {})}
                        </Button>
                      )}
                    </Group>
                  </Card>
                ))}
            </Stack>
          </Card>
        )}

        {mutationError && (
          <Card>
            <Text c='red'>{mutationError}</Text>
          </Card>
        )}

        {(scope !== 'admin' || canReply || canInternalNote) && detail.status !== 'closed' && (
          <Card>
            <Stack>
              {scope === 'admin' && (savedReplies.data?.length ?? 0) > 0 && (
                <Select
                  label={t('fields.savedReply', {})}
                  placeholder={t('fields.savedReplyPlaceholder', {})}
                  clearable
                  searchable
                  data={(savedReplies.data ?? [])
                    .filter(
                      (reply) =>
                        reply.enabled && (!reply.departmentUuid || reply.departmentUuid === detail.departmentUuid),
                    )
                    .map((reply) => ({ value: reply.uuid, label: reply.title }))}
                  onChange={(uuid) => {
                    const reply = savedReplies.data?.find((entry) => entry.uuid === uuid);
                    if (reply) setMessage(reply.body);
                  }}
                />
              )}
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
              {(scope !== 'admin' || canAttachments) && (
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
        ) : scope !== 'admin' ? (
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
                      serverUuid,
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

        {scope === 'admin' && canAssign && (
          <Card>
            <Group align='end'>
              <Select
                label={t('fields.assignedTo', {})}
                placeholder={t('fields.unassigned', {})}
                clearable
                searchable
                data={(staff.data ?? []).map((entry) => ({ value: entry.uuid, label: entry.username }))}
                value={selectedStaff}
                onChange={setSelectedStaff}
                flex={1}
              />
              <Button
                variant='default'
                loading={submitting}
                disabled={selectedStaff === detail.assignedStaffUuid}
                onClick={async () => {
                  await mutate(() => assignTicket(ticket, selectedStaff), t('notices.assignmentUpdated', {}));
                }}
              >
                {t('actions.assign', {})}
              </Button>
            </Group>
          </Card>
        )}

        {scope === 'admin' && canManageTags && (
          <Card>
            <Group align='end'>
              <TagsInput label={t('fields.tags', {})} value={ticketTags} onChange={setTicketTags} flex={1} />
              <Button
                variant='default'
                loading={submitting}
                onClick={async () => {
                  await mutate(() => updateTags(ticket, ticketTags), t('notices.tagsUpdated', {}));
                }}
              >
                {t('actions.save', {})}
              </Button>
            </Group>
          </Card>
        )}
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
      <Button variant='default' leftSection={<FontAwesomeIcon icon={faArrowLeft} />} onClick={() => navigate('..')}>
        {t('actions.backToTickets', {})}
      </Button>
    </Group>
  );

  const page =
    scope === 'admin' ? (
      <AdminContentContainer title={title} subtitle={t('pages.detail.subtitle', {})} contentRight={contentRight}>
        {content}
      </AdminContentContainer>
    ) : scope === 'server' ? (
      <ServerContentContainer title={title} contentRight={contentRight}>
        {content}
      </ServerContentContainer>
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
      <Modal opened={accessOpened} onClose={() => setAccessOpened(false)} title={t('access.request', {})} size='lg'>
        <Stack>
          <MultiSelect
            label={t('access.permissions', {})}
            data={(accessOptions.data?.permissions ?? []).map((permission) => ({
              value: permission,
              label: permission,
            }))}
            value={accessPermissions}
            onChange={setAccessPermissions}
          />
          <NumberInput
            label={t('access.duration', {})}
            min={1}
            max={accessOptions.data?.maxMinutes}
            value={accessDuration}
            onChange={(value) => typeof value === 'number' && setAccessDuration(value)}
          />
          <TextArea
            label={t('access.reason', {})}
            minRows={4}
            value={accessReason}
            onChange={(event) => setAccessReason(event.currentTarget.value)}
          />
          <ModalFooter>
            <Button
              loading={submitting}
              disabled={!accessReason.trim() || accessPermissions.length === 0}
              onClick={async () => {
                const succeeded = await mutate(
                  () =>
                    requestAccess(ticket, {
                      permissions: accessPermissions,
                      durationMinutes: accessDuration,
                      reason: accessReason.trim(),
                    }),
                  t('notices.accessRequested', {}),
                );
                if (succeeded) {
                  setAccessOpened(false);
                  setAccessReason('');
                }
              }}
            >
              {t('access.request', {})}
            </Button>
          </ModalFooter>
        </Stack>
      </Modal>
    </>
  );
}
