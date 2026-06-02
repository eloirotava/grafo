use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const APP_NAME: &str = "RotavaFlow";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const SCHEMA_VERSION: &str = "1.0";
const SIMULATION_MODULES: [&str; 4] = ["malha", "nos", "tubos", "equipamentos"];

type SharedState = Arc<AppState>;

struct AppState {
    started_at: Instant,
    project_counter: AtomicU64,
    simulation_counter: AtomicU64,
    projects: Mutex<HashMap<String, StoredProject>>,
    simulations: Mutex<HashMap<String, SimulationReport>>,
    metrics: Mutex<ServerMetrics>,
}

impl AppState {
    fn new() -> SharedState {
        Arc::new(Self {
            started_at: Instant::now(),
            project_counter: AtomicU64::new(0),
            simulation_counter: AtomicU64::new(0),
            projects: Mutex::new(HashMap::new()),
            simulations: Mutex::new(HashMap::new()),
            metrics: Mutex::new(ServerMetrics::default()),
        })
    }

    fn next_project_id(&self) -> String {
        format!(
            "proj-{}",
            self.project_counter.fetch_add(1, Ordering::Relaxed) + 1
        )
    }

    fn next_simulation_id(&self) -> String {
        format!(
            "sim-{}",
            self.simulation_counter.fetch_add(1, Ordering::Relaxed) + 1
        )
    }
}

#[derive(Default, Serialize)]
struct ServerMetrics {
    projects_saved: u64,
    validations_run: u64,
    simulations_started: u64,
    simulations_completed: u64,
    simulations_failed: u64,
    last_simulation_ms: Option<u128>,
}

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
    version: &'static str,
    schema_version: &'static str,
    modules: &'static [&'static str],
}

#[derive(Serialize)]
struct MetricsResponse {
    service: &'static str,
    uptime_seconds: u64,
    metrics: ServerMetrics,
}

#[derive(Clone, Deserialize, Serialize)]
struct ProjectInput {
    name: String,
    #[serde(default = "default_schema_version")]
    schema_version: String,
    #[serde(default)]
    nodes: Vec<Node>,
    #[serde(default)]
    ducts: Vec<Duct>,
    #[serde(default)]
    equipments: Vec<Equipment>,
    #[serde(default)]
    boundary_conditions: Vec<BoundaryCondition>,
}

#[derive(Clone, Serialize)]
struct StoredProject {
    id: String,
    updated_at_epoch_ms: u128,
    #[serde(flatten)]
    project: ProjectInput,
}

#[derive(Clone, Deserialize, Serialize)]
struct Node {
    id: String,
    label: Option<String>,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
}

#[derive(Clone, Deserialize, Serialize)]
struct Duct {
    id: String,
    from: String,
    to: String,
    length_m: f64,
    diameter_m: f64,
    #[serde(default)]
    roughness_m: f64,
}

#[derive(Clone, Deserialize, Serialize)]
struct Equipment {
    id: String,
    kind: String,
    node_id: String,
    #[serde(default)]
    pressure_delta_pa: f64,
}

#[derive(Clone, Deserialize, Serialize)]
struct BoundaryCondition {
    node_id: String,
    kind: String,
    value: f64,
    unit: String,
}

#[derive(Serialize)]
struct ProjectListResponse {
    count: usize,
    projects: Vec<ProjectSummary>,
}

#[derive(Serialize)]
struct ProjectSummary {
    id: String,
    name: String,
    schema_version: String,
    nodes: usize,
    ducts: usize,
    equipments: usize,
    boundary_conditions: usize,
    updated_at_epoch_ms: u128,
}

#[derive(Serialize)]
struct ValidationReport {
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Deserialize)]
struct SimulationRequest {
    project_id: Option<String>,
    project: Option<ProjectInput>,
}

#[derive(Clone, Serialize)]
struct SimulationReport {
    id: String,
    status: SimulationStatus,
    project_name: String,
    started_at_epoch_ms: u128,
    finished_at_epoch_ms: Option<u128>,
    iterations: u32,
    residual: f64,
    runtime_ms: u128,
    solver_version: &'static str,
    validation: ValidationReport,
    status_history: Vec<StatusTransition>,
    results: SimulationResults,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum SimulationStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Serialize)]
struct StatusTransition {
    status: SimulationStatus,
    epoch_ms: u128,
    message: String,
}

