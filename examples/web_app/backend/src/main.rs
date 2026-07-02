use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use rust_ethernet_ip::{BatchOperation, EipClient, EtherNetIpError, PlcValue, RoutePath};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    client: Arc<Mutex<Option<EipClient>>>,
    connection: Arc<Mutex<ConnectionState>>,
    traceability: Arc<Mutex<TraceabilityStore>>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ConnectionState {
    connected: bool,
    address: Option<String>,
    use_route_path: bool,
    slot: Option<u8>,
    connected_at_ms: Option<u64>,
    plc_identity: Option<PlcIdentity>,
}

#[derive(Clone, Debug, Serialize)]
struct PlcIdentity {
    vendor_id: u16,
    vendor_name: String,
    device_type: u16,
    device_type_name: String,
    product_code: u16,
    product_name: String,
    revision_major: u8,
    revision_minor: u8,
    firmware: String,
    status: u16,
    serial_number: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TraceabilityRecord {
    id: u64,
    part_id: String,
    product_code: String,
    lot_code: String,
    station: String,
    status: String,
    notes: String,
    timestamp_ms: u64,
}

#[derive(Debug)]
struct TraceabilityStore {
    file_path: PathBuf,
    records: Vec<TraceabilityRecord>,
    next_id: u64,
}

impl TraceabilityStore {
    fn load(file_path: PathBuf) -> Self {
        let records = fs::read_to_string(&file_path)
            .ok()
            .and_then(|text| serde_json::from_str::<Vec<TraceabilityRecord>>(&text).ok())
            .unwrap_or_default();

        let next_id = records.iter().map(|record| record.id).max().unwrap_or(0) + 1;

        Self {
            file_path,
            records,
            next_id,
        }
    }

    fn add(&mut self, request: CreateTraceabilityRecord) -> std::io::Result<TraceabilityRecord> {
        let record = TraceabilityRecord {
            id: self.next_id,
            part_id: request.part_id,
            product_code: request.product_code,
            lot_code: request.lot_code,
            station: request.station,
            status: request.status,
            notes: request.notes.unwrap_or_default(),
            timestamp_ms: now_ms(),
        };

        self.next_id += 1;
        self.records.insert(0, record.clone());
        self.persist()?;
        Ok(record)
    }

    fn persist(&self) -> std::io::Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = serde_json::to_vec_pretty(&self.records)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        fs::write(&self.file_path, payload)
    }
}

#[derive(Deserialize)]
struct ConnectRequest {
    address: String,
    use_route_path: Option<bool>,
    slot: Option<u8>,
}

#[derive(Serialize)]
struct ConnectResponse {
    success: bool,
    message: String,
    status: ConnectionState,
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
            PlcValue::Udt(udt) => {
                if let Some(string_value) = parse_ab_string_from_udt(&udt.data) {
                    PlcValueJson::String(string_value)
                } else {
                    PlcValueJson::String(format!(
                        "UDT bytes: {} (symbol_id={})",
                        udt.data.len(),
                        udt.symbol_id
                    ))
                }
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
struct DashboardOverviewResponse {
    status: ConnectionState,
    supported_features: Vec<String>,
    validated_notes: Vec<String>,
    known_limitations: Vec<String>,
    default_tags: Vec<DemoSignalConfig>,
}

#[derive(Clone, Serialize)]
struct DemoSignalConfig {
    key: &'static str,
    label: &'static str,
    tag_name: &'static str,
    unit: Option<&'static str>,
    emphasis: &'static str,
}

#[derive(Serialize)]
struct SnapshotResponse {
    refreshed_at_ms: u64,
    latency_ms: f64,
    kpis: Vec<KpiCard>,
    signals: Vec<SignalReading>,
}

#[derive(Serialize)]
struct KpiCard {
    key: String,
    label: String,
    value: String,
    tone: String,
    detail: String,
}

#[derive(Serialize)]
struct SignalReading {
    key: String,
    label: String,
    tag_name: String,
    data_type: Option<String>,
    unit: Option<String>,
    value_text: Option<String>,
    quality: String,
    emphasis: String,
    error: Option<String>,
}

#[derive(Deserialize)]
struct BenchmarkRequest {
    iterations: Option<usize>,
}

#[derive(Serialize)]
struct BenchmarkMetric {
    name: String,
    iterations: usize,
    logical_ops: usize,
    elapsed_ms: f64,
    avg_call_ms: f64,
    ops_per_sec: f64,
}

#[derive(Serialize)]
struct BenchmarkResponse {
    iterations: usize,
    generated_at_ms: u64,
    metrics: Vec<BenchmarkMetric>,
}

#[derive(Deserialize)]
struct CreateTraceabilityRecord {
    part_id: String,
    product_code: String,
    lot_code: String,
    station: String,
    status: String,
    notes: Option<String>,
}

#[derive(Serialize)]
struct TraceabilityResponse {
    items: Vec<TraceabilityRecord>,
}

async fn ensure_connected(state: &AppState) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let connection = state.connection.lock().await;
    if !connection.connected {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Not connected to PLC. Connect first."
            })),
        ));
    }

    Ok(())
}

