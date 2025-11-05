import React, { useMemo, useState, useCallback } from "react";
import {
  UiLayout,
  UiNode,
  ColumnNode,
  TextNode,
  ButtonNode,
  SpacerNode,
  ScrollTextNode,
  LogoNode,
  InputMoneyNode,
  // ⬇️ nuevos tipos
  InputTextNode,
  InputPasswordNode,
} from "./types";
import { invoke } from "@tauri-apps/api/core";

// helpers existentes
function alignToFlex(align?: string): React.CSSProperties["alignItems"] {
  switch (align) {
    case "start":
      return "flex-start";
    case "end":
      return "flex-end";
    case "center":
      return "center";
    case "stretch":
      return "stretch";
    default:
      return undefined;
  }
}

// ===== NUEVO: contexto de render =====
type RenderCtx = {
  inputs: Record<string, string>;
  setInput: (id: string, v: string) => void;
  flags: Record<string, boolean>;
  setFlag: (k: string, v: boolean) => void;
};

// ===== NUEVO: visibilidad condicional por flag =====
function isVisible(node: UiNode, flags: Record<string, boolean>): boolean {
  const flag = (node as any).visible_when_flag as string | undefined;
  if (!flag) return true;
  const expected =
    (node as any).visible_when === undefined ? true : (node as any).visible_when;
  const current = !!flags[flag];
  return current === expected;
}

