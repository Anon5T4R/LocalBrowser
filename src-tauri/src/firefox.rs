//! Resolução do Firefox, perfis dedicados, uBlock Origin e extensões (.xpi).

use crate::store::{Store, WebApp};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// URL canônica do uBlock Origin (assinado, xpi do Firefox).
const UBLOCK_URL: &str = "https://github.com/gorhill/uBlock/releases/latest/download/uBlock0_1.ff.xpi";
/// Id interno do uBlock no Gecko (o nome do xpi dentro do perfil TEM que ser o id).
pub const UBLOCK_ID: &str = "uBlock0@raymondhill.net";

/// Suprimir janela de console em TODO spawn (firefox, powershell).
pub fn no_window(cmd: &mut Command) {
    let _ = cmd; // no Linux/macOS não há console pra suprimir
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
}

// ---- resolução do binário ----

/// Ordem: override do usuário → caminhos conhecidos / PATH.
pub fn resolve_firefox(store: &Store) -> Result<PathBuf, String> {
    if let Some(p) = store.settings()?.firefox_path {
        let pb = PathBuf::from(&p);
        if pb.is_file() {
            return Ok(pb);
        }
        return Err(format!("Firefox configurado não existe: {p}"));
    }
    detect_firefox().ok_or_else(|| "Firefox não encontrado — configure o caminho".into())
}

/// Auto-detecção sem override (None se não achar).
pub fn detect_firefox() -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = if cfg!(windows) {
        let mut v = Vec::new();
        for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
            if let Ok(base) = std::env::var(var) {
                v.push(PathBuf::from(base).join("Mozilla Firefox").join("firefox.exe"));
            }
        }
        v
    } else if cfg!(target_os = "macos") {
        vec![PathBuf::from("/Applications/Firefox.app/Contents/MacOS/firefox")]
    } else {
        let mut v: Vec<PathBuf> = ["/usr/bin", "/usr/local/bin", "/snap/bin", "/opt/firefox"]
            .iter()
            .flat_map(|d| {
                [
                    PathBuf::from(d).join("firefox"),
                    PathBuf::from(d).join("firefox-esr"),
                    PathBuf::from(d).join("firefox-dev"),
                ]
            })
            .collect();
        if let Ok(path) = std::env::var("PATH") {
            for dir in path.split(':') {
                for name in ["firefox", "firefox-esr", "firefox-dev"] {
                    v.push(PathBuf::from(dir).join(name));
                }
            }
        }
        v.dedup();
        v
    };
    candidates.into_iter().find(|p| p.is_file())
}

// ---- perfil ----