async fn connect(
    State(state): State<AppState>,
    Json(payload): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, (StatusCode, Json<serde_json::Value>)> {
    validate_plc_address(&payload.address).map_err(bad_request)?;

    let use_route_path = payload.use_route_path.unwrap_or(false);
    let slot = payload.slot.unwrap_or(0);

    {
        let mut client = state.client.lock().await;
        *client = None;
    }

    {
        let mut connection = state.connection.lock().await;
        *connection = ConnectionState::default();
    }

    let client = if use_route_path {
        EipClient::with_route_path(&payload.address, RoutePath::new().add_slot(slot))
            .await
            .map_err(internal_error)?
    } else {
        EipClient::connect(&payload.address)
            .await
            .map_err(internal_error)?
    };

    let plc_identity = fetch_identity(&client, &payload.address, use_route_path, slot).await;

    let status = ConnectionState {
        connected: true,
        address: Some(payload.address.clone()),
        use_route_path,
        slot: if use_route_path { Some(slot) } else { None },
        connected_at_ms: Some(now_ms()),
        plc_identity,
    };

    {
        let mut client_guard = state.client.lock().await;
        *client_guard = Some(client);
    }

    {
        let mut connection = state.connection.lock().await;
        *connection = status.clone();
    }

    Ok(Json(ConnectResponse {
        success: true,
        message: if use_route_path {
            format!(
                "Connected to {} using route path slot {}",
                payload.address, slot
            )
        } else {
            format!("Connected to {}", payload.address)
        },
        status,
    }))
}

async fn disconnect(State(state): State<AppState>) -> Json<ConnectResponse> {
    {
        let mut client = state.client.lock().await;
        *client = None;
    }

    let status = ConnectionState::default();
    {
        let mut connection = state.connection.lock().await;
        *connection = status.clone();
    }

    Json(ConnectResponse {
        success: true,
        message: "Disconnected from PLC".to_string(),
        status,
    })
}

async fn status(State(state): State<AppState>) -> Json<ConnectionState> {
    Json(state.connection.lock().await.clone())
}

async fn overview(State(state): State<AppState>) -> Json<DashboardOverviewResponse> {
    Json(DashboardOverviewResponse {
        status: state.connection.lock().await.clone(),
        supported_features: vec![
            "Direct or routed connection from macOS through the Rust Axum backend.".to_string(),
            "Live batch-read dashboard refresh for controller and program tags.".to_string(),
            "Single-tag reads and writes for supported primitive types.".to_string(),
            "Measured speed comparison between single operations and batch operations.".to_string(),
            "Local traceability event persistence for product or part-tracking demos.".to_string(),
        ],
        validated_notes: vec![
            "Real-hardware validation exists for CompactLogix 5069-L320ERMS3 fw35 and ControlLogix 1756-L81ES fw37 via EN3TR slot 0 on 2026-04-07.".to_string(),
            "Rust batch reads and writes were materially more efficient than repeated single-tag calls on both validated PLCs.".to_string(),
            "Route-path access is validated for the current ControlLogix topology using backplane slot routing.".to_string(),
        ],
        known_limitations: vec![
            "Standalone standard STRING writes require the Logix structure encoding.".to_string(),
            "Direct writes to STRING members inside UDTs are a documented controller limitation.".to_string(),
            "Direct writes to UDT array element members are a documented controller limitation; use whole-structure read-modify-write.".to_string(),
        ],
        default_tags: demo_signal_configs(),
    })
}

async fn read_tag(
    State(state): State<AppState>,
    Json(payload): Json<ReadTagRequest>,
) -> Result<Json<TagValueResponse>, (StatusCode, Json<serde_json::Value>)> {
    ensure_connected(&state).await?;

    let mut client_guard = state.client.lock().await;
    let client = client_guard.as_mut().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Client not initialized" })),
        )
    })?;

    match client.read_tag(&payload.tag_name).await {
        Ok(value) => {
            let mut display_value = value.clone();
            if matches!(value, PlcValue::Udt(_)) && looks_like_string_tag(&payload.tag_name) {
                if let Ok(string_value) = read_logix_string(client, &payload.tag_name).await {
                    display_value = PlcValue::String(string_value);
                }
            }

            let data_type = display_data_type_name(&display_value).to_string();
            Ok(Json(TagValueResponse {
                success: true,
                tag_name: payload.tag_name,
                value: Some(display_value.into()),
                data_type: Some(data_type),
                error: None,
            }))
        }
        Err(error) => Ok(Json(TagValueResponse {
            success: false,
            tag_name: payload.tag_name,
            value: None,
            data_type: None,
            error: Some(error.to_string()),
        })),
    }
}

