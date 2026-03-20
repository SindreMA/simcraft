"use client";

import { usePathname } from "next/navigation";

const tabs = [
  { href: "/", label: "Quick Sim" },
  { href: "/top-gear", label: "Top Gear" },
];

export default function NavTabs() {
  const pathname = usePathname();

  return (
    <nav className="flex items-center gap-1 bg-surface-2 rounded-lg p-0.5 border border-border">
      {tabs.map((tab) => {
        const isActive =
          tab.href === "/"
            ? pathname === "/" || pathname.startsWith("/sim")
            : pathname.startsWith(tab.href);
        return (
          <a
            key={tab.href}
            href={tab.href}
            className={`px-4 py-1.5 text-[13px] font-medium rounded-md transition-all ${
              isActive
                ? "bg-gold text-black"
                : "text-muted hover:text-white"
            }`}
          >
            {tab.label}
          </a>
        );
      })}
    </nav>
  );
}
