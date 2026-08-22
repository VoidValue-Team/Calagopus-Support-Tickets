import { defineEnglishItem, defineTranslations } from 'shared';

const translations = defineTranslations({
  items: { ticket: defineEnglishItem('Ticket', 'Tickets') },
  translations: {
    nav: { support: 'Support' },
    pages: {
      account: { title: 'Support tickets', subtitle: 'Create and track your support conversations.' },
      server: { title: 'Server support', subtitle: 'Tickets related to this server.' },
      admin: { title: 'Support queue', subtitle: 'Manage customer requests and service levels.' },
      settings: { title: 'Support settings' },
    },
    actions: { create: 'Create ticket', submit: 'Submit ticket', open: 'Open ticket' },
    fields: { subject: 'Subject', message: 'Describe the problem', department: 'Department', priority: 'Priority' },
    columns: {
      code: 'Code',
      subject: 'Subject',
      department: 'Department',
      priority: 'Priority',
      status: 'Status',
      updated: 'Last reply',
      sla: 'SLA',
    },
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
