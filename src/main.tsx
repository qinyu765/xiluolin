import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";

function App() {
  return (
    <main className="app-shell">
      <section className="hero">
        <p className="eyebrow">AI 语音输入助手</p>
        <h1>XiLuoLin</h1>
        <p className="summary">
          面向办公、写作和编程场景，后续将把短语音整理成可直接使用的文本。
        </p>
      </section>

      <section className="status-panel" aria-label="当前开发状态">
        <h2>T002 项目骨架</h2>
        <ul>
          <li>Tauri v2 桌面应用骨架</li>
          <li>React + TypeScript + Vite 前端</li>
          <li>pnpm 依赖管理</li>
        </ul>
      </section>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
