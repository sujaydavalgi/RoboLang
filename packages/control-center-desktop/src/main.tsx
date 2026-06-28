import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ControlCenterPanel } from "@davalgi-spanda/web/ControlCenterPanel";
import "@davalgi-spanda/web/index.css";

const apiBase =
  import.meta.env.VITE_CONTROL_CENTER_URL ?? "http://127.0.0.1:8080";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <div className="app">
      <header>
        <div>
          <h1>Spanda Control Center</h1>
          <p className="subtitle">Desktop shell — enterprise operations dashboard</p>
        </div>
      </header>
      <ControlCenterPanel apiBase={apiBase} />
    </div>
  </StrictMode>,
);
