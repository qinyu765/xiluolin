import ReactDOM from "react-dom/client";

import { App } from "@/app/App";
import { enforceLightTheme } from "@/lib/theme";
import { IndicatorWindow } from "@/features/capture/IndicatorWindow";
import "./styles.css";

const isIndicatorWindow =
  new URLSearchParams(window.location.search).get("window") === "indicator";

void enforceLightTheme().catch((error) => {
  console.error("固定浅色主题失败", error);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  isIndicatorWindow ? <IndicatorWindow /> : <App />,
);
