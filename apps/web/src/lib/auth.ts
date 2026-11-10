import { createContext } from 'preact';
import { useContext } from 'preact/hooks';

import type { Identity } from '../types/api';
export const AuthContext = createContext<Identity | null>(null);
export function useAuth() {
  const value = useContext(AuthContext);
  if (!value) throw new Error('Authentication is not ready');
  return value;
}
