// Registro único do Service Worker, incluído por todas as páginas.
//
// O sw.js precisa ficar na RAIZ do site: um Service Worker só controla URLs
// dentro da pasta em que é servido, então um sw.js em /static/ nunca
// controlaria /index.html nem /pages/*.html.

(function () {
    if (!('serviceWorker' in navigator)) return;

    // Resolve a raiz a partir da URL deste próprio script (static/js/app.js),
    // funcionando tanto na raiz de um domínio quanto em subpasta (GitHub Pages).
    var root = new URL('../../', document.currentScript.src);

    window.addEventListener('load', function () {
        navigator.serviceWorker.register(new URL('sw.js', root), { scope: root.pathname })
            .then(function (reg) { console.log('✅ Modo offline ativo. Escopo:', reg.scope); })
            .catch(function (err) { console.warn('❌ Falha ao registar o Service Worker:', err); });
    });
})();