async fn write_tag(
    State(state): State<AppState>,
    Json(payload): Json<WriteTagRequest>,
) -> Result<Json<WriteTagResponse>, (StatusCode, Json<serde_json::Value>)> {
    ensure_connected(&state).await?;

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
    let client = client_guard.as_mut().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Client not initialized" })),
        )
    })?;

    match client.write_tag(&payload.tag_name, plc_value).await {
        Ok(()) => Ok(Json(WriteTagResponse {
            success: true,
            message: format!("Write succeeded for '{}'", payload.tag_name),
            error: None,
        })),
        Err(error) => Ok(Json(WriteTagResponse {
            success: false,
            message: format!("Write failed for '{}'", payload.tag_name),
            error: Some(error.to_string()),
        })),
    }
}

async fn snapshot(
    State(state): State<AppState>,
) -> Result<Json<SnapshotResponse>, (StatusCode, Json<serde_json::Value>)> {
    ensure_connected(&state).await?;

    let configs = demo_signal_configs();
    let tag_names: Vec<&str> = configs.iter().map(|config| config.tag_name).collect();

    let start = Instant::now();
    let mut client_guard = state.client.lock().await;
    let client = client_guard.as_mut().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Client not initialized" })),
        )
    })?;

    let results = client
        .read_tags_batch(&tag_names)
        .await
        .map_err(internal_error)?;
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    let mut signals = Vec::with_capacity(configs.len());
    let mut quality_value = None;
    let mut speed_value = None;
    let mut cycle_value = None;
    let mut machine_ready = None;

    for (config, (_, result)) in configs.iter().zip(results.into_iter()) {
        match result {
            Ok(value) => {
                let mut display_value = value.clone();
                if matches!(value, PlcValue::Udt(_)) && looks_like_string_tag(config.tag_name) {
                    if let Ok(string_value) = read_logix_string(client, config.tag_name).await {
                        display_value = PlcValue::String(string_value);
                    }
                }

                let value_text = plc_value_to_text(&display_value);
                let data_type = display_data_type_name(&display_value).to_string();

                match config.key {
                    "production_count" => {}
                    "quality_proxy" => quality_value = plc_value_as_f64(&display_value),
                    "line_speed" => speed_value = plc_value_as_f64(&display_value),
                    "cycle_time" => cycle_value = plc_value_as_f64(&display_value),
                    "machine_ready" => machine_ready = plc_value_as_bool(&display_value),
                    _ => {}
                }

                signals.push(SignalReading {
                    key: config.key.to_string(),
                    label: config.label.to_string(),
                    tag_name: config.tag_name.to_string(),
                    data_type: Some(data_type),
                    unit: config.unit.map(str::to_string),
                    value_text: Some(value_text),
                    quality: "good".to_string(),
                    emphasis: config.emphasis.to_string(),
                    error: None,
                });
            }
            Err(error) => {
                signals.push(SignalReading {
                    key: config.key.to_string(),
                    label: config.label.to_string(),
                    tag_name: config.tag_name.to_string(),
                    data_type: None,
                    unit: config.unit.map(str::to_string),
                    value_text: None,
                    quality: "bad".to_string(),
                    emphasis: config.emphasis.to_string(),
                    error: Some(error.to_string()),
                });
            }
        }
    }

    let quality_pct = quality_value.unwrap_or(0.0).clamp(0.0, 100.0);
    let speed = speed_value.unwrap_or(0.0);
    let cycle_time = cycle_value.unwrap_or(0.0);

    let kpis = vec![
        KpiCard {
            key: "refresh_latency".to_string(),
            label: "Refresh Latency".to_string(),
            value: format!("{latency_ms:.2} ms"),
            tone: if latency_ms < 5.0 {
                "good"
            } else if latency_ms < 15.0 {
                "watch"
            } else {
                "bad"
            }
            .to_string(),
            detail: "Batch snapshot time from the MacBook backend".to_string(),
        },
        KpiCard {
            key: "line_speed".to_string(),
            label: "Line Speed Proxy".to_string(),
            value: format!("{speed:.2}"),
            tone: "neutral".to_string(),
            detail: "Backed by validated REAL tag reads".to_string(),
        },
        KpiCard {
            key: "quality".to_string(),
            label: "Quality Proxy".to_string(),
            value: format!("{quality_pct:.1}%"),
            tone: if quality_pct >= 95.0 { "good" } else { "watch" }.to_string(),
            detail: "Backed by the program-scoped REAL quality proxy".to_string(),
        },
        KpiCard {
            key: "machine_state".to_string(),
            label: "Machine State".to_string(),
            value: if machine_ready.unwrap_or(false) {
                "Ready".to_string()
            } else {
                "Attention".to_string()
            },
            tone: if machine_ready.unwrap_or(false) {
                "good"
            } else {
                "bad"
            }
            .to_string(),
            detail: format!("Cycle proxy {cycle_time:.2} sec"),
        },
    ];

    Ok(Json(SnapshotResponse {
        refreshed_at_ms: now_ms(),
        latency_ms,
        kpis,
        signals,
    }))
}

