import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Routes, Route } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import App from "@/App";
import { Dashboard } from "@/views/Dashboard";
import { Minecraft } from "@/views/Minecraft";
import { LeagueOfLegends } from "@/views/LeagueOfLegends";
import { RuneScape } from "@/views/RuneScape";
import { PathOfExile } from "@/views/PathOfExile";
import { Tarkov } from "@/views/Tarkov";
import { Diagnostics } from "@/views/Diagnostics";
import { Recommendations } from "@/views/Recommendations";
import { GameLibrary } from "@/views/GameLibrary";
import { Sessions } from "@/views/Sessions";
import { Settings } from "@/views/Settings";
import "@/index.css";

document.documentElement.classList.add("dark");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TooltipProvider>
      <HashRouter>
        <Routes>
          <Route element={<App />}>
            <Route index element={<Dashboard />} />
            <Route path="library" element={<GameLibrary />} />
            <Route path="minecraft" element={<Minecraft />} />
            <Route path="league" element={<LeagueOfLegends />} />
            <Route path="runescape" element={<RuneScape />} />
            <Route path="poe" element={<PathOfExile />} />
            <Route path="tarkov" element={<Tarkov />} />
            <Route path="diagnostics" element={<Diagnostics />} />
            <Route path="recommendations" element={<Recommendations />} />
            <Route path="sessions" element={<Sessions />} />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
      </HashRouter>
      <Toaster position="bottom-right" />
    </TooltipProvider>
  </React.StrictMode>
);
