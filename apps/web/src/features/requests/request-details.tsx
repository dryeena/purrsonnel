import { useState } from 'preact/hooks';

import { RequestFollowUp } from '../../components/request-follow-up';
import { Action, Badge, ErrorBox, Icon, Loading, Modal, Status } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { requestDate } from '../../lib/alert-settings';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { isRequestOverdue } from '../../lib/request-presentation';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

import { RequestProgress } from './request-progress';

export function RequestDetails({ id, onClose }: { id: string; onClose: () => void }) {
  const me = useAuth();
  const { data: r, error, loading, refreshing, reload } = useData<Row>(`/requests/${id}`, 15000);
  const [cancelling, setCancelling] = useState(false);
  return (
    <Modal title="Request details" onClose={onClose} wide>
      {error ? (
        <div class="request-load-error">
          <ErrorBox message={error} />
          <button type="button" class="button secondary" onClick={reload}>
            Try again
          </button>
        </div>
      ) : loading ? (
        <Loading />
      ) : (
        r && (
          <>
            <div class="policy-top">
              <Status value={r.status} />
              <div class="tag-stack">
                {r.urgency === 'urgent' && <Badge tone="amber">Urgent</Badge>}
                {isRequestOverdue(r) && <Badge tone="red">Overdue</Badge>}
              </div>
            </div>
            <h2 class="request-title">{r.title}</h2>
            <p class="modal-description">
              {r.sender_name} → {r.recipient_name} · {requestDate(r.created_at)}
            </p>
            <RequestProgress request={r} />
            <div class="document-body request-body">{r.body}</div>
            {r.due_at && <p class="small-note">Needed by {requestDate(r.due_at)}</p>}
            {r.assigned_to_name && (
              <div class="info-note">
                <Icon name="check" />
                {r.assigned_to_name}{' '}
                {r.status === 'completed' ? 'completed this request.' : 'accepted this request.'}
              </div>
            )}
            <div class="row-actions request-decision-actions">
              {r.status === 'open' && r.is_recipient && (
                <Action
                  disabled={refreshing}
                  className="button primary"
                  successMessage="Request accepted. Your team can see you’re on it."
                  action={async () => {
                    await api(`/requests/${id}/respond`, 'POST', { action: 'accept' });
                    refresh();
                  }}
                >
                  I’m on it
                  <Icon name="check" size={16} />
                </Action>
              )}
              {r.status === 'accepted' &&
                (r.assigned_to_user_id === me.user.id || r.sender_user_id === me.user.id) && (
                  <Action
                    disabled={refreshing}
                    className="button primary"
                    successMessage="Request marked complete. Thank you for helping."
                    action={async () => {
                      await api(`/requests/${id}/respond`, 'POST', { action: 'complete' });
                      refresh();
                    }}
                  >
                    Mark completed
                    <Icon name="check" size={16} />
                  </Action>
                )}
              {r.sender_user_id === me.user.id &&
                ['open', 'accepted'].includes(r.status) &&
                !cancelling && (
                  <button type="button" class="text-link" onClick={() => setCancelling(true)}>
                    Cancel request
                  </button>
                )}
            </div>
            {cancelling && (
              <div class="confirm-inline">
                <p>Cancel this request? Your team will see that it’s no longer needed.</p>
                <div class="row-actions">
                  <Action
                    disabled={refreshing}
                    successMessage="Request cancelled. It is no longer awaiting action."
                    action={async () => {
                      await api(`/requests/${id}/respond`, 'POST', { action: 'cancel' });
                      setCancelling(false);
                      refresh();
                    }}
                  >
                    Yes, cancel request
                  </Action>
                  <button type="button" class="text-link" onClick={() => setCancelling(false)}>
                    Keep request
                  </button>
                </div>
              </div>
            )}
            <div class="request-updated">
              <p>
                {r.status === 'completed' && r.completed_at
                  ? `Completed ${requestDate(r.completed_at)}`
                  : `Last updated ${requestDate(r.updated_at)}`}
              </p>
              <button type="button" class="text-link" onClick={reload} disabled={refreshing}>
                {refreshing ? 'Refreshing…' : 'Refresh details'}
              </button>
            </div>
            <RequestFollowUp request={r} isSender={r.sender_user_id === me.user.id} />
          </>
        )
      )}
    </Modal>
  );
}
