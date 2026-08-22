import { useEffect, useMemo, useState } from "react";
import { api, type GraphData } from "../api.ts";
import { useI18n } from "../i18n.tsx";
import { useNav } from "../App.tsx";
import { ErrorState, LoadingState } from "../components/AsyncState.tsx";

const W = 900, H = 560;

/** 零依赖力导向布局(Fruchterman–Reingold 简版),确定性初始化(圆周),离线计算。 */
function layout(data: GraphData) {
  const n = data.nodes.length;
  const pos = new Map<string, { x: number; y: number }>();
  data.nodes.forEach((nd, i) => {
    const a = (i / Math.max(n, 1)) * 2 * Math.PI;
    pos.set(nd.slug, { x: W / 2 + Math.cos(a) * W / 4, y: H / 2 + Math.sin(a) * H / 4 });
  });
  if (n < 2) return pos;
  const k = Math.sqrt((W * H) / n) * 0.8;
  const adj = data.edges.map((e) => [e.source, e.target] as const);
  for (let it = 0; it < 350; it++) {
    const disp = new Map<string, { x: number; y: number }>();
    data.nodes.forEach((nd) => disp.set(nd.slug, { x: 0, y: 0 }));
    // 斥力
    for (let i = 0; i < n; i++) for (let j = i + 1; j < n; j++) {
      const a = pos.get(data.nodes[i]!.slug)!, b = pos.get(data.nodes[j]!.slug)!;
      let dx = a.x - b.x, dy = a.y - b.y; let d = Math.hypot(dx, dy) || 0.01;
      const f = (k * k) / d; dx = (dx / d) * f; dy = (dy / d) * f;
      const di = disp.get(data.nodes[i]!.slug)!, dj = disp.get(data.nodes[j]!.slug)!;
      di.x += dx; di.y += dy; dj.x -= dx; dj.y -= dy;
    }
    // 引力(边)
    for (const [s, tg] of adj) {
      const a = pos.get(s)!, b = pos.get(tg)!; if (!a || !b) continue;
      let dx = a.x - b.x, dy = a.y - b.y; const d = Math.hypot(dx, dy) || 0.01;
      const f = (d * d) / k; dx = (dx / d) * f; dy = (dy / d) * f;
      const ds = disp.get(s)!, dt = disp.get(tg)!;
      ds.x -= dx; ds.y -= dy; dt.x += dx; dt.y += dy;
    }
    const temp = 12 * (1 - it / 350);
    for (const nd of data.nodes) {
      const p = pos.get(nd.slug)!, d = disp.get(nd.slug)!;
      const len = Math.hypot(d.x, d.y) || 0.01;
      p.x += (d.x / len) * Math.min(len, temp);
      p.y += (d.y / len) * Math.min(len, temp);
      p.x = Math.max(40, Math.min(W - 40, p.x));
      p.y = Math.max(30, Math.min(H - 30, p.y));
    }
  }
  return pos;
}

export function Graph() {
  const { t } = useI18n();
  const nav = useNav();
  const [data, setData] = useState<GraphData | null>(null);
  const [hover, setHover] = useState<string | null>(null);
  const [error, setError] = useState("");

  async function load() {
    setError("");
    try { setData(await api.graph()); }
    catch (e) { setError((e as Error).message); }
  }
  useEffect(() => { void load(); }, [nav.refreshVersion]);
  const pos = useMemo(() => (data ? layout(data) : new Map()), [data]);

  if (error) return <ErrorState message={`${t("loadFailed")}: ${error}`} retry={() => void load()} />;
  if (!data) return <LoadingState label={t("loading")} />;
  if (data.nodes.length < 2) return <div className="view"><h1 className="h1">{t("graph")}</h1><p className="muted">{t("graphEmpty")}</p></div>;

  return (
    <div className="view">
      <h1 className="h1">{t("graph")}</h1>
      <div className="panel" style={{ padding: 0, overflow: "hidden" }}>
        <svg viewBox={`0 0 ${W} ${H}`} style={{ width: "100%", height: "70vh", display: "block" }}>
          {data.edges.map((e, i) => {
            const a = pos.get(e.source), b = pos.get(e.target);
            if (!a || !b) return null;
            const on = hover === e.source || hover === e.target;
            return <line key={i} x1={a.x} y1={a.y} x2={b.x} y2={b.y}
              stroke={on ? "var(--accent)" : "var(--border)"} strokeWidth={on ? 2 : 1} />;
          })}
          {data.nodes.map((nd) => {
            const p = pos.get(nd.slug)!; const on = hover === nd.slug;
            return (
              <g key={nd.slug} transform={`translate(${p.x},${p.y})`} className="graph-node" role="button" tabIndex={0}
                aria-label={nd.title} onClick={() => nav.openEntity(nd.slug)}
                onKeyDown={(event) => (event.key === "Enter" || event.key === " ") && nav.openEntity(nd.slug)}
                onMouseEnter={() => setHover(nd.slug)} onMouseLeave={() => setHover(null)}>
                <circle r={on ? 9 : 6} fill="var(--accent)" opacity={on ? 1 : 0.85} />
                <text x={11} y={4} fontSize={12} fill="var(--text)" fontFamily="var(--font-ui)">{nd.title}</text>
              </g>
            );
          })}
        </svg>
      </div>
      <p className="muted" style={{ marginTop: 8 }}>{data.nodes.length} {t("nodes")} · {data.edges.length} {t("links")}</p>
    </div>
  );
}
