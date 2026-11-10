// Transitional DTO for configurable server-defined fields. Prefer named interfaces for new APIs.
// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Existing dynamic endpoint contracts share this boundary.
export type Row = Record<string, any>;

export interface Grant {
  permission: string;
  scope_type: 'global' | 'department' | 'division';
  scope_id: string | null;
}

export interface Identity {
  user: Row;
  grants: Grant[];
  dev_mode: boolean;
  identities: Row[];
  csrf_token: string;
}
