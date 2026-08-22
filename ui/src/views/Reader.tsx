import { useEffect, useState } from "react";
import { api, type SourceDetail } from "../api.ts";
import { useI18n } from "../i18n.tsx";
import { useNav, type SourceRange } from "../App.tsx";
import { ErrorState, LoadingState, Notice } from "../components/AsyncState.tsx";
import { MarkdownContent } from "../components/Markdown.tsx";

export function Reader({ id, range }: { id: string | null; range: SourceRange | null }) {
  const { t } = useI18n();
  const nav = useNav();
  const [data, setData] = useState<SourceDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<"" | "success">("");

  async function load() {
    if (!id) return;
    setLoading(true); setError("");
    try { setData(await api.getSource(id)); }
    catch (e) { setError((e as Error).message); setData(null); }
    finally { setLoading(false); }
  }
  useEffect(() => { void load(); }, [id]);

  if (!id) return <ErrorState message={t("sourceNotFound")} retry={nav.back} />;
  if (loading) return <LoadingState label={t("loading")} />;
  if (error || !data) return <ErrorState message={`${t("sourceNotFound")}: ${error}`} retry={() => void load()} />;

  async function doWikify() {
    setBusy(true); setError(""); setResult("");
    try {
      await api.wikify(id!);
      await load();
      nav.refresh(); nav.notify(t("wikifySuccess")); setResult("success");
    } catch (e) { setError((e as Error).message); }
    finally { setBusy(false); }
  }

  const excerpt = range ? data.content.slice(Math.max(0, range.start), Math.min(data.content.length, range.end)) : "";
  return (
    <div className="view">
      <button className="back-link" onClick={nav.back}>← {t("queue")}</button>
      <div className="page-heading">
        <div><span className="eyebrow">{data.meta.kind}</span><h1 className="h1">{data.meta.title}</h1></div>
        <button className="btn primary" onClick={() => void doWikify()} disabled={busy}>{busy ? t("processing") : t("wikify")}</button>
      </div>
      {error && <Notice tone="error">{error}</Notice>}
      {result && <Notice>{t("wikifySuccess")}</Notice>}
      {excerpt && (
        <aside className="reference-excerpt">
          <strong>{t("referenceExcerpt")}</strong>
          <mark>{excerpt}</mark>
        </aside>
      )}
      <div className="split">
        <article className="panel">
          <div className="section-title">{t("reading")}</div>
          <MarkdownContent className="reading">{data.content}</MarkdownContent>
        </article>
        <article className="panel">
          <div className="section-title">{t("note")}</div>
          {data.note ? <MarkdownContent className="reading note-content">{stripFrontmatter(data.note)}</MarkdownContent>
            : <p className="muted">{t("processing")}</p>}
        </article>
      </div>
    </div>
  );
}

function stripFrontmatter(markdown: string): string {
  return markdown.replace(/^---[\s\S]*?---\n+/, "");
}
