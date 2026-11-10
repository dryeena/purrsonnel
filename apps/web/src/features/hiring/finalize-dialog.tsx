import { useState } from 'preact/hooks';

import { Form, Icon, Modal, Status } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { date } from '../../lib/date';
import { refresh } from '../../lib/events';
import { uuid } from '../../lib/uuid';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function Finalize({ application, onClose }: { application: Row; onClose: () => void }) {
  const [idempotencyKey] = useState(uuid);
  const { data } = useData<Row[]>('/interviews?page_size=100');
  const [saved, setSaved] = useState<Row | null>(null);
  const interviews = data?.filter((interview) => interview.application_id === application.id);

  return (
    <Modal title={`Final decision · ${application.applicant_name}`} onClose={onClose} wide>
      {saved ? (
        <div class="success-box">
          <Icon name="check" /> Application {saved.status}.{' '}
          {saved.status === 'approved' &&
            'Personnel assignment updated. Identity and permission grants remain separately managed.'}
        </div>
      ) : (
        <>
          <p class="modal-description">
            {application.position} · This is the hiring owner’s explicit decision. An HR override
            requires a reason.
          </p>
          <div class="panel-responses">
            {interviews?.map((interview) => (
              <div key={interview.id} class="final-panel">
                <strong>{date(interview.scheduled_start_at, true, interview.timezone)}</strong>
                {interview.panel.map((participant: Row) => (
                  <div key={participant.user_id}>
                    <span>{participant.display_name}</span>
                    <Status value={participant.response} />
                  </div>
                ))}
              </div>
            ))}
          </div>
          <Form
            fields={[
              {
                name: 'decision',
                label: 'Final decision',
                type: 'select',
                required: true,
                options: [
                  { value: 'approve', label: 'Approve & assign to department' },
                  { value: 'deny', label: 'Deny application' },
                ],
              },
              {
                name: 'reason',
                label: 'Decision reason / HR override reason',
                type: 'textarea',
                required: true,
                maxLength: 2000,
              },
              {
                name: 'confirm',
                label:
                  'I have reviewed the application and panel recommendations and confirm this final decision.',
                type: 'checkbox',
                required: true,
              },
            ]}
            submit="Confirm final decision"
            onSubmit={async (values) => {
              setSaved(
                await api(`/applications/${application.id}/finalize`, 'POST', {
                  decision: values.decision,
                  reason: values.reason,
                  idempotency_key: idempotencyKey,
                }),
              );
              refresh();
            }}
          />
        </>
      )}
    </Modal>
  );
}
