import { useState } from 'preact/hooks';

import { Form, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function localDate(value: string) {
  const d = new Date(value);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}T${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

export function InterviewEditor({ interview }: { interview: Row }) {
  const me = useAuth();
  const [open, setOpen] = useState(false);
  const { data: people } = useData<Row>(
    can(me, 'interviews.manage') ? '/users?page_size=100' : null,
  );
  if (!can(me, 'interviews.manage')) return null;
  return (
    <>
      <button type="button" class="text-link" onClick={() => setOpen(true)}>
        Edit schedule / panel
      </button>
      {open && (
        <Modal
          title={`Interview · ${interview.applicant_name}`}
          onClose={() => setOpen(false)}
          wide
        >
          <Form
            fields={[
              {
                name: 'scheduled_start_at',
                label: 'Start (your device timezone)',
                type: 'datetime-local',
                required: true,
                value: localDate(interview.scheduled_start_at),
              },
              {
                name: 'scheduled_end_at',
                label: 'End (your device timezone)',
                type: 'datetime-local',
                required: true,
                value: localDate(interview.scheduled_end_at),
              },
              {
                name: 'timezone',
                label: 'Display timezone',
                required: true,
                value: interview.timezone,
              },
              {
                name: 'location_label',
                label: 'Location',
                required: true,
                value: interview.location_label,
              },
              {
                name: 'location_url',
                label: 'HTTPS meeting link',
                type: 'url',
                value: interview.location_url,
              },
              {
                name: 'status',
                label: 'Status',
                type: 'select',
                required: true,
                value: interview.status,
                options: ['scheduled', 'completed', 'cancelled'].map((v) => ({
                  value: v,
                  label: v,
                })),
              },
            ]}
            onSubmit={async (b) => {
              await api(`/interviews/${interview.id}`, 'PATCH', {
                ...b,
                scheduled_start_at: new Date(b.scheduled_start_at).toISOString(),
                scheduled_end_at: new Date(b.scheduled_end_at).toISOString(),
                location_url: b.location_url || null,
              });
              setOpen(false);
              refresh();
            }}
          />
          <div class="detail-section">
            <h3>Add an interviewer</h3>
            <Form
              fields={[
                {
                  name: 'interviewer_user_id',
                  label: 'Interviewer',
                  type: 'select',
                  required: true,
                  options: people?.items
                    .filter((u: Row) => !interview.panel.some((p: Row) => p.user_id === u.id))
                    .map((u: Row) => ({ value: u.id, label: u.display_name })),
                },
              ]}
              submit="Add to panel"
              onSubmit={async (b) => {
                await api(`/interviews/${interview.id}/interviewers`, 'POST', b);
                setOpen(false);
                refresh();
              }}
            />
          </div>
        </Modal>
      )}
    </>
  );
}
