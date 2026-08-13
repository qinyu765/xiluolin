import ReactDOM from "react-dom/client";

import { App } from "@/app/App";
import { FallbackResultWindow } from "@/components/FallbackResultWindow";
import { IndicatorWindow } from "@/features/capture/IndicatorWindow";
import "./styles.css";

const isFallbackWindow =
  new URLSearchParams(window.location.search).get("window") === "fallback";
const isIndicatorWindow =
  new URLSearchParams(window.location.search).get("window") === "indicator";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  isIndicatorWindow ? (
    <IndicatorWindow />
  ) : isFallbackWindow ? (
    <FallbackResultWindow />
  ) : (
    <App />
  ),
);
