//! Persistência do LocalBrowser: apps, settings e registro de atalhos.
//!
//! Layout em `app_data_dir()`:
//! - `apps.json` — lista de web apps
//! - `settings.json` — override do caminho do Firefox
//! - `shortcuts.json` — registro dos atalhos criados (`[{appId, path}]`)
//! - `ublock/ublock.ff.xpi` — cache do uBlock Origin
//! - `extensions/<appId>/<gecko-id>.xpi` — extensões extras por app
//! - `profiles/<appId>/` — perfil Firefox dedicado por app

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    /// id interno do Gecko (ex.: `uBlock0@raymondhill.net`)
    pub id: String,
    /// nome amigável dado pelo usuário
    pub name: String,
    /// nome do arquivo em `extensions/<appId>/` (sempre `<gecko-id>.xpi`)
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebApp {
    pub id: String,
    pub name: String,
    pub url: String,
    /// abre em modo kiosk (sem chrome do navegador)
    pub kiosk: bool,
    /// pré-instala o uBlock Origin no perfil
    pub ublock: bool,
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// override manual do caminho do firefox
    pub firefox_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutEntry {
    pub app_id: String,
    pub path: String,
}

/// Raiz de dados: passada pelos commands (AppHandle) ou inferida no modo `--launch`.
#[derive(Clone)]
pub struct Store {
    pub root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Raiz de dados sem AppHandle (modo `--launch`): segue o mesmo diretório
    /// que o Tauri usa para `com.localbrowser.app`.
    pub fn infer_root() -> Result<PathBuf, String> {
        let dir = if cfg!(windows) {
            let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA não definido")?;
            PathBuf::from(appdata).join("com.localbrowser.app")
        } else {
            let base = std::env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                    PathBuf::from(home).join(".local/share")
                });
            base.join("com.localbrowser.app")
        };
        Ok(dir)
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        for d in ["ublock", "extensions", "profiles"] {
            fs::create_dir_all(self.root.join(d))
                .map_err(|e| format!("criar pasta {}: {e}", d))?;
        }
        Ok(())
    }

    fn read_json<T: for<'de> Deserialize<'de>>(&self, file: &str, default: T) -> Result<T, String> {
        let path = self.root.join(file);
        if !path.exists() {
            return Ok(default);
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("ler {file}: {e}"))?;
        serde_json::from_str(&raw).map_err(|e| format!("parse {file}: {e}"))
    }

    fn write_json<T: Serialize>(&self, file: &str, value: &T) -> Result<(), String> {
        let path = self.root.join(file);
        let raw = serde_json::to_string_pretty(value).map_err(|e| format!("serializar {file}: {e}"))?;
        fs::write(&path, raw).map_err(|e| format!("gravar {file}: {e}"))
    }

    pub fn apps(&self) -> Result<Vec<WebApp>, String> {
        self.read_json("apps.json", Vec::new())
    }

    pub fn save_apps(&self, apps: &[WebApp]) -> Result<(), String> {
        self.write_json("apps.json", &apps)
    }

    pub fn settings(&self) -> Result<Settings, String> {
        self.read_json("settings.json", Settings::default())
    }

    pub fn save_settings(&self, s: &Settings) -> Result<(), String> {
        self.write_json("settings.json", &s)
    }

    pub fn shortcuts(&self) -> Result<Vec<ShortcutEntry>, String> {
        self.read_json("shortcuts.json", Vec::new())
    }

    pub fn save_shortcuts(&self, s: &[ShortcutEntry]) -> Result<(), String> {
        self.write_json("shortcuts.json", &s)
    }

    /// Upsert por id; valida campos básicos.
    pub fn upsert_app(&self, app: WebApp) -> Result<WebApp, String> {
        if app.id.trim().is_empty() {
            return Err("id vazio".into());
        }
        if app.id.contains('/') || app.id.contains('\\') {
            return Err("id contém separador de caminho".into());
        }
        if app.name.trim().is_empty() || app.url.trim().is_empty() {
            return Err("nome e URL são obrigatórios".into());
        }
        let mut apps = self.apps()?;
        match apps.iter().position(|a| a.id == app.id) {
            Some(i) => apps[i] = app.clone(),
            None => apps.push(app.clone()),
        }
        self.save_apps(&apps)?;
        Ok(app)
    }

    /// Remove app + perfil + extensões; devolve a entrada de atalho (se houver)
    /// para o chamador apagar o arquivo depois.
    pub fn delete_app(&self, id: &str) -> Result<Option<ShortcutEntry>, String> {
        let mut apps = self.apps()?;
        let before = apps.len();
        apps.retain(|a| a.id != id);
        if apps.len() == before {
            return Ok(None); // não existia — ok
        }
        self.save_apps(&apps)?;

        // perfil e extensões
        let _ = fs::remove_dir_all(self.root.join("profiles").join(id));
        let _ = fs::remove_dir_all(self.root.join("extensions").join(id));

        // atalho registrado
        let mut shortcuts = self.shortcuts()?;
        let entry = shortcuts.iter().find(|s| s.app_id == id).cloned();
        shortcuts.retain(|s| s.app_id != id);
        self.save_shortcuts(&shortcuts)?;
        Ok(entry)
    }

    pub fn find_app(&self, id: &str) -> Result<WebApp, String> {
        self.apps()?
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("app não encontrado: {id}"))
    }

    // ---- helpers de caminho ----

    pub fn profile_dir(&self, id: &str) -> PathBuf {
        self.root.join("profiles").join(id)
    }

    pub fn ext_store_dir(&self, id: &str) -> PathBuf {
        self.root.join("extensions").join(id)
    }

    pub fn ublock_cache(&self) -> PathBuf {
        self.root.join("ublock").join("ublock.ff.xpi")
    }
}
