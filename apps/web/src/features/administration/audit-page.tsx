import { useState } from 'preact/hooks';

import { Badge, Empty, ErrorBox, Loading, PageHead, Panel } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { date } from '../../lib/date';
import type { Row } from '../../types/api';

export function Audit(_: { path?: string }) {
  const [page, setPage] = useState(1);
  const { data, error, loading } = useData<Row[]>(`/audit-events?page=${page}&page_size=25`);
  return (
    <>
      <PageHead
        eyebrow="A CLEAR RECORD"
        title="Audit log"
        description="Who changed what, and when. A history you can follow."
      />
      <Panel>
        {error ? (
          <ErrorBox message={error} />
        ) : loading ? (
          <Loading />
        ) : data?.length ? (
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>WHEN</th>
                  <th>ACTOR</th>
                  <th>ACTION</th>
                  <th>RECORD</th>
                  <th>CHANGES</th>
                </tr>
              </thead>
              <tbody>
                {data.map((e) => (
                  <tr key={e.id}>
                    <td class="nowrap">{date(e.created_at, true)}</td>
                    <td>{e.actor_name || 'System'}</td>
                    <td>
                      <Badge>{e.action}</Badge>
                    </td>
                    <td>
                      <span>{e.resource_type.replaceAll('_', ' ')}</span>
                      <small class="mono muted">{e.resource_id?.slice(0, 8) || '—'}</small>
                    </td>
                    <td>
                      <code class="audit-json">
                        {e.after_json ? JSON.stringify(e.after_json) : '—'}
                      </code>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <Empty title="No events on this page" />
        )}
        <div class="pagination">
          <span>Page {page} · read only history</span>
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
    </>
  );
}
