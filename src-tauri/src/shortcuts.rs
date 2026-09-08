//! Atalhos na área de trabalho: `.lnk` no Windows (via WScript.Shell no
//! PowerShell, sem janela), `.desktop` no Linux (Desktop + applications).
//! O atalho aponta para ESTE exe com `--launch <id>`, então a resolução do
//! Firefox e a sincronização de extensões continuam funcionando.
//!
//! O registro do que foi criado vive em `shortcuts.json` (o Store cuida).

use crate::firefox::no_window;
use crate::store::{ShortcutEntry, Store};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn desktop_dir() -> Result<PathBuf, String> {
    if cfg!(windows) {
        // OneDrive pode redirecionar o Desktop — o PowerShell resolve certo.
        let out = Command::new("powershell")
            .args(["-NoProfile", "-Command", "[Environment]::GetFolderPath('Desktop') | Write-Output -NoEnumerate"])
            .output();
        let out = out.map_err(|e| format!("powershell: {e}"))?;
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if path.is_empty() {
            return Err("não consegui achar a área de trabalho".into());
        }
        Ok(PathBuf::from(path))
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME não definido")?;
        // XDG_DESKTOP_DIR aponta pra um arquivo .dirname; aceitamos os dois
        if let Ok(x) = std::env::var("XDG_DESKTOP_DIR") {
            let p = PathBuf::from(x);
            if p.is_dir() {
                return Ok(p);
            }
        }
        Ok(PathBuf::from(home).join("Desktop"))
    }
}

fn current_exe() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("exe atual: {e}"))
}

/// Cria o atalho e registra em shortcuts.json. O ícone do atalho é o favicon
/// do site (Google s2, 128px); se o download falhar, segue com o ícone padrão.
pub fn create(store: &Store, app_id: &str, app_name: &str, url: &str) -> Result<(), String> {
    let exe = current_exe()?;
    let desktop = desktop_dir()?;
    let icon = site_icon(store, app_id, url);

    if cfg!(windows) {
        // nome do arquivo legível; aspas simples escapadas dobrando '
        let file = desktop.join(format!("LocalBrowser — {app_name}.lnk"));
        let icon_line = match &icon {
            Ok(ico) => format!("$s.IconLocation='{}',0; ", ico.display().to_string().replace('\'', "''")),
            Err(_) => String::new(),
        };
        let ps = format!(
            "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}'); \
             $s.TargetPath='{}'; {icon_line}\
             $s.Arguments='--launch {app_id}'; $s.Save()",
            file.display().to_string().replace('\'', "''"),
            exe.display().to_string().replace('\'', "''"),
        );
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", &ps]);
        no_window(&mut cmd);
        let status = cmd.status().map_err(|e| format!("powershell: {e}"))?;
        if !status.success() {
            return Err("falha ao criar o atalho".into());
        }
        register(store, app_id, &file)?;
    } else {
        let file = desktop.join(format!("localbrowser-{app_id}.desktop"));
        let apps_dir = PathBuf::from(
            std::env::var("HOME").map_err(|_| "HOME não definido")?,
        )
        .join(".local/share/applications");
        let icon_name = match &icon {
            Ok(ico) => ico.display().to_string(),
            Err(_) => "firefox".to_owned(),
        };
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName=LocalBrowser — {app_name}\n\
             Exec=\"{}\" --launch {app_id}\nIcon={icon_name}\nTerminal=false\n",
            exe.display(),
        );
        fs::write(&file, &entry).map_err(|e| format!("criar .desktop: {e}"))?;
        let _ = fs::create_dir_all(&apps_dir);
        let _ = fs::write(apps_dir.join(format!("localbrowser-{app_id}.desktop")), &entry);
        register(store, app_id, &file)?;
    }
    Ok(())
}

