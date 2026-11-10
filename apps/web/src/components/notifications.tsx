import { useState } from 'preact/hooks';

import { useData } from '../hooks/use-data';
import { date } from '../lib/date';
import { refresh } from '../lib/events';
import { humanize } from '../lib/text';
import { api } from '../services/api-client';
import type { Row } from '../types/api';

import { Action, Badge, Empty, Icon, Modal } from './ui';
export function Notifications() {
  const [open, setOpen] = useState(false);
  const { data, error } = useData<Row[]>('/notifications', 15000);
  const unread = data?.some((notification) => !notification.read_at) ?? false;
  return (
    <>
      <button
        type="button"
        class={`icon-button notification-button ${unread ? 'has-unread' : ''}`}
        aria-label={unread ? 'Notifications: unread messages' : 'Notifications'}
        onClick={() => setOpen(true)}
      >
        <Icon name="bell" size={21} />
        {unread && <i />}
      </button>
      {open && (
        <Modal title="Your notifications" onClose={() => setOpen(false)}>
          <a class="text-link" href="/requests" onClick={() => setOpen(false)}>
            Inbox & phone alert settings →
          </a>
          {error && (
            <p class="error-box" role="alert">
              {error}
            </p>
          )}
          {data?.length ? (
            <div class="note-list">
              {data.map((n) => (
                <article key={n.id}>
                  <div>
                    <strong>
                      {n.type === 'application_decision'
                        ? 'An application update'
                        : n.type === 'service_request'
                          ? 'A new request'
                          : n.type === 'request_update'
                            ? 'A request update'
                            : n.type === 'request_reminder'
                              ? 'A request reminder'
                              : n.type === 'request_escalated'
                                ? 'An unclaimed request needs help'
                                : humanize(n.type)}
                    </strong>
                    {!n.read_at && <Badge tone="plum">New</Badge>}
                  </div>
                  <p>
                    {n.payload_json.status
                      ? `Status: ${n.payload_json.status}`
                      : 'There is an update in your workspace.'}
                  </p>
                  <small>{date(n.created_at, true)}</small>
                  <div class="row-actions">
                    {n.payload_json.request_id &&
                      /^[0-9a-f-]{36}$/i.test(n.payload_json.request_id) && (
                        <a
                          class="text-link"
                          href={`/requests?id=${n.payload_json.request_id}`}
                          onClick={() => setOpen(false)}
                        >
                          View request →
                        </a>
                      )}
                    {!n.read_at && (
                      <Action
                        className="text-link"
                        action={async () => {
                          await api(`/notifications/${n.id}/read`, 'POST');
                          refresh();
                        }}
                      >
                        Mark read
                      </Action>
                    )}
                  </div>
                </article>
              ))}
            </div>
          ) : (
            !error && <Empty title="All quiet here">Updates meant for you will appear here.</Empty>
          )}
        </Modal>
      )}
    </>
  );
}