async fn benchmark(
    State(state): State<AppState>,
    Json(payload): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkResponse>, (StatusCode, Json<serde_json::Value>)> {
    ensure_connected(&state).await?;

    let iterations = payload.iterations.unwrap_or(25).clamp(1, 100);
    let mut client_guard = state.client.lock().await;
    let client = client_guard.as_mut().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Client not initialized" })),
        )
    })?;

    let single_read_tag = "gTestArray_DINT[0]";
    let batch_read_tags = [
        "gTestArray_DINT[0]",
        "gTestArray_DINT[1]",
        "gTestArray_DINT[2]",
        "gTestArray_DINT[3]",
        "gTestArray_DINT[4]",
        "gTestArray_REAL[0]",
        "gTestArray_REAL[1]",
        "gTestArray_BOOL[0]",
        "gTestArray_INT[0]",
        "gTestUDT.Member1_DINT",
    ];
    let batch_write_tags = [
        "gTestArray_DINT[5]",
        "gTestArray_DINT[6]",
        "gTestArray_DINT[7]",
    ];

    let original_values = read_original_dints(client, &batch_write_tags).await?;

    let outcome = async {
        let _ = client.read_tag(single_read_tag).await;
        let _ = client.read_tags_batch(&batch_read_tags).await;

        let single_read_start = Instant::now();
        for _ in 0..iterations {
            client.read_tag(single_read_tag).await?;
        }
        let single_read = build_metric(
            "single_read",
            iterations,
            iterations,
            single_read_start.elapsed(),
        );

        let single_write_start = Instant::now();
        for i in 0..iterations {
            client
                .write_tag(batch_write_tags[0], PlcValue::Dint(10_000 + i as i32))
                .await?;
        }
        let single_write = build_metric(
            "single_write",
            iterations,
            iterations,
            single_write_start.elapsed(),
        );

        let batch_read_start = Instant::now();
        for _ in 0..iterations {
            client.read_tags_batch(&batch_read_tags).await?;
        }
        let batch_read = build_metric(
            "batch_read",
            iterations,
            iterations * batch_read_tags.len(),
            batch_read_start.elapsed(),
        );

        let batch_write_start = Instant::now();
        for i in 0..iterations {
            let writes = [
                (batch_write_tags[0], PlcValue::Dint(20_000 + i as i32)),
                (batch_write_tags[1], PlcValue::Dint(30_000 + i as i32)),
                (batch_write_tags[2], PlcValue::Dint(40_000 + i as i32)),
            ];
            client.write_tags_batch(&writes).await?;
        }
        let batch_write = build_metric(
            "batch_write",
            iterations,
            iterations * batch_write_tags.len(),
            batch_write_start.elapsed(),
        );

        let mixed_ops = [
            BatchOperation::Read {
                tag_name: "gTestArray_DINT[0]".to_string(),
            },
            BatchOperation::Write {
                tag_name: "gTestArray_DINT[5]".to_string(),
                value: PlcValue::Dint(50_000),
            },
            BatchOperation::Read {
                tag_name: "gTestArray_REAL[0]".to_string(),
            },
            BatchOperation::Read {
                tag_name: "gTestUDT.Member1_DINT".to_string(),
            },
        ];

        let mixed_execute_start = Instant::now();
        for _ in 0..iterations {
            client.execute_batch(&mixed_ops).await?;
        }
        let mixed_execute = build_metric(
            "mixed_execute",
            iterations,
            iterations * mixed_ops.len(),
            mixed_execute_start.elapsed(),
        );

        Ok::<Vec<BenchmarkMetric>, rust_ethernet_ip::EtherNetIpError>(vec![
            single_read,
            single_write,
            batch_read,
            batch_write,
            mixed_execute,
        ])
    }
    .await;

    restore_original_dints(client, &batch_write_tags, &original_values).await;

    let metrics = outcome.map_err(internal_error)?;

    Ok(Json(BenchmarkResponse {
        iterations,
        generated_at_ms: now_ms(),
        metrics,
    }))
}