#[derive(Clone, Serialize)]
struct SimulationResults {
    node_count: usize,
    duct_count: usize,
    equipment_count: usize,
    estimated_flow_m3_s: f64,
    total_length_m: f64,
    pressure_delta_pa: f64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// --- ARQUIVOS ESTÁTICOS EMBUTIDOS ---
// Pega tudo da pasta 'static' e coloca dentro do .exe (JS, CSS, SW.js)
#[derive(RustEmbed)]
#[folder = "static"]
struct Assets;

// --- TEMPLATES ---
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "canvas.html")]
struct CanvasTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "nodes.html")]
struct NodesTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "ducts.html")]
struct DuctsTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "equipments.html")]
struct EquipsTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "simulation.html")]
struct SimTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "reports.html")]
struct ReportsTemplate {
    title: String,
}

#[derive(Template)]
#[template(path = "help.html")]
struct HelpTemplate {
    title: String,
}

#[tokio::main]
async fn main() {
    let app_state = AppState::new();
    let app = build_router(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("SERVER: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn build_router(app_state: SharedState) -> Router {
    Router::new()
        // --- PÁGINAS ---
        .route("/", get(home))
        .route("/canvas", get(canvas))
        .route("/nodes", get(nodes))
        .route("/ducts", get(ducts))
        .route("/equipments", get(equips))
        .route("/simulation", get(sim))
        .route("/reports", get(reports))
        .route("/help", get(help))
        // --- SAÚDE E OBSERVABILIDADE ---
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        // --- API DE PROJETOS ---
        .route("/api/projects", get(list_projects).post(create_project))
        .route("/api/projects/:id", get(get_project).put(update_project))
        .route("/api/projects/:id/validate", post(validate_stored_project))
        // --- API DE SIMULAÇÕES ---
        .route("/api/simulations", post(start_simulation))
        .route("/api/simulations/:id", get(get_simulation))
        // --- SERVICE WORKER NA RAIZ (Obrigatório para PWA) ---
        .route("/sw.js", get(sw_handler))
        // --- ARQUIVOS ESTÁTICOS (JS/CSS) ---
        .route("/static/*file", get(static_handler))
        .with_state(app_state)
}

// --- HANDLERS DE SAÚDE E OBSERVABILIDADE ---
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: APP_NAME,
        status: "ok",
        version: APP_VERSION,
        schema_version: SCHEMA_VERSION,
        modules: &SIMULATION_MODULES,
    })
}

async fn metrics(State(state): State<SharedState>) -> Json<MetricsResponse> {
    let metrics = state.metrics.lock().unwrap();

    Json(MetricsResponse {
        service: APP_NAME,
        uptime_seconds: state.started_at.elapsed().as_secs(),
        metrics: ServerMetrics {
            projects_saved: metrics.projects_saved,
            validations_run: metrics.validations_run,
            simulations_started: metrics.simulations_started,
            simulations_completed: metrics.simulations_completed,
            simulations_failed: metrics.simulations_failed,
            last_simulation_ms: metrics.last_simulation_ms,
        },
    })
}

// --- HANDLERS DE PROJETOS ---
async fn list_projects(State(state): State<SharedState>) -> Json<ProjectListResponse> {
    let projects = state.projects.lock().unwrap();
    let mut summaries = projects
        .values()
        .map(ProjectSummary::from)
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| left.id.cmp(&right.id));

    Json(ProjectListResponse {
        count: summaries.len(),
        projects: summaries,
    })
}

async fn create_project(
    State(state): State<SharedState>,
    Json(project): Json<ProjectInput>,
) -> impl IntoResponse {
    let validation = validate_project(&project);
    if !validation.valid {
        return (StatusCode::BAD_REQUEST, Json(validation)).into_response();
    }

    let id = state.next_project_id();
    let stored = StoredProject {
        id: id.clone(),
        updated_at_epoch_ms: now_epoch_ms(),
        project,
    };

    state
        .projects
        .lock()
        .unwrap()
        .insert(id.clone(), stored.clone());
    state.metrics.lock().unwrap().projects_saved += 1;
    println!(
        "PROJECT saved id={id} name={} nodes={} ducts={}",
        stored.project.name,
        stored.project.nodes.len(),
        stored.project.ducts.len()
    );

    (StatusCode::CREATED, Json(stored)).into_response()
}

async fn get_project(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.projects.lock().unwrap().get(&id).cloned() {
        Some(project) => Json(project).into_response(),
        None => not_found(format!("Projeto '{id}' não encontrado")),
    }
}