/// Remove o atalho registrado (arquivo + registro).
pub fn remove(store: &Store, app_id: &str) -> Result<(), String> {
    let mut shortcuts = store.shortcuts()?;
    let entry = shortcuts.iter().find(|s| s.app_id == app_id).cloned();
    if let Some(e) = entry {
        let _ = fs::remove_file(&e.path);
        // irmão .desktop no menu de aplicativos (Linux)
        if !cfg!(windows) {
            if let Ok(home) = std::env::var("HOME") {
                let p = PathBuf::from(home)
                    .join(".local/share/applications")
                    .join(format!("localbrowser-{app_id}.desktop"));
                let _ = fs::remove_file(p);
            }
        }
    }
    shortcuts.retain(|s| s.app_id != app_id);
    store.save_shortcuts(&shortcuts)?;
    Ok(())
}

pub fn exists(store: &Store, app_id: &str) -> bool {
    store
        .shortcuts()
        .map(|s| s.iter().any(|e| e.app_id == app_id))
        .unwrap_or(false)
}

fn register(store: &Store, app_id: &str, path: &PathBuf) -> Result<(), String> {
    let mut shortcuts = store.shortcuts()?;
    shortcuts.retain(|s| s.app_id != app_id);
    shortcuts.push(ShortcutEntry {
        app_id: app_id.to_owned(),
        path: path.display().to_string(),
    });
    store.save_shortcuts(&shortcuts)
}

/// Baixa o favicon do site e devolve o caminho de um `.ico` (PNG-in-ICO,
/// válido Vista+). Guarda `icons/<id>.{png,ico}` em app_data; re-baixar a
/// cada create = atualizar se o site trocar de ícone. Download falhando,
/// cai no `.ico` já em cache (ícone é enfeite — cache velho > nenhum).
fn site_icon(store: &Store, app_id: &str, url: &str) -> Result<PathBuf, String> {
    let host = host_of(url).ok_or_else(|| "sem host na URL".to_owned())?;
    let dir = store.root.join("icons");
    fs::create_dir_all(&dir).map_err(|e| format!("criar icons: {e}"))?;
    let ico_path = dir.join(format!("{app_id}.ico"));

    match download_site_icon(&dir, app_id, &host, &ico_path) {
        Ok(()) => Ok(ico_path),
        Err(e) => {
            if ico_path.is_file() {
                Ok(ico_path) // cache de uma tentativa antiga
            } else {
                Err(e)
            }
        }
    }
}

/// Baixa o favicon (Google s2, 128px, PNG) e grava png + ico.
fn download_site_icon(
    dir: &Path,
    app_id: &str,
    host: &str,
    ico_path: &Path,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("http: {e}"))?;
    let resp = client
        .get(format!("https://www.google.com/s2/favicons?domain={host}&sz=128"))
        .send()
        .map_err(|e| format!("favicon: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("favicon: HTTP {}", resp.status()));
    }
    let png = resp.bytes().map_err(|e| format!("favicon: {e}"))?;
    // sanidade: é um PNG de verdade e não o placeholder minúsculo
    if png.len() < 200 || png[..8] != [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return Err("favicon não parece um PNG válido".into());
    }

    let png_path = dir.join(format!("{app_id}.png"));
    fs::write(&png_path, &png).map_err(|e| format!("gravar png: {e}"))?;

    // ICO = ICONDIR (6 bytes) + 1 entrada (16 bytes) + o PNG
    let mut ico: Vec<u8> = Vec::with_capacity(png.len() + 22);
    ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]); // reserved, type=icon, count=1
    ico.push(0);
    ico.push(0); // 256x256 (0 = 256)
    ico.push(0);
    ico.push(0); // paleta/reservado
    ico.extend_from_slice(&1u16.to_le_bytes()); // planos
    ico.extend_from_slice(&32u16.to_le_bytes()); // bpp
    ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes()); // offset após cabeçalhos
    ico.extend_from_slice(&png);
    fs::write(ico_path, &ico).map_err(|e| format!("gravar ico: {e}"))?;
    Ok(())
}

/// Host da URL (sem www), pedaço usado pra buscar o favicon.
fn host_of(url: &str) -> Option<String> {
    let no_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = no_scheme.split(['/', '?', '#']).next()?.split('@').last()?;
    let host = host.split(':').next()?;
    let host = host.trim().to_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_owned();
    if host.is_empty() { None } else { Some(host) }
}