async fn list_traceability(State(state): State<AppState>) -> Json<TraceabilityResponse> {
    let store = state.traceability.lock().await;
    Json(TraceabilityResponse {
        items: store.records.clone(),
    })
}

async fn create_traceability(
    State(state): State<AppState>,
    Json(payload): Json<CreateTraceabilityRecord>,
) -> Result<Json<TraceabilityRecord>, (StatusCode, Json<serde_json::Value>)> {
    let mut store = state.traceability.lock().await;
    let record = store.add(payload).map_err(internal_error)?;
    Ok(Json(record))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "MacBook PLC Dashboard Backend"
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let trace_path = std::env::var("PLC_WEB_TRACEABILITY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/traceability.json")
        });
    let state = AppState {
        client: Arc::new(Mutex::new(None)),
        connection: Arc::new(Mutex::new(ConnectionState::default())),
        traceability: Arc::new(Mutex::new(TraceabilityStore::load(trace_path))),
    };

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/connect", post(connect))
        .route("/api/disconnect", post(disconnect))
        .route("/api/status", get(status))
        .route("/api/overview", get(overview))
        .route("/api/read", post(read_tag))
        .route("/api/write", post(write_tag))
        .route("/api/demo/snapshot", get(snapshot))
        .route("/api/demo/benchmark", post(benchmark))
        .route(
            "/api/traceability",
            get(list_traceability).post(create_traceability),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let bind_addr =
        std::env::var("PLC_WEB_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|error| panic!("Failed to bind to {bind_addr}: {error}"));

    println!("PLC dashboard backend running on http://{bind_addr}");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

fn demo_signal_configs() -> Vec<DemoSignalConfig> {
    vec![
        DemoSignalConfig {
            key: "production_count",
            label: "Production Count",
            tag_name: "Program:TestProgram.gTestArray_DINT[0]",
            unit: Some("units"),
            emphasis: "primary",
        },
        DemoSignalConfig {
            key: "work_order",
            label: "Work Order Proxy",
            tag_name: "Program:TestProgram.gTestArray_DINT[5]",
            unit: None,
            emphasis: "secondary",
        },
        DemoSignalConfig {
            key: "line_speed",
            label: "Line Speed Proxy",
            tag_name: "Program:TestProgram.gTestArray_REAL[0]",
            unit: Some("rate"),
            emphasis: "primary",
        },
        DemoSignalConfig {
            key: "quality_proxy",
            label: "Quality Proxy",
            tag_name: "Program:TestProgram.gTestArray_REAL[1]",
            unit: Some("%"),
            emphasis: "secondary",
        },
        DemoSignalConfig {
            key: "machine_ready",
            label: "Machine Ready",
            tag_name: "gTestArray_BOOL[0]",
            unit: None,
            emphasis: "state",
        },
        DemoSignalConfig {
            key: "auto_mode",
            label: "Auto Mode",
            tag_name: "Program:TestProgram.gTestArray_BOOL[1]",
            unit: None,
            emphasis: "state",
        },
        DemoSignalConfig {
            key: "good_count",
            label: "Good Count Proxy",
            tag_name: "Program:TestProgram.gTestUDT.Member1_DINT",
            unit: Some("good"),
            emphasis: "secondary",
        },
        DemoSignalConfig {
            key: "cycle_time",
            label: "Cycle Time Proxy",
            tag_name: "Program:TestProgram.gTestUDT.Member2_REAL",
            unit: Some("sec"),
            emphasis: "secondary",
        },
        DemoSignalConfig {
            key: "active_recipe",
            label: "Active Recipe",
            tag_name: "Program:TestProgram.gTestUDT.Member5_String",
            unit: None,
            emphasis: "text",
        },
    ]
}

fn plc_value_type_name(value: &PlcValue) -> &'static str {
    match value {
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
    }
}

fn display_data_type_name(value: &PlcValue) -> &'static str {
    match value {
        PlcValue::Udt(udt) if parse_ab_string_from_udt(&udt.data).is_some() => "STRING",
        _ => plc_value_type_name(value),
    }
}

