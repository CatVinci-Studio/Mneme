import { useEffect, useRef, useState } from "react";
import { api, type SearchHit } from "../api.ts";
import { useNav } from "../App.tsx";
import { useI18n } from "../i18n.tsx";
import { ErrorState, LoadingState } from "./AsyncState.tsx";

export function GlobalSearch() {
  const { t } = useI18n();
  const nav = useNav();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchHit[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault(); setOpen(true);
      }
      if (event.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  useEffect(() => { if (open) requestAnimationFrame(() => inputRef.current?.focus()); }, [open]);

  async function search() {
    const q = query.trim();
    if (!q) { setResults([]); return; }
    setLoading(true); setError("");
    try { setResults(await api.search(q)); }
    catch (e) { setError((e as Error).message); }
    finally { setLoading(false); }
  }

  function choose(hit: SearchHit) {
    nav.openEntity(hit.slug); setOpen(false); setQuery(""); setResults([]);
  }

  return (
    <>
      <button className="search-trigger" onClick={() => setOpen(true)} aria-label={t("searchKnowledge")}>
        <span>⌕</span><span>{t("searchKnowledge")}</span><kbd>⌘K</kbd>
      </button>
      {open && (
        <div className="overlay search-overlay" onMouseDown={() => setOpen(false)}>
          <section className="search-dialog" role="dialog" aria-modal="true" aria-label={t("searchKnowledge")} onMouseDown={(e) => e.stopPropagation()}>
            <div className="search-input-row">
              <input ref={inputRef} value={query} placeholder={t("searchPlaceholder")}
                onChange={(e) => setQuery(e.target.value)} onKeyDown={(e) => e.key === "Enter" && void search()} />
              <button className="btn primary" onClick={() => void search()} disabled={loading}>{t("searchAction")}</button>
            </div>
            <div className="search-results">
              {loading && <LoadingState label={t("searching")} />}
              {error && <ErrorState message={error} retry={() => void search()} />}
              {!loading && !error && results.map((hit) => (
                <button className="search-result" key={hit.slug} onClick={() => choose(hit)}>
                  <strong>{hit.title}</strong><span>{hit.snippet || hit.slug}</span>
                </button>
              ))}
              {!loading && !error && query.trim() && results.length === 0 && <p className="muted search-empty">{t("noResults")}</p>}
              {!query.trim() && <p className="muted search-empty">{t("searchHint")}</p>}
            </div>
          </section>
        </div>
      )}
    </>
  );
}
