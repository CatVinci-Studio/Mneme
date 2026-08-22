import { useEffect, useState } from "react";
import { api, type SourceMeta } from "../api.ts";
import { useI18n } from "../i18n.tsx";
import { useNav } from "../App.tsx";
import { ErrorState, LoadingState } from "../components/AsyncState.tsx";

export function Queue() {
  const { t, lang } = useI18n();
  const nav = useNav();
  const [items, setItems] = useState<SourceMeta[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  async function load() {
    setLoading(true); setError("");
    try { setItems(await api.listSources()); }
    catch (e) { setError((e as Error).message); }
    finally { setLoading(false); }
  }
  useEffect(() => { void load(); }, [nav.refreshVersion]);

  if (loading) return <LoadingState label={t("loading")} />;
  if (error) return <ErrorState message={`${t("loadFailed")}: ${error}`} retry={() => void load()} />;
  return (
    <div className="view content-page">
      <h1 className="h1">{t("queue")}</h1>
      {items.length === 0 && <div className="empty-state"><strong>{t("empty")}</strong></div>}
      <div className="card-list">
        {items.map((source) => (
          <button className="card source-card" key={source.id} onClick={() => nav.openSource(source.id)}>
            <h3>{source.title}</h3>
            <div className="row">
              <span className="badge kind">{source.kind}</span>
              {source.summarized && <span className="badge ok">{t("summarized")}</span>}
              <span className="muted">{source.word_count} {t("words")} · {formatDate(source.fetched_at, lang)}</span>
              {hostname(source.url) && <span className="muted">· {hostname(source.url)}</span>}
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}

function hostname(url?: string): string {
  if (!url) return "";
  try { return new URL(url).hostname; } catch { return ""; }
}
function formatDate(value: string, lang: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : new Intl.DateTimeFormat(lang === "zh" ? "zh-CN" : "en", { dateStyle: "medium" }).format(date);
}