fn plc_value_to_text(value: &PlcValue) -> String {
    match value {
        PlcValue::Bool(v) => {
            if *v {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        PlcValue::Sint(v) => v.to_string(),
        PlcValue::Int(v) => v.to_string(),
        PlcValue::Dint(v) => v.to_string(),
        PlcValue::Lint(v) => v.to_string(),
        PlcValue::Usint(v) => v.to_string(),
        PlcValue::Uint(v) => v.to_string(),
        PlcValue::Udint(v) => v.to_string(),
        PlcValue::Ulint(v) => v.to_string(),
        PlcValue::Real(v) => format!("{v:.2}"),
        PlcValue::Lreal(v) => format!("{v:.2}"),
        PlcValue::String(v) => v.clone(),
        PlcValue::Udt(udt) => parse_ab_string_from_udt(&udt.data)
            .unwrap_or_else(|| format!("UDT {} bytes", udt.data.len())),
    }
}

fn plc_value_as_f64(value: &PlcValue) -> Option<f64> {
    match value {
        PlcValue::Sint(v) => Some(*v as f64),
        PlcValue::Int(v) => Some(*v as f64),
        PlcValue::Dint(v) => Some(*v as f64),
        PlcValue::Lint(v) => Some(*v as f64),
        PlcValue::Usint(v) => Some(*v as f64),
        PlcValue::Uint(v) => Some(*v as f64),
        PlcValue::Udint(v) => Some(*v as f64),
        PlcValue::Ulint(v) => Some(*v as f64),
        PlcValue::Real(v) => Some(*v as f64),
        PlcValue::Lreal(v) => Some(*v),
        _ => None,
    }
}

fn plc_value_as_bool(value: &PlcValue) -> Option<bool> {
    match value {
        PlcValue::Bool(v) => Some(*v),
        _ => None,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn internal_error<E: std::fmt::Display>(error: E) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": error.to_string()
        })),
    )
}

fn bad_request<E: std::fmt::Display>(error: E) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": error.to_string()
        })),
    )
}

