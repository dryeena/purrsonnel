import { useState } from 'preact/hooks';

import { Form, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function VolunteerControls({ entry }: { entry: Row }) {
  const me = useAuth();
  const [open, setOpen] = useState('');
  const { data: org } = useData<Row>('/organization/tree');
  return (
    <>
      {entry.volunteer_user_id === me.user.id && entry.status === 'draft' && (
        <button type="button" class="text-link" onClick={() => setOpen('edit')}>
          Edit
        </button>
      )}
      {can(me, 'volunteer.time.admin') && ['approved', 'rejected'].includes(entry.status) && (
        <button type="button" class="text-link" onClick={() => setOpen('reopen')}>
          Reopen for correction
        </button>
      )}
      {open && (
        <Modal
          title={open === 'edit' ? 'Edit draft time' : 'Reopen decided time'}
          onClose={() => setOpen('')}
        >
          {open === 'reopen' ? (
            <>
              <p class="modal-description">
                The original record and decisions are preserved. Reopening returns the entry to
                draft for correction and a new approval.
              </p>
              <Form
                fields={[
                  {
                    name: 'reason',
                    label: 'Reason for correction',
                    type: 'textarea',
                    required: true,
                    maxLength: 2000,
                  },
                ]}
                submit="Reopen entry"
                onSubmit={async (b) => {
                  await api(`/volunteer/entries/${entry.id}/reopen`, 'POST', b);
                  setOpen('');
                  refresh();
                }}
              />
            </>
          ) : (
            <>
              <p class="modal-description">
                This editor records a manual duration. Saving replaces any previous start and end
                times on this draft.
              </p>
              <Form
                fields={[
                  {
                    name: 'department_id',
                    label: 'Department helped',
                    type: 'select',
                    required: true,
                    value: entry.department_id,
                    options: org?.departments.map((d: Row) => ({ value: d.id, label: d.name })),
                  },
                  {
                    name: 'work_date',
                    label: 'Work date',
                    type: 'date',
                    required: true,
                    value: entry.work_date,
                  },
                  {
                    name: 'minutes_worked',
                    label: 'Minutes worked',
                    type: 'number',
                    required: true,
                    min: 1,
                    max: org?.volunteer_max_minutes || 960,
                    value: entry.minutes_worked,
                  },
                  {
                    name: 'work_description',
                    label: 'Activity',
                    type: 'textarea',
                    required: true,
                    value: entry.work_description,
                    maxLength: 2000,
                  },
                ]}
                onSubmit={async (b) => {
                  await api(`/volunteer/entries/${entry.id}`, 'PATCH', {
                    ...b,
                    event_edition_id: entry.event_edition_id,
                  });
                  setOpen('');
                  refresh();
                }}
              />
            </>
          )}
        </Modal>
      )}
    </>
  );
}
