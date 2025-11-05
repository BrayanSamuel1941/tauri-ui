use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    // layout actual que la ventana debe estar mostrando
    pub current_layout: Arc<Mutex<String>>,
    // último layout válido conocido, por si llega uno roto
    pub last_good_layout: Arc<Mutex<String>>,

    // ===== estado adicional que en Android vive en Lua / variables globales =====
    pub reading: Arc<Mutex<bool>>,
    pub endpoint_snapshot: Arc<Mutex<String>>,       // equivalente a endpointSnapshot
    pub ack_endpoint_snapshot: Arc<Mutex<String>>,   // equivalente a ackEndpointSnapshot
    pub ack_init_snapshot: Arc<Mutex<bool>>,         // equivalente a ackInitSnapshot
    pub last_hb_millis: Arc<Mutex<i64>>,             // heartbeat
}

impl AppState {
    pub fn new(initial_layout: &str) -> Self {
        Self {
            current_layout: Arc::new(Mutex::new(initial_layout.to_string())),
            last_good_layout: Arc::new(Mutex::new(initial_layout.to_string())),
            reading: Arc::new(Mutex::new(false)),
            endpoint_snapshot: Arc::new(Mutex::new(String::new())),
            ack_endpoint_snapshot: Arc::new(Mutex::new(String::new())),
            ack_init_snapshot: Arc::new(Mutex::new(false)),
            last_hb_millis: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_layout(&self) -> String {
        self.current_layout.lock().unwrap().clone()
    }

    pub fn apply_layout_safely(&self, new_layout: &str) -> bool {
        // Validación básica: ¿es JSON válido?
        if serde_json::from_str::<serde_json::Value>(new_layout).is_err() {
            return false;
        }
        // si es válido, promovemos a current y lastGood
        *self.current_layout.lock().unwrap() = new_layout.to_string();
        *self.last_good_layout.lock().unwrap() = new_layout.to_string();
        true
    }

    pub fn restore_last_good(&self) {
        let last = self.last_good_layout.lock().unwrap().clone();
        *self.current_layout.lock().unwrap() = last;
    }

    pub fn set_reading(&self, r: bool) {
        *self.reading.lock().unwrap() = r;
    }
    pub fn get_reading(&self) -> bool {
        *self.reading.lock().unwrap()
    }

    pub fn set_status(&self, zmq: &str, ack: &str, ack_init: bool, last_hb: i64) {
        *self.endpoint_snapshot.lock().unwrap() = zmq.to_string();
        *self.ack_endpoint_snapshot.lock().unwrap() = ack.to_string();
        *self.ack_init_snapshot.lock().unwrap() = ack_init;
        *self.last_hb_millis.lock().unwrap() = last_hb;
    }
}
