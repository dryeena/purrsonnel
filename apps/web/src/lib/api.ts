// Compatibility entry point. New modules import the service, hook, type, or helper directly.
export { useData } from '../hooks/use-data';
export { setCsrf } from '../services/api-client';
export { ApiError } from '../services/api-client';
export { api } from '../services/api-client';
export { authenticate } from '../services/auth-service';
export type { Row } from '../types/api';
export type { Grant } from '../types/api';
export type { Identity } from '../types/api';
export { date } from './date';
export { refresh } from './events';
export { csvCell } from './exports';
export { downloadJson } from './exports';
export { downloadCsv } from './exports';
export { can } from './permissions';
export { global } from './permissions';
