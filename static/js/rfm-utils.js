(function () {
    const SCHEMA_VERSION = '1.0';

    function ensureSchemaVersion(project) {
        return {
            schema_version: project.schema_version || SCHEMA_VERSION,
            ...project,
        };
    }

    function validateProject(project) {
        const errors = [];
        const warnings = [];
        const nodes = (project?.nodes || []).filter((node) => node.type !== 'valve' && node.type !== 'A');
        const ducts = project?.ducts || [];
        const ductGeom = project?.duct_geom || [];
        const nodeComposition = project?.node_composition || [];
        const nodeIds = new Set();
        const ductIds = new Set();

        if (!project) {
            errors.push('Projeto vazio ou inválido.');
            return { valid: false, errors, warnings };
        }

        if (!project.schema_version) {
            warnings.push(`Arquivo sem schema_version; será salvo como ${SCHEMA_VERSION}.`);
        }

        nodes.forEach((node) => {
            if (!node.id) errors.push('Existe um nó sem id.');
            if (nodeIds.has(String(node.id))) errors.push(`Nó duplicado: ${node.id}.`);
            nodeIds.add(String(node.id));
        });

        ducts.forEach((duct) => {
            const label = duct.name || duct.id || 'sem tag';
            if (!duct.id) errors.push('Existe um duto sem id.');
            if (ductIds.has(String(duct.id))) errors.push(`Duto duplicado: ${duct.id}.`);
            ductIds.add(String(duct.id));

            if (!duct.start_id || !duct.end_id) errors.push(`Duto ${label} precisa estar conectado nas duas pontas.`);
            if (duct.start_id && !nodeIds.has(String(duct.start_id))) errors.push(`Duto ${label} referencia nó inicial inexistente.`);
            if (duct.end_id && !nodeIds.has(String(duct.end_id))) errors.push(`Duto ${label} referencia nó final inexistente.`);

            const segments = ductGeom.filter((segment) => String(segment.duct_id) === String(duct.id));
            if (segments.length === 0) warnings.push(`Duto ${label} não tem geometria detalhada; a simulação usará valores padrão.`);
            segments.forEach((segment) => {
                const segLabel = `${label}/tramo ${segment.seg_index ?? '?'}`;
                const length = Number(segment.L ?? segment.length ?? 0);
                const diameter = Number(segment.D ?? segment.diameter ?? 0);
                const roughness = Number(segment.rug ?? segment.roughness ?? 0);
                if (length <= 0) errors.push(`${segLabel} precisa de comprimento positivo.`);
                if (diameter <= 0) errors.push(`${segLabel} precisa de diâmetro positivo.`);
                if (roughness < 0) errors.push(`${segLabel} não pode ter rugosidade negativa.`);
            });
        });

        nodes.forEach((node) => {
            const comps = nodeComposition.filter((composition) => String(composition.node_id) === String(node.id));
            if (comps.length === 0) warnings.push(`Nó ${node.name || node.id} sem composição definida.`);
        });

        if (nodes.length < 2) errors.push('Crie pelo menos dois nós antes de salvar/simular uma malha completa.');
        if (ducts.length < 1) errors.push('Crie pelo menos um duto antes de salvar/simular uma malha completa.');

        return { valid: errors.length === 0, errors, warnings };
    }

    function formatValidation(report) {
        return [...report.errors, ...report.warnings].join('\n');
    }

    window.RotavaFlowRfm = {
        SCHEMA_VERSION,
        ensureSchemaVersion,
        validateProject,
        formatValidation,
    };
})();
