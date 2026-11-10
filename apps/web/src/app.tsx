import { useEffect, useRef, useState } from 'preact/hooks';
import Router, { route } from 'preact-router';

import { AppStatus } from './components/app-status';
import { Feedback } from './components/feedback';
import { InstallApp } from './components/install';
import { Notifications } from './components/notifications';
import { Avatar, Badge, ErrorBox, Icon, Loading } from './components/ui';
import { Audit, Lists, Organization, Reports, Settings } from './features/administration';
import { Dashboard } from './features/dashboard';
import { Applications, Interviews, Positions } from './features/hiring';
import { People } from './features/people';
import { Requests } from './features/requests';
import { Terms } from './features/terms';
import { Volunteers } from './features/volunteers';
import { AuthContext } from './lib/auth';
import { refresh } from './lib/events';
import { can } from './lib/permissions';
import { disablePush } from './lib/push';
import { confirmNavigation } from './lib/unsaved';
import { api, setCsrf } from './services/api-client';
import { authenticate } from './services/auth-service';
import type { Identity } from './types/api';

const nav = [
  { label: 'Overview', path: '/', icon: 'dashboard' },
  { label: 'Requests', path: '/requests', icon: 'bell' },
  { label: 'People', path: '/people', icon: 'people', permission: 'people.read' },
  { label: 'Organization', path: '/organization', icon: 'organization', permission: 'people.read' },
  { label: 'Volunteers', path: '/volunteer', icon: 'heart' },
  { label: 'Terms & policies', path: '/terms', icon: 'terms' },
  { label: 'Positions', path: '/positions', icon: 'briefcase' },
  { label: 'Applications', path: '/applications', icon: 'terms' },
  { label: 'Interviews', path: '/interviews', icon: 'calendar' },
  { label: 'Lists & bulk actions', path: '/lists', icon: 'list', permission: 'people.bulk_update' },
  { label: 'Settings', path: '/settings', icon: 'settings', permission: 'settings.manage' },
  { label: 'Audit log', path: '/audit', icon: 'audit', permission: 'audit.read' },
  { label: 'Reports', path: '/reports', icon: 'clipboard', permission: 'people.read' },
];

