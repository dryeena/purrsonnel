import { Icon } from '../../components/ui';

export function Task({
  icon,
  title,
  note,
  href,
  tone,
}: {
  icon: string;
  title: string;
  note: string;
  href: string;
  tone: string;
}) {
  return (
    <a class="task" href={href}>
      <span class={`task-icon ${tone}`}>
        <Icon name={icon} />
      </span>
      <span>
        <strong>{title}</strong>
        <small>{note}</small>
      </span>
      <Icon name="chevron" size={17} />
    </a>
  );
}
