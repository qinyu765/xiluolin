import ReactDOM from "react-dom/client";

import { App } from "@/app/App";
import { FallbackResultWindow } from "@/components/FallbackResultWindow";
import "./styles.css";

const isFallbackWindow =
  new URLSearchParams(window.location.search).get("window") === "fallback";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  isFallbackWindow ? <FallbackResultWindow /> : <App />,
);
