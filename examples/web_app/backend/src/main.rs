use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use rust_ethernet_ip::{EipClient, PlcValue};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

// Application state to hold the PLC client
#[derive(Clone)]
struct AppState {
    client: Arc<Mutex<Option<EipClient>>>,
    connected_address: Arc<Mutex<Option<String>>>,
}

// Request/Response types
#[derive(Deserialize)]
struct ConnectRequest {
    address: String,
}

#[derive(Serialize)]
struct ConnectResponse {
    success: bool,
    message: String,
}

#[derive(Deserialize)]
struct ReadTagRequest {
    tag_name: String,
}

#[derive(Serialize)]
struct TagValueResponse {
    success: bool,
    tag_name: String,
    value: Option<PlcValueJson>,
    data_type: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "value")]
enum PlcValueJson {
    #[serde(rename = "BOOL")]
    Bool(bool),
    #[serde(rename = "SINT")]
    Sint(i8),
    #[serde(rename = "INT")]
    Int(i16),
    #[serde(rename = "DINT")]
    Dint(i32),
    #[serde(rename = "LINT")]
    Lint(i64),
    #[serde(rename = "USINT")]
    Usint(u8),
    #[serde(rename = "UINT")]
    Uint(u16),
    #[serde(rename = "UDINT")]
    Udint(u32),
    #[serde(rename = "ULINT")]
    Ulint(u64),
    #[serde(rename = "REAL")]
    Real(f32),
    #[serde(rename = "LREAL")]
    Lreal(f64),
    #[serde(rename = "STRING")]
    String(String),
}

impl From<PlcValue> for PlcValueJson {
    fn from(value: PlcValue) -> Self {
        match value {
            PlcValue::Bool(v) => PlcValueJson::Bool(v),
            PlcValue::Sint(v) => PlcValueJson::Sint(v),
            PlcValue::Int(v) => PlcValueJson::Int(v),
            PlcValue::Dint(v) => PlcValueJson::Dint(v),
            PlcValue::Lint(v) => PlcValueJson::Lint(v),
            PlcValue::Usint(v) => PlcValueJson::Usint(v),
            PlcValue::Uint(v) => PlcValueJson::Uint(v),
            PlcValue::Udint(v) => PlcValueJson::Udint(v),
            PlcValue::Ulint(v) => PlcValueJson::Ulint(v),
            PlcValue::Real(v) => PlcValueJson::Real(v),
            PlcValue::Lreal(v) => PlcValueJson::Lreal(v),
            PlcValue::String(v) => PlcValueJson::String(v),
            // For UDT, convert to string representation
            PlcValue::Udt(_) => {
                PlcValueJson::String(format!("{:?}", value))
            }
        }
    }
}

#[derive(Deserialize)]
struct WriteTagRequest {
    tag_name: String,
    value: PlcValueJson,
}

#[derive(Serialize)]
struct WriteTagResponse {
    success: bool,
    message: String,
    error: Option<String>,
}

#[derive(Serialize)]
struct StatusResponse {
    connected: bool,
    address: Option<String>,
}

// Helper function to check if client is connected
async fn ensure_connected(state: &AppState) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let client = state.client.lock().await;
    if client.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Not connected to PLC. Please connect first."
            })),
        ));
    }
    Ok(())
}

// API Handlers

/// Connect to a PLC
async fn connect(
    State(state): State<AppState>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Disconnect existing connection if any
    {
        let mut client = state.client.lock().await;
        *client = None;
        let mut addr = state.connected_address.lock().await;
        *addr = None;
    }

    // Connect to new PLC
    match EipClient::connect(&payload.address).await {
        Ok(client) => {
            let mut state_client = state.client.lock().await;
            *state_client = Some(client);
            let mut addr = state.connected_address.lock().await;
            *addr = Some(payload.address.clone());

            Ok(Json(ConnectResponse {
                success: true,
                message: format!("Successfully connected to {}", payload.address),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to connect: {}", e)
            })),
        )),
    }
}

/// Disconnect from PLC
async fn disconnect(State(state): State<AppState>) -> Json<ConnectResponse> {
    let mut client = state.client.lock().await;
    *client = None;
    let mut addr = state.connected_address.lock().await;
    *addr = None;

    Json(ConnectResponse {
        success: true,
        message: "Disconnected from PLC".to_string(),
    })
}

/// Get connection status
async fn status(State(state): State<AppState>) -> Json<StatusResponse> {
    let addr = state.connected_address.lock().await;
    Json(StatusResponse {
        connected: addr.is_some(),
        address: addr.clone(),
    })
}

