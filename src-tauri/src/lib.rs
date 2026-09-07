//! LocalBrowser — sites como apps locais, sobre Firefox + uBlock Origin.

mod firefox;
mod shortcuts;
mod store;

use serde::Serialize;
use store::{Extension, Store, WebApp};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsView {
    firefox_path: Option<String>,
    detected_firefox: Option<String>,
}

fn store_of(app: &AppHandle) -> Result<Store, String> {
    let dir: PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let s = Store::new(dir);
    s.ensure_dirs()?;
    Ok(s)
}

fn detected_string() -> Option<String> {
    firefox::detect_firefox().map(|p| p.display().to_string())
}

#[tauri::command]
async fn get_settings(app: AppHandle) -> Result<SettingsView, String> {
    let s = store_of(&app)?;
    Ok(SettingsView {
        firefox_path: s.settings()?.firefox_path,
        detected_firefox: detected_string(),
    })
}

#[tauri::command]
async fn set_firefox_path(app: AppHandle, path: Option<String>) -> Result<(), String> {
    let s = store_of(&app)?;
    let mut st = s.settings()?;
    st.firefox_path = path;
    s.save_settings(&st)
}

#[tauri::command]
async fn list_apps(app: AppHandle) -> Result<Vec<WebApp>, String> {
    store_of(&app)?.apps()
}

#[tauri::command]
async fn upsert_app(
    app: AppHandle,
    web_app: WebApp,
    _lock: State<'_, Mutex<()>>,
) -> Result<WebApp, String> {
    let _g = _lock.lock().map_err(|_| "lock interno")?;
    store_of(&app)?.upsert_app(web_app)
}

#[tauri::command]
async fn delete_app(
    app: AppHandle,
    id: String,
    _lock: State<'_, Mutex<()>>,
) -> Result<(), String> {
    let _g = _lock.lock().map_err(|_| "lock interno")?;
    let s = store_of(&app)?;
    if let Some(entry) = s.delete_app(&id)? {
        // atalho registrado → apagar o arquivo também
        let _ = std::fs::remove_file(&entry.path);
    }
    Ok(())
}

#[tauri::command]
async fn launch_app(app: AppHandle, id: String) -> Result<(), String> {
    let s = store_of(&app)?;
    let wa = s.find_app(&id)?;
    firefox::launch(&s, &wa)
}

#[tauri::command]
async fn add_extension(
    app: AppHandle,
    id: String,
    name: String,
    src_path: String,
    _lock: State<'_, Mutex<()>>,
) -> Result<Extension, String> {
    let _g = _lock.lock().map_err(|_| "lock interno")?;
    let s = store_of(&app)?;
    let gecko_id = firefox::read_gecko_id(&PathBuf::from(&src_path))?;

    let file = format!("{gecko_id}.xpi");
    let dst_dir = s.ext_store_dir(&id);
    std::fs::create_dir_all(&dst_dir).map_err(|e| format!("criar pasta: {e}"))?;
    std::fs::copy(&src_path, dst_dir.join(&file)).map_err(|e| format!("copiar xpi: {e}"))?;

    let mut apps = s.apps()?;
    let wa = apps
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("app não encontrado: {id}"))?;
    let ext = Extension {
        id: gecko_id,
        name,
        file,
    };
    wa.extensions.push(ext.clone());
    s.save_apps(&apps)?;
    Ok(ext)
}

#[tauri::command]
async fn remove_extension(
    app: AppHandle,
    id: String,
    ext_id: String,
    _lock: State<'_, Mutex<()>>,
) -> Result<(), String> {
    let _g = _lock.lock().map_err(|_| "lock interno")?;
    let s = store_of(&app)?;
    let mut apps = s.apps()?;
    let wa = apps
        .iter_mut()
        .find(|a| a.id == id)
        .ok_or_else(|| format!("app não encontrado: {id}"))?;
    wa.extensions.retain(|e| e.id != ext_id);
    s.save_apps(&apps)?;

    // remove do estoque e do perfil
    let file = format!("{ext_id}.xpi");
    let _ = std::fs::remove_file(s.ext_store_dir(&id).join(&file));
    let _ = std::fs::remove_file(s.profile_dir(&id).join("extensions").join(&file));
    Ok(())
}

#[tauri::command]
async fn shortcut_exists(app: AppHandle, id: String) -> Result<bool, String> {
    Ok(shortcuts::exists(&store_of(&app)?, &id))
}

#[tauri::command]
async fn create_shortcut(app: AppHandle, id: String) -> Result<(), String> {
    let s = store_of(&app)?;
    let wa = s.find_app(&id)?;
    shortcuts::create(&s, &wa.id, &wa.name)
}

#[tauri::command]
async fn remove_shortcut(app: AppHandle, id: String) -> Result<(), String> {
    shortcuts::remove(&store_of(&app)?, &id)
}

/// Modo `--launch <id>`: dispara o web app sem abrir janela (é o que os
/// atalhos da área de trabalho chamam).
fn run_headless_launch() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let id = args
        .iter()
        .position(|a| a == "--launch")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .ok_or("--launch sem id")?;

    let root = Store::infer_root()?;
    let store = Store::new(root);
    store.ensure_dirs()?;
    let wa = store.find_app(&id)?;
    firefox::launch(&store, &wa)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Atalho da área de trabalho: `LocalBrowser.exe --launch spotify`
    if std::env::args().any(|a| a == "--launch") {
        match run_headless_launch() {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("localbrowser --launch: {e}");
                std::process::exit(1);
            }
        }
    }

    let mut builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 2ª instância: foca a janela da 1ª
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }));
    }

    builder
        .setup(|_app| {
            _app.manage(Mutex::new(()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_firefox_path,
            list_apps,
            upsert_app,
            delete_app,
            launch_app,
            add_extension,
            remove_extension,
            shortcut_exists,
            create_shortcut,
            remove_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("erro ao rodar o LocalBrowser");
}
