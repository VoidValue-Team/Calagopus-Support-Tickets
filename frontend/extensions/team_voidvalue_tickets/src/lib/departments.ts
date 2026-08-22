import type { Department } from '../schemas/tickets.ts';
import type { useExtTranslations } from '../translations.ts';

const defaultDepartments = {
  '00000000-0000-4000-8000-000000000001': 'technical',
  '00000000-0000-4000-8000-000000000002': 'billing',
  '00000000-0000-4000-8000-000000000003': 'sales',
  '00000000-0000-4000-8000-000000000004': 'abuse',
  '00000000-0000-4000-8000-000000000005': 'other',
} as const;

type Translator = ReturnType<typeof useExtTranslations>['t'];

export function getDepartmentName(department: Pick<Department, 'uuid' | 'name'>, t: Translator): string {
  const key = defaultDepartments[department.uuid as keyof typeof defaultDepartments];
  return key ? t(`defaultDepartments.${key}.name`, {}) : department.name;
}

export function getDepartmentDescription(department: Pick<Department, 'uuid' | 'description'>, t: Translator): string {
  const key = defaultDepartments[department.uuid as keyof typeof defaultDepartments];
  return key ? t(`defaultDepartments.${key}.description`, {}) : department.description;
}