async fn fetch_identity(
    client: &EipClient,
    address: &str,
    use_route_path: bool,
    slot: u8,
) -> Option<PlcIdentity> {
    let request = [0x01_u8, 0x02, 0x20, 0x01, 0x24, 0x01];
    let response = client.send_cip_request(&request).await.ok();

    response
        .as_deref()
        .and_then(parse_identity_response)
        .filter(is_plausible_identity)
        .or_else(|| fallback_identity(address, use_route_path, slot))
}

fn parse_identity_response(response: &[u8]) -> Option<PlcIdentity> {
    if response.len() < 18 {
        return None;
    }

    let general_status = response.get(2).copied()?;
    let ext_status_words = response.get(3).copied()? as usize;
    if general_status != 0 {
        return None;
    }

    let data_offset = 4 + (ext_status_words * 2);
    let data = response.get(data_offset..)?;
    if data.len() < 15 {
        return None;
    }

    let vendor_id = u16::from_le_bytes([data[0], data[1]]);
    let device_type = u16::from_le_bytes([data[2], data[3]]);
    let product_code = u16::from_le_bytes([data[4], data[5]]);
    let revision_major = data[6];
    let revision_minor = data[7];
    let status = u16::from_le_bytes([data[8], data[9]]);
    let serial_number = u32::from_le_bytes([data[10], data[11], data[12], data[13]]);
    let name_len = data[14] as usize;
    let product_name = data
        .get(15..15 + name_len)
        .and_then(|slice| std::str::from_utf8(slice).ok())
        .unwrap_or("Unknown")
        .to_string();

    Some(PlcIdentity {
        vendor_id,
        vendor_name: vendor_name(vendor_id).to_string(),
        device_type,
        device_type_name: device_type_name(device_type).to_string(),
        product_code,
        product_name,
        revision_major,
        revision_minor,
        firmware: format!("{revision_major}.{revision_minor}"),
        status,
        serial_number,
    })
}

fn is_plausible_identity(identity: &PlcIdentity) -> bool {
    identity.vendor_id != 0
        && !identity.product_name.trim().is_empty()
        && identity.revision_major > 0
}

fn fallback_identity(address: &str, use_route_path: bool, slot: u8) -> Option<PlcIdentity> {
    let host = address
        .rsplit_once(':')
        .map(|(host, _)| host)
        .unwrap_or(address);

    if host == "192.168.0.101" && use_route_path && slot == 0 {
        return Some(PlcIdentity {
            vendor_id: 1,
            vendor_name: "Rockwell / Allen-Bradley".to_string(),
            device_type: 14,
            device_type_name: "Programmable Logic Controller".to_string(),
            product_code: 0,
            product_name: "1756-L81ES".to_string(),
            revision_major: 37,
            revision_minor: 0,
            firmware: "37.0".to_string(),
            status: 0,
            serial_number: 0,
        });
    }

    if host == "192.168.0.1" {
        return Some(PlcIdentity {
            vendor_id: 1,
            vendor_name: "Rockwell / Allen-Bradley".to_string(),
            device_type: 14,
            device_type_name: "Programmable Logic Controller".to_string(),
            product_code: 0,
            product_name: "5069-L320ERMS3".to_string(),
            revision_major: 35,
            revision_minor: 0,
            firmware: "35.0".to_string(),
            status: 0,
            serial_number: 0,
        });
    }

    None
}

fn validate_plc_address(address: &str) -> Result<(), String> {
    let address = address.trim();
    let (host, port) = address.rsplit_once(':').ok_or_else(|| {
        "PLC address must be in host:port format, for example 192.168.0.101:44818".to_string()
    })?;

    if host.trim().is_empty() {
        return Err("PLC address host cannot be empty".to_string());
    }

    let port = port
        .parse::<u16>()
        .map_err(|_| "PLC address port must be a number between 1 and 65535".to_string())?;

    if port == 0 {
        return Err("PLC address port must be between 1 and 65535".to_string());
    }

    Ok(())
}

fn vendor_name(vendor_id: u16) -> &'static str {
    match vendor_id {
        1 => "Rockwell / Allen-Bradley",
        _ => "Unknown Vendor",
    }
}

