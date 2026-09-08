import { invoke } from "@tauri-apps/api/core";

/** Wrapper tipado do backend (contrato fixo com src-tauri/src/lib.rs). */

export interface Extension {
  id: string;
  name: string;
  file: string;
}

export interface WebApp {
  id: string;
  name: string;
  url: string;
  kiosk: boolean;
  ublock: boolean;
  /** web app do Firefox (-taskbar-tab): janela própria, ícone do site */
  appMode: boolean;
  extensions: Extension[];
}

export interface Settings {
  firefoxPath: string | null;
  detectedFirefox: string | null;
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function setFirefoxPath(path: string | null): Promise<void> {
  return invoke("set_firefox_path", { path });
}

export async function listApps(): Promise<WebApp[]> {
  return invoke<WebApp[]>("list_apps");
}

export async function upsertApp(app: WebApp): Promise<WebApp> {
  return invoke<WebApp>("upsert_app", { webApp: app });
}

export async function deleteApp(id: string): Promise<void> {
  return invoke("delete_app", { id });
}

export async function launchApp(id: string): Promise<void> {
  return invoke("launch_app", { id });
}

export async function addExtension(id: string, name: string, srcPath: string): Promise<Extension> {
  return invoke<Extension>("add_extension", { id, name, srcPath });
}

export async function removeExtension(id: string, extId: string): Promise<void> {
  return invoke("remove_extension", { id, extId });
}

export async function shortcutExists(id: string): Promise<boolean> {
  return invoke<boolean>("shortcut_exists", { id });
}

export async function createShortcut(id: string): Promise<void> {
  return invoke("create_shortcut", { id });
}

export async function removeShortcut(id: string): Promise<void> {
  return invoke("remove_shortcut", { id });
}

// ---- helpers puros ----

/** nome → id de app: minúsculo, sem acentos, separadores em '-'. */
export function slugify(name: string): string {
  const base = name
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return base || "app";
}

/** URL sem esquema ganha https:// (o usuário cola "open.spotify.com"). */
export function normalizeUrl(url: string): string {
  const trimmed = url.trim();
  if (!trimmed) return trimmed;
  return trimmed.includes("://") ? trimmed : `https://${trimmed}`;
}

/** host da URL, pra casar com presets ("https://a.b/x" → "a.b"). */
export function hostOf(url: string): string {
  try {
    return new URL(normalizeUrl(url)).host.replace(/^www\./, "");
  } catch {
    return "";
  }
}
