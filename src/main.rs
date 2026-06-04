use askama::Template;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;

const DEFAULT_ROOM: &str = "geral";
const DEFAULT_USER: &str = "Visitante";

#[derive(Clone)]
struct AppState {
    collaboration_tx: broadcast::Sender<CollaborationEvent>,
}

#[derive(Clone, Debug)]
struct CollaborationEvent {
    room: String,
    sender_id: String,
    payload: String,
}

#[derive(Deserialize)]
struct WsParams {
    room: Option<String>,
    user: Option<String>,
    client_id: Option<String>,
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
    let (collaboration_tx, _) = broadcast::channel(256);
    let state = Arc::new(AppState { collaboration_tx });

    // Roteador limpo: páginas, service worker, arquivos estáticos e WebSocket colaborativo
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
        // --- COLABORAÇÃO EM TEMPO REAL ---
        .route("/ws", get(ws_handler))
        // --- SERVICE WORKER NA RAIZ (Obrigatório para PWA) ---
        .route("/sw.js", get(sw_handler))
        // --- ARQUIVOS ESTÁTICOS (JS/CSS) ---
        .route("/static/*file", get(static_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("SERVER: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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

// --- HANDLER DE WEBSOCKET COLABORATIVO ---
async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let room = clean_param(params.room, DEFAULT_ROOM);
    let user = clean_param(params.user, DEFAULT_USER);
    let client_id = clean_param(params.client_id, "anon");

    ws.on_upgrade(move |socket| handle_socket(socket, state, room, user, client_id))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    room: String,
    user: String,
    client_id: String,
) {
    let mut rx = state.collaboration_tx.subscribe();
    println!("WS: {user} entrou na sala {room}");

    loop {
        tokio::select! {
            received = socket.recv() => {
                match received {
                    Some(Ok(Message::Text(payload))) => {
                        let _ = state.collaboration_tx.send(CollaborationEvent {
                            room: room.clone(),
                            sender_id: client_id.clone(),
                            payload,
                        });
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) | Some(Ok(Message::Binary(_))) => {}
                    Some(Err(err)) => {
                        eprintln!("WS erro em {user}/{room}: {err}");
                        break;
                    }
                }
            }
            broadcasted = rx.recv() => {
                match broadcasted {
                    Ok(event) if event.room == room && event.sender_id != client_id => {
                        if socket.send(Message::Text(event.payload)).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    println!("WS: {user} saiu da sala {room}");
}

fn clean_param(value: Option<String>, fallback: &str) -> String {
    value
        .map(|v| v.trim().chars().take(48).collect::<String>())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| fallback.to_string())
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
