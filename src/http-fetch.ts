/**
 * Shared fetch helpers for remote deploy and fleet clients.
 * @module
 */

export const REMOTE_HTTP_TIMEOUT_MS = 30_000;

export function remoteFetch(url: string, init: RequestInit = {}): Promise<Response> {
  // Issue one HTTP request with a bounded wait for connect and response body.
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), REMOTE_HTTP_TIMEOUT_MS);

  const upstreamSignal = init.signal;
  let onAbort: (() => void) | undefined;

  if (upstreamSignal) {
    if (upstreamSignal.aborted) {
      controller.abort();
    } else {
      onAbort = () => controller.abort();
      upstreamSignal.addEventListener("abort", onAbort, { once: true });
    }
  }

  return fetch(url, { ...init, signal: controller.signal }).finally(() => {
    clearTimeout(timeoutId);
    if (upstreamSignal && onAbort) {
      upstreamSignal.removeEventListener("abort", onAbort);
    }
  });
}
