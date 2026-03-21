"use client";

import { useEffect, useState, useCallback } from "react";

export default function WindowTitlebar() {
  const [isMaximized, setIsMaximized] = useState(false);

  const windowAction = useCallback(async (action: "minimize" | "toggleMaximize" | "close") => {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      if (action === "minimize") await win.minimize();
      else if (action === "toggleMaximize") {
        await win.toggleMaximize();
        setIsMaximized(await win.isMaximized());
      }
      else if (action === "close") await win.close();
    } catch {}
  }, []);

  useEffect(() => {
    if (!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) return;
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        setIsMaximized(await win.isMaximized());
        unlisten = await win.onResized(async () => {
          setIsMaximized(await win.isMaximized());
        });
      } catch {}
    })();
    return () => { unlisten?.(); };
  }, []);

  return (
    <div
      data-tauri-drag-region
      className="desktop-only"
    >
      <div
        data-tauri-drag-region
        className="h-8 flex items-center justify-between select-none bg-[#0a0a0a] border-b border-white/[0.06] sticky top-0 z-[100]"
      >
        <div data-tauri-drag-region className="flex items-center gap-2 pl-3">
          <div className="w-3.5 h-3.5 rounded-sm bg-gold/90 flex items-center justify-center">
            <svg className="w-2 h-2 text-black" viewBox="0 0 16 16" fill="currentColor">
              <path d="M3 2l10 6-10 6V2z" />
            </svg>
          </div>
          <span data-tauri-drag-region className="text-[11px] font-medium text-gray-500">
            SimHammer
          </span>
        </div>

        <div className="flex h-full">
          <button
            onClick={() => windowAction("minimize")}
            className="w-11 h-full flex items-center justify-center text-gray-500 hover:bg-white/[0.06] hover:text-gray-300 transition-colors"
          >
            <svg className="w-3 h-[1px]" viewBox="0 0 12 1" fill="currentColor">
              <rect width="12" height="1" />
            </svg>
          </button>
          <button
            onClick={() => windowAction("toggleMaximize")}
            className="w-11 h-full flex items-center justify-center text-gray-500 hover:bg-white/[0.06] hover:text-gray-300 transition-colors"
          >
            {isMaximized ? (
              <svg className="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1">
                <path d="M2 3h5v5H2z" />
                <path d="M3 3V2h5v5H7" />
              </svg>
            ) : (
              <svg className="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1">
                <rect x="1" y="1" width="8" height="8" />
              </svg>
            )}
          </button>
          <button
            onClick={() => windowAction("close")}
            className="w-11 h-full flex items-center justify-center text-gray-500 hover:bg-red-500 hover:text-white transition-colors"
          >
            <svg className="w-2.5 h-2.5" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M1 1l8 8M9 1l-8 8" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
