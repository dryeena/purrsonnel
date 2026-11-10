import { useState } from 'preact/hooks';

import { Icon } from '../../components/ui';
import type { Row } from '../../types/api';

export function QuestionBuilder({
  questions,
  setQuestions,
}: {
  questions: Row[];
  setQuestions: (q: Row[]) => void;
}) {
  const [label, setLabel] = useState('');
  const [type, setType] = useState('textarea');
  const [options, setOptions] = useState('');
  const [required, setRequired] = useState(true);
  return (
    <div class="question-builder">
      <h3>Role-specific questions</h3>
      <ol>
        {questions.map((q, i) => (
          <li key={i}>
            <span>
              {q.label}
              <small>
                {q.type}
                {q.required ? ' · required' : ''}
              </small>
            </span>
            <div>
              <button
                type="button"
                class="icon-button"
                disabled={i === 0}
                aria-label={`Move question ${i + 1} up`}
                onClick={() => {
                  const next = [...questions];
                  [next[i - 1], next[i]] = [next[i], next[i - 1]];
                  setQuestions(next);
                }}
              >
                ↑
              </button>
              <button
                type="button"
                class="icon-button"
                aria-label={`Remove question ${i + 1}`}
                onClick={() => setQuestions(questions.filter((_, n) => n !== i))}
              >
                ×
              </button>
            </div>
          </li>
        ))}
      </ol>
      <label class="field">
        <span>Question label</span>
        <input value={label} maxLength={240} onInput={(e) => setLabel(e.currentTarget.value)} />
      </label>
      <div class="inline-fields">
        <label class="field">
          <span>Answer type</span>
          <select value={type} onChange={(e) => setType(e.currentTarget.value)}>
            {[
              'text',
              'textarea',
              'number',
              'date',
              'boolean',
              'single_select',
              'multi_select',
              'url',
            ].map((v) => (
              <option key={v} value={v}>
                {v.replace('_', ' ')}
              </option>
            ))}
          </select>
        </label>
        <label class="check-field">
          <input
            type="checkbox"
            checked={required}
            onChange={(e) => setRequired(e.currentTarget.checked)}
          />
          Required
        </label>
      </div>
      {type.includes('select') && (
        <label class="field">
          <span>Choices, one per line</span>
          <textarea value={options} onInput={(e) => setOptions(e.currentTarget.value)} />
        </label>
      )}
      <button
        type="button"
        class="button secondary small-button"
        disabled={!label.trim() || (type.includes('select') && !options.trim())}
        onClick={() => {
          setQuestions([
            ...questions,
            {
              key: `custom_${crypto.randomUUID().replaceAll('-', '')}`,
              label,
              type,
              required,
              options: options
                .split('\n')
                .map((x) => x.trim())
                .filter(Boolean),
              max_length: 4000,
            },
          ]);
          setLabel('');
          setOptions('');
        }}
      >
        <Icon name="plus" size={15} />
        Add question
      </button>
    </div>
  );
}
