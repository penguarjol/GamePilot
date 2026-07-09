import { NavLink } from "react-router-dom";

const NAV_ITEMS = [
  { to: "/", icon: "\u25A6", label: "Dashboard" },
  { to: "/minecraft", icon: "\u25A3", label: "Minecraft" },
  { to: "/diagnostics", icon: "\u2699", label: "Diagnostics" },
  { to: "/recommendations", icon: "\u2691", label: "Recommendations" },
  { to: "/sessions", icon: "\u25F7", label: "Sessions" },
  { to: "/settings", icon: "\u2630", label: "Settings" },
];

export function Sidebar() {
  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <span className="sidebar-brand-icon">{"\u25C8"}</span>
        <span className="sidebar-brand-text">GamePilot</span>
      </div>
      <nav className="sidebar-nav">
        {NAV_ITEMS.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              `sidebar-link${isActive ? " active" : ""}`
            }
            end={item.to === "/"}
          >
            <span className="sidebar-link-icon">{item.icon}</span>
            <span className="sidebar-link-label">{item.label}</span>
          </NavLink>
        ))}
      </nav>
      <div className="sidebar-footer">
        <span className="sidebar-version">v0.1.0</span>
      </div>
    </aside>
  );
}
