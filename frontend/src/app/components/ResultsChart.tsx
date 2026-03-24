"use client";

import { useEffect, useState } from "react";

interface Ability {
  name: string;
  portion_dps: number;
  school: string;
  spell_id?: number;
}

interface ResultsChartProps {
  dps: number;
  dpsError: number;
  fightLength: number;
  playerName: string;
  playerClass: string;
  abilities: Ability[];
}

const iconCache = new Map<number, string>();

function useSpellIcons(spellIds: number[]) {
  const [icons, setIcons] = useState<Map<number, string>>(new Map());

  useEffect(() => {
    const missing = spellIds.filter((id) => id > 0 && !iconCache.has(id));
    if (missing.length === 0) {
      setIcons(new Map(iconCache));
      return;
    }

    let cancelled = false;
    Promise.all(
      missing.map(async (id) => {
        try {
          const res = await fetch(
            `https://nether.wowhead.com/tooltip/spell/${id}?dataEnv=1&locale=0`
          );
          if (!res.ok) return;
          const data = await res.json();
          if (data.icon) iconCache.set(id, data.icon);
        } catch {
          // ignore
        }
      })
    ).then(() => {
      if (!cancelled) setIcons(new Map(iconCache));
    });

    return () => { cancelled = true; };
  }, [spellIds.join(",")]); // eslint-disable-line react-hooks/exhaustive-deps

  return icons;
}

function SpellIcon({ icon }: { icon: string }) {
  return (
    <img
      src={`https://wow.zamimg.com/images/wow/icons/small/${icon}.jpg`}
      alt=""
      className="w-5 h-5 rounded-[3px] shrink-0"
    />
  );
}

const SCHOOL_COLORS: Record<string, string> = {
  physical: "#D4A843",
  holy: "#F5E6A3",
  fire: "#EF6461",
  nature: "#6BCB77",
  frost: "#6CB4EE",
  shadow: "#B07CD8",
  arcane: "#E88AED",
};

export default function ResultsChart({
  dps,
  dpsError,
  fightLength,
  playerName,
  playerClass,
  abilities,
}: ResultsChartProps) {
  const totalDps = dps || abilities.reduce((s, a) => s + a.portion_dps, 0);
  const top = abilities.slice(0, 15);
  const maxDps = top.length > 0 ? top[0].portion_dps : 1;
  const spellIds = top.map((a) => a.spell_id || 0);
  const icons = useSpellIcons(spellIds);

  return (
    <div className="space-y-6">
      <div className="card p-8 text-center">
        <p className="text-xs text-muted mb-4">
          {playerName} &middot; {playerClass}
        </p>
        <p className="text-5xl font-bold text-white tabular-nums tracking-tight">
          {Math.round(dps).toLocaleString()}
        </p>
        <p className="text-xs text-muted mt-2 uppercase tracking-widest">DPS</p>
        <div className="flex items-center justify-center gap-4 mt-3 text-xs text-gray-600">
          <span>&plusmn; {Math.round(dpsError).toLocaleString()}</span>
          <span className="w-px h-3 bg-border" />
          <span>{fightLength}s fight</span>
        </div>
      </div>

      <div className="card p-5">
        <h3 className="text-xs font-medium text-muted uppercase tracking-widest mb-4">
          Damage Breakdown
        </h3>
        <div className="space-y-1">
          {top.map((a, i) => {
            const color = SCHOOL_COLORS[a.school] || SCHOOL_COLORS.physical;
            const pct = totalDps > 0 ? (a.portion_dps / totalDps) * 100 : 0;
            const barWidth = maxDps > 0 ? (a.portion_dps / maxDps) * 100 : 0;
            const name = a.name.replace(/_/g, " ");

            return (
              <div key={i} className="group relative flex items-center h-7">
                {/* Background bar */}
                <div
                  className="absolute inset-y-0 left-0 rounded-r opacity-[0.08] group-hover:opacity-[0.14] transition-opacity"
                  style={{ width: `${barWidth}%`, backgroundColor: color }}
                />
                {/* Left edge accent */}
                <div
                  className="absolute left-0 top-1 bottom-1 w-[3px] rounded-full"
                  style={{ backgroundColor: color, opacity: 0.6 }}
                />
                {/* Content */}
                <span className="relative pl-3 text-[12px] text-gray-300 truncate flex-1 flex items-center gap-2">
                  {a.spell_id && icons.get(a.spell_id) ? (
                    <SpellIcon icon={icons.get(a.spell_id)!} />
                  ) : (
                    <span className="w-5 h-5 rounded-[3px] shrink-0 bg-surface-2" />
                  )}
                  {name}
                </span>
                <span className="relative text-[11px] font-mono tabular-nums text-gray-500 w-16 text-right shrink-0">
                  {Math.round(a.portion_dps).toLocaleString()}
                </span>
                <span className="relative text-[11px] font-mono tabular-nums text-gray-500 w-12 text-right shrink-0">
                  {pct.toFixed(1)}%
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
