use tauri::{AppHandle, Emitter};
use crate::state::AppState;
use serde_json::{Value, json};

const BROKER_ENDPOINT: &str = "tcp://34.70.157.148:5557";

// ===== base64 (API moderna) =====
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

// --------------------- helpers parsing ---------------------
fn parse_json_str(s: &str) -> Option<Value> {
    serde_json::from_str::<Value>(s).ok()
}
fn parse_base64_json(s: &str) -> Option<Value> {
    let bytes = STANDARD.decode(s).ok()?;
    let txt = String::from_utf8(bytes).ok()?;
    parse_json_str(&txt)
}

fn looks_like_json(bytes: &[u8]) -> bool {
    let mut it = bytes.iter().skip_while(|c| c.is_ascii_whitespace());
    matches!(it.next(), Some(b'{') | Some(b'['))
}

// --------------------- style → layout ---------------------
fn style_to_layout(style: &Value, screen_id: Option<&str>) -> Option<Value> {
    let bg = style.get("background").and_then(|v| v.as_str()).unwrap_or("#129ADA");
    let screens = style.get("screens")?.as_array()?;
    let pick = if let Some(id) = screen_id {
        screens.iter().find(|s| s.get("id").and_then(|v| v.as_str()) == Some(id))
    } else {
        screens.first()
    }?.clone();

    let children = pick.get("children").cloned().unwrap_or(json!([]));

    let mut layout = json!({
      "background": bg,
      "root": {
        "type": "column",
        "background": bg,
        "padding": 24,
        "gap": 12,
        "children": children
      }
    });

    if let Some(cd) = style.get("customer_display") {
        layout["customer_display"] = cd.clone();
    }
    if let Some(lg) = style.get("logo").and_then(|o| o.get("base64")).and_then(|v| v.as_str()) {
        layout["__style_logo_base64"] = Value::String(lg.to_string());
        layout["__style_logo_meta"] = style.get("logo").cloned().unwrap_or(json!({}));
    }

    Some(layout)
}

// --------------------- extrae layout de un Value ---------------------
fn extract_layout_from_value(v: &Value) -> Option<Value> {
    // Debug útil
    if let Some(obj) = v.as_object() {
        let keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
        let cmd_name = v.get("cmd").and_then(|c| c.get("name")).and_then(|n| n.as_str()).unwrap_or("");
        eprintln!("[ZMQ][dbg] keys={:?} cmd={}", keys, cmd_name);
    }

    // 1) Layout directo
    if v.get("root").is_some() {
        return Some(v.clone());
    }

    // 2) Top-level content (layout o style)
    if let Some(content) = v.get("content") {
        if content.get("root").is_some() {
            return Some(content.clone());
        }
        if content.get("screens").is_some() {
            return style_to_layout(content, None);
        }
    }

    // 3) Top-level style suelto (frame sólo con screens)
    if v.get("screens").is_some() {
        return style_to_layout(v, None);
    }

    // 4) Envelope
    if let Some(env) = v.get("envelope") {
        if let Some(content) = env.get("content") {
            if content.get("root").is_some() {
                return Some(content.clone());
            }
            if content.get("screens").is_some() {
                return style_to_layout(content, None);
            }
        }
    }

    // 5) Comandos
    if let Some(cmd) = v.get("cmd") {
        let name = cmd.get("name").and_then(|n| n.as_str()).unwrap_or_default();
        let args = cmd.get("args").cloned().unwrap_or(json!({}));

        match name {
            // ui.apply → args.content (root o style) o top-level content
            "ui.apply" | "ui.update" => {
                if let Some(content) = args.get("content") {
                    if content.get("root").is_some() {
                        return Some(content.clone());
                    }
                    if content.get("screens").is_some() {
                        return style_to_layout(content, None);
                    }
                }
                if let Some(content) = v.get("content") {
                    if content.get("root").is_some() {
                        return Some(content.clone());
                    }
                    if content.get("screens").is_some() {
                        return style_to_layout(content, None);
                    }
                }
            }

            // ui.style.apply → args.style / args.style_json / args.data_base64 / files[]
            "ui.style.apply" | "ui.style.update" => {
                // a) objeto style directo
                if let Some(style) = args.get("style") {
                    return style_to_layout(style, None);
                }
                // b) style en string JSON
                if let Some(txt) = args.get("style_json").and_then(|v| v.as_str()) {
                    if let Some(style) = parse_json_str(txt) {
                        return style_to_layout(&style, None);
                    }
                }
                // c) style en base64 (TU CASO)
                if let Some(b64) = args.get("data_base64").and_then(|v| v.as_str()) {
                    if let Some(style) = parse_base64_json(b64) {
                        return style_to_layout(&style, None);
                    }
                }
                // d) style referenciado en files (content/text/base64)
                if let Some(style) = find_style_in_files(v) {
                    return style_to_layout(&style, None);
                }
            }
            _ => {}
        }
    }

    None
}

