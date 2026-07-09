import { useEffect } from "react";
import { Outlet } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "./components/Sidebar";
import "./styles/theme.css";
import "./App.css";

function App() {
  useEffect(() => {
    invoke<string | null>("get_preference", { key: "theme" }).then((val) => {
      if (val === "light") {
        document.body.classList.add("light-theme");
      }
    }).catch(() => {});
  }, []);

  return (
    <div className="app-shell">
      <Sidebar />
      <main className="app-main">
        <Outlet />
      </main>
    </div>
  );
}

export default App;
