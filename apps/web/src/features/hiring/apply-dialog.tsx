import { useState } from 'preact/hooks';

import { Action, Form, Icon, Modal } from '../../components/ui';
import { refresh } from '../../lib/events';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

import { fieldsFor } from './application-fields';

export function Apply({
  position,
  onClose,
  draft,
}: {
  position: Row;
  onClose: () => void;
  draft?: Row;
}) {
  const [saved, setSaved] = useState<string | null>(draft?.id || null);
  const [submitted, setSubmitted] = useState(false);
  const questions = position.questions || [];
  return (
    <Modal title={submitted ? 'Application submitted' : position.title} onClose={onClose} wide>
      {submitted ? (
        <div class="success-box">
          <Icon name="check" />
          Your application is with the team. Follow its progress in Applications.
        </div>
      ) : (
        <>
          <p class="modal-description">
            {position.department} · Your authenticated identity supplies your myFWA ID. You can save
            your answers before submitting.
          </p>
          <Form
            draft
            fields={fieldsFor(questions, draft?.answers_json)}
            submit="Save draft"
            onSubmit={async (b) => {
              const answers = { ...b };
              if (saved) await api(`/applications/${saved}`, 'PATCH', { answers_json: answers });
              else {
                const result = await api('/applications', 'POST', {
                  position_posting_id: position.id,
                  answers_json: answers,
                });
                setSaved(result.id);
              }
              refresh();
            }}
          />
          {saved && (
            <div class="submit-draft">
              <p>Your draft is saved. When you’re ready, send it to the hiring team.</p>
              <Action
                className="button primary"
                action={async () => {
                  await api(`/applications/${saved}/submit`, 'POST');
                  setSubmitted(true);
                  refresh();
                }}
              >
                Submit application
                <Icon name="arrow" size={16} />
              </Action>
            </div>
          )}
        </>
      )}
    </Modal>
  );
}
