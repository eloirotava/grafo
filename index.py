import sqlite3
from flask import Flask, render_template, request, jsonify

app = Flask(__name__)

# --- Configuração de Banco de Dados ---
def get_db():
    conn = sqlite3.connect('mesh.sqlite')
    conn.row_factory = sqlite3.Row
    return conn

# --- Rotas de Páginas (Frontend) ---
@app.route("/")
def home():
    return render_template("home.html", title="Início")

@app.route("/canvas")
def canvas():
    return render_template("canvas.html", title="Editor")

@app.route("/nodes")
def nodes_page():
    return render_template("nodes.html", title="Nós e Fontes")

@app.route("/ducts")
def ducts_page():
    return render_template("ducts.html", title="Tubulação")

@app.route("/equipments")
def equips_page():
    return render_template("equipments.html", title="Equipamentos")

@app.route("/simulation")
def sim_page():
    return render_template("simulation.html", title="Simulação")

@app.route("/reports")
def rep_page():
    return render_template("reports.html", title="Relatórios")

@app.route("/help")
def help_page():
    return render_template("help.html", title="Ajuda")

# --- API e Banco de Dados ---

@app.get("/api/get-mesh")  # Atalho para @app.route(..., methods=["GET"])
def get_mesh():
    try:
        conn = get_db()
        cursor = conn.cursor()
        
        # Busca nós
        cursor.execute("SELECT * FROM nodes")
        nodes = [dict(row) for row in cursor.fetchall()]
        
        # Busca dutos
        cursor.execute("SELECT * FROM ducts")
        ducts = [dict(row) for row in cursor.fetchall()]
        
        conn.close()
        return jsonify({"nodes": nodes, "ducts": ducts})
    except Exception as e:
        print(f"Erro ao ler banco: {e}")
        return jsonify({"nodes": [], "ducts": []})

@app.post("/api/mesh-db") # Atalho para @app.route(..., methods=["POST"])
def save_mesh():
    # Em Flask, request.json retorna um dicionário (sem validação Pydantic)
    data = request.json 
    
    try:
        conn = get_db()
        cursor = conn.cursor()
        
        # Criação de tabelas (Idêntico ao original)
        cursor.execute("CREATE TABLE IF NOT EXISTS nodes (id TEXT PRIMARY KEY, name TEXT, type TEXT, x REAL, y REAL)")
        cursor.execute("CREATE TABLE IF NOT EXISTS ducts (id TEXT PRIMARY KEY, name TEXT, start_id TEXT, end_id TEXT, start_port INTEGER, end_port INTEGER)")
        
        # Limpa dados antigos
        cursor.execute("DELETE FROM nodes")
        cursor.execute("DELETE FROM ducts")
        
        # Inserção de Nós
        if 'nodes' in data:
            for n in data['nodes']:
                # Usa .get() para segurança caso algum campo falte
                cursor.execute("INSERT INTO nodes VALUES (?, ?, ?, ?, ?)", 
                             (n.get('id'), n.get('name'), n.get('type'), n.get('x'), n.get('y')))
        
        # Inserção de Dutos
        if 'ducts' in data:
            for d in data['ducts']:
                cursor.execute("INSERT INTO ducts VALUES (?, ?, ?, ?, ?, ?)", 
                             (d.get('id'), d.get('name'), d.get('start_id'), d.get('end_id'), d.get('start_port', 0), d.get('end_port', 0)))
        
        conn.commit()
        conn.close()
        return jsonify({"status": "success", "message": "Malha salva com sucesso!"})
    
    except Exception as e:
        return jsonify({"status": "error", "message": str(e)})

if __name__ == "__main__":
    # Roda o servidor. debug=True faz o reload automático ao salvar arquivos
    app.run(host="0.0.0.0", port=8000, debug=True)