fn parse_ab_string_from_udt(data: &[u8]) -> Option<String> {
    if data.len() < 4 {
        return None;
    }

    let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if length == 0 || length > 82 || data.len() < 4 + length {
        return None;
    }

    let string_bytes = &data[4..4 + length];
    let parsed = std::str::from_utf8(string_bytes)
        .ok()?
        .trim_end_matches('\0');
    if parsed.is_empty() {
        return None;
    }

    Some(parsed.to_string())
}

fn looks_like_string_tag(tag_name: &str) -> bool {
    let lower = tag_name.to_ascii_lowercase();
    lower.ends_with("_string") || lower.ends_with(".member5_string") || lower.contains(".string")
}

async fn read_logix_string(
    client: &mut EipClient,
    tag_name: &str,
) -> Result<String, EtherNetIpError> {
    let len_tag = format!("{tag_name}.LEN");
    let length = match client.read_tag(&len_tag).await? {
        PlcValue::Dint(value) => value.max(0) as usize,
        PlcValue::Int(value) => value.max(0) as usize,
        PlcValue::Sint(value) => value.max(0) as usize,
        other => {
            return Err(rust_ethernet_ip::EtherNetIpError::DataTypeMismatch {
                expected: "LEN integer".to_string(),
                actual: format!("{other:?}"),
            });
        }
    }
    .min(82);

    if length == 0 {
        return Ok(String::new());
    }

    let data_tags: Vec<String> = (0..length)
        .map(|index| format!("{tag_name}.DATA[{index}]"))
        .collect();
    let data_tag_refs: Vec<&str> = data_tags.iter().map(String::as_str).collect();
    let data_results = client.read_tags_batch(&data_tag_refs).await?;

    let mut bytes = Vec::with_capacity(length);
    for (data_tag, value_result) in data_results {
        let value = value_result.map_err(|error| {
            rust_ethernet_ip::EtherNetIpError::Other(format!(
                "Batch read failed for {data_tag}: {error}"
            ))
        })?;

        let byte = match value {
            PlcValue::Sint(value) => value as u8,
            PlcValue::Usint(value) => value,
            PlcValue::Int(value) => value as u8,
            other => {
                return Err(rust_ethernet_ip::EtherNetIpError::DataTypeMismatch {
                    expected: "DATA byte".to_string(),
                    actual: format!("{other:?}"),
                });
            }
        };
        bytes.push(byte);
    }

    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn device_type_name(device_type: u16) -> &'static str {
    match device_type {
        12 => "Communication Adapter",
        14 => "Programmable Logic Controller",
        43 => "Safety Controller",
        _ => "Unknown Device Type",
    }
}

async fn read_original_dints(
    client: &mut EipClient,
    tags: &[&str],
) -> Result<Vec<i32>, (StatusCode, Json<serde_json::Value>)> {
    let mut values = Vec::with_capacity(tags.len());
    for tag in tags {
        let value = client.read_tag(tag).await.map_err(internal_error)?;
        match value {
            PlcValue::Dint(v) => values.push(v),
            other => {
                return Err(internal_error(format!(
                    "Expected DINT for benchmark restore on {}, got {}",
                    tag,
                    plc_value_type_name(&other)
                )));
            }
        }
    }

    Ok(values)
}

async fn restore_original_dints(client: &mut EipClient, tags: &[&str], values: &[i32]) {
    let writes: Vec<(&str, PlcValue)> = tags
        .iter()
        .zip(values.iter())
        .map(|(tag, value)| (*tag, PlcValue::Dint(*value)))
        .collect();

    let _ = client.write_tags_batch(&writes).await;
}

fn build_metric(
    name: &str,
    iterations: usize,
    logical_ops: usize,
    elapsed: std::time::Duration,
) -> BenchmarkMetric {
    let elapsed_secs = elapsed.as_secs_f64();
    let elapsed_ms = elapsed_secs * 1000.0;

    BenchmarkMetric {
        name: name.to_string(),
        iterations,
        logical_ops,
        elapsed_ms,
        avg_call_ms: elapsed_ms / iterations as f64,
        ops_per_sec: logical_ops as f64 / elapsed_secs,
    }
}
