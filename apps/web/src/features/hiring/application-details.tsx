import { ErrorBox, Loading, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import type { Row } from '../../types/api';

import { Apply } from './apply-dialog';

export function ApplicationDetails({ id, onClose }: { id: string; onClose: () => void }) {
  const { data, error, loading } = useData<Row>(`/applications/${id}`);
  const me = useAuth();
  return loading ? (
    <Modal title="Application" onClose={onClose}>
      <Loading />
    </Modal>
  ) : error ? (
    <Modal title="Application" onClose={onClose}>
      <ErrorBox message={error} />
    </Modal>
  ) : data?.status === 'draft' && data.applicant_user_id === me.user.id ? (
    <Apply
      position={{ id: data.position_posting_id, title: data.position, questions: data.questions }}
      draft={data}
      onClose={onClose}
    />
  ) : (
    <Modal title={data?.position || 'Application'} onClose={onClose} wide>
      <div class="answer-list">
        {data?.questions.map((q: Row) => (
          <div key={q.key}>
            <h3>{q.label}</h3>
            <p>
              {data.answers_json[q.key] === undefined
                ? q.sensitive
                  ? 'Restricted or not provided'
                  : 'Not provided'
                : String(data.answers_json[q.key])}
            </p>
          </div>
        ))}
      </div>
    </Modal>
  );
}
