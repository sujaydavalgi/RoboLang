/**
 * Shared fetch helpers for remote deploy and fleet clients.
 * @module
 */

export const REMOTE_HTTP_TIMEOUT_MS = 30_000;

/**
 * Produces a sanitized URL string for error messages by removing query strings
 * and fragments. Returns a placeholder when parsing fails.
 *
 * @param url - Raw request URL.
 * @returns Origin plus pathname, or `"[unparseable-url]"` if parsing fails.
 * @example
 * safeUrlForError("https://api.example.com/deploy?token=secret")
 * // => "https://api.example.com/deploy"
 */
function safeUrlForError(url: string): string {
  try {
    const parsed = new URL(url);
    return `${parsed.origin}${parsed.pathname}`;
  } catch {
    return "[unparseable-url]";
  }
}

/**
 * Performs an HTTP fetch with a bounded total lifetime and abort-signal propagation.
 *
 * This helper wraps `fetch` to enforce a maximum duration of
 * `REMOTE_HTTP_TIMEOUT_MS` for the full request lifecycle. If the timeout elapses,
 * the request is aborted via an internal `AbortController`.
 *
 * If `init.signal` is provided, its abort is forwarded to the internal controller.
 * The forwarded abort reason is preserved when available; otherwise a default
 * `AbortError` DOMException is used.
 *
 * @param url - The request URL to fetch.
 * @param init - Standard fetch options. If `init.signal` is provided, abort events
 * are propagated to this request.
 * @returns A promise that resolves to the `Response` from `fetch`.
 * @throws Rejected when the request fails, is aborted by `init.signal`, or times out.
 */
export function remoteFetch(url: string, init: RequestInit = {}): Promise<Response> {
  // Issue one HTTP request with a bounded wait for connect and response body.
  const controller = new AbortController();
  const { signal: upstreamSignal, ...restInit } = init;
  let upstreamAbortHandler: (() => void) | undefined;
  const detachUpstreamAbortListener = () => {
    if (upstreamAbortHandler) {
      upstreamSignal?.removeEventListener("abort", upstreamAbortHandler);
    }
  };

  if (upstreamSignal?.aborted) {
    const reason =
      upstreamSignal.reason === undefined
        ? new DOMException("Remote fetch operation was aborted.", "AbortError")
        : upstreamSignal.reason;
    return Promise.reject(reason);
  }

  // Bound the full fetch lifecycle (connect + response/body read); expiry aborts via AbortController.
  const timeoutId = setTimeout(() => {
    detachUpstreamAbortListener();
    controller.abort(
      new DOMException(
        `Remote fetch timed out after ${REMOTE_HTTP_TIMEOUT_MS}ms for URL: ${safeUrlForError(url)}`,
        "TimeoutError",
      ),
    );
  }, REMOTE_HTTP_TIMEOUT_MS);

  if (upstreamSignal) {
    upstreamAbortHandler = () => {
      detachUpstreamAbortListener();
      clearTimeout(timeoutId);
      const reason =
        upstreamSignal.reason === undefined
          ? new DOMException("Remote fetch operation was aborted.", "AbortError")
          : upstreamSignal.reason;
      controller.abort(reason);
    };
    upstreamSignal.addEventListener("abort", upstreamAbortHandler, { once: true });
  }

  return fetch(url, { ...restInit, signal: controller.signal }).finally(() => {
    clearTimeout(timeoutId);
    detachUpstreamAbortListener();
  });
}