async fn update_project(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(project): Json<ProjectInput>,
) -> impl IntoResponse {
    let validation = validate_project(&project);
    if !validation.valid {
        return (StatusCode::BAD_REQUEST, Json(validation)).into_response();
    }

    let stored = StoredProject {
        id: id.clone(),
        updated_at_epoch_ms: now_epoch_ms(),
        project,
    };

    state
        .projects
        .lock()
        .unwrap()
        .insert(id.clone(), stored.clone());
    state.metrics.lock().unwrap().projects_saved += 1;
    println!(
        "PROJECT updated id={id} name={} nodes={} ducts={}",
        stored.project.name,
        stored.project.nodes.len(),
        stored.project.ducts.len()
    );

    Json(stored).into_response()
}

async fn validate_stored_project(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let project = match state.projects.lock().unwrap().get(&id).cloned() {
        Some(project) => project,
        None => return not_found(format!("Projeto '{id}' não encontrado")),
    };

    let report = validate_project(&project.project);
    state.metrics.lock().unwrap().validations_run += 1;
    println!(
        "VALIDATION project_id={id} valid={} errors={} warnings={}",
        report.valid,
        report.errors.len(),
        report.warnings.len()
    );

    Json(report).into_response()
}

// --- HANDLERS DE SIMULAÇÕES ---
async fn start_simulation(
    State(state): State<SharedState>,
    Json(request): Json<SimulationRequest>,
) -> impl IntoResponse {
    let project = match resolve_simulation_project(&state, request) {
        Ok(project) => project,
        Err(response) => return response,
    };

    let simulation_id = state.next_simulation_id();
    state.metrics.lock().unwrap().simulations_started += 1;
    let report = run_simulation(&simulation_id, &project.project);

    {
        let mut metrics = state.metrics.lock().unwrap();
        metrics.last_simulation_ms = Some(report.runtime_ms);
        match report.status {
            SimulationStatus::Completed => metrics.simulations_completed += 1,
            SimulationStatus::Failed => metrics.simulations_failed += 1,
            SimulationStatus::Queued | SimulationStatus::Running => {}
        }
    }

    println!(
        "SIMULATION id={} status={:?} project={} iterations={} residual={:.6} runtime_ms={}",
        report.id,
        report.status.as_log_label(),
        report.project_name,
        report.iterations,
        report.residual,
        report.runtime_ms
    );

    let status = match report.status {
        SimulationStatus::Completed => StatusCode::CREATED,
        SimulationStatus::Failed => StatusCode::BAD_REQUEST,
        SimulationStatus::Queued | SimulationStatus::Running => StatusCode::ACCEPTED,
    };

    state
        .simulations
        .lock()
        .unwrap()
        .insert(simulation_id, report.clone());

    (status, Json(report)).into_response()
}

async fn get_simulation(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.simulations.lock().unwrap().get(&id).cloned() {
        Some(report) => Json(report).into_response(),
        None => not_found(format!("Simulação '{id}' não encontrada")),
    }
}

fn resolve_simulation_project(
    state: &SharedState,
    request: SimulationRequest,
) -> Result<StoredProject, axum::response::Response> {
    if let Some(project) = request.project {
        return Ok(StoredProject {
            id: "inline".to_string(),
            updated_at_epoch_ms: now_epoch_ms(),
            project,
        });
    }

    let Some(project_id) = request.project_id else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Informe 'project_id' ou um 'project' inline para simular".to_string(),
            }),
        )
            .into_response());
    };

    state
        .projects
        .lock()
        .unwrap()
        .get(&project_id)
        .cloned()
        .ok_or_else(|| not_found(format!("Projeto '{project_id}' não encontrado")))
}

