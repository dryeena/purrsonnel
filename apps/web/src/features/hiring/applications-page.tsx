import { useState } from 'preact/hooks';

import { Avatar, Empty, ErrorBox, Loading, PageHead, Panel, Status } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { date } from '../../lib/date';
import { can, global } from '../../lib/permissions';
import type { Row } from '../../types/api';

import { ApplicationDetails } from './application-details';
import { Finalize } from './finalize-dialog';

export function Applications(_: { path?: string }) {
  const me = useAuth();
  const { data, error, loading } = useData<Row[]>('/applications?page_size=100');
  const [detail, setDetail] = useState<string | null>(null);
  const [final, setFinal] = useState<Row | null>(null);
  const [status, setStatus] = useState('');
  const inProgress = new URLSearchParams(location.search).get('in_progress') === '1';
  const rows = data?.filter(
    (a) =>
      (!status || a.status === status) &&
      (!inProgress || ['submitted', 'screening', 'interviewing'].includes(a.status)),
  );
  return (
    <>
      <PageHead
        eyebrow="THE NEXT GREAT TEAMMATE"
        title="Applications"
        description={
          can(me, 'applications.review')
            ? 'A thoughtful process for every person who raises their hand.'
            : 'Your next opportunity, one step at a time.'
        }
      />
      <Panel>
        <div class="directory-toolbar">
          <select
            value={status}
            aria-label="Filter applications by status"
            onChange={(e) => setStatus(e.currentTarget.value)}
          >
            <option value="">All statuses</option>
            {[
              'draft',
              'submitted',
              'screening',
              'interviewing',
              'offered',
              'approved',
              'rejected',
            ].map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
          <span class="result-count">{rows?.length || 0} applications</span>
          {inProgress && <span class="result-count">In progress</span>}
        </div>
        {error ? (
          <ErrorBox message={error} />
        ) : loading ? (
          <Loading />
        ) : rows?.length ? (
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>APPLICANT</th>
                  <th>POSITION</th>
                  <th>DEPARTMENT</th>
                  <th>STATUS</th>
                  <th>SUBMITTED</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {rows.map((a) => (
                  <tr key={a.id}>
                    <td>
                      <div class="person-cell">
                        <Avatar small name={a.applicant_name} />
                        <strong>{a.applicant_name}</strong>
                      </div>
                    </td>
                    <td>{a.position}</td>
                    <td>{a.department}</td>
                    <td>
                      <Status value={a.status} />
                    </td>
                    <td>{date(a.submitted_at)}</td>
                    <td>
                      <div class="row-actions">
                        <button type="button" class="text-link" onClick={() => setDetail(a.id)}>
                          View answers
                        </button>
                        {(a.hiring_owner_user_id === me.user.id ||
                          global(me, 'interviews.finalize')) &&
                          ['submitted', 'screening', 'interviewing', 'offered'].includes(
                            a.status,
                          ) && (
                            <button
                              type="button"
                              class="button secondary small-button"
                              onClick={() => setFinal(a)}
                            >
                              Final decision
                            </button>
                          )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <Empty title="No applications in this view">
            Find an open position to start your next chapter.
          </Empty>
        )}
      </Panel>
      {detail && <ApplicationDetails id={detail} onClose={() => setDetail(null)} />}{' '}
      {final && <Finalize application={final} onClose={() => setFinal(null)} />}
    </>
  );
}
