import { useSyncExternalStore } from "react";

/** i18n leve da UI (padrão da suíte, ver docs/planos/padrao-apps.md). */

export type Locale = "pt" | "en" | "es";

export const LOCALE_LABELS: Record<Locale, string> = {
  pt: "Português",
  en: "English",
  es: "Español",
};

const LOCALE_KEY = "localbrowser.locale";

const pt = {
  "app.subtitle": "sites como apps, sobre Firefox + uBlock",
  "top.add": "Adicionar",
  "top.settings": "Configurações",

  "banner.firefoxMissing": "Firefox não encontrado. Ele é necessário para abrir os apps.",
  "banner.firefoxPick": "Escolher firefox.exe",
  "banner.firefoxManual": "Usando: {path}",

  "grid.empty": "Nenhum app ainda",
  "grid.emptyHint": "Adicione um site (Spotify, YouTube Music…) ou uma URL própria — cada um abre numa janela Firefox limpa com uBlock.",

  "card.launch": "Abrir",
  "card.shortcut": "Atalho",
  "card.shortcutRemove": "Remover atalho",
  "card.edit": "Editar",
  "card.delete": "Excluir",
  "badge.kiosk": "kiosk",
  "badge.ublock": "uBlock",
  "badge.exts": "{n} ext.",

  "dlg.add": "Novo app",
  "dlg.edit": "Editar app",
  "dlg.name": "Nome",
  "dlg.url": "URL",
  "dlg.namePlaceholder": "ex.: Spotify",
  "dlg.urlPlaceholder": "ex.: open.spotify.com",
  "dlg.kiosk": "Modo kiosk (janela sem barra do navegador)",
  "dlg.ublock": "uBlock Origin pré-instalado",
  "dlg.presets": "Sugestões",
  "dlg.deleteTitle": "Excluir app",
  "dlg.deleteConfirm": "Excluir “{name}”? O perfil e o atalho também são removidos.",
  "dlg.cancel": "Cancelar",
  "dlg.save": "Salvar",
  "dlg.delete": "Excluir",

  "ext.title": "Extensões (.xpi)",
  "ext.add": "Adicionar .xpi",
  "ext.none": "Nenhuma extensão extra.",
  "ext.remove": "Remover",
  "ext.afterSave": "Salve o app antes de gerenciar extensões.",

  "toast.launched": "Abrindo {name}…",
  "toast.launchFail": "Não deu pra abrir: {err}",
  "toast.saved": "App salvo",
  "toast.deleted": "“{name}” excluído",
  "toast.shortcutCreated": "Atalho criado na área de trabalho",
  "toast.shortcutRemoved": "Atalho removido",
  "toast.err": "Erro: {err}",

  "dlg.ok": "OK",

  "settings.title": "Configurações",
  "settings.theme": "Tema",
  "settings.themeSystem": "Sistema",
  "settings.themeLight": "Claro",
  "settings.themeDark": "Escuro",
  "settings.themeNature": "Natureza",
  "settings.themeDarkBlue": "Azul escuro",
  "settings.themeCalmGreen": "Verde calmo",
  "settings.themePastelPink": "Rosa pastel",
  "settings.themePunkPrincess": "PunkPrincess",
  "settings.language": "Idioma",
  "settings.about":
    " — abre sites como apps locais: janelas Firefox com perfil dedicado, uBlock Origin por padrão e suporte a extensões. Parte da suíte Local.",
} as const;

export type MessageKey = keyof typeof pt;

const en: Record<MessageKey, string> = {
  "app.subtitle": "sites as apps, on Firefox + uBlock",
  "top.add": "Add",
  "top.settings": "Settings",

  "banner.firefoxMissing": "Firefox not found. It's required to open the apps.",
  "banner.firefoxPick": "Pick firefox executable",
  "banner.firefoxManual": "Using: {path}",

  "grid.empty": "No apps yet",
  "grid.emptyHint": "Add a site (Spotify, YouTube Music…) or your own URL — each one opens in a clean Firefox window with uBlock.",

  "card.launch": "Open",
  "card.shortcut": "Shortcut",
  "card.shortcutRemove": "Remove shortcut",
  "card.edit": "Edit",
  "card.delete": "Delete",
  "badge.kiosk": "kiosk",
  "badge.ublock": "uBlock",
  "badge.exts": "{n} ext.",

  "dlg.add": "New app",
  "dlg.edit": "Edit app",
  "dlg.name": "Name",
  "dlg.url": "URL",
  "dlg.namePlaceholder": "e.g. Spotify",
  "dlg.urlPlaceholder": "e.g. open.spotify.com",
  "dlg.kiosk": "Kiosk mode (window without browser chrome)",
  "dlg.ublock": "uBlock Origin pre-installed",
  "dlg.presets": "Suggestions",
  "dlg.deleteTitle": "Delete app",
  "dlg.deleteConfirm": "Delete “{name}”? The profile and shortcut are removed too.",
  "dlg.cancel": "Cancel",
  "dlg.save": "Save",
  "dlg.delete": "Delete",

  "ext.title": "Extensions (.xpi)",
  "ext.add": "Add .xpi",
  "ext.none": "No extra extensions.",
  "ext.remove": "Remove",
  "ext.afterSave": "Save the app before managing extensions.",

  "toast.launched": "Opening {name}…",
  "toast.launchFail": "Couldn't open: {err}",
  "toast.saved": "App saved",
  "toast.deleted": "“{name}” deleted",
  "toast.shortcutCreated": "Shortcut created on the desktop",
  "toast.shortcutRemoved": "Shortcut removed",
  "toast.err": "Error: {err}",

  "dlg.ok": "OK",

  "settings.title": "Settings",
  "settings.theme": "Theme",
  "settings.themeSystem": "System",
  "settings.themeLight": "Light",
  "settings.themeDark": "Dark",
  "settings.themeNature": "Nature",
  "settings.themeDarkBlue": "Dark blue",
  "settings.themeCalmGreen": "Calm green",
  "settings.themePastelPink": "Pastel pink",
  "settings.themePunkPrincess": "PunkPrincess",
  "settings.language": "Language",
  "settings.about":
    " — opens sites as local apps: Firefox windows with a dedicated profile, uBlock Origin by default and extension support. Part of the Local suite.",
};

