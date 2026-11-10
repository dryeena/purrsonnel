import { useEffect, useState } from 'preact/hooks';
import { route } from 'preact-router';

import { PhoneAlerts } from '../../components/phone-alerts';
import {
  Avatar,
  Badge,
  Empty,
  ErrorBox,
  Icon,
  Loading,
  Modal,
  PageHead,
  Panel,
  Status,
} from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { requestDate } from '../../lib/alert-settings';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { isRequestOverdue, relativeTime } from '../../lib/request-presentation';
import type { Row } from '../../types/api';

import { Compose } from './compose-dialog';
import { RequestDetails } from './request-details';

export function Requests(incoming: { path?: string; id?: string }) {
  const me = useAuth();
  const [tab, setTab] = useState('inbox');
  const [page, setPage] = useState(1);
  const [compose, setCompose] = useState(false);
  const [selected, setSelected] = useState<string | null>(
    new URLSearchParams(location.search).get('id'),
  );
  const [sent, setSent] = useState<Row | null>(null);
  const [q, setQ] = useState('');
  const [search, setSearch] = useState('');
  const [status, setStatus] = useState('all');
  useEffect(() => {
    if (incoming.id) setSelected(incoming.id);
  }, [incoming.id]);
  useEffect(() => {
    if (q === search) return;
    const timer = setTimeout(() => {
      setSearch(q);
      setPage(1);
    }, 250);
    return () => clearTimeout(timer);
  }, [q, search]);
  const { data, error, loading, refreshing, updatedAt, reload } = useData<Row[]>(
    `/requests?page=${page}&page_size=25&view=${tab}&status=${status}&q=${encodeURIComponent(search)}`,
    15000,
  );
  const rows = data?.filter((r) =>
    tab === 'sent' ? r.sender_user_id === me.user.id : r.is_recipient,
  );
  const filtered = !!search || status !== 'all';
  const clearFilters = () => {
    setQ('');
    setSearch('');
    setStatus('all');
    setPage(1);
  };
  const emptyTitle = filtered
    ? 'No requests match these filters'
    : page > 1
      ? 'No more requests on this page'
      : tab === 'inbox'
        ? 'A little breathing room.'
        : 'Your next helping hand starts here.';
  const emptyAction = filtered ? (
    <button type="button" class="button secondary" onClick={clearFilters}>
      Clear filters
    </button>
  ) : page > 1 ? (
    <button type="button" class="button secondary" onClick={() => setPage((p) => p - 1)}>
      Go to previous page
    </button>
  ) : tab === 'sent' && can(me, 'requests.send') ? (
    <button type="button" class="button primary" onClick={() => setCompose(true)}>
      Send your first request
      <Icon name="arrow" size={16} />
    </button>
  ) : undefined;
  return (
    <>
      <PageHead
        eyebrow="A HELPING HAND, ONE TAP AWAY"
        title="Requests"
        description="Ask a teammate or a department for help. See who’s on it and when it’s done."
      >
        {can(me, 'requests.send') && (
          <button type="button" class="button primary" onClick={() => setCompose(true)}>
            <Icon name="plus" size={17} />
            Send a request
          </button>
        )}
      </PageHead>
      <PhoneAlerts />
      <div class="tabs" role="tablist" aria-label="Request views">
        {[
          ['inbox', 'For me & my teams'],
          ['sent', 'Sent by me'],
        ].map(([key, label]) => (
          <button
            type="button"
            key={key}
            role="tab"
            aria-selected={tab === key}
            class={tab === key ? 'active' : ''}
            onClick={() => {
              setTab(key);
              setPage(1);
            }}
          >
            {label}
          </button>
        ))}
      </div>
      <Panel>
        <div class="request-controls">
          <div class="input-icon">
            <Icon name="search" size={17} />
            <input
              aria-label="Search requests"
              placeholder="Find a request…"
              value={q}
              maxLength={160}
              onInput={(e) => setQ(e.currentTarget.value)}
            />
          </div>
          <label class="request-status-filter">
            <span>Status</span>
            <select
              aria-label="Filter request status"
              value={status}
              onChange={(e) => {
                setStatus(e.currentTarget.value);
                setPage(1);
              }}
            >
              {[
                ['all', 'All requests'],
                ['active', 'Needs action'],
                ['open', 'Not yet accepted'],
                ['accepted', 'In progress'],
                ['completed', 'Completed'],
                ['cancelled', 'Cancelled'],
              ].map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            class="icon-button"
            aria-label="Refresh requests"
            disabled={refreshing}
            onClick={reload}
          >
            {refreshing ? (
              <span class="spinner button-spinner" aria-hidden="true" />
            ) : (
              <Icon name="refresh" size={18} />
            )}
          </button>
        </div>
        {(filtered || updatedAt) && (
          <div class="request-list-meta">
            <span>
              {filtered ? 'Filtered view' : 'Your authorized requests'}
              {!loading && rows?.length ? ` · ${rows.length} on this page` : ''}
            </span>
            {filtered ? (
              <button type="button" class="text-link" onClick={clearFilters}>
                Clear filters
              </button>
            ) : (
              updatedAt && (
                <small>
                  Last loaded{' '}
                  {new Intl.DateTimeFormat('en-US', { hour: 'numeric', minute: '2-digit' }).format(
                    updatedAt,
                  )}
                </small>
              )
            )}
          </div>
        )}
        {error ? (
          <div class="request-load-error">
            <ErrorBox message={error} />
            <button type="button" class="button secondary" onClick={reload}>
              Try again
              <Icon name="refresh" size={16} />
            </button>
          </div>
        ) : loading ? (
          <Loading />
        ) : rows?.length ? (
          <div class="request-list">
            {rows.map((r) => (
              <button
                type="button"
                key={r.id}
                class={`request-row ${isRequestOverdue(r) ? 'is-overdue' : ''}`}
                onClick={() => setSelected(r.id)}
              >
                <Avatar name={r.sender_name} />
                <div class="request-summary">
                  <div>
                    <strong>{r.title}</strong>
                    {r.urgency === 'urgent' && <Badge tone="amber">Urgent</Badge>}
                    {isRequestOverdue(r) && <Badge tone="red">Overdue</Badge>}
                  </div>
                  <p>
                    {r.sender_name} <span aria-hidden="true">→</span>
                    <span class="sr-only">to</span> {r.recipient_name}
                  </p>
                  <small>
                    {r.assigned_to_name ? (
                      `${r.assigned_to_name} is on it`
                    ) : (
                      <time dateTime={r.created_at} title={requestDate(r.created_at)}>
                        {relativeTime(r.created_at)}
                      </time>
                    )}
                    {r.due_at ? ` · Due ${requestDate(r.due_at)}` : ''}
                  </small>
                </div>
                <Status value={r.status} />
                <Icon name="chevron" size={16} />
              </button>
            ))}
          </div>
        ) : (
          <Empty title={emptyTitle} icon={tab === 'inbox' ? 'check' : 'bell'} action={emptyAction}>
            {filtered
              ? 'Try a different phrase or broaden the status filter.'
              : tab === 'inbox'
                ? 'Requests addressed to you and your teams will land here. We’ll keep the details together.'
                : 'Choose a person or team, tell them what would help, and follow the progress here.'}
          </Empty>
        )}
        <div class="pagination">
          <span>Page {page}</span>
          <div>
            <button
              type="button"
              class="button secondary small-button"
              disabled={page === 1}
              onClick={() => setPage((p) => p - 1)}
            >
              Previous
            </button>
            <button
              type="button"
              class="button secondary small-button"
              disabled={!data || data.length < 25}
              onClick={() => setPage((p) => p + 1)}
            >
              Next
            </button>
          </div>
        </div>
      </Panel>
      {compose && (
        <Compose
          onClose={() => setCompose(false)}
          onSent={(result) => {
            setCompose(false);
            setSent(result);
            setTab('sent');
            setPage(1);
            clearFilters();
            refresh();
          }}
        />
      )}
      {selected && (
        <RequestDetails
          key={selected}
          id={selected}
          onClose={() => {
            setSelected(null);
            if (new URLSearchParams(location.search).has('id')) route('/requests', true);
          }}
        />
      )}
      {sent && (
        <Modal title="Your request is sent" onClose={() => setSent(null)}>
          <div class="sent-receipt">
            <span>
              <Icon name="check" size={27} />
            </span>
            <h3>Your request is in the right place.</h3>
            <p>
              {sent.replayed
                ? 'This request was already sent. We kept the original instead of creating a duplicate.'
                : `${sent.recipient_count} ${sent.recipient_count === 1 ? 'person has' : 'people have'} a new request in Purrsonnel.`}
            </p>
          </div>
          <p class="modal-description">
            {sent.push_queued > 0
              ? `Phone alerts are queued for ${sent.push_queued} subscribed device${sent.push_queued === 1 ? '' : 's'}. Quiet hours may defer delivery; device settings still apply.`
              : sent.replayed
                ? 'You can check the current status below.'
                : 'No phone alerts were queued. Recipients can still find the request in their inbox. Phone alerts depend on subscriptions, preferences, and workspace configuration.'}
          </p>
          {sent.push_suppressed > 0 && (
            <p class="small-note">
              {sent.push_suppressed} device alert{sent.push_suppressed === 1 ? ' was' : 's were'}{' '}
              suppressed by recipient preferences.
            </p>
          )}
          <button
            type="button"
            class="button primary receipt-action"
            onClick={() => {
              setSelected(sent.id);
              setSent(null);
            }}
          >
            View request & follow-up options
            <Icon name="arrow" size={16} />
          </button>
        </Modal>
      )}
    </>
  );
}
