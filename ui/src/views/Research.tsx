import { useState } from "react";
import { api } from "../api.ts";
import { useI18n } from "../i18n.tsx";
import { useNav } from "../App.tsx";
import { Notice } from "../components/AsyncState.tsx";
import { MarkdownContent } from "../components/Markdown.tsx";

export function Research() {
  const { t } = useI18n();
  const nav = useNav();
  const [q, setQ] = useState("");
  const [busy, setBusy] = useState(false);
  const [res, setRes] = useState<{ answer: string; sources: { slug: string; title: string }[] } | null>(null);
  const [error, setError] = useState("");

  async function ask() {
    if (!q.trim()) return;
    setBusy(true); setError("");
    try { setRes(await api.query(q.trim())); }
    catch (e) { setError((e as Error).message); }
    finally { setBusy(false); }
  }

  return (
    <div className="view">
      <h1 className="h1">{t("research")}</h1>
      <div className="row" style={{ gap: 8 }}>
        <input value={q} placeholder={t("askPlaceholder")} onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && ask()} />
        <button className="btn primary" onClick={ask} disabled={busy}>{busy ? t("processing") : t("ask")}</button>
      </div>
      {error && <Notice tone="error">{t("askFailed")}: {error}</Notice>}
      {res && (
        <div className="panel" style={{ marginTop: 16 }}>
          <div className="section-title">{t("answer")}</div>
          <MarkdownContent className="answer reading">{res.answer}</MarkdownContent>
          {res.sources.length > 0 && (
            <div className="row" style={{ marginTop: 12 }}>
              {res.sources.map((s) => <button key={s.slug} className="wikilink" onClick={() => nav.openEntity(s.slug)}>[[{s.slug}]]</button>)}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
