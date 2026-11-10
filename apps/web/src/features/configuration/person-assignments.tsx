import { useState } from 'preact/hooks';

import { Form } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { global } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function PersonAssignments({ person }: { person: Row }) {
  const me = useAuth();
  const [open, setOpen] = useState(false);
  const { data: org } = useData<Row>('/organization/tree');
  const { data: people } = useData<Row>(global(me, 'people.edit') ? '/users?page_size=100' : null);
  if (!global(me, 'people.edit')) return null;
  return (
    <div class="detail-section">
      <h3>Memberships & manager</h3>
      <p class="muted">Primary membership and reporting relationships are explicit assignments.</p>
      <button type="button" class="button secondary" onClick={() => setOpen(!open)}>
        Edit organization assignments
      </button>
      {open && (
        <Form
          fields={[
            {
              name: 'department_ids',
              label: 'Department memberships (select multiple)',
              type: 'multi_select',
              value: person.departments.map((d: Row) => d.id),
              options: org?.departments.map((d: Row) => ({ value: d.id, label: d.name })),
            },
            {
              name: 'primary_department_id',
              label: 'Primary department',
              type: 'select',
              value: person.primary_department_id,
              options: org?.departments.map((d: Row) => ({ value: d.id, label: d.name })),
            },
            {
              name: 'manager_id',
              label: 'Manager',
              type: 'select',
              value: person.managers[0]?.id,
              options: people?.items
                .filter((u: Row) => u.id !== person.id)
                .map((u: Row) => ({ value: u.id, label: u.display_name })),
            },
          ]}
          submit="Save assignments"
          onSubmit={async (b) => {
            if (!b.primary_department_id) b.primary_department_id = null;
            if (!b.manager_id) {
              delete b.manager_id;
              b.clear_manager = true;
            }
            await api(`/users/${person.id}`, 'PATCH', b);
            setOpen(false);
            refresh();
          }}
        />
      )}
    </div>
  );
}