fn run_simulation(simulation_id: &str, project: &ProjectInput) -> SimulationReport {
    let started = Instant::now();
    let started_at_epoch_ms = now_epoch_ms();
    let validation = validate_project(project);
    let total_length_m = project.ducts.iter().map(|duct| duct.length_m).sum::<f64>();
    let pressure_delta_pa = project
        .equipments
        .iter()
        .map(|equipment| equipment.pressure_delta_pa)
        .sum::<f64>();
    let avg_diameter_m = if project.ducts.is_empty() {
        0.0
    } else {
        project
            .ducts
            .iter()
            .map(|duct| duct.diameter_m)
            .sum::<f64>()
            / project.ducts.len() as f64
    };
    let hydraulic_capacity = avg_diameter_m.powi(2) * total_length_m.max(1.0).sqrt();
    let estimated_flow_m3_s = (pressure_delta_pa.abs().sqrt() * hydraulic_capacity) / 10_000.0;
    let iterations = if validation.valid {
        12 + project.nodes.len() as u32 + (project.ducts.len() as u32 * 2)
    } else {
        0
    };
    let residual = if validation.valid {
        1.0 / f64::from(iterations).powi(2)
    } else {
        1.0
    };
    let runtime_ms = started.elapsed().as_millis();

    let valid = validation.valid;

    SimulationReport {
        id: simulation_id.to_string(),
        status: if valid {
            SimulationStatus::Completed
        } else {
            SimulationStatus::Failed
        },
        project_name: project.name.clone(),
        started_at_epoch_ms,
        finished_at_epoch_ms: Some(now_epoch_ms()),
        iterations,
        residual,
        runtime_ms,
        solver_version: APP_VERSION,
        validation,
        status_history: build_status_history(started_at_epoch_ms, valid),
        results: SimulationResults {
            node_count: project.nodes.len(),
            duct_count: project.ducts.len(),
            equipment_count: project.equipments.len(),
            estimated_flow_m3_s,
            total_length_m,
            pressure_delta_pa,
        },
    }
}

fn build_status_history(started_at_epoch_ms: u128, valid: bool) -> Vec<StatusTransition> {
    let finished_status = if valid {
        SimulationStatus::Completed
    } else {
        SimulationStatus::Failed
    };
    let finished_message = if valid {
        "Simulação finalizada com convergência estimada"
    } else {
        "Simulação recusada por erros de validação"
    };

    vec![
        StatusTransition {
            status: SimulationStatus::Queued,
            epoch_ms: started_at_epoch_ms,
            message: "Simulação recebida e enfileirada".to_string(),
        },
        StatusTransition {
            status: SimulationStatus::Running,
            epoch_ms: now_epoch_ms(),
            message: "Validação e cálculo hidráulico simplificado iniciados".to_string(),
        },
        StatusTransition {
            status: finished_status,
            epoch_ms: now_epoch_ms(),
            message: finished_message.to_string(),
        },
    ]
}

