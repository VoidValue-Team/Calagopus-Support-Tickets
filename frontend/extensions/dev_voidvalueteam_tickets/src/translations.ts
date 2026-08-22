import { defineEnglishItem, defineTranslations } from 'shared';

const translations = defineTranslations({
  items: { ticket: defineEnglishItem('Ticket', 'Tickets') },
  translations: {
    nav: { support: 'Support' },
    pages: {
      account: { title: 'Support tickets', subtitle: 'Create and track your support conversations.' },
      admin: { title: 'Support queue', subtitle: 'Manage customer requests and service levels.' },
      detail: { title: 'Support ticket', subtitle: 'Conversation and ticket management.' },
      settings: { title: 'Support settings' },
    },
    actions: {
      create: 'Create ticket',
      submit: 'Submit ticket',
      open: 'Open ticket',
      reply: 'Reply',
      sendReply: 'Send reply',
      internalNote: 'Internal note',
      internalNoteDescription: 'Only support staff can see internal notes.',
      addInternalNote: 'Add internal note',
      updateStatus: 'Update status',
      close: 'Close ticket',
      reopen: 'Reopen ticket',
      backToTickets: 'Back to tickets',
      search: 'Search tickets',
    },
    fields: {
      subject: 'Subject',
      message: 'Describe the problem',
      department: 'Department',
      priority: 'Priority',
      status: 'Status',
      customer: 'Customer',
      server: 'Affected server (optional)',
      serverDescription: 'Select a server only when the request is related to one.',
      noServer: 'Not related to a server',
      createdAt: 'Created',
    },
    columns: {
      code: 'Code',
      subject: 'Subject',
      customer: 'Customer',
      server: 'Server',
      department: 'Department',
      priority: 'Priority',
      status: 'Status',
      updated: 'Last reply',
      sla: 'SLA',
    },
    statuses: {
      open: 'Open',
      awaiting_staff: 'Awaiting staff',
      awaiting_customer: 'Awaiting customer',
      in_progress: 'In progress',
      resolved: 'Resolved',
      closed: 'Closed',
    },
    priorities: { low: 'Low', normal: 'Normal', high: 'High', urgent: 'Urgent' },
    sla: { breached: 'Breached', ok: 'OK' },
    messageTypes: {
      customer: 'Customer',
      staff: 'Support staff',
      internal_note: 'Internal note',
      system: 'System',
    },
    notices: {
      ticketCreated: 'Ticket created.',
      replySent: 'Reply sent.',
      statusUpdated: 'Ticket status updated.',
    },
    errors: { ticketUnavailable: 'The ticket could not be loaded or is no longer available.' },
    empty: 'No support tickets found.',
    quickActions: {
      category: 'Support',
      open: 'Open Support',
      unassigned: 'Unassigned Tickets',
      urgent: 'Urgent Tickets',
    },
  },
});
export const useExtTranslations = translations.useTranslations.bind(translations);
export const getExtTranslations = translations.getTranslations.bind(translations);
export default translations;
