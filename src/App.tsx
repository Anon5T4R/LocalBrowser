import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import SettingsModal from "./components/SettingsModal";
import Toasts from "./components/Toasts";
import { PRESETS, type Preset } from "./lib/catalog";
import {
  addExtension,
  createShortcut,
  deleteApp,
  getSettings,
  hostOf,
  launchApp,
  listApps,
  normalizeUrl,
  removeExtension,
  removeShortcut,
  setFirefoxPath,
  shortcutExists,
  slugify,
  upsertApp,
  type Settings,
  type WebApp,
} from "./lib/backend";
import { t } from "./lib/i18n";
import { useUi } from "./state/ui";

/** App.tsx — grid de web apps + banner do Firefox. Subcomponentes no nível do
 * módulo (gotcha da suíte: subcomponente interno = remontagem a cada render). */

export default function App() {
  const ui = useUi();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [apps, setApps] = useState<WebApp[]>([]);
  const [shortcutIds, setShortcutIds] = useState<Set<string>>(new Set());
  const [editing, setEditing] = useState<WebApp | null>(null); // null = fechado
  const [adding, setAdding] = useState(false);
  const [deleting, setDeleting] = useState<WebApp | null>(null);

  const err = (e: unknown) => ui.pushToast("error", t("toast.err", { err: String(e) }));

  const reload = useCallback(async () => {
    try {
      const list = await listApps();
      setApps(list);
      const flags = await Promise.all(list.map((a) => shortcutExists(a.id)));
      setShortcutIds(new Set(list.filter((_, i) => flags[i]).map((a) => a.id)));
    } catch (e) {
      err(e);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    reload();
    getSettings().then(setSettings).catch(err);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const pickFirefox = async () => {
    try {
      const picked = await open({ multiple: false, directory: false, title: "Firefox" });
      if (typeof picked === "string") {
        await setFirefoxPath(picked);
        setSettings(await getSettings());
      }
    } catch (e) {
      err(e);
    }
  };

  const launch = async (a: WebApp) => {
    try {
      await launchApp(a.id);
      ui.pushToast("ok", t("toast.launched", { name: a.name }));
    } catch (e) {
      ui.pushToast("error", t("toast.launchFail", { err: String(e) }));
    }
  };

  const toggleShortcut = async (a: WebApp) => {
    try {
      if (shortcutIds.has(a.id)) {
        await removeShortcut(a.id);
        setShortcutIds((s) => {
          const n = new Set(s);
          n.delete(a.id);
          return n;
        });
        ui.pushToast("info", t("toast.shortcutRemoved"));
      } else {
        await createShortcut(a.id);
        setShortcutIds((s) => new Set(s).add(a.id));
        ui.pushToast("ok", t("toast.shortcutCreated"));
      }
    } catch (e) {
      err(e);
    }
  };

  const save = async (a: WebApp) => {
    try {
      await upsertApp(a);
      ui.pushToast("ok", t("toast.saved"));
      setEditing(null);
      setAdding(false);
      await reload();
    } catch (e) {
      err(e);
    }
  };

  const confirmDelete = async () => {
    if (!deleting) return;
    const a = deleting;
    setDeleting(null);
    try {
      await deleteApp(a.id);
      ui.pushToast("info", t("toast.deleted", { name: a.name }));
      await reload();
    } catch (e) {
      err(e);
    }
  };

  const firefoxMissing =
    settings !== null && !settings.detectedFirefox && !settings.firefoxPath;
  const firefoxManual = settings?.firefoxPath ?? null;

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="brand-name">LocalBrowser</span>
          <span className="muted small">{t("app.subtitle")}</span>
        </div>
        <div className="toolbar-fill" />
        {firefoxManual && (
          <span className="muted small firefox-path" title={firefoxManual}>
            {t("banner.firefoxManual", { path: firefoxManual.split(/[\\/]/).pop() ?? firefoxManual })}
          </span>
        )}
        <button className="primary" onClick={() => setAdding(true)}>
          + {t("top.add")}
        </button>
        <button
          onClick={() => ui.setSettingsOpen(true)}
          title={t("top.settings")}
          aria-label={t("top.settings")}
        >
          ⚙
        </button>
      </header>

      {firefoxMissing && (
        <div className="banner-warn">
          <span>⚠ {t("banner.firefoxMissing")}</span>
          <button onClick={pickFirefox}>{t("banner.firefoxPick")}</button>
        </div>
      )}

      <main className="grid">
        {apps.length === 0 ? (
          <div className="empty">
            <p className="big-empty">{t("grid.empty")}</p>
            <p className="muted">{t("grid.emptyHint")}</p>
          </div>
        ) : (
          apps.map((a) => (
            <AppCard
              key={a.id}
              app={a}
              hasShortcut={shortcutIds.has(a.id)}
              onLaunch={() => launch(a)}
              onToggleShortcut={() => toggleShortcut(a)}
              onEdit={() => setEditing(a)}
              onDelete={() => setDeleting(a)}
            />
          ))
        )}
      </main>

      {adding && (
        <EditModal
          existing={null}
          takenIds={apps.map((a) => a.id)}
          onCancel={() => setAdding(false)}
          onSave={save}
          onError={err}
        />
      )}
      {editing && (
        <EditModal
          existing={editing}
          takenIds={apps.filter((a) => a.id !== editing.id).map((a) => a.id)}
          onCancel={() => setEditing(null)}
          onSave={save}
          onError={err}
        />
      )}
      {deleting && (
        <div className="modal-backdrop" onClick={() => setDeleting(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>{t("dlg.deleteTitle")}</h2>
            <p>{t("dlg.deleteConfirm", { name: deleting.name })}</p>
            <div className="modal-actions">
              <button onClick={() => setDeleting(null)}>{t("dlg.cancel")}</button>
              <button className="danger" onClick={confirmDelete}>
                {t("dlg.delete")}
              </button>
            </div>
          </div>
        </div>
      )}

      <SettingsModal />
      <Toasts />
    </div>
  );
}

/** Emoji do preset que casa com o host da URL, senão a inicial do nome. */
function iconFor(app: WebApp): string {
  const host = hostOf(app.url);
  const preset = PRESETS.find((p) => hostOf(p.url) === host);
  if (preset) return preset.emoji;
  return (app.name.trim()[0] ?? "?").toUpperCase();
}

function AppCard(props: {
  app: WebApp;
  hasShortcut: boolean;
  onLaunch: () => void;
  onToggleShortcut: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const { app, hasShortcut } = props;
  return (
    <div className="card app-card">
      <div className="app-card-head">
        <span className="app-icon" aria-hidden>
          {iconFor(app)}
        </span>
        <div className="app-card-title">
          <strong>{app.name}</strong>
          <span className="muted small ellipsis">{hostOf(app.url) || app.url}</span>
        </div>
      </div>
      <div className="badges">
        {app.appMode && <span className="badge">{t("badge.app")}</span>}
        {app.ublock && <span className="badge">{t("badge.ublock")}</span>}
        {app.kiosk && <span className="badge">{t("badge.kiosk")}</span>}
        {app.extensions.length > 0 && (
          <span className="badge">{t("badge.exts", { n: app.extensions.length })}</span>
        )}
      </div>
      <div className="app-card-actions">
        <button className="primary" onClick={props.onLaunch}>
          {t("card.launch")}
        </button>
        <button
          className={hasShortcut ? "active" : ""}
          title={hasShortcut ? t("card.shortcutRemove") : t("card.shortcut")}
          onClick={props.onToggleShortcut}
        >
          {hasShortcut ? "★" : "☆"}
        </button>
        <button onClick={props.onEdit}>{t("card.edit")}</button>
        <button onClick={props.onDelete} aria-label={t("card.delete")}>
          🗑
        </button>
      </div>
    </div>
  );
}

function EditModal(props: {
  existing: WebApp | null;
  takenIds: string[];
  onCancel: () => void;
  onSave: (app: WebApp) => void;
  onError: (e: unknown) => void;
}) {
  const { existing, takenIds } = props;
  const [name, setName] = useState(existing?.name ?? "");
  const [url, setUrl] = useState(existing?.url ?? "");
  const [kiosk, setKiosk] = useState(existing?.kiosk ?? false);
  const [ublock, setUblock] = useState(existing?.ublock ?? true);
  const [appMode, setAppMode] = useState(existing?.appMode ?? true);
  const [exts, setExts] = useState(existing?.extensions ?? []);

  const pickPreset = (p: Preset) => {
    setName(p.name);
    setUrl(p.url);
  };

  const submit = () => {
    const id =
      existing?.id ??
      (() => {
        let base = slugify(name);
        let id = base;
        let n = 2;
        while (takenIds.includes(id)) id = `${base}-${n++}`;
        return id;
      })();
    props.onSave({
      id,
      name: name.trim(),
      url: normalizeUrl(url),
      kiosk,
      ublock,
      appMode,
      extensions: exts,
    });
  };

  const addExt = async () => {
    if (!existing) return;
    try {
      const picked = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Firefox extension", extensions: ["xpi"] }],
      });
      if (typeof picked === "string") {
        const fileName = picked.split(/[\\/]/).pop() ?? "ext";
        const ext = await addExtension(existing.id, fileName.replace(/\.xpi$/i, ""), picked);
        setExts((v) => [...v, ext]);
      }
    } catch (e) {
      props.onError(e);
    }
  };

  const removeExt = async (extId: string) => {
    if (!existing) return;
    try {
      await removeExtension(existing.id, extId);
      setExts((v) => v.filter((e) => e.id !== extId));
    } catch (e) {
      props.onError(e);
    }
  };

  const valid = name.trim().length > 0 && url.trim().length > 0;

  return (
    <div className="modal-backdrop" onClick={props.onCancel}>
      <div className="modal edit-modal" onClick={(e) => e.stopPropagation()}>
        <h2>{existing ? t("dlg.edit") : t("dlg.add")}</h2>

        {!existing && (
          <>
            <p className="muted small">{t("dlg.presets")}</p>
            <div className="preset-grid">
              {PRESETS.map((p) => (
                <button key={p.id} className="preset-btn" onClick={() => pickPreset(p)}>
                  <span aria-hidden>{p.emoji}</span> {p.name}
                </button>
              ))}
            </div>
          </>
        )}

        <label className="field">
          <span>{t("dlg.name")}</span>
          <input
            value={name}
            placeholder={t("dlg.namePlaceholder")}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <label className="field">
          <span>{t("dlg.url")}</span>
          <input
            value={url}
            placeholder={t("dlg.urlPlaceholder")}
            onChange={(e) => setUrl(e.target.value)}
          />
        </label>

        <label className="check">
          <input
            type="checkbox"
            checked={appMode}
            onChange={(e) => {
              setAppMode(e.target.checked);
              if (e.target.checked) setKiosk(false); // modo app já é janela limpa
            }}
          />
          {t("dlg.appMode")}
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={kiosk}
            disabled={appMode}
            onChange={(e) => setKiosk(e.target.checked)}
          />
          {t("dlg.kiosk")}
        </label>
        <label className="check">
          <input type="checkbox" checked={ublock} onChange={(e) => setUblock(e.target.checked)} />
          {t("dlg.ublock")}
        </label>

        <div className="ext-section">
          <div className="ext-head">
            <span className="muted small">{t("ext.title")}</span>
            {existing ? (
              <button className="small" onClick={addExt}>
                {t("ext.add")}
              </button>
            ) : null}
          </div>
          {!existing && <p className="muted small">{t("ext.afterSave")}</p>}
          {existing && exts.length === 0 && <p className="muted small">{t("ext.none")}</p>}
          {exts.map((e) => (
            <div key={e.id} className="ext-row">
              <span className="ellipsis">{e.name}</span>
              <button className="small" onClick={() => removeExt(e.id)}>
                {t("ext.remove")}
              </button>
            </div>
          ))}
        </div>

        <div className="modal-actions">
          <button onClick={props.onCancel}>{t("dlg.cancel")}</button>
          <button className="primary" disabled={!valid} onClick={submit}>
            {t("dlg.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
