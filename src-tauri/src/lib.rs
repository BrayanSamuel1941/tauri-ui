#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;
mod broker; // 👈 añade el módulo del listener

use state::AppState;
use broker::start_zmq_listener; // 👈 importa la función

use tauri::{AppHandle, Wry, Emitter};

use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;

// =====================
// 1. Generadores de layout (igual a tu versión)
// =====================

fn build_base_layout() -> String {
    let json = r###"
{
  "background": "#FFFFFF",
  "root": {
    "type": "column",
    "background": "#FFFFFF",
    "padding": 24,
    "gap": 12,
    "children": [
      { "type": "text", "id": "txt_title", "text": "Inicio", "align": "center", "size": 22, "bold": true, "color": "#111827" },
      { "type": "spacer", "id": "sp_start_1", "height": 12 },
      { "type": "button", "id": "btn_proceed", "text": "Proceder al cobro", "on_click": "go_payment", "align": "center", "tint": "#2962FF", "text_color": "#FFFFFF", "enabled": true }
    ]
  },
  "customer_display": { "text": "Bienvenido", "size": 24, "align": "center", "use_logo": true, "bg_color": "#000000" }
}
"###;
    json.to_string()
}

fn build_payment_layout(reading: bool, last_result: &str) -> String {
    let (rc_text, rc_click, rc_tint) = if reading {
        ("Cancelar lectura", "btn_cancel_msr", "#DC2626")
    } else {
        ("Leer banda magnética", "btn_read_msr", "#D97706")
    };
    let escaped_result = last_result.replace('"', "\\\"");

    let raw = format!(r###"{{
  "background": "#FFFFFF",
  "root": {{
    "type": "column",
    "background": "#FFFFFF",
    "padding": 24,
    "gap": 8,
    "children": [
      {{ "type": "button", "id": "btn_back", "text": "Regresar", "on_click": "nav_back", "align": "start", "tint": "#111827", "text_color": "#FFFFFF", "icon": "back" }},
      {{ "type": "spacer", "id": "sp_pay_2", "height": 8 }},
      {{ "type": "button", "id": "btn_print", "text": "Imprimir mensaje", "on_click": "print_from_button", "align": "center", "tint": "#2563EB", "text_color": "#FFFFFF", "icon": "print" }},
      {{ "type": "button", "id": "btn_read_cancel", "text": "{rc_text}", "on_click": "{rc_click}", "align": "center", "tint": "{rc_tint}", "text_color": "#FFFFFF", "icon": "info" }},
      {{ "type": "spacer", "id": "sp_pay_1", "height": 12 }},
      {{ "type": "text", "id": "txt_result_title", "text": "Resultado de lectura", "align": "start", "size": 14, "bold": true }},
      {{ "type": "text", "id": "msr_name", "text": "Nombre: {{msr.cardholderName}}", "align": "start", "size": 12 }},
      {{ "type": "text", "id": "msr_t1", "text": "Track 1: {{msr.track1}}", "align": "start", "size": 12 }},
      {{ "type": "text", "id": "msr_t2", "text": "Track 2: {{msr.track2}}", "align": "start", "size": 12 }},
      {{ "type": "text", "id": "msr_t3", "text": "Track 3: {{msr.track3}}", "align": "start", "size": 12 }},
      {{ "type": "scroll", "id": "msr_result", "weight": 1, "padding": 12, "text": "{escaped_result}" }}
    ]
  }},
  "customer_display": {{ "text": "Bienvenido", "size": 24, "align": "center", "use_logo": true, "bg_color": "#000000" }}
}}"###,
        rc_text = rc_text, rc_click = rc_click, rc_tint = rc_tint, escaped_result = escaped_result,
    );
    raw.replace('\'', "\"")
}

fn emit_layout_update(app: &AppHandle<Wry>, json: &str) {
    let _ = app.emit("layout_update", json.to_string());
}

#[tauri::command]
fn get_ui_layout(state: tauri::State<AppState>) -> Result<String, String> {
    Ok(state.get_layout())
}

#[tauri::command]
fn on_ui_event(
    event_id: String,
    state: tauri::State<AppState>,
    app: tauri::AppHandle
) -> Result<Option<String>, String> {
    // ⬇️ Ignorar nav_to:* (navegación local en el front)
    if event_id.starts_with("nav_to:") {
        println!("[UI] nav_to recibido (ignorado en backend): {}", event_id);
        return Ok(None);
    }

    let mut new_layout: Option<String> = None;

    match event_id.as_str() {
        "go_payment" | "btn_proceed" => {
            let reading = state.get_reading();
            let last_result = "— sin lectura aún —";
            new_layout = Some(build_payment_layout(reading, last_result));
        }
        "nav_back" => new_layout = Some(build_base_layout()),
        "btn_read_msr" => { state.set_reading(true); new_layout = Some(build_payment_layout(true, "Leyendo banda magnética.")); }
        "btn_cancel_msr" => { state.set_reading(false); new_layout = Some(build_payment_layout(false, "Lectura cancelada por el usuario")); }
        "print_from_button" => { let reading = state.get_reading(); new_layout = Some(build_payment_layout(reading, "Enviando a impresora.")); }
        other => eprintln!("Evento no manejado: {}", other),
    }

    if let Some(candidate) = new_layout {
        if state.apply_layout_safely(&candidate) {
            emit_layout_update(&app, &candidate);
            return Ok(Some(candidate));
        } else {
            state.restore_last_good();
            let fallback = state.get_layout();
            emit_layout_update(&app, &fallback);
            return Ok(Some(fallback));
        }
    }
    Ok(None)
}


// Opcional: heartbeat simulado (igual que tu versión)
async fn heartbeat_ticker(_app: AppHandle<Wry>, state: AppState) {
    loop {
        let now = Utc::now().timestamp_millis();
        state.set_status("tcp://34.70.157.148:5557", "http://34.70.157.148:8080/ack", true, now);
        sleep(Duration::from_secs(15)).await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_layout = build_base_layout();
    let app_state = AppState::new(&initial_layout);

    tauri::Builder::default()
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![ get_ui_layout, on_ui_event ])
        .setup(move |app| {
            // 🔸 Arranca el listener ZMQ (aquí es donde “escucha y aplica”)
            {
                let state_for_broker = app_state.clone();
                let handle_for_broker = app.handle().clone();
                start_zmq_listener(handle_for_broker, state_for_broker);
            }

            // 🔸 (Opcional) heartbeat
            {
                let state_for_hb = app_state.clone();
                let handle_for_hb = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    heartbeat_ticker(handle_for_hb, state_for_hb).await;
                });
            }

            // ❌ Quitamos el polling HTTP que pegaba a otra IP/endpoint no “pull”.
            // Si luego expones un SSE/long-poll en el gateway, lo activamos aquí.

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
