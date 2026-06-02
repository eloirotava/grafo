(function () {
    const STORAGE_KEY = 'malha_db_v2';

    function readProject() {
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            return raw ? JSON.parse(raw) : null;
        } catch (error) {
            console.warn('[project-summary] Falha ao ler projeto:', error);
            return null;
        }
    }

    function countProject(data) {
        const nodes = (data?.nodes || []).filter((node) => node.type !== 'valve' && node.type !== 'A');
        const valves = (data?.valves || []).length || (data?.nodes || []).filter((node) => node.type === 'valve' || node.type === 'A').length;
        const ducts = data?.ducts || [];
        const segments = data?.duct_geom || [];
        const compositions = data?.node_composition || [];
        return { nodes: nodes.length, ducts: ducts.length, valves, segments: segments.length, compositions: compositions.length };
    }

    function renderSummary(container, data) {
        if (!data) {
            container.className = 'project-summary project-summary-warning';
            container.innerHTML = `
                <strong>Nenhum projeto carregado.</strong>
                <span>Comece pelo Editor P&ID ou carregue um arquivo .rfm.</span>
                <a href="canvas.html">Abrir Editor P&ID</a>
            `;
            return;
        }

        const counts = countProject(data);
        const warnings = [];
        if (counts.nodes === 0) warnings.push('sem nós');
        if (counts.ducts === 0) warnings.push('sem dutos');
        if (counts.segments === 0 && counts.ducts > 0) warnings.push('dutos sem geometria detalhada');

        container.className = `project-summary ${warnings.length ? 'project-summary-warning' : 'project-summary-ok'}`;
        container.innerHTML = `
            <strong>Projeto atual</strong>
            <span>${counts.nodes} nós</span>
            <span>${counts.ducts} dutos</span>
            <span>${counts.valves} equipamentos/válvulas</span>
            <span>${counts.segments} tramos</span>
            ${warnings.length ? `<em>Atenção: ${warnings.join(', ')}.</em>` : '<em>Malha pronta para configuração.</em>'}
        `;
    }

    function injectStyles() {
        if (document.getElementById('project-summary-style')) return;
        const style = document.createElement('style');
        style.id = 'project-summary-style';
        style.textContent = `
            .project-summary { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin: 0 0 18px; padding: 12px 14px; border-radius: 6px; border: 1px solid #d9e2ec; background: #fff; color: #334; font-size: 13px; box-shadow: 0 1px 3px rgba(0,0,0,0.04); }
            .project-summary strong { color: #0078d7; margin-right: 4px; }
            .project-summary span { background: #eef5ff; border: 1px solid #d7e8ff; color: #23527c; padding: 3px 8px; border-radius: 999px; }
            .project-summary em { font-style: normal; color: #667; margin-left: auto; }
            .project-summary a { color: #0078d7; font-weight: 600; margin-left: auto; }
            .project-summary-warning { border-color: #f0d49b; background: #fff8e8; }
            .project-summary-warning span { background: #fff; border-color: #f0d49b; color: #8a5a00; }
            .project-summary-warning em { color: #8a5a00; }
            .project-summary-ok { border-color: #b7dfc2; background: #f2fbf4; }
        `;
        document.head.appendChild(style);
    }

    window.RotavaFlowProject = { readProject, countProject };

    document.addEventListener('DOMContentLoaded', () => {
        const container = document.getElementById('project-summary');
        if (!container) return;
        injectStyles();
        renderSummary(container, readProject());
    });
})();
