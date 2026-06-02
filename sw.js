const CACHE_NAME = 'rotavaflow-static-v1';
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
    './static/manifest.json',
    './static/js/fabric.min.js',
    './static/js/chart.js',
    './static/js/wrapper.js',
    './static/wasm/fluxo.js',
    './static/wasm/fluxo.wasm'
];

self.addEventListener('install', (event) => {
    self.skipWaiting();
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS_TO_CACHE))
    );
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

    event.respondWith(
        caches.match(event.request).then((cached) => {
            if (cached) return cached;

            return fetch(event.request).then((response) => {
                if (response.ok) {
                    const responseClone = response.clone();
                    caches.open(CACHE_NAME).then((cache) => cache.put(event.request, responseClone));
                }
                return response;
            });
        })
    );
});
