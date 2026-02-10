use axum::{
    extract::{Json, Path, State},
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use askama::Template;
use rust_embed::RustEmbed; // <--- BIBLIOTECA NOVA
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::net::SocketAddr;

// --- DADOS ---
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Node { id: String, name: String, #[sqlx(rename = "type")] tipo: String, x: f64, y: f64 }

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Duct { id: String, name: String, start_id: Option<String>, end_id: Option<String>, start_port: i32, end_port: i32 }

#[derive(Debug, Serialize, Deserialize)]
struct MeshData { nodes: Vec<Node>, ducts: Vec<Duct> }

// --- ARQUIVOS ESTÁTICOS EMBUTIDOS ---
// Isso diz ao Rust: "Na hora de compilar, vá na pasta 'static', 
// leia tudo e guarde os bytes dentro do executável final".
#[derive(RustEmbed)]
#[folder = "static"]
struct Assets;

// --- TEMPLATES (Registrando TODAS as telas) ---
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

// --- ESTADO ---
#[derive(Clone)]
struct AppState { pool: SqlitePool }

#[tokio::main]
async fn main() {
    let db_url = "sqlite://mesh.sqlite?mode=rwc";
    let pool = SqlitePoolOptions::new().connect(db_url).await.unwrap();

    // Cria as tabelas se não existirem
    sqlx::query("CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, name TEXT, type TEXT, x REAL, y REAL)").execute(&pool).await.unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS ducts (id TEXT PRIMARY KEY, name TEXT, start_id TEXT, end_id TEXT, start_port INTEGER, end_port INTEGER)").execute(&pool).await.unwrap();

    let app = Router::new()
        // --- ROTAS DAS PÁGINAS ---
        .route("/", get(home))
        .route("/canvas", get(canvas))
        .route("/nodes", get(nodes))
        .route("/ducts", get(ducts))
        .route("/equipments", get(equips))
        .route("/simulation", get(sim))
        .route("/reports", get(reports))
        .route("/help", get(help))
        // --- API ---
        .route("/api/get-mesh", get(get_mesh))
        .route("/api/mesh-db", post(save_mesh))
        // --- ROTA DE ARQUIVOS ESTÁTICOS (MÁGICA AQUI) ---
        // O *file pega qualquer caminho, tipo "js/fabric.min.js"
        .route("/static/*file", get(static_handler)) 
        .with_state(AppState { pool });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("☢️  SERVER ONLINE: http://{}", addr);
    println!("📦 Modo Monólito: Arquivos estáticos estão embutidos no EXE.");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- HANDLER DE ARQUIVOS ESTÁTICOS ---
async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    
    match Assets::get(path) {
        Some(content) => {
            // Descobre se é JS, CSS, PNG, etc automaticamente
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// --- HANDLERS DE PÁGINAS ---
async fn home() -> impl IntoResponse { HomeTemplate { title: "Início".to_string() } }
async fn canvas() -> impl IntoResponse { CanvasTemplate { title: "Editor P&ID".to_string() } }
async fn nodes() -> impl IntoResponse { NodesTemplate { title: "Nós".to_string() } }
async fn ducts() -> impl IntoResponse { DuctsTemplate { title: "Tubos".to_string() } }
async fn equips() -> impl IntoResponse { EquipsTemplate { title: "Equipamentos".to_string() } }
async fn sim() -> impl IntoResponse { SimTemplate { title: "Simulação".to_string() } }
async fn reports() -> impl IntoResponse { ReportsTemplate { title: "Relatórios".to_string() } }
async fn help() -> impl IntoResponse { HelpTemplate { title: "Ajuda".to_string() } }

// --- API HANDLERS ---
async fn get_mesh(State(s): State<AppState>) -> Json<MeshData> {
    let nodes = sqlx::query_as::<_, Node>("SELECT * FROM nodes").fetch_all(&s.pool).await.unwrap_or_default();
    let ducts = sqlx::query_as::<_, Duct>("SELECT * FROM ducts").fetch_all(&s.pool).await.unwrap_or_default();
    Json(MeshData { nodes, ducts })
}

async fn save_mesh(State(s): State<AppState>, Json(p): Json<MeshData>) -> impl IntoResponse {
    let mut tx = s.pool.begin().await.unwrap();
    sqlx::query("DELETE FROM nodes").execute(&mut *tx).await.unwrap();
    sqlx::query("DELETE FROM ducts").execute(&mut *tx).await.unwrap();
    for n in p.nodes { sqlx::query("INSERT INTO nodes VALUES (?,?,?,?,?)").bind(n.id).bind(n.name).bind(n.tipo).bind(n.x).bind(n.y).execute(&mut *tx).await.unwrap(); }
    for d in p.ducts { sqlx::query("INSERT INTO ducts VALUES (?,?,?,?,?,?)").bind(d.id).bind(d.name).bind(d.start_id).bind(d.end_id).bind(d.start_port).bind(d.end_port).execute(&mut *tx).await.unwrap(); }
    tx.commit().await.unwrap();
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}