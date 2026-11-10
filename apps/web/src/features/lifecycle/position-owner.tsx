import { useState } from 'preact/hooks';

import { Form, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function PositionOwner({ position }: { position: Row }) {
  const me = useAuth();
  const [open, setOpen] = useState(false);
  const { data: people } = useData<Row>(
    can(me, 'positions.manage') ? '/users?page_size=100' : null,
  );
  if (!can(me, 'positions.manage')) return null;
  return (
    <>
      <button type="button" class="text-link" onClick={() => setOpen(true)}>
        Change hiring owner
      </button>
      {open && (
        <Modal title="Reassign final hiring authority" onClose={() => setOpen(false)}>
          <p class="modal-description">
            The new owner must have explicit finalization permission. This change and your reason
            are recorded in the audit history.
          </p>
          <Form
            fields={[
              {
                name: 'hiring_owner_user_id',
                label: 'New hiring owner',
                type: 'select',
                required: true,
                options: people?.items.map((u: Row) => ({ value: u.id, label: u.display_name })),
              },
              {
                name: 'reason',
                label: 'Reason for reassignment',
                type: 'textarea',
                required: true,
                maxLength: 1000,
              },
            ]}
            submit="Reassign owner"
            onSubmit={async (b) => {
              await api(`/positions/${position.id}/owner`, 'POST', b);
              setOpen(false);
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