// Busca un objeto style en files[] (content/text/base64)
fn find_style_in_files(root: &Value) -> Option<Value> {
    let files = root.get("files")?.as_array()?;
    let mut candidates: Vec<&Value> = files.iter().collect();
    candidates.sort_by_key(|f| {
        let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let score = if name.to_ascii_lowercase().contains("style") { 0 } else { 1 };
        (score, name.len())
    });

    for f in candidates {
        if let Some(c) = f.get("content") {
            if c.get("screens").is_some() { return Some(c.clone()); }
            if let Some(txt) = c.as_str() {
                if let Some(v) = parse_json_str(txt) {
                    if v.get("screens").is_some() { return Some(v); }
                }
            }
        }
        if let Some(txt) = f.get("text").and_then(|v| v.as_str()) {
            if let Some(v) = parse_json_str(txt) {
                if v.get("screens").is_some() { return Some(v); }
            }
        }
        for key in &["content_b64", "base64", "bytes_b64"] {
            if let Some(b64) = f.get(*key).and_then(|v| v.as_str()) {
                if let Some(v) = parse_base64_json(b64) {
                    if v.get("screens").is_some() { return Some(v); }
                }
            }
        }
    }
    None
}

// --------------------- principal: prueba TODOS los frames ---------------------
fn emit_layout_update(app: &AppHandle, json: &str) {
    let _ = app.emit("layout_update", json.to_string());
}

pub fn start_zmq_listener(app: AppHandle, state: AppState) {
    std::thread::spawn(move || {
        let endpoint = std::env::var("TAURI_ZMQ_SUB").unwrap_or_else(|_| BROKER_ENDPOINT.to_string());

        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::SUB).expect("[ZMQ] no se pudo crear SUB");
        socket.set_subscribe(b"").expect("[ZMQ] no se pudo suscribir");
        socket.connect(&endpoint).expect("[ZMQ] no se pudo conectar al broker");

        eprintln!("[ZMQ] SUB conectado a {endpoint}");

        loop {
            let frames = match socket.recv_multipart(0) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("[ZMQ] error recibiendo: {e:?}");
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    continue;
                }
            };

            let mut applied = false;

            // 1) intenta con TODOS los frames JSON
            for bytes in frames.iter().filter(|b| looks_like_json(b)) {
                if let Ok(txt) = String::from_utf8(bytes.clone()) {
                    if let Some(v) = parse_json_str(&txt) {
                        if let Some(layout_v) = extract_layout_from_value(&v) {
                            if let Ok(layout_json) = serde_json::to_string(&layout_v) {
                                if state.apply_layout_safely(&layout_json) {
                                    emit_layout_update(&app, &layout_json);
                                    applied = true;
                                    break;
                                } else {
                                    state.restore_last_good();
                                    let fallback = state.get_layout();
                                    emit_layout_update(&app, &fallback);
                                    applied = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // 2) si no aplicó, aún puede que el *style* venga DENTRO del envelope como base64
            if !applied {
                if let Some(last_json_bytes) = frames.iter().rev().find(|b| looks_like_json(b)) {
                    if let Ok(txt) = String::from_utf8(last_json_bytes.clone()) {
                        if let Some(v) = parse_json_str(&txt) {
                            // intenta leer args.data_base64 si está
                            if let Some(cmd) = v.get("cmd") {
                                let args = cmd.get("args").cloned().unwrap_or(json!({}));
                                if let Some(b64) = args.get("data_base64").and_then(|x| x.as_str()) {
                                    if let Some(style) = parse_base64_json(b64) {
                                        if let Some(layout_v) = style_to_layout(&style, None) {
                                            if let Ok(layout_json) = serde_json::to_string(&layout_v) {
                                                if state.apply_layout_safely(&layout_json) {
                                                    emit_layout_update(&app, &layout_json);
                                                    applied = true;
                                                } else {
                                                    state.restore_last_good();
                                                    let fallback = state.get_layout();
                                                    emit_layout_update(&app, &fallback);
                                                    applied = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !applied {
                if let Some(bytes) = frames.iter().rev().find(|b| looks_like_json(b)) {
                    if let Ok(txt) = String::from_utf8(bytes.clone()) {
                        let preview = if txt.len() > 240 { &txt[..240] } else { &txt };
                        eprintln!("[ZMQ] ❌ sin layout (tras revisar todos los frames). preview: {}", preview);
                    } else {
                        eprintln!("[ZMQ] ❌ sin layout (frame JSON no UTF-8).");
                    }
                } else {
                    eprintln!("[ZMQ] ❌ sin layout (no hubo frames JSON).");
                }
            }
        }
    });
}
