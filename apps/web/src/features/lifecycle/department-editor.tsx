import { useState } from 'preact/hooks';

import { Form, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function DepartmentEditor({ department }: { department: Row }) {
  const me = useAuth();
  const [open, setOpen] = useState(false);
  const { data: people } = useData<Row>(
    can(me, 'organization.manage') ? `/users?department=${department.id}&page_size=100` : null,
  );
  if (!can(me, 'organization.manage')) return null;
  return (
    <>
      <button type="button" class="button secondary" onClick={() => setOpen(true)}>
        Edit department
      </button>
      {open && (
        <Modal title={`Edit ${department.name}`} onClose={() => setOpen(false)}>
          <Form
            fields={[
              { name: 'name', label: 'Department name', required: true, value: department.name },
              {
                name: 'description',
                label: 'Description',
                type: 'textarea',
                value: department.description,
              },
              {
                name: 'lead_user_id',
                label: 'Department lead',
                type: 'select',
                value: department.lead_user_id,
                options: people?.items.map((u: Row) => ({ value: u.id, label: u.display_name })),
              },
            ]}
            onSubmit={async (b) => {
              if (!b.lead_user_id) {
                delete b.lead_user_id;
                b.clear_lead = true;
              }
              await api(`/departments/${department.id}`, 'PATCH', b);
              setOpen(false);
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
