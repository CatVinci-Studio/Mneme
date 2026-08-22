import type { ReactNode } from "react";
import { useI18n } from "../i18n.tsx";

export function LoadingState({ label = "Loading…" }: { label?: string }) {
  return <div className="state-box" role="status"><span className="loader" aria-hidden="true" />{label}</div>;
}

export function ErrorState({ message, retry }: { message: string; retry?: () => void }) {
  const { t } = useI18n();
  return (
    <div className="state-box error" role="alert">
      <span>{message}</span>
      {retry && <button className="btn" onClick={retry}>{t("retry")}</button>}
    </div>
  );
}

export function Notice({ tone = "success", children }: { tone?: "success" | "error" | "info"; children: ReactNode }) {
  return <div className={`notice ${tone}`} role={tone === "error" ? "alert" : "status"}>{children}</div>;
}
