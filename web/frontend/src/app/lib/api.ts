// API URL detection: Tauri desktop uses a fixed local server port
export const API_URL =
  typeof window !== "undefined" &&
  (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
    ? "http://localhost:17384"
    : process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000";