export function App() {
  const [identity, setIdentity] = useState<Identity>();
  const [error, setError] = useState('');
  const [currentUrl, setCurrentUrl] = useState(location.pathname + location.search);
  const path = currentUrl.split('?')[0];
  const [mobile, setMobile] = useState(false);
  const [search, setSearch] = useState('');
  const [switching, setSwitching] = useState(false);
  const sidebar = useRef<HTMLElement>(null);
  async function boot() {
    setError('');
    try {
      setIdentity(await authenticate());
    } catch (e) {
      setError((e as Error).message);
    }
  }
  useEffect(() => {
    void boot();
  }, []);
  useEffect(() => {
    const update = () => {
      void api<Identity>('/auth/me')
        .then((next) => {
          setIdentity((prior) => (prior?.user.id === next.user.id ? next : prior));
        })
        .catch(() => {});
    };
    window.addEventListener('purrsonnel:refresh', update);
    return () => window.removeEventListener('purrsonnel:refresh', update);
  }, []);
  useEffect(() => {
    if (!mobile) return;
    const old = document.activeElement as HTMLElement | null;
    const main = document.querySelector('.main-shell');
    const overflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    main?.setAttribute('inert', '');
    sidebar.current?.querySelector<HTMLElement>('a')?.focus();
    const key = (event: KeyboardEvent) => {
      if (document.querySelector('[role=dialog]')) return;
      if (event.key === 'Escape') {
        event.preventDefault();
        setMobile(false);
      }
      if (event.key === 'Tab') {
        const items = sidebar.current?.querySelectorAll<HTMLElement>(
          'a[href],button:not([disabled]),select',
        );
        if (!items?.length) return;
        const first = items[0],
          last = items[items.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }
    };
    const resize = () => {
      if (!matchMedia('(max-width: 720px)').matches) setMobile(false);
    };
    document.addEventListener('keydown', key);
    window.addEventListener('resize', resize);
    return () => {
      document.body.style.overflow = overflow;
      main?.removeAttribute('inert');
      document.removeEventListener('keydown', key);
      window.removeEventListener('resize', resize);
      if (old?.isConnected) old.focus();
    };
  }, [mobile]);
  if (error && !identity)
    return (
      <>
        <AppStatus />
        <div class="boot">
          <img src="/branding/favicon.svg" width="64" alt="Purrsonnel" />
          <h1>Let’s get connected.</h1>
          <p>
            Check your connection and try again. If this keeps happening, contact your team
            administrator.
          </p>
          <ErrorBox message={error} />
          <button type="button" class="button primary" onClick={boot}>
            Try again
          </button>
          <InstallApp />
        </div>
      </>
    );
  if (!identity)
    return (
      <div class="boot">
        <img src="/branding/favicon.svg" width="64" alt="Purrsonnel" />
        <Loading />
      </div>
    );
  const current = nav.find((n) => (n.path === '/' ? path === '/' : path.startsWith(n.path)));
  async function switchIdentity(userId: string) {
    if (!confirmNavigation()) return;
    setSwitching(true);
    setError('');
    try {
      await disablePush();
      const result = await api('/auth/dev/switch', 'POST', { user_id: userId });
      setCsrf(result.csrf_token);
      const next = await api<Identity>('/auth/me');
      setIdentity(next);
      setCsrf(next.csrf_token);
      route('/');
      refresh();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setSwitching(false);
    }
  }
  return (
    <AuthContext.Provider value={identity}>
      <a class="skip-link" href="#main">
        Skip to content
      </a>
      <Feedback />
      <div class="app-shell">
        <aside
          ref={sidebar}
          class={`sidebar ${mobile ? 'is-open' : ''}`}
          aria-label="Workspace navigation"
        >
          <a class="brand" href="/" aria-label="Purrsonnel home">
            <span class="wordmark">
              <img src="/branding/purrsonnel-logo.png" alt="Purrsonnel" />
            </span>
            <span class="brand-sub">THE PEOPLE BEHIND THE MAGIC</span>
          </a>
          <div class="workspace">
            <span class="workspace-mark">F</span>
            <div>
              <strong>Furry Weekend Atlanta</strong>
              <small>Personnel workspace</small>
            </div>
            <span class="workspace-dot" />
          </div>
          <div class="nav-label">WORKSPACE</div>
          <nav aria-label="Main navigation">
            {nav
              .filter((n) => !n.permission || can(identity, n.permission))
              .map((n) => (
                <a
                  key={n.path}
                  href={n.path}
                  class={`${current?.path === n.path ? 'active' : ''} ${n.path === '/lists' ? 'nav-divider' : ''}`}
                  aria-current={current?.path === n.path ? 'page' : undefined}
                  onClick={() => setMobile(false)}
                >
                  <Icon name={n.icon} size={19} />
                  <span>{n.label}</span>
                </a>
              ))}
          </nav>
          <div class="sidebar-bottom">
            <InstallApp />
            <a class="profile" href="/people?self=1" onClick={() => setMobile(false)}>
              <Avatar name={identity.user.display_name} />
              <div>
                <strong>{identity.user.display_name}</strong>
                <small>
                  {can(identity, 'people.bulk_update')
                    ? 'HR workspace'
                    : identity.user.level || 'Community member'}
                </small>
              </div>
              <Icon name="chevron" size={16} />
            </a>
          </div>
        </aside>
        {mobile && (
          <button
            type="button"
            class="sidebar-shade"
            aria-label="Close navigation"
            onClick={() => setMobile(false)}
          />
        )}
        <div class="main-shell">
          <header class="topbar">
            <div class="breadcrumb">
              <button
                type="button"
                class="icon-button mobile-menu"
                aria-label="Open navigation"
                aria-expanded={mobile}
                onClick={() => setMobile(true)}
              >
                <Icon name="menu" />
              </button>
              <a href="/">Workspace</a>
              <Icon name="chevron" size={13} />
              <strong>{current?.label || 'Volunteer approval'}</strong>
            </div>
            <div class="topbar-right">
              {can(identity, 'people.read') && (
                <form
                  class="global-search"
                  onSubmit={(e) => {
                    e.preventDefault();
                    if (confirmNavigation()) route(`/people?q=${encodeURIComponent(search)}`);
                  }}
                >
                  <Icon name="search" size={16} />
                  <input
                    aria-label="Search people"
                    value={search}
                    onInput={(e) => setSearch(e.currentTarget.value)}
                    placeholder="Find a person…"
                  />
                  <kbd>↵</kbd>
                </form>
              )}
              <Notifications key={identity.user.id} />
              <a class="topbar-avatar" href="/people?self=1" aria-label="My profile">
                <Avatar name={identity.user.display_name} small />
              </a>
            </div>
          </header>
          <AppStatus />
          {identity.dev_mode && (
            <div class="dev-strip">
              <span>
                <b>DEV MODE</b> Fictional data · development authentication
              </span>
              <label>
                Viewing as{' '}
                <select
                  aria-label="Development identity"
                  value={identity.user.id}
                  disabled={switching}
                  onChange={(e) => {
                    void switchIdentity(e.currentTarget.value);
                  }}
                >
                  {identity.identities.map((u) => (
                    <option key={u.id} value={u.id}>
                      {u.display_name} · {u.email.split('@')[0].replace('dev.', '')}
                    </option>
                  ))}
                </select>
              </label>
            </div>
          )}
          {error && <ErrorBox message={error} />}
          <main id="main" tabIndex={-1} key={identity.user.id}>
            <Router
              onChange={(e) => {
                const pathname = e.url.split('?')[0];
                setCurrentUrl(e.url);
                setMobile(false);
                document.title = `${nav.find((n) => n.path === pathname)?.label || (pathname === '/volunteer/approve' ? 'Volunteer approval' : 'Purrsonnel')} · Purrsonnel`;
                setTimeout(() => document.getElementById('main')?.focus(), 0);
              }}
            >
              <Dashboard path="/" />
              <Requests path="/requests" />
              <People path="/people" />
              <Organization path="/organization" />
              <Terms path="/terms" />
              <Volunteers path="/volunteer" />
              <Volunteers path="/volunteer/approve" />
              <Positions path="/positions" />
              <Applications path="/applications" />
              <Interviews path="/interviews" />
              <Lists path="/lists" />
              <Settings path="/settings" />
              <Audit path="/audit" />
              <Reports path="/reports" />
              <NotFound default />
            </Router>
          </main>
          <footer class="app-footer">
            <span>Purrsonnel · People make the magic happen.</span>
            <span>Made for FWA {identity.dev_mode && <Badge>Development</Badge>}</span>
          </footer>
          <nav class="mobile-dock" aria-label="Quick navigation">
            {[
              { path: '/', label: 'Home', icon: 'dashboard' },
              { path: '/requests', label: 'Requests', icon: 'bell' },
              { path: '/volunteer', label: 'Volunteer', icon: 'heart' },
              { path: '/people?self=1', label: 'Me', icon: 'people' },
            ].map((n) => (
              <a
                key={n.path}
                href={n.path}
                aria-current={
                  (n.path.includes('?') ? currentUrl === n.path : path === n.path)
                    ? 'page'
                    : undefined
                }
              >
                <Icon name={n.icon} size={19} />
                <span>{n.label}</span>
              </a>
            ))}
          </nav>
        </div>
      </div>
    </AuthContext.Provider>
  );
}
function NotFound(_: { default?: boolean }) {
  return (
    <div class="boot">
      <h1>This page wandered off.</h1>
      <a class="button primary" href="/">
        Back to overview
      </a>
    </div>
  );
}
