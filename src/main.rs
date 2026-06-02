use askama::Template;
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::Serialize;
use std::net::SocketAddr;

const APP_NAME: &str = "RotavaFlow";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const SIMULATION_MODULES: [&str; 4] = ["malha", "nos", "tubos", "equipamentos"];

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
    version: &'static str,
    modules: &'static [&'static str],
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
    // Roteador limpo: páginas, status, service worker e arquivos estáticos.
    let app = Router::new()
        // --- PÁGINAS ---
        .route("/", get(home))
        .route("/canvas", get(canvas))
        .route("/nodes", get(nodes))
        .route("/ducts", get(ducts))
        .route("/equipments", get(equips))
        .route("/simulation", get(sim))
        .route("/reports", get(reports))
        .route("/help", get(help))
        // --- SAÚDE DO SERVIDOR ---
        .route("/health", get(health))
        // --- SERVICE WORKER NA RAIZ (Obrigatório para PWA) ---
        .route("/sw.js", get(sw_handler))
        // --- ARQUIVOS ESTÁTICOS (JS/CSS) ---
        .route("/static/*file", get(static_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("SERVER: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- HANDLER DE SAÚDE DO SERVIDOR ---
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: APP_NAME,
        status: "ok",
        version: APP_VERSION,
        modules: &SIMULATION_MODULES,
    })
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
