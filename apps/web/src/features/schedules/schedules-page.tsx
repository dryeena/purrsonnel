/* eslint-disable max-lines -- The calendar and its linked dialogs share state. */
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
    [lastShift, setLastShift] = useState<Row | null>(null),
    [copyShift, setCopyShift] = useState<Row | null>(null),
    [selectedShift, setSelectedShift] = useState<Row | null>(null),
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
    ':' +
    String(d.getMinutes()).padStart(2, '0');
  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
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
              setCopyShift(null);
              setAdd(true);
            }}
          >
            <Icon name="plus" size={16} /> Add shift
          </button>
        )}
      </PageHead>
      <label class="field">
        Event
        <select
          value={eventId}
          onChange={(e) => {
            const id = e.currentTarget.value;
            setEventId(id);
            const event = org?.events.find((entry: Row) => entry.id === id);
            if (event?.starts_at) {
              const start = new Date(event.starts_at);
              start.setHours(0, 0, 0, 0);
              start.setDate(start.getDate() - 1);
              setWeek(start);
            }
          }}
        >
          <option value="">All active events</option>
          {org?.events.map((e: Row) => (
            <option key={e.id} value={e.id}>
              {e.name}
            </option>
          ))}
        </select>
      </label>
      <section class="schedule-calendar" aria-label="Weekly shift calendar">
        {departments.length > 0 && (
          <p class="small-note">
            Hold Shift while selecting a time to copy your last created shift.
          </p>
        )}
        <div class="schedule-calendar-header">
          <button
            type="button"
            class="button secondary"
            onClick={() => {
              const today = new Date();
              today.setHours(0, 0, 0, 0);
              today.setDate(today.getDate() - today.getDay());
              setWeek(today);
            }}
          >
            Today
          </button>
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
              <strong
                key={d.toISOString()}
                class={d.toDateString() === new Date().toDateString() ? 'schedule-today' : ''}
              >
                <span>{d.toLocaleDateString(undefined, { weekday: 'short' })}</span>
                <b>{d.getDate()}</b>
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
                        <button
                          type="button"
                          key={s.id}
                          class="schedule-slot-shift"
                          onClick={() => setSelectedShift(s)}
                        >
                          <strong>{s.title}</strong>
                          <small>
                            {new Date(s.starts_at).toLocaleTimeString([], {
                              hour: 'numeric',
                              minute: '2-digit',
                            })}{' '}
                            –{' '}
                            {new Date(s.ends_at).toLocaleTimeString([], {
                              hour: 'numeric',
                              minute: '2-digit',
                            })}
                          </small>
                        </button>
                      ))}
                      {departments.length > 0 && (
                        <button
                          type="button"
                          aria-label={'Add shift ' + slot.toLocaleString()}
                          onClick={(event) => {
                            setSelectedStart(localInput(slot));
                            setCopyShift(event.shiftKey ? lastShift : null);
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
      {selectedShift && (
        <Modal title={selectedShift.title} onClose={() => setSelectedShift(null)}>
          <p>
            {selectedShift.department} · {selectedShift.event}
          </p>
          <p>
            {date(selectedShift.starts_at, true, timezone)} –{' '}
            {date(selectedShift.ends_at, true, timezone)}
          </p>
          {selectedShift.location_label && <p>{selectedShift.location_label}</p>}
          <p>
            {selectedShift.assigned_count} of {selectedShift.slots} places filled
          </p>
          <p>
            {selectedShift.assigned.map((person: Row) => person.display_name).join(', ') ||
              'No one assigned yet'}
          </p>
          {selectedShift.can_manage && selectedShift.assigned_count < selectedShift.slots && (
            <button
              type="button"
              class="button primary"
              onClick={() => {
                setAssign(selectedShift);
                setSelectedShift(null);
              }}
            >
              Assign staff
            </button>
          )}
          {!selectedShift.claimed_by_me &&
            selectedShift.assigned_count < selectedShift.slots &&
            me.user.volunteer_eligible && (
              <Action
                action={async () => {
                  await api('/schedules/' + selectedShift.id + '/claim', 'POST', {});
                  setSelectedShift(null);
                  refresh();
                }}
              >
                Claim opening
              </Action>
            )}
        </Modal>
      )}
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
                <Icon name="calendar" size={16} /> {date(s.starts_at, true, timezone)} –{' '}
                {date(s.ends_at, true, timezone)}
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
                value: copyShift?.event_edition_id || eventId,
                options: org?.events.map((e: Row) => ({ value: e.id, label: e.name })),
              },
              {
                name: 'department_id',
                label: 'Department',
                type: 'select',
                required: true,
                value: copyShift?.department_id,
                options: departments.map((d: Row) => ({ value: d.id, label: d.name })),
              },
              { name: 'title', label: 'Shift title', required: true, value: copyShift?.title },
              {
                name: 'starts_at',
                label: 'Starts',
                type: 'datetime-local',
                required: true,
                value: selectedStart,
              },
              {
                name: 'ends_at',
                label: 'Ends',
                type: 'datetime-local',
                required: true,
                value:
                  copyShift && selectedStart
                    ? localInput(
                        new Date(
                          new Date(selectedStart).getTime() +
                            (new Date(copyShift.ends_at).getTime() -
                              new Date(copyShift.starts_at).getTime()),
                        ),
                      )
                    : selectedStart
                      ? localInput(new Date(new Date(selectedStart).getTime() + 60 * 60 * 1000))
                      : undefined,
              },
              { name: 'location_label', label: 'Location', value: copyShift?.location_label },
              {
                name: 'slots',
                label: 'Openings',
                type: 'number',
                required: true,
                value: copyShift?.slots ?? 1,
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
              setLastShift({ ...b });
              setAdd(false);
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
