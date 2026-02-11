const CACHE_NAME = 'restobox-offline-v2'; // <--- Mudei para v2 (Sempre mude isso pra forçar update)
const ASSETS_TO_CACHE = [
    '/',
    '/canvas',
    '/nodes',
    '/ducts',
    '/equipments',
    '/simulation',
    '/reports',
    '/help',
    '/static/js/fabric.min.js',
    '/static/js/chart.js',
    '/static/js/wrapper.js',  
    '/static/wasm/fluxo.js',  
    '/static/wasm/fluxo.wasm',
    '/static/manifest.json' // Adicionei o manifesto se você criou
];

// 1. INSTALL: Baixa tudo e FORÇA a entrada imediata (Pula o Waiting)
self.addEventListener('install', (event) => {
    console.log('[SW] Instalando v2...');
    
    // O self.skipWaiting() chuta o SW antigo imediatamente!
    self.skipWaiting(); 

    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(ASSETS_TO_CACHE);
        })
    );
});

// 2. ACTIVATE: Limpa o lixo e assume o controle das abas abertas
self.addEventListener('activate', (event) => {
    console.log('[SW] Ativando v2 e limpando caches antigos...');
    
    event.waitUntil(
        caches.keys().then((keyList) => {
            return Promise.all(keyList.map((key) => {
                // Se o cache não for o v2, apaga!
                if (key !== CACHE_NAME) {
                    console.log('[SW] Removendo cache antigo:', key);
                    return caches.delete(key);
                }
            }));
        })
    );
    
    // O clients.claim() faz a aba obedecer o novo SW sem precisar recarregar
    return self.clients.claim(); 
});

// 3. FETCH: (Manteve igual) Intercepta e serve do cache se offline
self.addEventListener('fetch', (event) => {
    if (event.request.method !== 'GET') return;

    event.respondWith(
        fetch(event.request)
            .then((response) => {
                // Rede funcionou? Atualiza o cache (Stale-while-revalidate)
                const responseClone = response.clone();
                caches.open(CACHE_NAME).then((cache) => {
                    cache.put(event.request, responseClone);
                });
                return response;
            })
            .catch(() => {
                // Rede falhou? Usa o cache
                return caches.match(event.request).then(cached => {
                    return cached || new Response("Offline e sem cache.", { status: 404 });
                });
            })
    );
});