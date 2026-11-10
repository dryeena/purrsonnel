import { useState } from 'preact/hooks';

import { Empty, ErrorBox, Form, Loading, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { uuid } from '../../lib/uuid';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function Compose({ onClose, onSent }: { onClose: () => void; onSent: (r: Row) => void }) {
  const { data, error, loading } = useData<Row>('/requests/recipients');
  const [kind, setKind] = useState('team');
  const [key] = useState(uuid);
  const options =
    kind === 'team'
      ? data?.departments
          .filter((d: Row) => d.recipient_count > 0 && d.recipient_count <= 250)
          .map((d: Row) => ({ value: d.id, label: `${d.name} · ${d.recipient_count} recipients` }))
      : data?.people.map((p: Row) => ({
          value: p.id,
          label: `${p.display_name}${p.department ? ` · ${p.department}` : ''}`,
        }));
  return (
    <Modal title="Send a request" onClose={onClose} wide>
      <p class="modal-description">
        A team request goes to its current active members. One teammate can accept it so everyone
        knows who’s helping.
      </p>
      <label class="field">
        <span>Send to</span>
        <select value={kind} onChange={(e) => setKind(e.currentTarget.value)}>
          <option value="team">A department / team</option>
          <option value="person">One person</option>
        </select>
      </label>
      {error ? (
        <ErrorBox message={error} />
      ) : loading ? (
        <Loading />
      ) : !options?.length ? (
        <Empty title="No recipients in your scope">
          Ask your administrator to assign request-sending access for the appropriate department.
        </Empty>
      ) : (
        <Form
          fields={[
            {
              name: 'recipient',
              label: kind === 'team' ? 'Which team?' : 'Who needs this?',
              type: 'select',
              required: true,
              options,
            },
            {
              name: 'title',
              label: 'What do you need?',
              required: true,
              maxLength: 160,
              placeholder: 'e.g. Extra hands at the welcome desk',
              help: 'A short, specific title helps people respond quickly.',
            },
            {
              name: 'body',
              label: 'Details',
              type: 'textarea',
              required: true,
              maxLength: 4000,
              placeholder: 'Where to go, what to bring, and who to check in with…',
              help: 'Include where to go and what a helpful response looks like.',
            },
            {
              name: 'urgency',
              label: 'Priority',
              type: 'select',
              required: true,
              value: 'normal',
              options: [
                { value: 'normal', label: 'Normal' },
                { value: 'urgent', label: 'Urgent · needs prompt attention' },
              ],
            },
            {
              name: 'due_at',
              label: 'Needed by (optional, your device timezone)',
              type: 'datetime-local',
            },
          ]}
          submit="Send request"
          onSubmit={async (values) => {
            onSent(
              await api('/requests', 'POST', {
                title: values.title,
                body: values.body,
                urgency: values.urgency,
                due_at: values.due_at ? new Date(values.due_at).toISOString() : null,
                recipient_user_id: kind === 'person' ? values.recipient : null,
                recipient_department_id: kind === 'team' ? values.recipient : null,
                idempotency_key: key,
              }),
            );
          }}
        />
      )}
    </Modal>
  );
}
