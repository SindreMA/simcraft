"use client";

import { usePathname } from "next/navigation";
import { useSimContext } from "./SimContext";
import FightStyleSelector from "./FightStyleSelector";

function parseCharacterInfo(input: string) {
  if (!input) return null;
  const nameMatch = input.match(/^(\w+)="(.+)"$/m);
  const specMatch = input.match(/^spec=(\w+)/m);
  if (!nameMatch) return null;
  return {
    className: nameMatch[1],
    name: nameMatch[2],
    spec: specMatch?.[1] || "unknown",
  };
}

export default function SimSharedConfig() {
  const pathname = usePathname();
  const { simcInput, setSimcInput, fightStyle, setFightStyle } =
    useSimContext();

  const isSimPage = pathname === "/quick-sim" || pathname === "/top-gear";
  if (!isSimPage) return null;

  const detectedInfo = parseCharacterInfo(simcInput);

  return (
    <div className="space-y-6 mb-6">
      <div className="card p-5 space-y-3">
        <label className="label-text">SimC Addon Export</label>
        <textarea
          value={simcInput}
          onChange={(e) => setSimcInput(e.target.value)}
          placeholder="Paste your SimC addon export here…"
          className="input-field h-44 font-mono text-xs resize-y"
        />
        {detectedInfo && (
          <p className="text-xs text-gold">
            {detectedInfo.name} &middot; {detectedInfo.spec}{" "}
            {detectedInfo.className}
          </p>
        )}
      </div>

      <FightStyleSelector value={fightStyle} onChange={setFightStyle} />
    </div>
  );
}
