const CACHE_NAME = 'rotavaflow-v3'; // Versão nova para forçar a limpeza do cache antigo
const ASSETS_TO_CACHE = [
    '../index.html',
    '../pages/canvas.html',
    '../pages/nodes.html',
    '../pages/ducts.html',
    '../pages/equipments.html',
    '../pages/simulation.html',
    '../pages/reports.html',
    '../pages/help.html',
    './js/fabric.min.js',
    './js/chart.js',
    './js/wrapper.js',  
    './wasm/fluxo.js',  
    './wasm/fluxo.wasm',
    './manifest.json'
];

self.addEventListener('install', (event) => {
    console.log('[SW] Instalando v3...');
    self.skipWaiting(); 
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(ASSETS_TO_CACHE);
        })
    );
});

self.addEventListener('activate', (event) => {
    console.log('[SW] Ativando v3 e limpando caches antigos...');
    event.waitUntil(
        caches.keys().then((keyList) => {
            return Promise.all(keyList.map((key) => {
                if (key !== CACHE_NAME) {
                    console.log('[SW] Removendo cache antigo:', key);
                    return caches.delete(key);
                }
            }));
        })
    );
    return self.clients.claim(); 
});

self.addEventListener('fetch', (event) => {
    if (event.request.method !== 'GET') return;
    event.respondWith(
        fetch(event.request)
            .then((response) => {
                const responseClone = response.clone();
                caches.open(CACHE_NAME).then((cache) => {
                    cache.put(event.request, responseClone);
                });
                return response;
            })
            .catch(() => {
                return caches.match(event.request).then(cached => {
                    return cached || new Response("Offline e sem cache.", { status: 404 });
                });
            })
    );
});