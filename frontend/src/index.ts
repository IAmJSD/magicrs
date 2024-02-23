// Load the bridge before all else.
import "./bridge/implementation";

// Load the main CSS.
import "./index.css";

// @ts-expect-error: Load font awesome.
import("@fortawesome/fontawesome-free/css/all.css");

import { createRoot } from "react-dom/client";
import { createElement } from "react";
import App from "./components/App";

createRoot(document.getElementById("app")).render(createElement(App));
