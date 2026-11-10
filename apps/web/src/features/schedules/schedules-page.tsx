import { useState } from 'preact/hooks';

import { Empty, ErrorBox, Form, Icon, Loading, Modal, PageHead, Panel } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { date } from '../../lib/date';
import { refresh } from '../../lib/events';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
export function Schedules(_: { path?: string }) {
  const me = useAuth(),
    [eventId, setEventId] = useState(''),
    [add, setAdd] = useState(false);
  const { data: org } = useData<Row>('/organization/tree'),
    { data, error, loading } = useData<Row[]>(
      '/schedules' + (eventId ? '?event_id=' + eventId : ''),
    );
  const departments =
    org?.departments.filter((d: Row) =>
      me.grants.some(
        (g) =>
          g.permission === 'people.edit' &&
          (g.scope_type === 'global' ||
            (g.scope_type === 'department' && g.scope_id === d.id) ||
            (g.scope_type === 'division' && g.scope_id === d.division_id)),
      ),
    ) || [];
  return (
    <>
      <PageHead
        eyebrow="THE WEEKEND AT A GLANCE"
        title="Schedules"
        description="Plan coverage, share openings, and see where everyone needs to be."
      >
        {departments.length > 0 && (
          <button type="button" class="button primary" onClick={() => setAdd(true)}>
            <Icon name="plus" size={16} /> Add shift
          </button>
        )}
      </PageHead>
      <label class="field">
        Event
        <select value={eventId} onChange={(e) => setEventId(e.currentTarget.value)}>
          <option value="">All active events</option>
          {org?.events.map((e: Row) => (
            <option key={e.id} value={e.id}>
              {e.name}
            </option>
          ))}
        </select>
      </label>
      {error ? (
        <ErrorBox message={error} />
      ) : loading ? (
        <Loading />
      ) : data?.length ? (
        <div class="card-grid">
          {data.map((s) => (
            <Panel key={s.id}>
              <strong>{s.title}</strong>
              <p>
                {s.department} · {s.event}
              </p>
              <p>
                <Icon name="calendar" size={16} /> {date(s.starts_at, true)}
              </p>
              <p>
                {s.assigned_count}/{s.slots} filled
              </p>
              <p class="small-note">
                {s.assigned.map((u: Row) => u.display_name).join(', ') || 'Open shift'}
              </p>
              {!s.claimed_by_me && s.assigned_count < s.slots && me.user.volunteer_eligible && (
                <button
                  class="button secondary"
                  type="button"
                  onClick={async () => {
                    await api('/schedules/' + s.id + '/claim', 'POST', {});
                    refresh();
                  }}
                >
                  Claim opening
                </button>
              )}
            </Panel>
          ))}
        </div>
      ) : (
        <Empty title="No shifts scheduled yet">Choose an event and add the first shift.</Empty>
      )}
      {add && (
        <Modal title="Add a shift" onClose={() => setAdd(false)}>
          <Form
            submit="Publish shift"
            fields={[
              {
                name: 'event_edition_id',
                label: 'Event',
                type: 'select',
                required: true,
                options: org?.events.map((e: Row) => ({ value: e.id, label: e.name })),
              },
              {
                name: 'department_id',
                label: 'Department',
                type: 'select',
                required: true,
                options: departments.map((d: Row) => ({ value: d.id, label: d.name })),
              },
              { name: 'title', label: 'Shift title', required: true },
              { name: 'starts_at', label: 'Starts', type: 'datetime-local', required: true },
              { name: 'ends_at', label: 'Ends', type: 'datetime-local', required: true },
              { name: 'location_label', label: 'Location' },
              {
                name: 'slots',
                label: 'Openings',
                type: 'number',
                required: true,
                value: 1,
                min: 1,
                max: 200,
              },
            ]}
            onSubmit={async (b) => {
              await api('/schedules', 'POST', {
                ...b,
                starts_at: new Date(b.starts_at).toISOString(),
                ends_at: new Date(b.ends_at).toISOString(),
              });
              setAdd(false);
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
