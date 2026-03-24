"use client";

import { createContext, useCallback, useContext, useState, type ReactNode } from "react";

interface SimContextType {
  simcInput: string;
  setSimcInput: (v: string) => void;
  fightStyle: string;
  setFightStyle: (v: string) => void;
  threads: number;
  setThreads: (v: number) => void;
  selectedTalent: string;
  setSelectedTalent: (v: string) => void;
  targetCount: number;
  setTargetCount: (v: number) => void;
  fightLength: number;
  setFightLength: (v: number) => void;
  customApl: string;
  setCustomApl: (v: string) => void;
  // Expert Mode injection points
  simcHeader: string;
  setSimcHeader: (v: string) => void;
  simcBasePlayer: string;
  setSimcBasePlayer: (v: string) => void;
  simcRaidActors: string;
  setSimcRaidActors: (v: string) => void;
  simcPostCombos: string;
  setSimcPostCombos: (v: string) => void;
  simcFooter: string;
  setSimcFooter: (v: string) => void;
}

const SimContext = createContext<SimContextType | null>(null);

export function useSimContext() {
  const ctx = useContext(SimContext);
  if (!ctx) throw new Error("useSimContext must be used within SimProvider");
  return ctx;
}

function readStoredThreads(): number {
  if (typeof window === "undefined") return 0;
  const v = localStorage.getItem("simhammer_threads");
  if (v == null) return 0;
  const n = parseInt(v, 10);
  return Number.isFinite(n) && n > 0 ? n : 0;
}

export function SimProvider({ children }: { children: ReactNode }) {
  const [simcInput, setSimcInput] = useState("");
  const [fightStyle, setFightStyle] = useState("Patchwerk");
  const [threads, _setThreads] = useState(readStoredThreads);
  const [selectedTalent, setSelectedTalent] = useState("");
  const [targetCount, setTargetCount] = useState(1);
  const [fightLength, setFightLength] = useState(300);
  const [customApl, setCustomApl] = useState("");
  const [simcHeader, setSimcHeader] = useState("");
  const [simcBasePlayer, setSimcBasePlayer] = useState("");
  const [simcRaidActors, setSimcRaidActors] = useState("");
  const [simcPostCombos, setSimcPostCombos] = useState("");
  const [simcFooter, setSimcFooter] = useState("");

  const setThreads = useCallback((v: number) => {
    _setThreads(v);
    try { localStorage.setItem("simhammer_threads", String(v)); } catch {}
  }, []);

  return (
    <SimContext.Provider
      value={{ simcInput, setSimcInput, fightStyle, setFightStyle, threads, setThreads, selectedTalent, setSelectedTalent, targetCount, setTargetCount, fightLength, setFightLength, customApl, setCustomApl, simcHeader, setSimcHeader, simcBasePlayer, setSimcBasePlayer, simcRaidActors, setSimcRaidActors, simcPostCombos, setSimcPostCombos, simcFooter, setSimcFooter }}
    >
      {children}
    </SimContext.Provider>
  );
}
