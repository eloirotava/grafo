const isLocalFile = window.location.protocol === 'file:';
const pwaScriptUrl = document.currentScript?.src || new URL('static/js/pwa.js', window.location.href).href;
const serviceWorkerUrl = new URL('../../sw.js', pwaScriptUrl);
const serviceWorkerScope = new URL('../../', pwaScriptUrl).pathname;

if ('serviceWorker' in navigator && !isLocalFile) {
    window.addEventListener('load', async () => {
        try {
            const registration = await navigator.serviceWorker.register(serviceWorkerUrl, {
                scope: serviceWorkerScope,
            });
            console.info('✅ Modo Offline Ativado! Scope:', registration.scope);
        } catch (error) {
            console.warn('❌ Falha no Service Worker:', error);
        }
    });
}
