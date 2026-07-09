import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Routes, Route } from "react-router-dom";
import App from "./App";
import { Dashboard } from "./views/Dashboard";
import { Minecraft } from "./views/Minecraft";
import { Diagnostics } from "./views/Diagnostics";
import { Recommendations } from "./views/Recommendations";
import { Sessions } from "./views/Sessions";
import { Settings } from "./views/Settings";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <Routes>
        <Route element={<App />}>
          <Route index element={<Dashboard />} />
          <Route path="minecraft" element={<Minecraft />} />
          <Route path="diagnostics" element={<Diagnostics />} />
          <Route path="recommendations" element={<Recommendations />} />
          <Route path="sessions" element={<Sessions />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </HashRouter>
  </React.StrictMode>
);
