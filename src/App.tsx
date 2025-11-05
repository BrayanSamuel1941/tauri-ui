import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Renderer from "./ui/Renderer";
import { UiLayout } from "./ui/types";

export default function App() {
  const [layout, setLayout] = useState<UiLayout | null>(null);

  // 1. pedir el layout inicial al backend Rust (AppState.current_layout)
  useEffect(() => {
    (async () => {
      const layoutStr = await invoke<string>("get_ui_layout");
      const initial = JSON.parse(layoutStr);
      (window as any).__lastLayout = initial; // ← NUEVO
      setLayout(initial);
    })();
  }, []);

  useEffect(() => {
    const unlistenPromise = listen<string>("layout_update", (event) => {
      const parsed = JSON.parse(event.payload);
      (window as any).__lastLayout = parsed; // ← NUEVO
      setLayout(parsed);
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // 3. render dinámico
  return <Renderer layout={layout} />;
}
