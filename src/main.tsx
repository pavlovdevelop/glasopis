import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/global.css";

const root = document.getElementById("root");
if (!root) throw new Error("Липсва елемент #root");

// Плаващият прозорец е прозрачен — фонът се задава само за него.
if (window.location.hash.startsWith("#/overlay")) {
  document.body.classList.add("body--overlay");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
