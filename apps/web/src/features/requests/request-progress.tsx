import { Icon } from '../../components/ui';
import type { Row } from '../../types/api';

export function RequestProgress({ request }: { request: Row }) {
  if (request.status === 'cancelled')
    return (
      <div class="request-cancelled">
        <Icon name="close" size={17} />
        This request was cancelled. No further action is needed.
      </div>
    );
  const current = request.status === 'completed' ? 2 : request.status === 'accepted' ? 1 : 0;
  return (
    <ol class="request-progress" aria-label="Request progress">
      {['Sent', 'Accepted', 'Completed'].map((label, index) => (
        <li
          key={label}
          class={index <= current ? 'reached' : ''}
          aria-current={index === current ? 'step' : undefined}
        >
          <span class="progress-dot" aria-hidden="true">
            {index <= current ? <Icon name="check" size={12} /> : index + 1}
          </span>
          <span>
            {label}
            {index > current && <small>Not yet</small>}
          </span>
        </li>
      ))}
    </ol>
  );
}
