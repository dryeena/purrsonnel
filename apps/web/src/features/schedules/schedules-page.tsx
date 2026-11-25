import { Fragment } from 'preact';
import { useState } from 'preact/hooks';

import {
  Action,
  Empty,
  ErrorBox,
  Form,
  Icon,
  Loading,
  Modal,
  PageHead,
  Panel,
} from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { date } from '../../lib/date';
import { refresh } from '../../lib/events';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
export function Schedules(_: { path?: string }) {
  const me = useAuth(),
    [eventId, setEventId] = useState(''),
    [add, setAdd] = useState(false),
    [week, setWeek] = useState(() => {
      const d = new Date();
      d.setHours(0, 0, 0, 0);
      d.setDate(d.getDate() - d.getDay());
      return d;
    }),
    [selectedStart, setSelectedStart] = useState(''),
    [assign, setAssign] = useState<Row | null>(null);
  const { data: org } = useData<Row>('/organization/tree'),
    { data, error, loading } = useData<Row[]>(
      '/schedules' + (eventId ? '?event_id=' + eventId : ''),
    );
  const { data: people } = useData<Row[]>(
    assign ? '/schedules/staff?department_id=' + assign.department_id : null,
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
  const days = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(week);
    d.setDate(d.getDate() + i);
    return d;
  });
  const localInput = (d: Date) =>
    [
      d.getFullYear(),
      String(d.getMonth() + 1).padStart(2, '0'),
      String(d.getDate()).padStart(2, '0'),
    ].join('-') +
    'T' +
    String(d.getHours()).padStart(2, '0') +
    ':00';
  return (
    <>
      <PageHead
        eyebrow="THE WEEKEND AT A GLANCE"
        title="Schedules"
        description="Plan coverage, share openings, and see where everyone needs to be."
      >
        {departments.length > 0 && (
          <button
            type="button"
            class="button primary"
            onClick={() => {
              setSelectedStart('');
              setAdd(true);
            }}
          >
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
      <section class="schedule-calendar" aria-label="Weekly shift calendar">
        <div class="schedule-calendar-header">
          <button
            type="button"
            class="button secondary"
            onClick={() =>
              setWeek(new Date(week.getFullYear(), week.getMonth(), week.getDate() - 7))
            }
          >
            Previous week
          </button>
          <strong>
            {days[0].toLocaleDateString()} – {days[6].toLocaleDateString()}
          </strong>
          <button
            type="button"
            class="button secondary"
            onClick={() =>
              setWeek(new Date(week.getFullYear(), week.getMonth(), week.getDate() + 7))
            }
          >
            Next week
          </button>
        </div>
        <div class="schedule-calendar-scroll">
          <div class="schedule-calendar-grid">
            <span>Time</span>
            {days.map((d) => (
              <strong key={d.toISOString()}>
                {d.toLocaleDateString(undefined, {
                  weekday: 'short',
                  month: 'short',
                  day: 'numeric',
                })}
              </strong>
            ))}
            {Array.from({ length: 24 }, (_, i) => i).map((hour) => (
              <Fragment key={hour}>
                <span key={'hour-' + hour}>
                  {hour % 12 || 12} {hour < 12 ? 'AM' : 'PM'}
                </span>
                {days.map((day) => {
                  const slot = new Date(day);
                  slot.setHours(hour);
                  const shifts =
                    data?.filter((s) => {
                      const starts = new Date(s.starts_at);
                      return (
                        starts.getFullYear() === slot.getFullYear() &&
                        starts.getMonth() === slot.getMonth() &&
                        starts.getDate() === slot.getDate() &&
                        starts.getHours() === hour
                      );
                    }) || [];
                  return (
                    <div class="schedule-slot" key={localInput(slot)}>
                      {shifts.map((s) => (
                        <span key={s.id} class="schedule-slot-shift">
                          {s.title}
                        </span>
                      ))}
                      {departments.length > 0 && (
                        <button
                          type="button"
                          aria-label={'Add shift ' + slot.toLocaleString()}
                          onClick={() => {
                            setSelectedStart(localInput(slot));
                            setAdd(true);
                          }}
                        >
                          +
                        </button>
                      )}
                    </div>
                  );
                })}
              </Fragment>
            ))}
          </div>
        </div>
      </section>
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
                <Icon name="calendar" size={16} /> {date(s.starts_at, true)} –{' '}
                {date(s.ends_at, true)}
              </p>
              {s.location_label && <p>{s.location_label}</p>}
              <p>
                {s.assigned_count}/{s.slots} filled
              </p>
              <p class="small-note">
                {s.assigned.map((u: Row) => u.display_name).join(', ') || 'Open shift'}
              </p>
              {s.can_manage && s.assigned_count < s.slots && (
                <button type="button" class="button secondary" onClick={() => setAssign(s)}>
                  Assign staff
                </button>
              )}
              {!s.claimed_by_me && s.assigned_count < s.slots && me.user.volunteer_eligible && (
                <Action
                  action={async () => {
                    await api('/schedules/' + s.id + '/claim', 'POST', {});
                    refresh();
                  }}
                >
                  Claim opening
                </Action>
              )}
            </Panel>
          ))}
        </div>
      ) : (
        <Empty title="No shifts scheduled yet">Choose an event and add the first shift.</Empty>
      )}
      {assign && (
        <Modal title="Assign staff" onClose={() => setAssign(null)}>
          <Form
            fields={[
              {
                name: 'user_id',
                label: 'Staff member',
                type: 'select',
                required: true,
                options: people?.map((p: Row) => ({ value: p.id, label: p.display_name })),
              },
            ]}
            submit="Assign"
            onSubmit={async (b) => {
              await api('/schedules/' + assign.id + '/assignments', 'POST', b);
              setAssign(null);
              refresh();
            }}
          />
        </Modal>
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
                value: eventId,
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
              {
                name: 'starts_at',
                label: 'Starts',
                type: 'datetime-local',
                required: true,
                value: selectedStart,
              },
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
