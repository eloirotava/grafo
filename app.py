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
    id: str
    name: str
    type: str
    x: float
    y: float

class Duct(BaseModel):
    id: str
    name: str
    start_id: Optional[str]
    end_id: Optional[str]
    start_port: int = 0
    end_port: int = 0

class MeshData(BaseModel):
    nodes: List[Node]
    ducts: List[Duct]

@app.get("/", response_class=HTMLResponse)
async def read_root(request: Request):
    return templates.TemplateResponse("index.html", {"request": request})

@app.post("/api/mesh-db")
async def save_mesh(data: MeshData):
    try:
        conn = sqlite3.connect('mesh.sqlite')
        cursor = conn.cursor()
        
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY, 
                name TEXT,
                type TEXT, 
                x REAL, 
                y REAL
            )
        """)
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS ducts (
                id TEXT PRIMARY KEY, 
                name TEXT, 
                start_id TEXT, 
                end_id TEXT,
                start_port INTEGER,
                end_port INTEGER
            )
        """)
        
        cursor.execute("DELETE FROM nodes")
        cursor.execute("DELETE FROM ducts")
        
        for n in data.nodes:
            cursor.execute("INSERT INTO nodes (id, name, type, x, y) VALUES (?, ?, ?, ?, ?)", 
                           (n.id, n.name, n.type, n.x, n.y))
            
        for d in data.ducts:
            cursor.execute("INSERT INTO ducts (id, name, start_id, end_id, start_port, end_port) VALUES (?, ?, ?, ?, ?, ?)", 
                           (d.id, d.name, d.start_id, d.end_id, d.start_port, d.end_port))
        
        conn.commit()
        conn.close()
        return {"status": "success", "message": "Malha salva com sucesso!"}
    except Exception as e:
        return {"status": "error", "message": str(e)}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)