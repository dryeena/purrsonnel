import { useEffect, useState } from 'preact/hooks';

import {
  Avatar,
  Badge,
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
import { refresh } from '../../lib/events';
import { downloadCsv } from '../../lib/exports';
import { can, global } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import { AddPersonButton } from '../lifecycle';

import { Person } from './person-details';

export function People(incoming: { path?: string; q?: string; person?: string; self?: string }) {
  const me = useAuth();
  const params = new URLSearchParams(location.search);
  const [q, setQ] = useState(params.get('q') || '');
  const [search, setSearch] = useState(q);
  const [department, setDepartment] = useState('');
  const [status, setStatus] = useState('');
  const [sort, setSort] = useState('display_name');
  const [order, setOrder] = useState('asc');
  const [page, setPage] = useState(1);
  const [selected, setSelected] = useState<string[]>([]);
  const [person, setPerson] = useState<string | null>(
    params.get('person') || (params.has('self') ? me.user.id : null),
  );
  const [save, setSave] = useState(false);
  useEffect(() => {
    if (incoming.q !== undefined) setQ(incoming.q);
  }, [incoming.q]);
  useEffect(() => {
    if (incoming.person) setPerson(incoming.person);
    if (incoming.self) setPerson(me.user.id);
  }, [incoming.person, incoming.self, me.user.id]);
  useEffect(() => {
    const timer = setTimeout(() => {
      setSearch(q);
      setPage(1);
    }, 250);
    return () => clearTimeout(timer);
  }, [q]);
  const { data, error, loading } = useData<Row>(
    `/users?q=${encodeURIComponent(search)}${department ? `&department=${department}` : ''}${status ? `&status=${status}` : ''}&sort=${sort}&order=${order}&page=${page}&page_size=15`,
  );
  const { data: org } = useData<Row>('/organization/tree');
  const { data: peopleFields } = useData<Row[]>('/people-page-fields');
  const chooseSort = (next: string) => {
    setOrder(next === sort && order === 'asc' ? 'desc' : 'asc');
    setSort(next);
    setPage(1);
  };
  const sortLabel = (key: string, label: string) => (
    <button type="button" class="table-sort" onClick={() => chooseSort(key)}>
      {label} {sort === key ? (order === 'asc' ? '↑' : '↓') : '↕'}
    </button>
  );
  const filter = {
    op: 'and',
    args: [
      ...(search ? [{ op: 'contains', field: 'display_name', value: search }] : []),
      ...(status ? [{ op: 'eq', field: 'status', value: status }] : []),
      ...(department
        ? [
            {
              op: 'eq',
              field: 'department',
              value: org?.departments.find((d: Row) => d.id === department)?.name,
            },
          ]
        : []),
    ],
  };
  return (
    <>
      <PageHead
        eyebrow="GOOD PEOPLE. ONE PLACE."
        title="People"
        description="Get to know the people who bring your weekend to life."
      >
        <button
          type="button"
          class="button secondary"
          onClick={() =>
            downloadCsv('purrsonnel-people.csv', data?.items || [], [
              'display_name',
              'myfwa_id',
              'department',
              'level',
              'status',
              'service_years',
            ])
          }
          disabled={!data?.items.length}
        >
          <Icon name="download" size={16} />
          Export page
        </button>
        {global(me, 'people.read') && (
          <button type="button" class="button primary" onClick={() => setSave(true)}>
            <Icon name="plus" size={17} />
            Save this view
          </button>
        )}
        {can(me, 'people.bulk_update') && (
          <a class="button secondary" href="/lists">
            Bulk actions
          </a>
        )}
        <AddPersonButton />
      </PageHead>
      <Panel>
        <div class="directory-toolbar">
          <div class="input-icon">
            <Icon name="search" size={17} />
            <input
              aria-label="Search directory"
              placeholder="Search name or myFWA ID…"
              value={q}
              onInput={(e) => setQ(e.currentTarget.value)}
            />
          </div>
          <select
            aria-label="Filter by department"
            value={department}
            onChange={(e) => {
              setDepartment(e.currentTarget.value);
              setPage(1);
            }}
          >
            <option value="">All departments</option>
            {org?.departments.map((d: Row) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </select>
          <select
            aria-label="Filter by status"
            value={status}
            onChange={(e) => {
              setStatus(e.currentTarget.value);
              setPage(1);
            }}
          >
            <option value="">All statuses</option>
            {['active', 'inactive', 'former', 'pending_identity'].map((s) => (
              <option key={s} value={s}>
                {s.replace('_', ' ')}
              </option>
            ))}
          </select>
          <span class="result-count">{data?.total ?? '…'} people</span>
        </div>
        {selected.length > 0 && (
          <div class="selection-bar">
            <strong>{selected.length} selected</strong>
            <button
              type="button"
              class="text-link"
              onClick={() =>
                downloadCsv(
                  'purrsonnel-selected.csv',
                  data?.items.filter((r: Row) => selected.includes(r.id)) || [],
                  ['display_name', 'myfwa_id', 'department', 'status'],
                )
              }
            >
              Export selected
            </button>
            <button type="button" class="text-link" onClick={() => setSelected([])}>
              Clear
            </button>
            {can(me, 'people.bulk_update') && (
              <a class="text-link" href="/lists">
                Review bulk actions →
              </a>
            )}
          </div>
        )}
        {error ? (
          <ErrorBox message={error} />
        ) : loading ? (
          <Loading />
        ) : data?.items.length ? (
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>
                    <input
                      type="checkbox"
                      aria-label="Select current page"
                      checked={data.items.every((u: Row) => selected.includes(u.id))}
                      onChange={(e) =>
                        setSelected(e.currentTarget.checked ? data.items.map((u: Row) => u.id) : [])
                      }
                    />
                  </th>
                  <th>{sortLabel('display_name', 'PERSON')}</th>
                  <th>{sortLabel('department', 'DEPARTMENTS')}</th>
                  <th>{sortLabel('level', 'LEVEL')}</th>
                  <th>{sortLabel('verified_years', 'SERVICE')}</th>
                  <th>{sortLabel('status', 'STATUS')}</th>
                  {peopleFields?.map((field) => (
                    <th key={field.id}>{field.label}</th>
                  ))}
                  <th />
                </tr>
              </thead>
              <tbody>
                {data.items.map((u: Row) => (
                  <tr key={u.id}>
                    <td>
                      <input
                        type="checkbox"
                        aria-label={`Select ${u.display_name}`}
                        checked={selected.includes(u.id)}
                        onChange={(e) =>
                          setSelected(
                            e.currentTarget.checked
                              ? [...selected, u.id]
                              : selected.filter((id) => id !== u.id),
                          )
                        }
                      />
                    </td>
                    <td>
                      <button
                        type="button"
                        class="person-cell table-person"
                        onClick={() => setPerson(u.id)}
                      >
                        <Avatar name={u.display_name} />
                        <span>
                          <strong>{u.display_name}</strong>
                          <small>{u.myfwa_id || 'Identity pending'}</small>
                        </span>
                      </button>
                    </td>
                    <td>
                      <div class="tag-stack">
                        {u.departments.length ? (
                          u.departments.map((d: Row) => (
                            <Badge key={d.id}>
                              {d.name}
                              {d.id === u.primary_department_id ? ' · primary' : ''}
                            </Badge>
                          ))
                        ) : (
                          <span class="muted">Unassigned</span>
                        )}
                      </div>
                    </td>
                    <td>
                      {u.level || <span class="muted">Community</span>}
                      {u.volunteer_eligible && (
                        <small class="volunteer-label">♡ Volunteer eligible</small>
                      )}
                    </td>
                    <td>
                      <strong>{u.verified_years}</strong>
                      <span class="muted"> years</span>
                    </td>
                    <td>
                      <Status value={u.status} />
                    </td>
                    {peopleFields?.map((field) => (
                      <td key={field.id}>
                        {u.profile_fields?.[field.key] ?? <span class="muted">—</span>}
                      </td>
                    ))}
                    <td>
                      <button
                        type="button"
                        class="icon-button"
                        aria-label={`View ${u.display_name}`}
                        onClick={() => setPerson(u.id)}
                      >
                        <Icon name="chevron" size={17} />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <Empty title="No people match these filters">
            Try another name or broaden your filters.
          </Empty>
        )}
        <div class="pagination">
          <span>
            Page {page} · {data?.total ?? 0} matches
          </span>
          <div>
            <button
              type="button"
              class="button secondary small-button"
              disabled={page <= 1}
              onClick={() => setPage((p) => p - 1)}
            >
              Previous
            </button>
            <button
              type="button"
              class="button secondary small-button"
              disabled={!data || page * 15 >= data.total}
              onClick={() => setPage((p) => p + 1)}
            >
              Next
            </button>
          </div>
        </div>
      </Panel>
      {person && <Person id={person} onClose={() => setPerson(null)} />}{' '}
      {save && (
        <Modal title="Save this people view" onClose={() => setSave(false)}>
          <p class="modal-description">Keep these search and filter criteria as a private list.</p>
          <Form
            fields={[{ name: 'name', label: 'List name', required: true }]}
            submit="Save list"
            onSubmit={async (b) => {
              await api('/saved-lists', 'POST', { name: b.name, filter_ast_json: filter });
              setSave(false);
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
