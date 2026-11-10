import type { Identity } from '../types/api';

export function can(identity: Identity, permission: string) {
  return identity.grants.some((g) => g.permission === permission);
}

export function global(identity: Identity, permission: string) {
  return identity.grants.some((g) => g.permission === permission && g.scope_type === 'global');
}
