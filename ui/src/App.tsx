import { createContext, useContext, useEffect, useState } from "react";
import { I18nProvider, useI18n } from "./i18n.tsx";
import { api } from "./api.ts";
import { Queue } from "./views/Queue.tsx";
import { Reader } from "./views/Reader.tsx";
import { Entity } from "./views/Entity.tsx";
import { Research } from "./views/Research.tsx";
import { Settings } from "./views/Settings.tsx";
import { Graph } from "./views/Graph.tsx";
import { AddDialog } from "./components/AddDialog.tsx";
import { GlobalSearch } from "./components/GlobalSearch.tsx";

export type View = "queue" | "reader" | "entity" | "graph" | "research" | "settings";
export interface SourceRange { start: number; end: number }

interface Nav {
  view: View;
  go: (view: View) => void;
  back: () => void;
  source: string | null;
  sourceRange: SourceRange | null;
  openSource: (id: string, range?: SourceRange) => void;
  entity: string | null;
  openEntity: (slug: string) => void;
  refreshVersion: number;
  refresh: () => void;
  notify: (message: string, tone?: "success" | "error") => void;
}
const NavCtx = createContext<Nav>(null as unknown as Nav);
export const useNav = () => useContext(NavCtx);

function initialTheme(): "light" | "dark" {
  const saved = localStorage.getItem("mneme-theme");
  if (saved === "light" || saved === "dark") return saved;
  return matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function Shell() {
  const { t, lang } = useI18n();
  const [view, setView] = useState<View>("queue");
  const [source, setSource] = useState<string | null>(null);
  const [sourceRange, setSourceRange] = useState<SourceRange | null>(null);
  const [entity, setEntity] = useState<string | null>(null);
  const [theme, setTheme] = useState<"light" | "dark">(initialTheme);
  const [addOpen, setAddOpen] = useState(false);
  const [provider, setProvider] = useState("—");
  const [connection, setConnection] = useState<"checking" | "online" | "offline">("checking");
  const [refreshVersion, setRefreshVersion] = useState(0);
  const [toast, setToast] = useState<{ message: string; tone: "success" | "error" } | null>(null);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("mneme-theme", theme);
  }, [theme]);
  useEffect(() => { document.documentElement.lang = lang; }, [lang]);

  async function checkConnection() {
    try {
      const config = await api.config();
      setProvider(config.provider); setConnection("online");
    } catch {
      setProvider("—"); setConnection("offline");
    }
  }
  useEffect(() => {
    void checkConnection();
    const timer = window.setInterval(() => void checkConnection(), 30_000);
    return () => window.clearInterval(timer);
  }, []);

  function notify(message: string, tone: "success" | "error" = "success") {
    setToast({ message, tone });
    window.setTimeout(() => setToast(null), 3500);
  }
  function go(next: View) {
    if (next === "entity") setEntity(null);
    setView(next);
  }
  function back() {
    if (view === "reader") { setSourceRange(null); setView("queue"); }
    else if (view === "entity" && entity) { setEntity(null); }
    else if (view === "graph") { setEntity(null); setView("entity"); }
    else setView("queue");
  }

  const nav: Nav = {
    view, go, back,
    source, sourceRange,
    openSource: (id, range) => { setSource(id); setSourceRange(range ?? null); setView("reader"); },
    entity, openEntity: (slug) => { setEntity(slug); setView("entity"); },
    refreshVersion, refresh: () => setRefreshVersion((value) => value + 1), notify,
  };

  const NAV: { view: View; label: string }[] = [
    { view: "queue", label: t("queue") },
    { view: "entity", label: t("wiki") },
    { view: "graph", label: t("graph") },
    { view: "research", label: t("research") },
    { view: "settings", label: t("settings") },
  ];
  const activeView = view === "reader" ? "queue" : view;

  return (
    <NavCtx.Provider value={nav}>
      <div className="shell">
        <header className="topbar">
          <span className="brand">Mneme<small>Μνήμη</small></span>
          <GlobalSearch />
          <span className="spacer" />
          <button className="btn primary" onClick={() => setAddOpen(true)}>+ {t("add")}</button>
        </header>

        <nav className="sidenav" aria-label={t("mainNavigation")}>
          {NAV.map((item) => (
            <button key={item.view} className={`nav-${item.view}${activeView === item.view ? " active" : ""}`}
              onClick={() => go(item.view)}>{item.label}</button>
          ))}
        </nav>

        <main className="main">
          {view === "queue" && <Queue />}
          {view === "reader" && <Reader id={source} range={sourceRange} />}
          {view === "entity" && <Entity slug={entity} />}
          {view === "graph" && <Graph />}
          {view === "research" && <Research />}
          {view === "settings" && <Settings theme={theme} setTheme={setTheme} onConfigSaved={() => void checkConnection()} />}
        </main>

        <footer className={`statusbar ${connection}`}>
          <span><span className="dot" />{connection === "checking" ? t("checking") : connection === "online" ? `${t("serviceOnline")} · ${provider}` : t("serviceOffline")}</span>
          <span className="muted">Mneme · read-it-later → LLM Wiki</span>
        </footer>
      </div>
      {addOpen && <AddDialog onClose={() => setAddOpen(false)} onDone={() => {
        setAddOpen(false); nav.refresh(); setView("queue"); notify(t("addedSuccess"));
      }} />}
      {toast && <div className={`toast ${toast.tone}`} role="status">{toast.message}</div>}
    </NavCtx.Provider>
  );
}

export function App() {
  return <I18nProvider><Shell /></I18nProvider>;
}