// ======= Render de nodos (ahora recibe ctx) =======
function renderNode(
  node: UiNode,
  key: React.Key | undefined,
  ctx: RenderCtx
): React.ReactNode {
  // visibilidad por flag
  if (!isVisible(node, ctx.flags)) return null;

  switch (node.type) {
    case "column": {
      const col = node as ColumnNode;
      const style: React.CSSProperties = {
        display: "flex",
        flexDirection: "column",
        backgroundColor: col.background || undefined,
        padding: col.padding ? col.padding : undefined,
        rowGap: col.gap ? col.gap : undefined,
        alignItems: alignToFlex(col.align),
        flex: 1,
        width: "100%",
        minHeight: 0,
      };
      return (
        <div key={key} style={style}>
          {col.children?.map((child, idx) => renderNode(child, idx, ctx))}
        </div>
      );
    }

    case "text": {
      const t = node as TextNode;
      const style: React.CSSProperties = {
        fontWeight: t.bold ? "600" : "400",
        fontSize: t.size ? t.size : 14,
        color: t.color || "#111827",
        textAlign: t.align as any,
        alignSelf: t.align ? alignToFlex(t.align) : undefined,
        whiteSpace: "pre-wrap",
      };
      return (
        <div key={key} style={style}>
          {t.text}
        </div>
      );
    }

    case "button": {
      const b = node as ButtonNode;
      const style: React.CSSProperties = {
        alignSelf: b.align ? alignToFlex(b.align) : undefined,
        backgroundColor: b.tint || "#2962FF",
        color: b.text_color || "#FFFFFF",
        borderRadius: 8,
        padding: "12px 16px",
        fontSize: 16,
        fontWeight: 500,
        opacity: b.enabled === false ? 0.4 : 1,
        border: "none",
        cursor: b.enabled === false ? "not-allowed" : "pointer",
        display: "flex",
        flexDirection: "row",
        justifyContent: "center",
      };

      // dentro de case "button" -> handleClick
const handleClick = async () => {
  if (b.enabled === false) return;

  const eventId = b.on_click || b.id || "";
  if (!eventId) return;

  // 1) Navegación local (NO mandar a Rust)
if (eventId.startsWith("nav_to:")) {
  const to = eventId.substring("nav_to:".length);
  const comingFromLogin = ctx.flags["screen_login"] === true && to !== "login";

  if (comingFromLogin) {
    // toma "user_pass" o, si no existe, el respaldo global; quita espacios
    const pass = (ctx.inputs["user_pass"] ?? ctx.inputs["__pwd_val"] ?? "").trim();
    const SECRET = "password123#";
    if (pass !== SECRET) {
      ctx.setFlag("login_error", true);
      return; // NO navega si es incorrecta
    } else {
      ctx.setFlag("login_error", false);
    }
  }

  // conmutar “pantallas” por flags
  if (to === "start") {
    ctx.setFlag("screen_login", false);
    ctx.setFlag("screen_start", true);
  } else if (to === "login") {
    ctx.setFlag("screen_login", true);
    ctx.setFlag("screen_start", false);
    ctx.setFlag("login_error", false);
  }
  return; // no invoques on_ui_event con nav_to
}

  // 2) Eventos “de negocio” sí van a Rust
  try {
    await invoke("on_ui_event", { eventId });
  } catch {
    /* opcional: ignora si aún no implementaste el handler */
  }
};

      return (
        <button key={key} style={style} onClick={handleClick}>
          {b.text}
        </button>
      );
    }

    case "spacer": {
      const sp = node as SpacerNode;
      return (
        <div key={key} style={{ height: sp.height ?? 12, width: "100%" }} />
      );
    }

    case "scroll": {
      const sc = node as ScrollTextNode;
      return (
        <div
          key={key}
          style={{
            flex: sc.weight ?? 1,
            overflowY: "auto",
            padding: sc.padding ?? 0,
            border: "1px solid #ddd",
            borderRadius: 8,
            fontFamily: "monospace",
            fontSize: 12,
            whiteSpace: "pre-wrap",
            color: sc.color || undefined,
          }}
        >
          {sc.text}
        </div>
      );
    }

    case "logo": {
      const lg = node as LogoNode;
      const base64 = (window as any).__lastLayout?.__style_logo_base64 as
        | string
        | undefined;
      const meta = (window as any).__lastLayout?.__style_logo_meta || {};
      const w = lg.width ?? meta.width ?? 160;
      const h = lg.height ?? meta.height ?? 80;
      const wrap: React.CSSProperties = {
        display: "flex",
        justifyContent:
          lg.align === "center"
            ? "center"
            : lg.align === "end"
            ? "flex-end"
            : "flex-start",
      };
      return (
        <div key={key} style={wrap}>
          {base64 ? (
            <img
              src={`data:image/png;base64,${base64}`}
              width={w}
              height={h}
              style={{ objectFit: "contain" }}
            />
          ) : null}
        </div>
      );
    }

    case "input_money": {
      const m = node as InputMoneyNode;
      const style: React.CSSProperties = {
        alignSelf: m.align ? alignToFlex(m.align) : undefined,
        borderRadius: 8,
        padding: "10px 12px",
        fontSize: 16,
        border: "1px solid #ddd",
        width: "100%",
        maxWidth: 360,
      };
      return (
        <input
          key={key}
          placeholder={m.hint || "Monto"}
          defaultValue={m.value || ""}
          style={style}
        />
      );
    }

    // ===== NUEVO: input_text =====
    case "input_text": {
      const t = node as InputTextNode;
      const style: React.CSSProperties = {
        alignSelf: t.align ? alignToFlex(t.align) : undefined,
        borderRadius: 8,
        padding: "10px 12px",
        fontSize: 16,
        border: "1px solid #ddd",
        width: "100%",
        maxWidth: 360,
      };
      const id = t.id || "input_text";
      const value = ctx.inputs[id] ?? t.value ?? "";
      return (
        <input
          key={key}
          type="text"
          placeholder={t.hint || ""}
          value={value}
          onChange={(e) => ctx.setInput(id, e.currentTarget.value)}
          style={style}
        />
      );
    }

    // ===== NUEVO: input_password =====
case "input_password": {
  const p = node as InputPasswordNode;
  const style: React.CSSProperties = {
    alignSelf: p.align ? alignToFlex(p.align) : undefined,
    borderRadius: 8,
    padding: "10px 12px",
    fontSize: 16,
    border: "1px solid #ddd",
    width: "100%",
    maxWidth: 360,
  };
  const id = p.id || "input_password";
  const value = ctx.inputs[id] ?? p.value ?? "";
  return (
    <input
      key={key}
      type="password"
      placeholder={p.hint || ""}
      value={value}
      onChange={(e) => {
        const v = e.currentTarget.value;
        ctx.setInput(id, v);          // por id (idealmente "user_pass")
        ctx.setInput("__pwd_val", v); // respaldo global
      }}
      style={style}
    />
  );
}

    default:
      return (
        <div key={key} style={{ color: "red", fontSize: 12 }}>
          [Nodo no soportado: {node.type}]
        </div>
      );
  }
}

export default function Renderer({ layout }: { layout: UiLayout | null }) {
  // ===== NUEVO: estado de inputs y flags (pantallas + error) =====
  const [inputs, setInputs] = useState<Record<string, string>>({});
  const [flags, setFlags] = useState<Record<string, boolean>>({
    screen_login: true,   // login visible al inicio
    screen_start: false,  // start oculto al inicio
    login_error: false,   // sin error
  });

  const setInput = useCallback((id: string, v: string) => {
    setInputs((prev) => ({ ...prev, [id]: v }));
  }, []);
  const setFlag = useCallback((k: string, v: boolean) => {
    setFlags((prev) => ({ ...prev, [k]: v }));
  }, []);

  const ctx: RenderCtx = useMemo(
    () => ({ inputs, setInput, flags, setFlag }),
    [inputs, flags, setInput, setFlag]
  );

  if (!layout) {
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          backgroundColor: "#000",
          color: "#fff",
          height: "100vh",
          alignItems: "center",
          justifyContent: "center",
          fontFamily: "sans-serif",
        }}
      >
        Cargando UI…
      </div>
    );
  }

  const bg = layout.background || "#FFFFFF";
  return (
    <div
      style={{
        backgroundColor: bg,
        color: "#111827",
        width: "100vw",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {renderNode(layout.root, "root", ctx)}
    </div>
  );
}
