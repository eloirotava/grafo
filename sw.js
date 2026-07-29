// Service Worker do RotavaFlow.
// Fica na raiz de propósito: o escopo de um SW é limitado à pasta em que é
// servido, e daqui ele controla tanto / quanto /pages/.
// Todos os caminhos abaixo são relativos a este ficheiro, então o site também
// funciona servido numa subpasta (ex.: GitHub Pages em /grafo/).

const CACHE_NAME = 'rotavaflow-v4';

const ASSETS_TO_CACHE = [
    './',
    './index.html',
    './pages/canvas.html',
    './pages/nodes.html',
    './pages/ducts.html',
    './pages/equipments.html',
    './pages/simulation.html',
    './pages/reports.html',
    './pages/help.html',
    './static/css/app.css',
    './static/js/app.js',
    './static/js/fabric.min.js',
    './static/js/chart.js',
    './static/js/wrapper.js',
    './static/wasm/fluxo.js',
    './static/wasm/fluxo.wasm',
    './static/manifest.json',
    './static/icon.png'
];

self.addEventListener('install', (event) => {
    self.skipWaiting();
    event.waitUntil(
        caches.open(CACHE_NAME).then(async (cache) => {
            // Um a um em vez de cache.addAll(): addAll é atómico, e um único
            // recurso em falta descartaria silenciosamente todo o cache offline.
            const results = await Promise.allSettled(
                ASSETS_TO_CACHE.map((url) => cache.add(url))
            );
            results.forEach((r, i) => {
                if (r.status === 'rejected') {
                    console.warn('[SW] Não foi possível pré-carregar:', ASSETS_TO_CACHE[i], r.reason);
                }
            });
        })
    );
});

self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys()
            .then((keys) => Promise.all(
                keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k))
            ))
            // clients.claim() tem de estar dentro do waitUntil para ser aguardado.
            .then(() => self.clients.claim())
    );
});

// Só guardamos respostas próprias e bem sucedidas: respostas opacas
// (cross-origin) e erros no cache deixariam a app partida quando offline.
function isCacheable(response) {
    return response && response.ok && response.type === 'basic';
}

self.addEventListener('fetch', (event) => {
    const request = event.request;
    if (request.method !== 'GET') return;

    const url = new URL(request.url);
    if (url.origin !== self.location.origin) return;

    // Navegação (HTML): rede primeiro, para apanhar páginas atualizadas;
    // cai para o cache quando offline.
    if (request.mode === 'navigate') {
        event.respondWith(
            fetch(request)
                .then((response) => {
                    if (isCacheable(response)) {
                        const copy = response.clone();
                        event.waitUntil(caches.open(CACHE_NAME).then((c) => c.put(request, copy)));
                    }
                    return response;
                })
                .catch(async () => {
                    return (await caches.match(request))
                        || (await caches.match('./index.html'))
                        || new Response('Offline e sem cache.', {
                            status: 503,
                            headers: { 'Content-Type': 'text/plain; charset=utf-8' }
                        });
                })
        );
        return;
    }

    // Restantes recursos (JS/CSS/WASM): cache primeiro, revalidando em segundo
    // plano. Evita esperar pela rede para 350 KB de WASM a cada carregamento.
    event.respondWith(
        caches.match(request).then((cached) => {
            const network = fetch(request)
                .then((response) => {
                    if (isCacheable(response)) {
                        const copy = response.clone();
                        event.waitUntil(caches.open(CACHE_NAME).then((c) => c.put(request, copy)));
                    }
                    return response;
                })
                .catch(() => cached);

            return cached || network;
        })
    );
});