fn validate_project(project: &ProjectInput) -> ValidationReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut node_ids = HashSet::new();
    let mut duct_ids = HashSet::new();
    let mut equipment_ids = HashSet::new();

    if project.schema_version != SCHEMA_VERSION {
        warnings.push(format!(
            "Versão de schema '{}' recebida; servidor espera '{}'",
            project.schema_version, SCHEMA_VERSION
        ));
    }

    if project.name.trim().is_empty() {
        errors.push("Nome do projeto é obrigatório".to_string());
    }

    if project.nodes.is_empty() {
        errors.push("Inclua pelo menos um nó na malha".to_string());
    }

    if project.ducts.is_empty() {
        errors.push("Inclua pelo menos um tubo para simular escoamento".to_string());
    }

    for node in &project.nodes {
        if node.id.trim().is_empty() {
            errors.push("Nó com id vazio".to_string());
        } else if !node_ids.insert(node.id.clone()) {
            errors.push(format!("Nó duplicado: '{}'", node.id));
        }
    }

    for duct in &project.ducts {
        if duct.id.trim().is_empty() {
            errors.push("Tubo com id vazio".to_string());
        } else if !duct_ids.insert(duct.id.clone()) {
            errors.push(format!("Tubo duplicado: '{}'", duct.id));
        }

        if !node_ids.contains(&duct.from) {
            errors.push(format!(
                "Tubo '{}' referencia nó de origem inexistente '{}'",
                duct.id, duct.from
            ));
        }

        if !node_ids.contains(&duct.to) {
            errors.push(format!(
                "Tubo '{}' referencia nó de destino inexistente '{}'",
                duct.id, duct.to
            ));
        }

        if duct.from == duct.to {
            errors.push(format!(
                "Tubo '{}' conecta o mesmo nó nas duas pontas",
                duct.id
            ));
        }

        if duct.length_m <= 0.0 {
            errors.push(format!("Tubo '{}' precisa ter length_m positivo", duct.id));
        }

        if duct.diameter_m <= 0.0 {
            errors.push(format!(
                "Tubo '{}' precisa ter diameter_m positivo",
                duct.id
            ));
        }

        if duct.roughness_m < 0.0 {
            errors.push(format!(
                "Tubo '{}' não pode ter roughness_m negativo",
                duct.id
            ));
        }
    }

    for equipment in &project.equipments {
        if equipment.id.trim().is_empty() {
            errors.push("Equipamento com id vazio".to_string());
        } else if !equipment_ids.insert(equipment.id.clone()) {
            errors.push(format!("Equipamento duplicado: '{}'", equipment.id));
        }

        if !node_ids.contains(&equipment.node_id) {
            errors.push(format!(
                "Equipamento '{}' referencia nó inexistente '{}'",
                equipment.id, equipment.node_id
            ));
        }
    }

    for condition in &project.boundary_conditions {
        if !node_ids.contains(&condition.node_id) {
            errors.push(format!(
                "Condição de contorno referencia nó inexistente '{}'",
                condition.node_id
            ));
        }

        if condition.kind.trim().is_empty() {
            errors.push(format!(
                "Condição no nó '{}' precisa de kind",
                condition.node_id
            ));
        }

        if condition.unit.trim().is_empty() {
            errors.push(format!(
                "Condição no nó '{}' precisa de unit",
                condition.node_id
            ));
        }
    }

    if project.boundary_conditions.is_empty() {
        warnings.push("Nenhuma condição de contorno definida".to_string());
    }

    if project.equipments.is_empty() {
        warnings.push(
            "Nenhum equipamento definido; a simulação usará pressão diferencial zero".to_string(),
        );
    }

    ValidationReport {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

impl From<&StoredProject> for ProjectSummary {
    fn from(project: &StoredProject) -> Self {
        Self {
            id: project.id.clone(),
            name: project.project.name.clone(),
            schema_version: project.project.schema_version.clone(),
            nodes: project.project.nodes.len(),
            ducts: project.project.ducts.len(),
            equipments: project.project.equipments.len(),
            boundary_conditions: project.project.boundary_conditions.len(),
            updated_at_epoch_ms: project.updated_at_epoch_ms,
        }
    }
}

impl Clone for ServerMetrics {
    fn clone(&self) -> Self {
        Self {
            projects_saved: self.projects_saved,
            validations_run: self.validations_run,
            simulations_started: self.simulations_started,
            simulations_completed: self.simulations_completed,
            simulations_failed: self.simulations_failed,
            last_simulation_ms: self.last_simulation_ms,
        }
    }
}

impl Clone for ValidationReport {
    fn clone(&self) -> Self {
        Self {
            valid: self.valid,
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
        }
    }
}

impl SimulationStatus {
    fn as_log_label(&self) -> &'static str {
        match self {
            SimulationStatus::Queued => "queued",
            SimulationStatus::Running => "running",
            SimulationStatus::Completed => "completed",
            SimulationStatus::Failed => "failed",
        }
    }
}

fn default_schema_version() -> String {
    SCHEMA_VERSION.to_string()
}

fn now_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX_EPOCH")
        .as_millis()
}

fn not_found(message: String) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse { error: message }),
    )
        .into_response()
}

// --- HANDLER DO SERVICE WORKER ---
async fn sw_handler() -> impl IntoResponse {
    match Assets::get("sw.js") {
        Some(content) => (
            [(header::CONTENT_TYPE, "application/javascript")],
            content.data,
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// --- HANDLER DE ARQUIVOS ESTÁTICOS ---
async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// --- HANDLERS DE PÁGINAS ---
async fn home() -> impl IntoResponse {
    HomeTemplate {
        title: "Início".to_string(),
    }
}
async fn canvas() -> impl IntoResponse {
    CanvasTemplate {
        title: "Editor P&ID".to_string(),
    }
}
async fn nodes() -> impl IntoResponse {
    NodesTemplate {
        title: "Nós".to_string(),
    }
}
async fn ducts() -> impl IntoResponse {
    DuctsTemplate {
        title: "Tubos".to_string(),
    }
}
async fn equips() -> impl IntoResponse {
    EquipsTemplate {
        title: "Equipamentos".to_string(),
    }
}
async fn sim() -> impl IntoResponse {
    SimTemplate {
        title: "Simulação".to_string(),
    }
}
async fn reports() -> impl IntoResponse {
    ReportsTemplate {
        title: "Relatórios".to_string(),
    }
}
async fn help() -> impl IntoResponse {
    HelpTemplate {
        title: "Ajuda".to_string(),
    }
}
