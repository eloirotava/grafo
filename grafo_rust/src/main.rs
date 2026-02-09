use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use askama::Template;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;

// --- ESTRUTURAS DE DADOS (Igual ao seu app.py) ---
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Node {
    id: String,
    name: String,
    #[serde(rename = "type")] // JSON usa "type"
    #[sqlx(rename = "type")]  // Banco usa "type"
    tipo: String,             // Rust usa "tipo" (type é reservado)
    x: f64,
    y: f64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Duct {
    id: String,
    name: String,
    start_id: Option<String>,
    end_id: Option<String>,
    start_port: i32,
    end_port: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MeshData {
    nodes: Vec<Node>,
    ducts: Vec<Duct>,
}

// --- TEMPLATES (A Mágica da Compilação) ---
// ATENÇÃO: Use ASPAS DUPLAS nos caminhos!
#[derive(Template)]
#[template(path = "home.html")] 
struct HomeTemplate { title: String }

#[derive(Template)]
#[template(path = "canvas.html")]
struct CanvasTemplate { title: String }

#[derive(Template)]
#[template(path = "nodes.html")]
struct NodesTemplate { title: String }

#[derive(Template)]
#[template(path = "ducts.html")]
struct DuctsTemplate { title: String }

#[derive(Template)]
#[template(path = "equipments.html")]
struct EquipsTemplate { title: String }

#[derive(Template)]
#[template(path = "simulation.html")]
struct SimTemplate { title: String }

#[derive(Template)]
#[template(path = "reports.html")]
struct ReportsTemplate { title: String }

#[derive(Template)]
#[template(path = "help.html")]
struct HelpTemplate { title: String }

// --- ESTADO DO APP ---
#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[tokio::main]
async fn main() {
    // Conecta no SQLite (cria o arquivo se não existir)
    let db_url = "sqlite://mesh.sqlite?mode=rwc";
    let pool = SqlitePoolOptions::new()
        .max_connections(5) // Poucas conexões pra não estourar o Termux
        .connect(db_url)
        .await
        .expect("Erro ao conectar no SQLite");

    // Cria tabelas na inicialização
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, name TEXT, type TEXT, x REAL, y REAL);
         CREATE TABLE IF NOT EXISTS ducts (id TEXT PRIMARY KEY, name TEXT, start_id TEXT, end_id TEXT, start_port INTEGER, end_port INTEGER);"
    )
    .execute(&pool)
    .await
    .unwrap();

    let state = AppState { pool };

    // Rotas (Igualzinho ao FastAPI)
    let app = Router::new()
        .route("/", get(home))
        .route("/canvas", get(canvas))
        .route("/nodes", get(nodes))
        .route("/ducts", get(ducts))
        .route("/equipments", get(equips))
        .route("/simulation", get(sim))
        .route("/reports", get(reports))
        .route("/help", get(help))
        .route("/api/get-mesh", get(get_mesh_api))
        .route("/api/mesh-db", post(save_mesh_api))
        .with_state(state);

    // Roda na porta 8000
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("☢️  SERVER ONLINE: http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- HANDLERS (Views) ---
async fn home() -> impl IntoResponse { HomeTemplate { title: "Início".to_string() } }
async fn canvas() -> impl IntoResponse { CanvasTemplate { title: "Editor P&ID".to_string() } }
async fn nodes() -> impl IntoResponse { NodesTemplate { title: "Nós".to_string() } }
async fn ducts() -> impl IntoResponse { DuctsTemplate { title: "Tubos".to_string() } }
async fn equips() -> impl IntoResponse { EquipsTemplate { title: "Equipamentos".to_string() } }
async fn sim() -> impl IntoResponse { SimTemplate { title: "Simulação".to_string() } }
async fn reports() -> impl IntoResponse { ReportsTemplate { title: "Relatórios".to_string() } }
async fn help() -> impl IntoResponse { HelpTemplate { title: "Ajuda".to_string() } }

// --- API (JSON) ---
async fn get_mesh_api(State(state): State<AppState>) -> Json<MeshData> {
    let nodes = sqlx::query_as::<_, Node>("SELECT * FROM nodes")
        .fetch_all(&state.pool).await.unwrap_or_default();
    let ducts = sqlx::query_as::<_, Duct>("SELECT * FROM ducts")
        .fetch_all(&state.pool).await.unwrap_or_default();
    Json(MeshData { nodes, ducts })
}

async fn save_mesh_api(State(state): State<AppState>, Json(payload): Json<MeshData>) -> impl IntoResponse {
    let mut tx = state.pool.begin().await.unwrap();
    
    // Limpa tudo e regrava (Simples e brutal, funciona bem pra grafos pequenos)
    sqlx::query("DELETE FROM nodes").execute(&mut *tx).await.unwrap();
    sqlx::query("DELETE FROM ducts").execute(&mut *tx).await.unwrap();

    for n in payload.nodes {
        sqlx::query("INSERT INTO nodes VALUES (?, ?, ?, ?, ?)")
            .bind(n.id).bind(n.name).bind(n.tipo).bind(n.x).bind(n.y)
            .execute(&mut *tx).await.unwrap();
    }
    for d in payload.ducts {
        sqlx::query("INSERT INTO ducts VALUES (?, ?, ?, ?, ?, ?)")
            .bind(d.id).bind(d.name).bind(d.start_id).bind(d.end_id).bind(d.start_port).bind(d.end_port)
            .execute(&mut *tx).await.unwrap();
    }
    
    tx.commit().await.unwrap();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}