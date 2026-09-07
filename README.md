# LocalBrowser

Sites rodando **como aplicativos** — cada um numa janela Firefox com perfil
dedicado (cookies/logins próprios, sem histórico do navegador pessoal),
**uBlock Origin pré-instalado** por padrão e suporte a extensões extras
(`.xpi`). Atalhos na área de trabalho abrem o app direto, passando pelo
LocalBrowser (que resolve o Firefox e sincroniza as extensões).

- **Base:** Firefox real, detectado automaticamente (ou caminho manual).
- **uBlock:** xpi oficial baixado uma vez e replicado nos perfis.
- **Extensões:** adicione qualquer `.xpi` por app (o id interno Gecko é lido
  do próprio arquivo).
- **Kiosk:** opcional por app, abre sem a barra do navegador.
- Padrão da suíte Local: temas (8), i18n EN/PT/ES, 100% offline.

Parte da suíte Local. Icon: globo âmbar/teal sobre quadrado escuro.
