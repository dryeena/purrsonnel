import { useState } from 'preact/hooks';

import {
  Avatar,
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
import { can, global } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import { InterviewEditor } from '../lifecycle';

import { Finalize } from './finalize-dialog';
import { Notes } from './notes-dialog';

export function Interviews(_: { path?: string }) {
  const me = useAuth();
  const { data, error, loading } = useData<Row[]>('/interviews?page_size=100');
  const { data: apps } = useData<Row[]>('/applications?page_size=100');
  const { data: people } = useData<Row>(
    can(me, 'interviews.manage') ? '/users?page_size=100' : null,
  );
  const [schedule, setSchedule] = useState(false);
  const [panel, setPanel] = useState<string[]>([]);
  const [recommend, setRecommend] = useState<Row | null>(null);
  const [final, setFinal] = useState<Row | null>(null);
  const [notes, setNotes] = useState<Row | null>(null);
  return (
    <>
      <PageHead
        eyebrow="MAKE ROOM FOR A CONVERSATION"
        title="Interviews"
        description="Meet the person behind the application. Give everyone a fair hearing."
      >
        {can(me, 'interviews.manage') && (
          <button type="button" class="button primary" onClick={() => setSchedule(true)}>
            <Icon name="plus" size={16} />
            Schedule interview
          </button>
        )}
      </PageHead>
      {error ? (
        <ErrorBox message={error} />
      ) : loading ? (
        <Loading />
      ) : data?.length ? (
        <div class="interview-grid">
          {data.map((i) => (
            <Panel key={i.id}>
              <div class="interview-card">
                <div class="policy-top">
                  <div class="person-cell">
                    <Avatar name={i.applicant_name} />
                    <div>
                      <h2>{i.applicant_name}</h2>
                      <p>{i.position}</p>
                    </div>
                  </div>
                  <Status value={i.status} />
                </div>
                <div class="interview-details">
                  <p>
                    <Icon name="calendar" size={17} />
                    {date(i.scheduled_start_at, true, i.timezone)}
                  </p>
                  <p>
                    <Icon name="clock" size={17} />
                    {Math.round(
                      (new Date(i.scheduled_end_at).getTime() -
                        new Date(i.scheduled_start_at).getTime()) /
                        60000,
                    )}{' '}
                    minutes · {i.timezone}
                  </p>
                  <p>
                    <Icon name="organization" size={17} />
                    {i.location_label}
                  </p>
                  {i.location_url?.startsWith('https://') && (
                    <a class="text-link" href={i.location_url} target="_blank" rel="noreferrer">
                      Open meeting link ↗
                    </a>
                  )}
                </div>
                <div class="panel-responses">
                  <h3>Panel recommendations</h3>
                  {i.panel.map((p: Row) => (
                    <div key={p.user_id}>
                      <span>{p.display_name}</span>
                      <Status value={p.response} />
                    </div>
                  ))}
                </div>
                <p class="small-note">
                  Recommendations inform the hiring owner. They don’t automatically decide the
                  outcome.
                </p>
                <div class="row-actions">
                  <InterviewEditor interview={i} />
                  {i.panel.some((p: Row) => p.user_id === me.user.id) &&
                    i.application_status === 'interviewing' && (
                      <button type="button" class="button primary" onClick={() => setRecommend(i)}>
                        My recommendation
                      </button>
                    )}
                  {i.applicant_user_id !== me.user.id && (
                    <button type="button" class="button secondary" onClick={() => setNotes(i)}>
                      Notes
                    </button>
                  )}
                  {(i.hiring_owner_user_id === me.user.id || global(me, 'interviews.finalize')) &&
                    ['submitted', 'screening', 'interviewing', 'offered'].includes(
                      i.application_status,
                    ) && (
                      <button
                        type="button"
                        class="text-link"
                        onClick={() =>
                          setFinal({
                            id: i.application_id,
                            position: i.position,
                            applicant_name: i.applicant_name,
                          })
                        }
                      >
                        Final decision →
                      </button>
                    )}
                </div>
              </div>
            </Panel>
          ))}
        </div>
      ) : (
        <Panel>
          <Empty title="No conversations scheduled yet">
            Upcoming interviews will have a home here.
          </Empty>
        </Panel>
      )}
      {schedule && (
        <Modal title="Schedule an interview" onClose={() => setSchedule(false)} wide>
          <div class="detail-section">
            <h3>Interview panel</h3>
            <p class="muted">
              Members must have explicit interview decision permission in this department.
            </p>
            <div class="checkbox-grid">
              {people?.items.map((u: Row) => (
                <label key={u.id} class="check-field">
                  <input
                    type="checkbox"
                    checked={panel.includes(u.id)}
                    onChange={(e) =>
                      setPanel(
                        e.currentTarget.checked
                          ? [...panel, u.id]
                          : panel.filter((id) => id !== u.id),
                      )
                    }
                  />
                  {u.display_name}
                </label>
              ))}
            </div>
          </div>
          <Form
            fields={[
              {
                name: 'application_id',
                label: 'Application',
                type: 'select',
                required: true,
                options: apps
                  ?.filter((a) => ['submitted', 'screening', 'interviewing'].includes(a.status))
                  .map((a) => ({ value: a.id, label: `${a.applicant_name} · ${a.position}` })),
              },
              {
                name: 'scheduled_start_at',
                label: 'Start (your device timezone)',
                type: 'datetime-local',
                required: true,
              },
              {
                name: 'scheduled_end_at',
                label: 'End (your device timezone)',
                type: 'datetime-local',
                required: true,
              },
              {
                name: 'timezone',
                label: 'Display timezone (IANA)',
                required: true,
                value: 'America/New_York',
              },
              { name: 'location_label', label: 'Location', required: true },
              { name: 'location_url', label: 'HTTPS meeting link', type: 'url' },
            ]}
            submit="Schedule interview"
            onSubmit={async (b) => {
              await api('/interviews', 'POST', {
                ...b,
                scheduled_start_at: new Date(b.scheduled_start_at).toISOString(),
                scheduled_end_at: new Date(b.scheduled_end_at).toISOString(),
                location_url: b.location_url || null,
                panel,
              });
              setSchedule(false);
              setPanel([]);
              refresh();
            }}
          />
        </Modal>
      )}
      {recommend && (
        <Modal
          title={`Recommendation · ${recommend.applicant_name}`}
          onClose={() => setRecommend(null)}
        >
          <Form
            fields={[
              {
                name: 'decision',
                label: 'Recommendation',
                type: 'select',
                required: true,
                options: [
                  { value: 'approve', label: 'Approve' },
                  { value: 'deny', label: 'Deny' },
                ],
              },
              {
                name: 'note',
                label: 'Feedback (required for denial)',
                type: 'textarea',
                maxLength: 2000,
              },
            ]}
            submit="Record recommendation"
            onSubmit={async (b) => {
              await api(`/interviews/${recommend.id}/my-decision`, 'PUT', b);
              setRecommend(null);
              refresh();
            }}
          />
        </Modal>
      )}
      {final && <Finalize application={final} onClose={() => setFinal(null)} />}{' '}
      {notes && <Notes interview={notes} onClose={() => setNotes(null)} />}
    </>
  );
}
