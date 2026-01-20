import sqlite3
import uvicorn
from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
from pydantic import BaseModel
from typing import List, Optional

app = FastAPI()
templates = Jinja2Templates(directory="templates")

# --- Modelos ---
class Node(BaseModel):
    id: str; name: str; type: str; x: float; y: float
class Duct(BaseModel):
    id: str; name: str; start_id: Optional[str]; end_id: Optional[str]; start_port: int = 0; end_port: int = 0
class MeshData(BaseModel):
    nodes: List[Node]; ducts: List[Duct]

# --- Rotas de Páginas ---
@app.get("/", response_class=HTMLResponse)
async def home(r: Request): return templates.TemplateResponse("home.html", {"request": r, "title": "Início"})

@app.get("/canvas", response_class=HTMLResponse)
async def canvas(r: Request): return templates.TemplateResponse("canvas.html", {"request": r, "title": "Editor"})

@app.get("/nodes", response_class=HTMLResponse)
async def nodes_page(r: Request): return templates.TemplateResponse("nodes.html", {"request": r, "title": "Nós e Fontes"})

@app.get("/ducts", response_class=HTMLResponse)
async def ducts_page(r: Request): return templates.TemplateResponse("ducts.html", {"request": r, "title": "Tubulação"})

@app.get("/equipments", response_class=HTMLResponse)
async def equips_page(r: Request): return templates.TemplateResponse("equipments.html", {"request": r, "title": "Equipamentos"})

@app.get("/simulation", response_class=HTMLResponse)
async def sim_page(r: Request): return templates.TemplateResponse("simulation.html", {"request": r, "title": "Simulação"})

@app.get("/reports", response_class=HTMLResponse)
async def rep_page(r: Request): return templates.TemplateResponse("reports.html", {"request": r, "title": "Relatórios"})

@app.get("/help", response_class=HTMLResponse)
async def help_page(r: Request): return templates.TemplateResponse("help.html", {"request": r, "title": "Ajuda"})

# --- API e Banco de Dados ---
def get_db():
    conn = sqlite3.connect('mesh.sqlite')
    conn.row_factory = sqlite3.Row
    return conn

@app.get("/api/get-mesh")
async def get_mesh():
    try:
        conn = get_db()
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM nodes")
        nodes = [dict(row) for row in cursor.fetchall()]
        cursor.execute("SELECT * FROM ducts")
        ducts = [dict(row) for row in cursor.fetchall()]
        conn.close()
        return {"nodes": nodes, "ducts": ducts}
    except:
        return {"nodes": [], "ducts": []}

@app.post("/api/mesh-db")
async def save_mesh(data: MeshData):
    try:
        conn = get_db()
        cursor = conn.cursor()
        cursor.execute("CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, name TEXT, type TEXT, x REAL, y REAL)")
        cursor.execute("CREATE TABLE IF NOT EXISTS ducts (id TEXT PRIMARY KEY, name TEXT, start_id TEXT, end_id TEXT, start_port INTEGER, end_port INTEGER)")
        
        cursor.execute("DELETE FROM nodes"); cursor.execute("DELETE FROM ducts")
        
        for n in data.nodes: cursor.execute("INSERT INTO nodes VALUES (?, ?, ?, ?, ?)", (n.id, n.name, n.type, n.x, n.y))
        for d in data.ducts: cursor.execute("INSERT INTO ducts VALUES (?, ?, ?, ?, ?, ?)", (d.id, d.name, d.start_id, d.end_id, d.start_port, d.end_port))
        
        conn.commit(); conn.close()
        return {"status": "success", "message": "Malha salva com sucesso!"}
    except Exception as e:
        return {"status": "error", "message": str(e)}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)