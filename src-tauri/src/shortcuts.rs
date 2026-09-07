//! Atalhos na área de trabalho: `.lnk` no Windows (via WScript.Shell no
//! PowerShell, sem janela), `.desktop` no Linux (Desktop + applications).
//! O atalho aponta para ESTE exe com `--launch <id>`, então a resolução do
//! Firefox e a sincronização de extensões continuam funcionando.
//!
//! O registro do que foi criado vive em `shortcuts.json` (o Store cuida).

use crate::firefox::no_window;
use crate::store::{ShortcutEntry, Store};
use std::fs;
use std::path::PathBuf;
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

/// Cria o atalho e registra em shortcuts.json.
pub fn create(store: &Store, app_id: &str, app_name: &str) -> Result<(), String> {
    let exe = current_exe()?;
    let desktop = desktop_dir()?;

    if cfg!(windows) {
        // nome do arquivo legível; aspas simples escapadas dobrando '
        let file = desktop.join(format!("LocalBrowser — {app_name}.lnk"));
        let ps = format!(
            "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}'); \
             $s.TargetPath='{}'; $s.Arguments='--launch {app_id}'; $s.Save()",
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
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName=LocalBrowser — {app_name}\n\
             Exec=\"{}\" --launch {app_id}\nIcon=firefox\nTerminal=false\n",
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
