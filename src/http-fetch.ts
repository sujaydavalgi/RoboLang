/**
 * Shared fetch helpers for remote deploy and fleet clients.
 * @module
 */

export const REMOTE_HTTP_TIMEOUT_MS = 30_000;

export function remoteFetch(url: string, init: RequestInit = {}): Promise<Response> {
  // Issue one HTTP request with a bounded wait for connect and response body.
  return fetch(url, { ...init, signal: AbortSignal.timeout(REMOTE_HTTP_TIMEOUT_MS) });
}
