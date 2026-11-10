import { useState } from 'preact/hooks';

import {
  Action,
  Badge,
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
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function Terms(_: { path?: string }) {
  const me = useAuth();
  const [tab, setTab] = useState('required');
  const [document, setDocument] = useState<Row | null>(null);
  const [create, setCreate] = useState(false);
  const [revision, setRevision] = useState<Row | null>(null);
  const [receipt, setReceipt] = useState<Row | null>(null);
  const { data: mine, error, loading } = useData<Row[]>('/terms/applicable');
  const { data: managed } = useData<Row[]>(can(me, 'terms.manage') ? '/terms' : null);
  const { data: compliance } = useData<Row[]>(
    can(me, 'terms.read_compliance') ? '/terms/compliance' : null,
  );
  const pending = mine?.filter((t) => !t.signed_at).length || 0;
  const current = mine?.filter((t) => (tab === 'signed' ? !!t.signed_at : !t.signed_at)) || [];
  return (
    <>
      <PageHead
        eyebrow="A SHARED COMMITMENT"
        title="Terms & policies"
        description="Clear expectations help us take good care of each other."
      >
        {can(me, 'terms.manage') && (
          <button type="button" class="button primary" onClick={() => setCreate(true)}>
            <Icon name="plus" size={16} />
            Create policy
          </button>
        )}
      </PageHead>
      <div class="tabs" role="tablist" aria-label="Policy views">
        {[
          ['required', `Required for me${pending ? ` · ${pending}` : ''}`],
          ['signed', 'Acknowledged'],
          ...(can(me, 'terms.manage') ? [['manage', 'Manage policies']] : []),
          ...(can(me, 'terms.read_compliance') ? [['compliance', 'Compliance']] : []),
        ].map(([key, label]) => (
          <button
            type="button"
            key={key}
            role="tab"
            aria-selected={tab === key}
            onClick={() => setTab(key)}
            class={tab === key ? 'active' : ''}
          >
            {label}
          </button>
        ))}
      </div>
      {error ? (
        <ErrorBox message={error} />
      ) : loading ? (
        <Loading />
      ) : tab === 'manage' ? (
        <div class="card-grid">
          {managed?.map((t) => (
            <Panel key={t.id}>
              <div class="policy-card">
                <span class="stat-icon plum">
                  <Icon name="terms" />
                </span>
                <h2>{t.title}</h2>
                <p class="muted">Audience: {t.audience}</p>
                <div class="version-list">
                  {t.versions.map((v: Row) => (
                    <div key={v.id}>
                      <span>Version {v.version_number}</span>
                      <Badge
                        tone={v.superseded_at ? 'neutral' : v.published_at ? 'green' : 'amber'}
                      >
                        {v.superseded_at ? 'Superseded' : v.published_at ? 'Published' : 'Draft'}
                      </Badge>
                      {!v.published_at && (
                        <Action
                          action={async () => {
                            await api(`/terms/${t.id}/publish`, 'POST', { version_id: v.id });
                            refresh();
                          }}
                        >
                          Publish
                        </Action>
                      )}
                    </div>
                  ))}
                </div>
                <button type="button" class="button secondary" onClick={() => setRevision(t)}>
                  Draft new version
                  <Icon name="plus" size={15} />
                </button>
              </div>
            </Panel>
          ))}
        </div>
      ) : tab === 'compliance' ? (
        <Panel
          title="Current version acknowledgments"
          description="Each applicable policy is tracked separately."
        >
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>PERSON</th>
                  <th>POLICY</th>
                  <th>VERSION</th>
                  <th>ACKNOWLEDGMENT</th>
                </tr>
              </thead>
              <tbody>
                {compliance?.map((r) => (
                  <tr key={r.id}>
                    <td>{r.display_name}</td>
                    <td>{r.title}</td>
                    <td>v{r.version_number}</td>
                    <td>
                      <Badge tone={r.signed_at ? 'green' : 'amber'}>
                        {r.signed_at ? date(r.signed_at) : 'Pending'}
                      </Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Panel>
      ) : current.length ? (
        <div class="card-grid">
          {current.map((t) => (
            <Panel key={t.id}>
              <div class="policy-card">
                <div class="policy-top">
                  <span class="stat-icon plum">
                    <Icon name="terms" />
                  </span>
                  <Badge tone={t.signed_at ? 'green' : 'amber'}>
                    {t.signed_at ? 'Acknowledged' : 'Action needed'}
                  </Badge>
                </div>
                <h2>{t.title}</h2>
                <p class="muted">
                  Version {t.version_number} · Published {date(t.published_at)}
                </p>
                <p class="document-excerpt">{t.body_markdown.slice(0, 180)}…</p>
                <button
                  type="button"
                  class={`button ${t.signed_at ? 'secondary' : 'primary'}`}
                  onClick={() => setDocument(t)}
                >
                  {t.signed_at ? 'View signed document' : 'Read & acknowledge'}
                  <Icon name="arrow" size={16} />
                </button>
                {t.signed_at && (
                  <small class="receipt-note">
                    Acknowledged {date(t.signed_at)} by {t.typed_name}
                  </small>
                )}
              </div>
            </Panel>
          ))}
        </div>
      ) : (
        <Panel>
          <Empty title={tab === 'signed' ? 'No acknowledgments yet' : 'You’re all up to date'}>
            {tab === 'signed'
              ? 'Your acknowledged documents will appear here.'
              : 'All your currently applicable policies are acknowledged.'}
          </Empty>
        </Panel>
      )}
      {document && (
        <Modal title={document.title} onClose={() => setDocument(null)} wide>
          <div class="document-meta">
            <Badge tone="plum">Version {document.version_number}</Badge>
            <span>Published {date(document.published_at)}</span>
          </div>
          <div class="document-body">{document.body_markdown}</div>
          <p class="hash-line">Document SHA-256: {document.body_sha256}</p>
          {document.signed_at ? (
            <div class="success-box">
              <Icon name="check" /> Acknowledged by {document.typed_name} on{' '}
              {date(document.signed_at, true)}.
            </div>
          ) : (
            <Form
              fields={[
                { name: 'typed_name', label: 'Your full name', required: true, maxLength: 200 },
                {
                  name: 'consent',
                  label: 'I have read this exact version and agree to acknowledge it.',
                  type: 'checkbox',
                  required: true,
                },
              ]}
              submit="Acknowledge this version"
              onSubmit={async (b) => {
                setReceipt(await api(`/term-versions/${document.version_id}/sign`, 'POST', b));
                setDocument(null);
                refresh();
              }}
            />
          )}
        </Modal>
      )}
      {receipt && (
        <Modal title="Your acknowledgment is recorded" onClose={() => setReceipt(null)}>
          <div class="success-box">
            <Icon name="check" /> Thank you, {receipt.typed_name}.
          </div>
          <div class="detail-section">
            <p>Recorded {date(receipt.server_received_at, true)}.</p>
            <p class="hash-line">Receipt {receipt.id}</p>
            <p class="hash-line">Document SHA-256: {receipt.document_sha256}</p>
          </div>
        </Modal>
      )}
      {create && (
        <Modal title="Create a policy draft" onClose={() => setCreate(false)} wide>
          <Form
            fields={[
              { name: 'title', label: 'Policy title', required: true },
              {
                name: 'audience',
                label: 'Who is this for?',
                type: 'select',
                required: true,
                options: [
                  { value: 'all', label: 'Everyone' },
                  { value: 'volunteers', label: 'Volunteer-eligible people only' },
                  { value: 'personnel', label: 'Personnel only' },
                ],
              },
              {
                name: 'body_markdown',
                label: 'Policy text',
                type: 'textarea',
                required: true,
                maxLength: 50000,
              },
            ]}
            submit="Save draft"
            onSubmit={async (b) => {
              await api('/terms', 'POST', b);
              setCreate(false);
              refresh();
            }}
          />
        </Modal>
      )}
      {revision && (
        <Modal title={`New version · ${revision.title}`} onClose={() => setRevision(null)} wide>
          <p class="modal-description">
            Previous published versions and acknowledgments are preserved. Publishing this draft
            will require a fresh acknowledgment.
          </p>
          <Form
            fields={[
              {
                name: 'body_markdown',
                label: 'New policy text',
                type: 'textarea',
                required: true,
                maxLength: 50000,
                value: revision.versions[0]?.body_markdown,
              },
            ]}
            submit="Create new draft"
            onSubmit={async (b) => {
              await api(`/terms/${revision.id}/versions`, 'POST', b);
              setRevision(null);
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
