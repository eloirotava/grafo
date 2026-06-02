const CACHE_NAME = 'rotavaflow-static-v2';
const APP_SHELL = [
    './',
    './index.html',
    './pages/canvas.html',
    './pages/nodes.html',
    './pages/ducts.html',
    './pages/equipments.html',
    './pages/simulation.html',
    './pages/reports.html',
    './pages/help.html',
    './static/manifest.json',
    './static/js/pwa.js',
    './static/js/fabric.min.js',
    './static/js/chart.js',
    './static/js/wrapper.js',
    './static/wasm/fluxo.js',
    './static/wasm/fluxo.wasm'
];

self.addEventListener('install', (event) => {
    self.skipWaiting();
    event.waitUntil(caches.open(CACHE_NAME).then((cache) => cache.addAll(APP_SHELL)));
});

self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then((keys) => Promise.all(
            keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
        ))
    );
    return self.clients.claim();
});

self.addEventListener('fetch', (event) => {
    if (event.request.method !== 'GET') return;

    event.respondWith(staleWhileRevalidate(event.request));
});

async function staleWhileRevalidate(request) {
    const cache = await caches.open(CACHE_NAME);
    const cached = await cache.match(request);

    const fetched = fetch(request)
        .then((response) => {
            if (response.ok) cache.put(request, response.clone());
            return response;
        })
        .catch(() => cached || fallbackResponse(request));

    return cached || fetched;
}

function fallbackResponse(request) {
    if (request.mode === 'navigate') {
        return caches.match('./index.html');
    }

    return new Response('Offline e sem cache.', {
        status: 404,
        headers: { 'Content-Type': 'text/plain; charset=utf-8' }
    });
}
