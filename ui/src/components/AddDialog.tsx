import { useEffect, useRef, useState } from "react";
import { api } from "../api.ts";
import { useI18n } from "../i18n.tsx";

export function AddDialog({ onClose, onDone }: { onClose: () => void; onDone: () => void }) {
  const { t } = useI18n();
  const [url, setUrl] = useState("");
  const [text, setText] = useState("");
  const [title, setTitle] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const urlRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    urlRef.current?.focus();
    const onKey = (event: KeyboardEvent) => { if (event.key === "Escape" && !busy) onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [busy, onClose]);

  async function submit() {
    setBusy(true); setErr("");
    try {
      await api.addSource(url.trim() ? { url: url.trim(), title: title || undefined } : { text, title: title || undefined });
      onDone();
    } catch (e) { setErr((e as Error).message); setBusy(false); }
  }

  return (
    <div className="overlay" onMouseDown={onClose}>
      <div className="dialog" role="dialog" aria-modal="true" aria-labelledby="add-dialog-title" onMouseDown={(e) => e.stopPropagation()}>
        <h2 id="add-dialog-title">{t("addTitle")}</h2>
        <div className="field">
          <label htmlFor="add-url">{t("url")}</label>
          <input id="add-url" ref={urlRef} value={url} onChange={(e) => setUrl(e.target.value)} placeholder="https://…" />
        </div>
        <div className="field">
          <label htmlFor="add-text">{t("orPasteText")}</label>
          <textarea id="add-text" rows={6} value={text} onChange={(e) => setText(e.target.value)} disabled={!!url.trim()} />
        </div>
        <div className="field">
          <label htmlFor="add-title">{t("titleOpt")}</label>
          <input id="add-title" value={title} onChange={(e) => setTitle(e.target.value)} />
        </div>
        {err && <p style={{ color: "var(--warn)" }}>{err}</p>}
        <div className="actions">
          <button className="btn" onClick={onClose} disabled={busy}>{t("cancel")}</button>
          <button className="btn primary" onClick={submit} disabled={busy || (!url.trim() && !text.trim())}>
            {busy ? t("processing") : t("save")}
          </button>
        </div>
      </div>
    </div>
  );
}
