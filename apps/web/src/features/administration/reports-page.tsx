import { Icon, PageHead, Panel } from '../../components/ui';
import { useAuth } from '../../lib/auth';
import { can } from '../../lib/permissions';

export function Reports(_: { path?: string }) {
  const me = useAuth();
  const reports = [
    {
      title: 'Volunteer hours',
      description: 'Approved volunteer time by person, department, and event.',
      href: '/volunteer?tab=report',
      available: can(me, 'volunteer.time.approve'),
    },
    {
      title: 'People directory export',
      description: 'Filter people, then export the current page or selected people as CSV.',
      href: '/people',
      available: can(me, 'people.read'),
    },
    {
      title: 'Audit log',
      description: 'A read-only history of workspace changes.',
      href: '/audit',
      available: can(me, 'audit.read'),
    },
  ].filter((report) => report.available);
  return (
    <>
      <PageHead
        eyebrow="REPORTING"
        title="Reports"
        description="Find the information you are authorized to review or export."
      />
      <div class="card-grid">
        {reports.map((report) => (
          <Panel key={report.href}>
            <div class="policy-card">
              <span class="stat-icon plum">
                <Icon name="list" />
              </span>
              <h2>{report.title}</h2>
              <p class="muted">{report.description}</p>
              <a class="button secondary" href={report.href}>
                Open report <Icon name="arrow" size={16} />
              </a>
            </div>
          </Panel>
        ))}
      </div>
    </>
  );
}
