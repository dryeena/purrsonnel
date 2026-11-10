import { useState } from 'preact/hooks';

import { useData } from '../hooks/use-data';
import { followUpPayload, reminderReady, requestDate } from '../lib/alert-settings';
import { refresh } from '../lib/events';
import { notify } from '../lib/feedback';
import { api } from '../services/api-client';
import type { Row } from '../types/api';

import { Action, Badge, ErrorBox, Form, Icon, Loading, Modal } from './ui';

const activityLabels: Record<string, string> = {
  sent: 'Request sent',
  accepted: 'Request accepted',
  completed: 'Request completed',
  cancelled: 'Request cancelled',
  reminded: 'Reminder sent',
  follow_up_changed: 'Follow-up schedule updated',
  escalated: 'Escalated to the selected person',
  reminder_skipped: 'Scheduled reminder skipped',
  escalation_skipped: 'Scheduled escalation skipped',
};
export function DeliverySummary({ delivery: d }: { delivery: Row }) {
  return (
    <section class="request-delivery" aria-label="Phone delivery summary">
      <h3>
        <Icon name="bell" size={17} />
        Phone delivery
      </h3>
      <div class="delivery-counts">
        <span>
          <strong>{d.sent}</strong>Accepted by push service
        </span>
        <span>
          <strong>{d.queued}</strong>Queued
          {d.deferred > 0 ? ` · ${d.deferred} waiting / retrying` : ''}
        </span>
        <span>
          <strong>{d.suppressed}</strong>Suppressed by preferences
        </span>
        <span>
          <strong>{d.failed}</strong>Failed or expired
        </span>
      </div>
      <p class="small-note">
        Counts cover device alerts for this request and its follow-ups. Push-service acceptance does
        not confirm that a phone buzzed or a person read it. Recipients’ device details and settings
        are private.
      </p>
    </section>
  );
}
export function RequestFollowUp({ request: r, isSender }: { request: Row; isSender: boolean }) {
  const { data, error, loading, reload } = useData<Row>(`/requests/${r.id}/follow-up`, 15000);
  const [editing, setEditing] = useState(false);
  const active = ['open', 'accepted'].includes(r.status);
  const f = data?.follow_up;
  return (
    <section class="request-follow-through">
      {isSender && data?.delivery && <DeliverySummary delivery={data.delivery} />}
      <div class="follow-up-heading">
        <h3>
          <Icon name="clock" size={17} />
          Follow-through
        </h3>
        {isSender && active && (
          <button type="button" class="text-link" onClick={() => setEditing(true)}>
            {f?.reminder_at || f?.escalation_at ? 'Change schedule' : 'Schedule follow-up'}
          </button>
        )}
      </div>
      {error ? (
        <>
          <ErrorBox message={error} />
          <button type="button" class="text-link" onClick={reload}>
            Reload follow-through
          </button>
        </>
      ) : loading ? (
        <Loading />
      ) : (
        <>
          {f?.reminder_at && (
            <p class="follow-up-line">
              <Badge>
                {f.reminder_processed_at ? 'Processed' : active ? 'Scheduled' : 'Stopped'}
              </Badge>
              Reminder · {requestDate(f.reminder_at)}
            </p>
          )}
          {f?.escalation_at && (
            <p class="follow-up-line">
              <Badge>
                {f.escalation_processed_at
                  ? 'Processed'
                  : r.status === 'open'
                    ? 'Scheduled'
                    : 'Stopped'}
              </Badge>
              If unclaimed, ask {f.escalation_name} · {requestDate(f.escalation_at)}
            </p>
          )}
          {!f?.reminder_at && !f?.escalation_at && (
            <p class="small-note">No automatic follow-ups scheduled.</p>
          )}
          {!active && (
            <p class="small-note">
              This request is closed. No further reminders or escalations will be sent.
            </p>
          )}
          {isSender && active && (
            <div class="follow-up-actions">
              <Action
                disabled={!reminderReady(r)}
                successMessage="Reminder saved to the recipients’ inboxes. Phone alerts follow their preferences."
                action={async () => {
                  await api(`/requests/${r.id}/remind`, 'POST', {});
                  refresh();
                }}
              >
                Send a reminder
                <Icon name="bell" size={15} />
              </Action>
              <small>
                {r.status === 'accepted'
                  ? 'Only the person who accepted gets the reminder.'
                  : 'Reminds the active people who can accept this request.'}{' '}
                Allow 15 minutes between reminders, starting when the request was sent.
              </small>
            </div>
          )}
          {isSender && active && (f?.reminder_at || f?.escalation_at) && (
            <Action
              className="text-link"
              successMessage="Automatic follow-ups removed."
              action={async () => {
                await api(`/requests/${r.id}/follow-up`, 'PUT', {
                  reminder_at: null,
                  escalation_at: null,
                  escalation_user_id: null,
                });
                refresh();
              }}
            >
              Remove automatic follow-ups
            </Action>
          )}
          <h3 class="request-history-heading">Request history</h3>
          {data?.activity?.length ? (
            <ol class="request-history">
              {data.activity.map((event: Row) => (
                <li key={event.id}>
                  <span class="history-dot" aria-hidden="true" />
                  <div>
                    <strong>{activityLabels[event.kind] || 'Request updated'}</strong>
                    <p>
                      {event.actor_name || 'Automatic follow-up'} · {requestDate(event.created_at)}
                    </p>
                    {event.kind === 'reminded' && (
                      <small>
                        {event.details.recipient_count} active recipient
                        {event.details.recipient_count === 1 ? '' : 's'}
                      </small>
                    )}
                    {event.details.reason && (
                      <small>
                        {event.details.reason === 'already_accepted_or_closed'
                          ? 'The request was already accepted or closed.'
                          : event.details.reason === 'closed_or_recent_reminder'
                            ? 'The request was closed or recently reminded.'
                            : event.details.reason === 'no_active_recipients'
                              ? 'There were no active recipients to remind.'
                              : 'The selected person was unavailable or sending access had changed.'}
                      </small>
                    )}
                  </div>
                </li>
              ))}
            </ol>
          ) : (
            <p class="small-note">New activity will appear here as this request progresses.</p>
          )}
          {data?.activity?.length === 100 && (
            <p class="small-note">
              Showing the latest 100 events. Earlier audited activity is retained.
            </p>
          )}
        </>
      )}
      {editing && (
        <ScheduleEditor
          request={r}
          onClose={() => setEditing(false)}
          onSaved={() => {
            setEditing(false);
            refresh();
          }}
        />
      )}
    </section>
  );
}
function ScheduleEditor({
  request: r,
  onClose,
  onSaved,
}: {
  request: Row;
  onClose: () => void;
  onSaved: () => void;
}) {
  const { data, error, loading, reload } = useData<Row>(
    r.status === 'open' ? '/requests/recipients' : null,
  );
  const intervals = [
    { value: '15', label: 'In 15 minutes' },
    { value: '30', label: 'In 30 minutes' },
    { value: '60', label: 'In 1 hour' },
    { value: '120', label: 'In 2 hours' },
    { value: '240', label: 'In 4 hours' },
    { value: '1440', label: 'In 1 day' },
  ];
  return (
    <Modal title="Schedule request follow-up" onClose={onClose}>
      <p class="modal-description">
        A reminder goes to the team while the request is open, or just its assignee after
        acceptance. Escalation adds your chosen person to this request only if it remains unclaimed.
        They will be able to read its details and accept it.
      </p>
      <p class="small-note">
        Saving replaces the current schedule. Times are calculated when you save. Each scheduled
        step runs once; closed requests are skipped. Phone preferences always apply.
      </p>
      {error ? (
        <>
          <ErrorBox message={error} />
          <button type="button" class="button secondary" onClick={reload}>
            Try again
          </button>
        </>
      ) : loading ? (
        <Loading />
      ) : (
        <Form
          fields={[
            {
              name: 'reminder_after',
              label: 'Send a reminder',
              type: 'select',
              options: intervals,
            },
            ...(r.status === 'open'
              ? [
                  {
                    name: 'escalation_after',
                    label: 'If still unclaimed, escalate',
                    type: 'select',
                    options: intervals,
                  },
                  {
                    name: 'escalation_user_id',
                    label: 'Escalate to',
                    type: 'select',
                    options: (data?.people || []).map((person: Row) => ({
                      value: person.id,
                      label: `${person.display_name}${person.department ? ` · ${person.department}` : ''}`,
                    })),
                    help: 'Choose someone in your sending scope who should help resolve this request.',
                  },
                ]
              : []),
          ]}
          submit="Save follow-up schedule"
          onSubmit={async (values) => {
            await api(`/requests/${r.id}/follow-up`, 'PUT', followUpPayload(values));
            notify('Follow-up schedule saved.');
            onSaved();
          }}
        />
      )}
    </Modal>
  );
}
