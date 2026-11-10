import type { FieldSpec } from '../../components/ui';
import type { Row } from '../../types/api';

export function fieldsFor(questions: Row[], answers: Row = {}): FieldSpec[] {
  return questions.map((q) => ({
    name: q.key,
    label: q.label,
    type: q.type === 'boolean' ? 'checkbox' : q.type === 'single_select' ? 'select' : q.type,
    required: q.required,
    value: answers[q.key],
    maxLength: q.max_length,
    options: q.options?.map((o: string) => ({ value: o, label: o })),
    help: q.sensitive ? 'Restricted response. Visible only to authorized personnel.' : undefined,
  }));
}
