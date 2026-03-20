"use client";

import { useEffect, useState } from "react";
import { API_URL } from "../lib/api";

export default function SystemInfo() {
  const [threads, setThreads] = useState<number | null>(null);
  const [mode, setMode] = useState<string | null>(null);

  useEffect(() => {
    fetch(`${API_URL}/health`)
      .then((res) => res.json())
      .then((data) => {
        if (data.threads) setThreads(data.threads);
        if (data.mode) setMode(data.mode);
      })
      .catch(() => {});
  }, []);

  if (!mode || mode !== "desktop") return null;

  return (
    <span className="text-[10px] text-muted font-mono bg-surface-2 border border-border px-1.5 py-0.5 rounded">
      {threads} threads
    </span>
  );
}
