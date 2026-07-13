import { NavLink } from "react-router-dom";
import { Separator } from "@/components/ui/separator";

const NAV_ITEMS = [
  { to: "/", icon: "\u25A6", label: "Dashboard" },
  { to: "/library", icon: "\u2637", label: "Library" },
  { to: "/minecraft", icon: "\u25A3", label: "Minecraft" },
  { to: "/league", icon: "\u2694", label: "League" },
  { to: "/runescape", icon: "\u2726", label: "RuneScape" },
  { to: "/poe", icon: "\u25C7", label: "Path of Exile" },
  { to: "/tarkov", icon: "\u2316", label: "Tarkov" },
  { to: "/diagnostics", icon: "\u2699", label: "Diagnostics" },
  { to: "/recommendations", icon: "\u2691", label: "Recommendations" },
  { to: "/sessions", icon: "\u25F7", label: "Sessions" },
  { to: "/settings", icon: "\u2630", label: "Settings" },
];

export function Sidebar() {
  return (
    <aside className="w-56 bg-sidebar border-r border-sidebar-border flex flex-col shrink-0">
      <div className="flex items-center gap-2.5 px-5 py-4">
        <span className="text-primary text-xl font-bold">{"\u25C8"}</span>
        <span className="text-sidebar-foreground font-bold text-base tracking-tight">GamePilot</span>
      </div>

      <Separator />

      <nav className="flex-1 flex flex-col gap-0.5 px-3 py-3">
        {NAV_ITEMS.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `flex items-center gap-3 px-4 py-2.5 text-sm rounded-lg transition-colors ${
                isActive
                  ? "bg-primary/15 text-primary border-l-2 border-primary font-medium"
                  : "text-sidebar-foreground hover:bg-sidebar-accent border-l-2 border-transparent"
              }`
            }
            end={item.to === "/"}
          >
            <span className="text-base w-5 text-center">{item.icon}</span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>

      <Separator />

      <div className="px-5 py-3">
        <span className="text-xs text-muted-foreground">v0.1.0</span>
      </div>
    </aside>
  );
}