const es: Record<MessageKey, string> = {
  "app.subtitle": "sitios como apps, sobre Firefox + uBlock",
  "top.add": "Añadir",
  "top.settings": "Configuración",

  "banner.firefoxMissing": "Firefox no encontrado. Es necesario para abrir las apps.",
  "banner.firefoxPick": "Elegir ejecutable de Firefox",
  "banner.firefoxManual": "Usando: {path}",

  "grid.empty": "Aún no hay apps",
  "grid.emptyHint": "Añade un sitio (Spotify, YouTube Music…) o una URL propia — cada uno abre en una ventana limpia de Firefox con uBlock.",

  "card.launch": "Abrir",
  "card.shortcut": "Acceso",
  "card.shortcutRemove": "Quitar acceso",
  "card.edit": "Editar",
  "card.delete": "Eliminar",
  "badge.kiosk": "kiosk",
  "badge.ublock": "uBlock",
  "badge.exts": "{n} ext.",

  "dlg.add": "Nueva app",
  "dlg.edit": "Editar app",
  "dlg.name": "Nombre",
  "dlg.url": "URL",
  "dlg.namePlaceholder": "ej.: Spotify",
  "dlg.urlPlaceholder": "ej.: open.spotify.com",
  "dlg.kiosk": "Modo kiosk (ventana sin barra del navegador)",
  "dlg.ublock": "uBlock Origin preinstalado",
  "dlg.presets": "Sugerencias",
  "dlg.deleteTitle": "Eliminar app",
  "dlg.deleteConfirm": "¿Eliminar “{name}”? El perfil y el acceso también se quitan.",
  "dlg.cancel": "Cancelar",
  "dlg.save": "Guardar",
  "dlg.delete": "Eliminar",

  "ext.title": "Extensiones (.xpi)",
  "ext.add": "Añadir .xpi",
  "ext.none": "Sin extensiones extra.",
  "ext.remove": "Quitar",
  "ext.afterSave": "Guarda la app antes de gestionar extensiones.",

  "toast.launched": "Abriendo {name}…",
  "toast.launchFail": "No se pudo abrir: {err}",
  "toast.saved": "App guardada",
  "toast.deleted": "“{name}” eliminada",
  "toast.shortcutCreated": "Acceso creado en el escritorio",
  "toast.shortcutRemoved": "Acceso quitado",
  "toast.err": "Error: {err}",

  "dlg.ok": "OK",

  "settings.title": "Configuración",
  "settings.theme": "Tema",
  "settings.themeSystem": "Sistema",
  "settings.themeLight": "Claro",
  "settings.themeDark": "Oscuro",
  "settings.themeNature": "Naturaleza",
  "settings.themeDarkBlue": "Azul oscuro",
  "settings.themeCalmGreen": "Verde tranquilo",
  "settings.themePastelPink": "Rosa pastel",
  "settings.themePunkPrincess": "PunkPrincess",
  "settings.language": "Idioma",
  "settings.about":
    " — abre sitios como apps locales: ventanas de Firefox con perfil dedicado, uBlock Origin por defecto y soporte de extensiones. Parte de la suite Local.",
};

const DICTS: Record<Locale, Record<MessageKey, string>> = { pt, en, es };

export function detectLocale(): Locale {
  const l = (typeof navigator !== "undefined" ? navigator.language : "pt").toLowerCase();
  if (l.startsWith("en")) return "en";
  if (l.startsWith("es")) return "es";
  return "pt";
}

function loadLocale(): Locale {
  const v = typeof localStorage !== "undefined" ? localStorage.getItem(LOCALE_KEY) : null;
  return v === "pt" || v === "en" || v === "es" ? v : detectLocale();
}

let current: Locale = loadLocale();
const listeners = new Set<() => void>();

export function getLocale(): Locale {
  return current;
}

export function setLocale(locale: Locale) {
  if (locale === current) return;
  current = locale;
  try {
    localStorage.setItem(LOCALE_KEY, locale);
  } catch {
    /* localStorage indisponível */
  }
  for (const l of listeners) l();
}

function subscribe(l: () => void) {
  listeners.add(l);
  return () => listeners.delete(l);
}

export function useLocale(): Locale {
  return useSyncExternalStore(subscribe, getLocale);
}

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  let msg: string = DICTS[current][key] ?? pt[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      msg = msg.split(`{${k}}`).join(String(v));
    }
  }
  return msg;
}
