import { useState } from 'preact/hooks';

import { Form, Icon, Modal } from '../../components/ui';
import { date } from '../../lib/date';
import { refresh } from '../../lib/events';
import { uuid } from '../../lib/uuid';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function ManualApproval({ entry, onClose }: { entry: Row; onClose: () => void }) {
  const [idempotencyKey] = useState(uuid);
  const [success, setSuccess] = useState(false);

  return (
    <Modal title={success ? 'Decision recorded' : 'Review volunteer time'} onClose={onClose}>
      {success ? (
        <div class="success-box">
          <Icon name="check" /> The volunteer’s time entry has been updated.
        </div>
      ) : (
        <>
          <p class="modal-description">
            This records a manual staff decision and revokes any active approval QR.
          </p>
          <div class="approval-summary">
            <h3>{entry.volunteer_name}</h3>
            <p>
              {entry.department} · {date(entry.work_date)}
            </p>
            <strong class="approval-hours">{(entry.minutes_worked / 60).toFixed(1)} hours</strong>
            <p>{entry.work_description}</p>
          </div>
          <Form
            fields={[
              {
                name: 'decision',
                label: 'Your decision',
                type: 'select',
                required: true,
                options: [
                  { value: 'approve', label: 'Approve hours' },
                  { value: 'deny', label: 'Reject hours' },
                ],
              },
              {
                name: 'note',
                label: 'Decision note (required for rejection)',
                type: 'textarea',
                maxLength: 2000,
              },
              {
                name: 'confirm',
                label: 'I reviewed this entry and confirm this decision.',
                type: 'checkbox',
                required: true,
              },
            ]}
            submit="Confirm manual decision"
            onSubmit={async (body) => {
              await api('/volunteer/manual-decisions', 'POST', {
                entry_id: entry.id,
                decision: body.decision,
                note: body.note,
                idempotency_key: idempotencyKey,
              });
              setSuccess(true);
              refresh();
            }}
          />
        </>
      )}
    </Modal>
  );
}