fn user_js() -> &'static str {
    "\
user_pref(\"extensions.autoDisableScopes\", 0);
user_pref(\"extensions.startupScanScopes\", 15);
user_pref(\"browser.shell.checkDefaultBrowser\", false);
user_pref(\"browser.aboutwelcome.enabled\", false);
user_pref(\"datareporting.policy.dataSubmissionEnabled\", false);
user_pref(\"toolkit.legacyUserProfileCustomizations.stylesheets\", true);
"
}

/// Cria/atualiza o perfil dedicado do app (idempotente).
pub fn ensure_profile(store: &Store, app: &WebApp) -> Result<PathBuf, String> {
    let dir = store.profile_dir(&app.id);
    fs::create_dir_all(dir.join("extensions"))
        .map_err(|e| format!("criar perfil: {e}"))?;
    fs::write(dir.join("user.js"), user_js()).map_err(|e| format!("escrever user.js: {e}"))?;
    Ok(dir)
}

// ---- uBlock ----

/// Garante o xpi do uBlock em cache (baixa se faltar). Retorna o caminho.
pub fn ensure_ublock(store: &Store) -> Result<PathBuf, String> {
    let cache = store.ublock_cache();
    if cache.is_file() {
        return Ok(cache);
    }
    fs::create_dir_all(cache.parent().unwrap_or(&store.root))
        .map_err(|e| format!("criar pasta ublock: {e}"))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("http: {e}"))?;
    let resp = client
        .get(UBLOCK_URL)
        .send()
        .map_err(|e| format!("baixar uBlock: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("baixar uBlock: HTTP {}", resp.status()));
    }
    let bytes = resp.bytes().map_err(|e| format!("baixar uBlock: {e}"))?;
    // sanidade: é um zip e tem tamanho crível (>100 KB)
    if bytes.len() < 100_000 || bytes.first() != Some(&b'P') || bytes.get(1) != Some(&b'K') {
        return Err("download do uBlock não parece um xpi válido".into());
    }
    fs::write(&cache, &bytes).map_err(|e| format!("gravar uBlock: {e}"))?;
    Ok(cache)
}

/// Copia o uBlock (se habilitado) e as extensões extras para o perfil.
/// Sobrescrever = atualizar a versão em lançamentos futuros.
pub fn sync_extensions(store: &Store, app: &WebApp) -> Result<(), String> {
    let ext_dir = store.profile_dir(&app.id).join("extensions");
    fs::create_dir_all(&ext_dir).map_err(|e| format!("criar extensions: {e}"))?;

    if app.ublock {
        let src = ensure_ublock(store)?;
        let dst = ext_dir.join(format!("{UBLOCK_ID}.xpi"));
        fs::copy(&src, &dst).map_err(|e| format!("instalar uBlock: {e}"))?;
    } else {
        // desligou o uBlock → remove do perfil se estava lá
        let _ = fs::remove_file(ext_dir.join(format!("{UBLOCK_ID}.xpi")));
    }

    for ext in &app.extensions {
        let src = store.ext_store_dir(&app.id).join(&ext.file);
        if src.is_file() {
            let dst = ext_dir.join(&ext.file);
            fs::copy(&src, &dst).map_err(|e| format!("instalar {}: {e}", ext.name))?;
        }
    }
    Ok(())
}

// ---- id interno de um xpi ----

fn read_zip_entry(path: &Path, name: &str) -> Result<Vec<u8>, String> {
    let f = fs::File::open(path).map_err(|e| format!("abrir xpi: {e}"))?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| format!("abrir xpi: {e}"))?;
    let mut entry = z.by_name(name).map_err(|_| format!("xpi sem {name}"))?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf).map_err(|e| format!("ler {name}: {e}"))?;
    Ok(buf)
}

fn gecko_id_from_manifest(raw: &[u8]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(raw).ok()?;
    v.pointer("/browser_specific_settings/gecko/id")
        .or_else(|| v.pointer("/applications/gecko/id"))?
        .as_str()
        .map(str::to_owned)
}

fn gecko_id_from_rdf(raw: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(raw);
    // em:id="..." (formato antigo, atributo) ou <em:id>...</em:id>
    let attr = s
        .split("em:id=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next());
    let el = s
        .split("<em:id>")
        .nth(1)
        .and_then(|rest| rest.split("</em:id>").next());
    attr.or(el).map(|id| id.trim().to_owned())
}

/// Lê o id interno (gecko.id / em:id) de um .xpi arbitrário.
pub fn read_gecko_id(xpi: &Path) -> Result<String, String> {
    if let Ok(raw) = read_zip_entry(xpi, "manifest.json") {
        if let Some(id) = gecko_id_from_manifest(&raw) {
            return Ok(id);
        }
    }
    if let Ok(raw) = read_zip_entry(xpi, "install.rdf") {
        if let Some(id) = gecko_id_from_rdf(&raw) {
            return Ok(id);
        }
    }
    Err("não achei o id interno da extensão (manifest.json/install.rdf)".into())
}

// ---- launch ----

/// Prepara o perfil e dispara o Firefox. Não espera o processo.
pub fn launch(store: &Store, app: &WebApp) -> Result<(), String> {
    let firefox = resolve_firefox(store)?;
    ensure_profile(store, app)?;
    sync_extensions(store, app)?;

    let dir = store.profile_dir(&app.id);
    let mut cmd = Command::new(&firefox);
    cmd.arg("-no-remote").arg("-profile").arg(&dir);
    if app.kiosk {
        cmd.arg("-kiosk");
    }
    cmd.arg(&app.url);
    no_window(&mut cmd);
    cmd.spawn()
        .map_err(|e| format!("iniciar Firefox: {e}"))?;
    Ok(())
}
