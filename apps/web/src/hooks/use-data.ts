import { useEffect, useRef, useState } from 'preact/hooks';

import { api } from '../services/api-client';

export function useData<T>(path: string | null, pollMs = 0) {
  const loadedPath = useRef<string | null>(null);
  const [data, setData] = useState<T>();
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);
  const [version, setVersion] = useState(0);
  const [refreshing, setRefreshing] = useState(false);
  const [updatedAt, setUpdatedAt] = useState<number>();
  useEffect(() => {
    const listener = () => setVersion((v) => v + 1);
    window.addEventListener('purrsonnel:refresh', listener);
    return () => window.removeEventListener('purrsonnel:refresh', listener);
  }, []);
  useEffect(() => {
    if (!path || !pollMs) return;
    const timer = setInterval(() => {
      if (document.visibilityState !== 'hidden') setVersion((v) => v + 1);
    }, pollMs);
    return () => clearInterval(timer);
  }, [path, pollMs]);
  useEffect(() => {
    let alive = true;
    if (!path) {
      setLoading(false);
      setRefreshing(false);
      setData(undefined);
      setUpdatedAt(undefined);
      return;
    }
    setLoading(loadedPath.current !== path);
    setRefreshing(true);
    setError('');
    api<T>(path)
      .then((v) => {
        if (alive) {
          loadedPath.current = path;
          setData(v);
          setUpdatedAt(Date.now());
        }
      })
      .catch((e) => {
        if (alive) setError(e.message);
      })
      .finally(() => {
        if (alive) {
          setLoading(false);
          setRefreshing(false);
        }
      });
    return () => {
      alive = false;
    };
  }, [path, version]);
  return { data, error, loading, refreshing, updatedAt, reload: () => setVersion((v) => v + 1) };
}
