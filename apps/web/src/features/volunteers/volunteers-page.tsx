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
  Status,
} from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { date } from '../../lib/date';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import { VolunteerControls } from '../lifecycle';

import { Scanner } from './approval-scanner';
import { QrModal } from './qr-dialog';

export function Volunteers(_: { path?: string }) {
  const me = useAuth();
  const [newEntry, setNewEntry] = useState(false);
  const [qr, setQr] = useState<Row | null>(null);
  const [scan, setScan] = useState(location.pathname.endsWith('/approve'));
  const [tab, setTab] = useState('entries');
  const [timeMode, setTimeMode] = useState(false);
  const {
    data: entries,
    error,
    loading,
  } = useData<Row[]>('/volunteer/entries?page_size=100', 15000);
  const { data: org } = useData<Row>('/organization/tree');
  const { data: report } = useData<Row[]>('/volunteer/reports/hours');
  const approve = can(me, 'volunteer.time.approve');
  const rows =
    tab === 'mine' ? entries?.filter((e) => e.volunteer_user_id === me.user.id) : entries;
  const minutes =
    entries
      ?.filter((e) => e.volunteer_user_id === me.user.id && e.status === 'approved')
      .reduce((n, e) => n + e.minutes_worked, 0) || 0;
  return (
    <>
      <PageHead
        eyebrow="EVERY HELPING HAND MATTERS"
        title="Volunteers"
        description="Keep track of the time and care that make FWA possible."
      >
        {approve && (
          <button type="button" class="button secondary" onClick={() => setScan(true)}>
            <Icon name="qr" size={17} />
            Scan approval QR
          </button>
        )}
        {me.user.volunteer_eligible && (
          <button type="button" class="button primary" onClick={() => setNewEntry(true)}>
            <Icon name="plus" size={17} />
            Log volunteer time
          </button>
        )}
      </PageHead>
      <div class="volunteer-banner">
        <span class="stat-icon plum">
          <Icon name="heart" size={24} />
        </span>
        <div>
          <h2>A little time. A big difference.</h2>
          <p>
            {me.user.volunteer_eligible
              ? `You’ve contributed ${(minutes / 60).toFixed(1)} approved hours. Thank you for showing up.`
              : 'Help someone find their place. Share the volunteer page with your team.'}
          </p>
        </div>
        {(me.user.level || approve) && (
          <button
            type="button"
            class="button secondary"
            onClick={() => setQr({ url: `${location.origin}/volunteer`, shortcut: true })}
          >
            <Icon name="qr" size={17} />
            Show volunteer signup / time-entry QR
          </button>
        )}
      </div>
      <div class="tabs" role="tablist" aria-label="Volunteer views">
        {[
          ['entries', approve ? 'Team entries' : 'Time entries'],
          ['mine', 'My time'],
          ['report', 'Hours report'],
        ].map(([key, label]) => (
          <button
            type="button"
            key={key}
            role="tab"
            aria-selected={tab === key}
            class={tab === key ? 'active' : ''}
            onClick={() => setTab(key)}
          >
            {label}
          </button>
        ))}
      </div>
      <Panel>
        {error ? (
          <ErrorBox message={error} />
        ) : loading ? (
          <Loading />
        ) : tab === 'report' ? (
          report?.length ? (
            <div class="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>VOLUNTEER</th>
                    <th>DEPARTMENT</th>
                    <th>EVENT</th>
                    <th>APPROVED HOURS</th>
                  </tr>
                </thead>
                <tbody>
                  {report.map((r) => (
                    <tr key={`${r.user_id}:${r.department}:${r.event ?? 'none'}`}>
                      <td>{r.display_name}</td>
                      <td>{r.department}</td>
                      <td>{r.event || 'No edition assigned'}</td>
                      <td>{(r.minutes / 60).toFixed(1)}h</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <Empty title="Good things take a little time">Approved hours will appear here.</Empty>
          )
        ) : rows?.length ? (
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>VOLUNTEER & ACTIVITY</th>
                  <th>DATE</th>
                  <th>DEPARTMENT</th>
                  <th>TIME</th>
                  <th>STATUS</th>
                  <th>ACTIONS</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((e) => (
                  <tr key={e.id}>
                    <td>
                      <strong>{e.volunteer_name}</strong>
                      <small class="table-description">{e.work_description}</small>
                      {e.decided_by_name && (
                        <small class="muted">Reviewed by {e.decided_by_name}</small>
                      )}
                      {e.decision_note && (
                        <small class="table-description">{e.decision_note}</small>
                      )}
                    </td>
                    <td>{date(e.work_date)}</td>
                    <td>{e.department}</td>
                    <td class="nowrap">{(e.minutes_worked / 60).toFixed(1)}h</td>
                    <td>
                      <Status value={e.status} />
                    </td>
                    <td>
                      <div class="row-actions">
                        <VolunteerControls entry={e} />
                        {e.volunteer_user_id === me.user.id && e.status === 'draft' && (
                          <Action
                            action={async () => {
                              await api(`/volunteer/entries/${e.id}/submit`, 'POST');
                              refresh();
                            }}
                          >
                            Submit
                          </Action>
                        )}
                        {e.volunteer_user_id === me.user.id && e.status === 'submitted' && (
                          <Action
                            action={async () => {
                              setQr(await api(`/volunteer/entries/${e.id}/approval-qr`, 'POST'));
                            }}
                          >
                            Show approval QR
                          </Action>
                        )}
                        {e.volunteer_user_id === me.user.id &&
                          ['draft', 'submitted'].includes(e.status) && (
                            <Action
                              className="text-link"
                              action={async () => {
                                await api(`/volunteer/entries/${e.id}/withdraw`, 'POST');
                                refresh();
                              }}
                            >
                              Withdraw
                            </Action>
                          )}
                        {e.volunteer_user_id !== me.user.id && e.status === 'submitted' && (
                          <span class="muted">Ask volunteer for QR</span>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <Empty title="Your next helping hand starts here">
            Log an entry when you’ve finished volunteering.
          </Empty>
        )}
      </Panel>
      {newEntry && (
        <Modal title="Log volunteer time" onClose={() => setNewEntry(false)}>
          <p class="modal-description">
            Record the department you helped, even if it’s outside your home team.
          </p>
          <label class="check-field">
            <input
              type="checkbox"
              checked={timeMode}
              onChange={(e) => setTimeMode(e.currentTarget.checked)}
            />
            Use start and end times
          </label>
          <Form
            key={String(timeMode)}
            fields={[
              {
                name: 'department_id',
                label: 'Department helped',
                type: 'select',
                required: true,
                options: org?.departments.map((d: Row) => ({ value: d.id, label: d.name })),
              },
              {
                name: 'event_edition_id',
                label: 'Event',
                type: 'select',
                options: org?.events.map((d: Row) => ({ value: d.id, label: d.name })),
              },
              {
                name: 'work_date',
                label: 'Work date',
                type: 'date',
                required: true,
                value: new Date().toLocaleDateString('en-CA'),
              },
              ...(timeMode
                ? [
                    {
                      name: 'started_at',
                      label: 'Start (your device timezone)',
                      type: 'datetime-local',
                      required: true,
                    },
                    {
                      name: 'ended_at',
                      label: 'End (your device timezone)',
                      type: 'datetime-local',
                      required: true,
                    },
                  ]
                : [
                    {
                      name: 'minutes_worked',
                      label: 'Minutes worked',
                      type: 'number',
                      required: true,
                      min: 1,
                      max: org?.volunteer_max_minutes || 960,
                      value: 90,
                    },
                  ]),
              {
                name: 'work_description',
                label: 'What did you help with?',
                type: 'textarea',
                required: true,
                maxLength: 2000,
              },
            ]}
            submit="Save draft"
            onSubmit={async (b) => {
              if (!b.event_edition_id) b.event_edition_id = null;
              if (timeMode) {
                b.started_at = new Date(b.started_at).toISOString();
                b.ended_at = new Date(b.ended_at).toISOString();
              }
              await api('/volunteer/entries', 'POST', b);
              setNewEntry(false);
              refresh();
            }}
          />
        </Modal>
      )}
      {qr && <QrModal data={qr} onClose={() => setQr(null)} />}{' '}
      {scan && <Scanner onClose={() => setScan(false)} />}
    </>
  );
}
