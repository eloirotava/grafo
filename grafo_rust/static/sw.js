const CACHE_NAME = 'restobox-offline-v1';
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
    '/static/js/chart.js'
];

// 1. Instalação: Baixa e salva tudo no cache
self.addEventListener('install', (event) => {
    console.log('[Service Worker] Instalando e cacheando tudo...');
    event.waitUntil(
        caches.open(CACHE_NAME).then((cache) => {
            return cache.addAll(ASSETS_TO_CACHE);
        })
    );
});

// 2. Ativação: Limpa caches antigos se mudar a versão
self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then((keyList) => {
            return Promise.all(keyList.map((key) => {
                if (key !== CACHE_NAME) {
                    return caches.delete(key);
                }
            }));
        })
    );
    return self.clients.claim();
});

// 3. Interceptação (A Mágica): Tenta Rede -> Se falhar, usa Cache
self.addEventListener('fetch', (event) => {
    // Ignora requisições que não sejam GET (ex: POST para salvar)
    if (event.request.method !== 'GET') return;

    event.respondWith(
        fetch(event.request)
            .then((response) => {
                // Se a rede funcionou, atualiza o cache (pra próxima vez)
                const responseClone = response.clone();
                caches.open(CACHE_NAME).then((cache) => {
                    cache.put(event.request, responseClone);
                });
                return response;
            })
            .catch(() => {
                // Se a rede falhou (Servidor OFF), retorna do cache
                console.log('[Service Worker] Rede falhou. Usando cache para:', event.request.url);
                return caches.match(event.request);
            })
    );
});