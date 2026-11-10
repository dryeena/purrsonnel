import type { ComponentChildren } from 'preact';

import { humanize } from '../../lib/text';

import { Icon } from './icon';

export function Badge({
  children,
  tone = 'neutral',
}: {
  children: ComponentChildren;
  tone?: string;
}) {
  return <span class={`badge ${tone}`}>{children}</span>;
}

export function Status({ value }: { value: string }) {
  const tone =
    value === 'accepted'
      ? 'blue'
      : ['active', 'approved', 'open', 'completed', 'approve'].includes(value)
        ? 'green'
        : ['submitted', 'interviewing', 'scheduled', 'pending', 'previewed', 'screening'].includes(
              value,
            )
          ? 'amber'
          : ['rejected', 'deny', 'former', 'partially_failed'].includes(value)
            ? 'red'
            : 'neutral';
  return <Badge tone={tone}>{humanize(value)}</Badge>;
}

export function Avatar({ name, small = false }: { name: string; small?: boolean }) {
  return (
    <span class={`avatar ${small ? 'small' : ''}`} aria-hidden="true">
      {name
        .split(' ')
        .map((n) => n[0])
        .slice(0, 2)
        .join('')}
    </span>
  );
}

export function Empty({
  title = 'Nothing here yet',
  children,
  icon = 'heart',
  action,
}: {
  title?: string;
  children?: ComponentChildren;
  icon?: string;
  action?: ComponentChildren;
}) {
  return (
    <div class="empty">
      <span class="empty-icon">
        <Icon name={icon} size={28} />
      </span>
      <h3>{title}</h3>
      <p>{children || 'When there’s something to show, you’ll find it here.'}</p>
      {action && <div class="empty-action">{action}</div>}
    </div>
  );
}

export function Loading() {
  return (
    <div class="loading loading-stack" role="status">
      <div>
        <span class="spinner" aria-hidden="true" />
        Gathering the details…
      </div>
      <div class="loading-placeholder" aria-hidden="true">
        <i />
        <i />
        <i />
      </div>
    </div>
  );
}

export function ErrorBox({ message }: { message: string }) {
  return (
    <div class="error-box" role="alert">
      {message}
    </div>
  );
}

export function PageHead({
  eyebrow,
  title,
  description,
  children,
}: {
  eyebrow?: string;
  title: string;
  description: string;
  children?: ComponentChildren;
}) {
  return (
    <div class="page-head">
      <div>
        {eyebrow && <div class="eyebrow">{eyebrow}</div>}
        <h1>{title}</h1>
        <p>{description}</p>
      </div>
      <div class="page-actions">{children}</div>
    </div>
  );
}

export function Panel({
  title,
  description,
  children,
  action,
  className = '',
}: {
  title?: string;
  description?: string;
  children: ComponentChildren;
  action?: ComponentChildren;
  className?: string;
}) {
  return (
    <section class={`panel ${className}`}>
      {title && (
        <div class="panel-head">
          <div>
            <h2>{title}</h2>
            {description && <p>{description}</p>}
          </div>
          {action}
        </div>
      )}
      {children}
    </section>
  );
}
