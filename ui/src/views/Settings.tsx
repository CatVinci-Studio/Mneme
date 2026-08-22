import { useEffect, useState } from "react";
import { api, type AppConfigView, type LintReport } from "../api.ts";
import { useI18n, type Lang } from "../i18n.tsx";

const PROVIDER_LABELS: Record<string, string> = {
  openai: "OpenAI", chatglm: "ChatGLM (智谱)", deepseek: "DeepSeek", qwen: "Qwen (通义千问)", llamacpp: "llama.cpp", custom: "自定义 / Custom", mock: "Demo",
};

export function Settings({ theme, setTheme, onConfigSaved }: { theme: "light" | "dark"; setTheme: (t: "light" | "dark") => void; onConfigSaved: () => void }) {
  const { t, lang, setLang } = useI18n();
  const [cfg, setCfg] = useState<AppConfigView | null>(null);
  const [provider, setProvider] = useState("openai");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [retriever, setRetriever] = useState("builtin");
  const [syncRemote, setSyncRemote] = useState("");
  const [status, setStatus] = useState("");
  const [lint, setLint] = useState<LintReport | null>(null);
  const [syncMsg, setSyncMsg] = useState("");

  useEffect(() => {
    api.config().then((c) => {
      setCfg(c); setProvider(c.provider && c.providers.includes(c.provider) ? c.provider : "openai");
      setModel(c.model ?? ""); setBaseUrl(c.baseUrl ?? ""); setRetriever(c.retriever ?? "builtin");
      setSyncRemote(c.syncRemote ?? "");
    }).catch(() => {});
  }, []);

  async function save() {
    setStatus("…");
    try {
      const c = await api.setConfig({ provider, apiKey: apiKey || undefined, model: model.trim(), baseUrl: baseUrl.trim(), retriever, syncRemote: syncRemote.trim() });
      setCfg(c); setApiKey(""); setStatus(t("saved")); onConfigSaved();
      setTimeout(() => setStatus(""), 1500);
    } catch (e) { setStatus((e as Error).message); }
  }
  async function runBackup() {
    setSyncMsg("…");
    try { const r = await api.sync(); setSyncMsg(r.message); } catch (e) { setSyncMsg((e as Error).message); }
  }

  const needsKey = provider !== "mock" && provider !== "custom";
  const options = cfg?.providers ?? ["openai", "chatglm", "deepseek", "custom", "mock"];

  return (
    <div className="view settings">
      <h1 className="h1">{t("settings")}</h1>

      <div className="group">
        <h3>{t("aiProvider")}</h3>
        <p className="hint">{t("keyHint")}</p>
        <div className="field">
          <label>{t("provider")}</label>
          <select value={provider} onChange={(e) => setProvider(e.target.value)}>
            {options.map((p) => <option key={p} value={p}>{PROVIDER_LABELS[p] ?? p}</option>)}
          </select>
        </div>
        {provider === "custom" && (
          <div className="field">
            <label>{t("baseUrl")}</label>
            <input value={baseUrl} placeholder="https://… /v1" onChange={(e) => setBaseUrl(e.target.value)} />
          </div>
        )}
        {(needsKey || provider === "custom") && (
          <div className="field">
            <label>{t("apiKey")} {cfg?.hasKey && <span className="badge ok">{t("apiKeySet")}</span>}</label>
            <input type="password" value={apiKey} placeholder={cfg?.hasKey ? "••••••••" : "sk-…"} onChange={(e) => setApiKey(e.target.value)} />
          </div>
        )}
        <div className="field">
          <label>{t("model")}</label>
          <input value={model} placeholder="gpt-4o / glm-4-plus / deepseek-chat" onChange={(e) => setModel(e.target.value)} />
        </div>
        <div className="field">
          <label>{t("retrieval")}</label>
          <select value={retriever} onChange={(e) => setRetriever(e.target.value)}>
            <option value="builtin">{t("builtin")}</option>
          </select>
        </div>
        <div className="row">
          <button className="btn primary" onClick={save}>{t("saveCfg")}</button>
          {status && <span className="muted">{status}</span>}
          {cfg && <span className="badge"><span className="dot" />{cfg.provider}</span>}
        </div>
      </div>

      <div className="group">
        <h3>{t("appearance")}</h3>
        <div className="field">
          <label>{t("theme")}</label>
          <select value={theme} onChange={(e) => setTheme(e.target.value as "light" | "dark")}>
            <option value="light">{t("light")}</option>
            <option value="dark">{t("dark")}</option>
          </select>
        </div>
        <div className="field">
          <label>{t("uiLang")}</label>
          <select value={lang} onChange={(e) => setLang(e.target.value as Lang)}>
            <option value="zh">中文</option>
            <option value="en">English</option>
          </select>
        </div>
      </div>

      <div className="group">
        <h3>{t("health")}</h3>
        <div className="row">
          <button className="btn" onClick={async () => {
            try { setLint(await api.lint()); } catch (e) { setStatus((e as Error).message); }
          }}>{t("runLint")}</button>
          {lint && <span className="muted">{lint.entities} {t("entitiesN")} · {lint.facts} {t("factsN")} · {lint.orphans.length} {t("orphans")} · {lint.contradictions.length} {t("contradictionsN")}</span>}
        </div>
        {lint && lint.orphans.length > 0 && <p className="hint" style={{ marginTop: 8 }}>{t("orphans")}: {lint.orphans.join(", ")}</p>}
      </div>

      <div className="group">
        <h3>{t("backup")}</h3>
        <p className="hint">{t("backupHint")}</p>
        <div className="field">
          <label>{t("gitRemote")}</label>
          <input value={syncRemote} placeholder="git@github.com:me/my-vault.git" onChange={(e) => setSyncRemote(e.target.value)} />
        </div>
        <div className="row">
          <button className="btn" onClick={save}>{t("saveCfg")}</button>
          <button className="btn primary" onClick={runBackup}>{t("backupNow")}</button>
          {syncMsg && <span className="muted">{syncMsg}</span>}
        </div>
      </div>
    </div>
  );
}
