import { useState } from 'preact/hooks';

import {
  Action,
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
import { downloadCsv } from '../../lib/exports';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

import { describeFilter } from './filter-builder';
import { FilterBuilder } from './filter-builder';

export function Lists(_: { path?: string }) {
  const me = useAuth();
  const [editing, setEditing] = useState<Row | null>(null);
  const { data, error, loading } = useData<Row[]>('/saved-lists');
  const { data: jobs } = useData<Row[]>('/bulk-changes');
  const [create, setCreate] = useState(false);
  const [filter, setFilter] = useState<Row>({ op: 'and', args: [] });
  const [results, setResults] = useState<Row | null>(null);
  const [chosen, setChosen] = useState<Row | null>(null);
  const [preview, setPreview] = useState<Row | null>(null);
  const [outcome, setOutcome] = useState<Row | null>(null);
  return (
    <>
      <PageHead
        eyebrow="LESS BUSYWORK. MORE PEOPLE."
        title="Lists & bulk actions"
        description="Save a useful view. Review every change. Keep a clear record."
      >
        <button type="button" class="button primary" onClick={() => setCreate(true)}>
          <Icon name="plus" size={16} />
          Create saved list
        </button>
      </PageHead>
      {error ? (
        <ErrorBox message={error} />
      ) : loading ? (
        <Loading />
      ) : data?.length ? (
        <div class="card-grid">
          {data.map((l) => (
            <Panel key={l.id}>
              <div class="policy-card">
                <span class="stat-icon plum">
                  <Icon name="list" />
                </span>
                <h2>{l.name}</h2>
                <p class="muted">Private list · updated {date(l.updated_at)}</p>
                <div class="filter-summary">{describeFilter(l.filter_ast_json)}</div>
                <div class="row-actions">
                  {l.owner_user_id === me.user.id && (
                    <button type="button" class="text-link" onClick={() => setEditing(l)}>
                      Edit / share
                    </button>
                  )}
                  <Action
                    action={async () => {
                      setResults(await api(`/saved-lists/${l.id}/run`, 'POST'));
                      setChosen(l);
                    }}
                  >
                    Run list
                    <Icon name="arrow" size={15} />
                  </Action>
                  <Action
                    className="text-link"
                    action={async () => {
                      await api(`/saved-lists/${l.id}`, 'DELETE');
                      refresh();
                    }}
                  >
                    Delete
                  </Action>
                </div>
              </div>
            </Panel>
          ))}
        </div>
      ) : (
        <Panel>
          <Empty title="Your useful views, saved">
            Create a list for returning volunteers, a department, or people who need a follow-up.
          </Empty>
        </Panel>
      )}
      <Panel title="Bulk change history" description="Reviewed snapshots and per-person outcomes.">
        {jobs?.length ? (
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>CREATED</th>
                  <th>OPERATION</th>
                  <th>PEOPLE</th>
                  <th>STATUS</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {jobs.map((j) => (
                  <tr key={j.id}>
                    <td>{date(j.created_at, true)}</td>
                    <td>
                      {j.operation_json.field} → {String(j.operation_json.value)}
                    </td>
                    <td>{j.preview_count}</td>
                    <td>
                      <Status value={j.status} />
                    </td>
                    <td>
                      {j.result_json && (
                        <button
                          type="button"
                          class="text-link"
                          onClick={() =>
                            downloadCsv('bulk-results.csv', j.result_json, ['id', 'status'])
                          }
                        >
                          Download results
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <Empty title="No bulk changes yet">
            Completed changes and conflicts will be recorded here.
          </Empty>
        )}
      </Panel>
      {editing && (
        <Modal title="Edit saved list" onClose={() => setEditing(null)} wide>
          <FilterBuilder
            value={editing.filter_ast_json}
            onChange={(f) => setEditing({ ...editing, filter_ast_json: f })}
          />
          <Form
            fields={[
              { name: 'name', label: 'List name', required: true, value: editing.name },
              {
                name: 'shared_scope',
                label: 'Sharing',
                type: 'select',
                required: true,
                value: editing.shared_scope,
                options: [
                  { value: 'private', label: 'Only me' },
                  { value: 'hr', label: 'HR with global people access' },
                ],
              },
            ]}
            onSubmit={async (b) => {
              await api(`/saved-lists/${editing.id}`, 'PATCH', {
                ...b,
                filter_ast_json: editing.filter_ast_json,
              });
              setEditing(null);
              refresh();
            }}
          />
        </Modal>
      )}
      {create && (
        <Modal title="Create a saved list" onClose={() => setCreate(false)} wide>
          <FilterBuilder value={filter} onChange={setFilter} />
          <Form
            fields={[{ name: 'name', label: 'List name', required: true, maxLength: 120 }]}
            submit="Save private list"
            onSubmit={async (b) => {
              await api('/saved-lists', 'POST', { name: b.name, filter_ast_json: filter });
              setCreate(false);
              setFilter({ op: 'and', args: [] });
              refresh();
            }}
          />
        </Modal>
      )}
      {results && chosen && (
        <Modal
          title={`${chosen.name} · ${results.total} people`}
          onClose={() => {
            setResults(null);
            setChosen(null);
          }}
          wide
        >
          <div class="member-list">
            {results.items.map((u: Row) => (
              <div key={u.id} class="person-cell">
                <Avatar name={u.display_name} small />
                <strong>{u.display_name}</strong>
                <Status value={u.status} />
              </div>
            ))}
          </div>
          <div class="info-note">
            <Icon name="audit" />
            <span>
              Changes use a reviewed snapshot. People edited after preview are skipped and reported
              as conflicts.
            </span>
          </div>
          <Form
            fields={[
              {
                name: 'operation',
                label: 'Change to preview',
                type: 'select',
                required: true,
                options: [
                  { value: 'inactive', label: 'Set status: inactive' },
                  { value: 'active', label: 'Set status: active' },
                  { value: 'former', label: 'Set status: former' },
                  { value: 'volunteer_true', label: 'Enable volunteer eligibility' },
                  { value: 'volunteer_false', label: 'Disable volunteer eligibility' },
                ],
              },
            ]}
            submit="Preview changes"
            onSubmit={async (b) => {
              const operation = b.operation.startsWith('volunteer_')
                ? { field: 'volunteer_eligible', value: b.operation === 'volunteer_true' }
                : { field: 'status', value: b.operation };
              setPreview(
                await api('/bulk-changes/preview', 'POST', {
                  filter: chosen.filter_ast_json,
                  operation,
                }),
              );
              setResults(null);
            }}
          />
        </Modal>
      )}
      {preview && (
        <Modal title="Review bulk changes" onClose={() => setPreview(null)} wide>
          <p class="modal-description">
            {preview.count} people · {preview.operation.field} will become{' '}
            <strong>{String(preview.operation.value)}</strong>. No permissions will be assigned.
          </p>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>PERSON</th>
                  <th>BEFORE</th>
                  <th>AFTER</th>
                </tr>
              </thead>
              <tbody>
                {preview.targets.map((t: Row) => (
                  <tr key={t.id}>
                    <td>{t.display_name}</td>
                    <td>{String(t.before)}</td>
                    <td>{String(preview.operation.value)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Form
            fields={[
              {
                name: 'confirmed',
                label: `I confirm these changes for ${preview.count} people.`,
                type: 'checkbox',
                required: true,
              },
            ]}
            submit="Apply reviewed snapshot"
            onSubmit={async (b) => {
              setOutcome(
                await api('/bulk-changes/execute', 'POST', {
                  preview_id: preview.id,
                  confirmed: b.confirmed,
                }),
              );
              setPreview(null);
              refresh();
            }}
          />
        </Modal>
      )}
      {outcome && (
        <Modal title="Bulk update results" onClose={() => setOutcome(null)}>
          <Status value={outcome.status} />
          <p>
            {outcome.results.filter((r: Row) => r.status === 'updated').length} updated ·{' '}
            {outcome.results.filter((r: Row) => r.status === 'conflict').length} conflicts
          </p>
          <button
            type="button"
            class="button secondary"
            onClick={() => downloadCsv('bulk-results.csv', outcome.results, ['id', 'status'])}
          >
            <Icon name="download" size={16} />
            Download results
          </button>
        </Modal>
      )}
    </>
  );
}
