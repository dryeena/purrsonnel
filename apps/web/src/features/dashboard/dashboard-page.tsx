import {
  Avatar,
  Badge,
  Empty,
  ErrorBox,
  Icon,
  Loading,
  PageHead,
  Panel,
  Status,
} from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { date } from '../../lib/date';
import { can } from '../../lib/permissions';
import type { Row } from '../../types/api';

import { Task } from './task';

export function Dashboard(_: { path?: string }) {
  const identity = useAuth();
  const hr = can(identity, 'people.read');
  const { data, error, loading } = useData<Row>('/dashboard');
  const { data: interviews } = useData<Row[]>('/interviews');
  const { data: terms } = useData<Row[]>('/terms/applicable');
  const { data: people } = useData<Row>(hr ? '/users?page_size=5&sort=display_name' : null);
  if (loading) return <Loading />;
  if (error) return <ErrorBox message={error} />;
  if (!data) return null;
  const c = data.counts;
  const max = Math.max(...data.departments.map((d: Row) => Number(d.count)), 1);
  return (
    <>
      <PageHead
        eyebrow="PEOPLE MAKE THE MAGIC HAPPEN."
        title={`Hello, ${identity.user.display_name.split(' ')[0]}.`}
        description={
          hr
            ? 'Here’s what’s happening with your people. Let’s make a great weekend.'
            : 'Your team, your time, and your next steps. All in one place.'
        }
      >
        <span class="date-chip">
          <Icon name="calendar" size={16} />
          {date(new Date().toISOString())}
        </span>
      </PageHead>
      <section class="welcome-banner">
        <div class="welcome-copy">
          <Badge tone="plum">YOUR FWA WORKSPACE</Badge>
          <h2>
            People make
            <br />
            the magic happen.
          </h2>
          <p>
            A home for the teams, helping hands, and little details
            <br class="desktop" /> that bring Furry Weekend Atlanta together.
          </p>
          <a class="button primary" href={hr ? '/people' : '/volunteer'}>
            {hr ? 'Meet your people' : 'My volunteer time'}
            <Icon name="arrow" size={17} />
          </a>
        </div>
        <div class="brand-art">
          <div class="art-ring ring-one" />
          <div class="art-ring ring-two" />
          <span class="art-spark spark-one">✧</span>
          <span class="art-spark spark-two">✦</span>
          <img
            src="/branding/purrsonnel-logo.png"
            alt="Purrsonnel cat peeking over a personnel ID badge"
          />
          <span class="art-caption">People make the magic happen.</span>
        </div>
      </section>
      <div class="stats-grid">
        {(hr
          ? [
              {
                label: 'People in your scope',
                value: c.people,
                icon: 'people',
                note: `${c.staff} active personnel`,
                href: '/people',
                tone: 'plum',
              },
              {
                label: 'Hours awaiting approval',
                value: c.pending_approvals,
                icon: 'clock',
                note: 'A little recognition goes a long way',
                href: '/volunteer',
                tone: 'amber',
              },
              {
                label: 'Applications in progress',
                value: c.applications,
                icon: 'briefcase',
                note: 'Find your next great teammate',
                href: '/applications?in_progress=1',
                tone: 'blue',
              },
              {
                label: 'Your pending policies',
                value: c.pending_terms,
                icon: 'terms',
                note: 'Keep everyone on the same page',
                href: '/terms',
                tone: 'green',
              },
            ]
          : [
              {
                label: 'Verified service years',
                value: identity.user.verified_years,
                icon: 'people',
                note: 'Every year you showed up counts',
                href: `/people?person=${identity.user.id}`,
                tone: 'plum',
              },
              {
                label: 'Approved volunteer hours',
                value: (c.volunteer_minutes / 60).toFixed(1),
                icon: 'heart',
                note: 'Thank you for lending a hand',
                href: '/volunteer',
                tone: 'green',
              },
              {
                label: 'My applications',
                value: c.applications,
                icon: 'briefcase',
                note: 'Your next chapter starts here',
                href: '/applications?in_progress=1',
                tone: 'blue',
              },
              {
                label: 'Pending policies',
                value: c.pending_terms,
                icon: 'terms',
                note: 'A quick read, a shared commitment',
                href: '/terms',
                tone: 'amber',
              },
            ]
        ).map((stat) => (
          <a key={stat.label} class="stat-card" href={stat.href}>
            <div class="stat-top">
              <span class={`stat-icon ${stat.tone}`}>
                <Icon name={stat.icon} />
              </span>
              <Icon name="arrow" size={16} />
            </div>
            <div class="stat-value">{stat.value}</div>
            <h3>{stat.label}</h3>
            <p>{stat.note}</p>
          </a>
        ))}
      </div>
      <div class="dashboard-grid">
        <div class="dashboard-main">
          <Panel
            title="A few things to take care of"
            description="Small steps that keep the weekend moving."
            action={<span class="panel-tiny">YOUR NEXT STEPS</span>}
          >
            <div class="task-list">
              {c.incoming_requests > 0 && (
                <Task
                  icon="bell"
                  title={`${c.incoming_requests} requests need a helping hand`}
                  note="See what your teammates need and let them know help is on the way."
                  href="/requests"
                  tone="plum"
                />
              )}
              {c.pending_approvals > 0 && hr && (
                <Task
                  icon="heart"
                  title={`${c.pending_approvals} volunteer ${c.pending_approvals === 1 ? 'entry needs' : 'entries need'} a review`}
                  note="Review hours and give your volunteers a little credit."
                  href="/volunteer"
                  tone="amber"
                />
              )}
              {c.applications > 0 && (
                <Task
                  icon="briefcase"
                  title={
                    hr
                      ? `${c.applications} applications are in progress`
                      : 'Check in on your applications'
                  }
                  note={
                    hr
                      ? 'Thoughtful people are ready to join your team.'
                      : 'Follow along with your next opportunity.'
                  }
                  href="/applications?in_progress=1"
                  tone="blue"
                />
              )}
              {terms
                ?.filter((t) => !t.signed_at)
                .map((t) => (
                  <Task
                    key={t.version_id}
                    icon="terms"
                    title={t.title}
                    note={`Version ${t.version_number} · Your acknowledgment is needed`}
                    href="/terms"
                    tone="plum"
                  />
                ))}
              {c.pending_terms === 0 && c.pending_approvals === 0 && c.applications === 0 && (
                <Empty title="You’re all caught up">A little breathing room. Enjoy it.</Empty>
              )}
            </div>
          </Panel>
          <Panel
            title={hr ? 'People, at a glance' : 'Your people & profile'}
            description={
              hr
                ? 'A few friendly faces from your workspace.'
                : 'The details that connect you to your team.'
            }
            action={
              <a class="text-link" href="/people">
                {hr ? 'View directory' : 'My profile'}
                <Icon name="arrow" size={14} />
              </a>
            }
          >
            {hr ? (
              <div class="table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>PERSON</th>
                      <th>DEPARTMENT</th>
                      <th>STATUS</th>
                    </tr>
                  </thead>
                  <tbody>
                    {people?.items.map((u: Row) => (
                      <tr key={u.id}>
                        <td>
                          <a class="person-cell" href={`/people?person=${u.id}`}>
                            <Avatar name={u.display_name} small />
                            <span>
                              <strong>{u.display_name}</strong>
                              <small>{u.myfwa_id}</small>
                            </span>
                          </a>
                        </td>
                        <td>{u.department || 'Unassigned'}</td>
                        <td>
                          <Status value={u.status} />
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div class="profile-summary">
                <Avatar name={identity.user.display_name} />
                <div>
                  <h3>{identity.user.display_name}</h3>
                  <p>
                    {identity.user.departments.map((d: Row) => d.name).join(' · ') ||
                      'Your next team is waiting.'}
                  </p>
                  <p>{identity.user.service_years.join(', ') || 'No verified service years yet'}</p>
                </div>
              </div>
            )}
          </Panel>
        </div>
        <div class="dashboard-side">
          <Panel title="Coming up" action={<Icon name="calendar" size={18} />}>
            <div class="upcoming-list">
              {interviews
                ?.filter(
                  (i) => new Date(i.scheduled_start_at) >= new Date() && i.status === 'scheduled',
                )
                .slice(0, 3)
                .map((i) => (
                  <a key={i.id} class="upcoming-item" href="/interviews">
                    <div class="calendar-tile">
                      <span>
                        {new Date(i.scheduled_start_at).toLocaleString('en-US', {
                          month: 'short',
                          timeZone: 'America/New_York',
                        })}
                      </span>
                      <strong>
                        {new Date(i.scheduled_start_at).toLocaleString('en-US', {
                          day: 'numeric',
                          timeZone: 'America/New_York',
                        })}
                      </strong>
                    </div>
                    <div>
                      <strong>{i.applicant_name}</strong>
                      <p>{i.position}</p>
                      <small>{date(i.scheduled_start_at, true, i.timezone)}</small>
                    </div>
                  </a>
                ))}
              {!interviews?.filter(
                (i) => new Date(i.scheduled_start_at) >= new Date() && i.status === 'scheduled',
              ).length && (
                <Empty title="Room on the calendar">Upcoming interviews will appear here.</Empty>
              )}
            </div>
            <a class="panel-bottom-link" href="/interviews">
              Go to interviews
              <Icon name="arrow" size={15} />
            </a>
          </Panel>
          {hr && (
            <Panel title="A team effort" description="Active memberships by department.">
              <div class="department-bars">
                {data.departments.map((d: Row, i: number) => (
                  <div key={d.id}>
                    <div>
                      <span>
                        <i class={`dot dot-${i % 3}`} />
                        {d.name}
                      </span>
                      <strong>{d.count}</strong>
                    </div>
                    <div class="bar-track">
                      <div
                        style={{ width: `${(Number(d.count) / max) * 100}%` }}
                        class={`bar bar-${i % 3}`}
                      />
                    </div>
                  </div>
                ))}
              </div>
              <a class="panel-bottom-link" href="/organization">
                Explore organization
                <Icon name="arrow" size={15} />
              </a>
            </Panel>
          )}
          <div class="kind-card">
            <Icon name="heart" size={24} />
            <h3>Every helping hand matters.</h3>
            <p>Make it easy for someone to join in. Share the volunteer page with your team.</p>
            <a href="/volunteer" class="text-link">
              Find a way to help
              <Icon name="arrow" size={14} />
            </a>
          </div>
        </div>
      </div>
    </>
  );
}
