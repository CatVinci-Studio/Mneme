import { useEffect, useState } from "react";
import { api, type EntityPage, type EntitySummary } from "../api.ts";
import { useI18n } from "../i18n.tsx";
import { useNav } from "../App.tsx";
import { ErrorState, LoadingState } from "../components/AsyncState.tsx";

export function Entity({ slug }: { slug: string | null }) {
  if (!slug) return <EntityList />;
  return <EntityDetail slug={slug} />;
}

function EntityList() {
  const { t } = useI18n();
  const nav = useNav();
  const [items, setItems] = useState<EntitySummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  async function load() {
    setLoading(true); setError("");
    try { setItems(await api.listEntities()); }
    catch (e) { setError((e as Error).message); }
    finally { setLoading(false); }
  }
  useEffect(() => { void load(); }, [nav.refreshVersion]);
  if (loading) return <LoadingState label={t("loading")} />;
  if (error) return <ErrorState message={`${t("loadFailed")}: ${error}`} retry={() => void load()} />;
  return (
    <div className="view content-page">
      <div className="page-heading">
        <h1 className="h1">{t("wiki")}</h1>
        {items.length > 1 && <button className="btn" onClick={() => nav.go("graph")}>{t("graph")} →</button>}
      </div>
      {items.length === 0 && <div className="empty-state"><strong>{t("noEntities")}</strong></div>}
      <div className="card-list">
        {items.map((entity) => (
          <button className="card entity-card" key={entity.slug} onClick={() => nav.openEntity(entity.slug)}>
            <h3>{entity.title} <span className="badge kind">{entity.kind}</span></h3>
            <span className="muted">{entity.summary}</span>
          </button>
        ))}
      </div>
    </div>
  );
}

function EntityDetail({ slug }: { slug: string }) {
  const { t } = useI18n();
  const nav = useNav();
  const [data, setData] = useState<{ page: EntityPage; backlinks: string[] } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  async function load() {
    setLoading(true); setError("");
    try { setData(await api.getEntity(slug)); }
    catch (e) { setError((e as Error).message); setData(null); }
    finally { setLoading(false); }
  }
  useEffect(() => { void load(); }, [slug, nav.refreshVersion]);
  if (loading) return <LoadingState label={t("loading")} />;
  if (error || !data) return <ErrorState message={`${t("entityNotFound")}: ${error}`} retry={() => void load()} />;
  const page = data.page;

  return (
    <div className="view">
      <button className="back-link" onClick={nav.back}>← {t("wiki")}</button>
      <div className="entity-detail">
        <article>
          <div className="page-heading compact">
            <div><span className="eyebrow">{page.kind}</span><h1 className="h1">{page.title}</h1></div>
          </div>
          <p className="reading entity-summary">{page.summary}</p>

          <div className="section-title">{t("facts")}</div>
          <ul className="facts reading">
            {page.facts.map((fact) => (
              <li key={fact.id}>{fact.text}
                <button className="provenance" title={`src:${fact.prov.source_id}@${fact.prov.start}-${fact.prov.end}`}
                  aria-label={`${t("referenceExcerpt")}: ${fact.prov.source_id}`}
                  onClick={() => nav.openSource(fact.prov.source_id, { start: fact.prov.start, end: fact.prov.end })}>↩</button>
              </li>
            ))}
          </ul>

          {page.history.length > 0 && <>
            <div className="section-title">{t("history")}</div>
            <ul className="facts history reading">{page.history.map((item) => <li key={item.id}>{item.text}</li>)}</ul>
          </>}
          {page.contradictions.length > 0 && <>
            <div className="section-title">{t("contradictions")}</div>
            {page.contradictions.map((item, index) => <div className="contradiction reading" key={index}>⚠ {item.note}</div>)}
          </>}
        </article>

        <aside className="entity-aside">
          {page.links.length > 0 && <>
            <div className="section-title">{t("related")}</div>
            {page.links.map((link) => <button className="wikilink" key={link.target_slug} onClick={() => nav.openEntity(link.target_slug)}>[[{link.target_slug}]]</button>)}
          </>}
          {data.backlinks.length > 0 && <>
            <div className="section-title">{t("backlinks")}</div>
            {data.backlinks.map((backlink) => <button className="wikilink" key={backlink} onClick={() => nav.openEntity(backlink)}>[[{backlink}]]</button>)}
          </>}
          <div className="section-title">{t("sources")}</div>
          {page.sources.map((source) => <button className="source-link" key={source} onClick={() => nav.openSource(source)}>{source}</button>)}
        </aside>
      </div>
    </div>
  );
}
