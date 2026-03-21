"use client";

import { createContext, useContext, useState, type ReactNode } from "react";

interface SimContextType {
  simcInput: string;
  setSimcInput: (v: string) => void;
  fightStyle: string;
  setFightStyle: (v: string) => void;
  threads: number;
  setThreads: (v: number) => void;
}

const SimContext = createContext<SimContextType | null>(null);

export function useSimContext() {
  const ctx = useContext(SimContext);
  if (!ctx) throw new Error("useSimContext must be used within SimProvider");
  return ctx;
}

export function SimProvider({ children }: { children: ReactNode }) {
  const [simcInput, setSimcInput] = useState("");
  const [fightStyle, setFightStyle] = useState("Patchwerk");
  const [threads, setThreads] = useState(0);

  return (
    <SimContext.Provider
      value={{ simcInput, setSimcInput, fightStyle, setFightStyle, threads, setThreads }}
    >
      {children}
    </SimContext.Provider>
  );
}