/// Read a tag from the PLC
async fn read_tag(
    State(state): State<AppState>,
    Json(payload): Json<ReadTagRequest>,
) -> Result<Json<TagValueResponse>, (StatusCode, Json<serde_json::Value>)> {
    ensure_connected(&state).await?;

    let mut client_guard = state.client.lock().await;
    if let Some(ref mut client) = *client_guard {
        match client.read_tag(&payload.tag_name).await {
            Ok(value) => {
                let value_json: PlcValueJson = value.clone().into();
                let data_type = match &value {
                    PlcValue::Bool(_) => "BOOL",
                    PlcValue::Sint(_) => "SINT",
                    PlcValue::Int(_) => "INT",
                    PlcValue::Dint(_) => "DINT",
                    PlcValue::Lint(_) => "LINT",
                    PlcValue::Usint(_) => "USINT",
                    PlcValue::Uint(_) => "UINT",
                    PlcValue::Udint(_) => "UDINT",
                    PlcValue::Ulint(_) => "ULINT",
                    PlcValue::Real(_) => "REAL",
                    PlcValue::Lreal(_) => "LREAL",
                    PlcValue::String(_) => "STRING",
                    PlcValue::Udt(_) => "UDT",
                };

                Ok(Json(TagValueResponse {
                    success: true,
                    tag_name: payload.tag_name,
                    value: Some(value_json),
                    data_type: Some(data_type.to_string()),
                    error: None,
                }))
            }
            Err(e) => Ok(Json(TagValueResponse {
                success: false,
                tag_name: payload.tag_name,
                value: None,
                data_type: None,
                error: Some(format!("{}", e)),
            })),
        }
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Client not initialized"
            })),
        ))
    }
}

/// Write a tag to the PLC
async fn write_tag(
    State(state): State<AppState>,
    Json(payload): Json<WriteTagRequest>,
) -> Result<Json<WriteTagResponse>, (StatusCode, Json<serde_json::Value>)> {
    ensure_connected(&state).await?;

    // Convert JSON value to PlcValue
    let plc_value = match payload.value {
        PlcValueJson::Bool(v) => PlcValue::Bool(v),
        PlcValueJson::Sint(v) => PlcValue::Sint(v),
        PlcValueJson::Int(v) => PlcValue::Int(v),
        PlcValueJson::Dint(v) => PlcValue::Dint(v),
        PlcValueJson::Lint(v) => PlcValue::Lint(v),
        PlcValueJson::Usint(v) => PlcValue::Usint(v),
        PlcValueJson::Uint(v) => PlcValue::Uint(v),
        PlcValueJson::Udint(v) => PlcValue::Udint(v),
        PlcValueJson::Ulint(v) => PlcValue::Ulint(v),
        PlcValueJson::Real(v) => PlcValue::Real(v),
        PlcValueJson::Lreal(v) => PlcValue::Lreal(v),
        PlcValueJson::String(v) => PlcValue::String(v),
    };

    let mut client_guard = state.client.lock().await;
    if let Some(ref mut client) = *client_guard {
        match client.write_tag(&payload.tag_name, plc_value).await {
            Ok(_) => Ok(Json(WriteTagResponse {
                success: true,
                message: format!("Successfully wrote to tag '{}'", payload.tag_name),
                error: None,
            })),
            Err(e) => Ok(Json(WriteTagResponse {
                success: false,
                message: format!("Failed to write to tag '{}'", payload.tag_name),
                error: Some(format!("{}", e)),
            })),
        }
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Client not initialized"
            })),
        ))
    }
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "PLC Web Backend"
    }))
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create application state
    let state = AppState {
        client: Arc::new(Mutex::new(None)),
        connected_address: Arc::new(Mutex::new(None)),
    };

    // Build the router
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/connect", post(connect))
        .route("/api/disconnect", post(disconnect))
        .route("/api/status", get(status))
        .route("/api/read", post(read_tag))
        .route("/api/write", post(write_tag))
        .layer(CorsLayer::permissive()) // Allow all CORS for development
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");
    
    println!("🚀 PLC Web Backend server running on http://0.0.0.0:3000");
    println!("📡 API endpoints:");
    println!("   GET  /api/health     - Health check");
    println!("   POST /api/connect    - Connect to PLC");
    println!("   POST /api/disconnect - Disconnect from PLC");
    println!("   GET  /api/status     - Get connection status");
    println!("   POST /api/read       - Read a tag");
    println!("   POST /api/write      - Write a tag");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

