import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { t } from "./i18n";
import { useSettings } from "./useSettings";
import Settings from "./Settings";

type Status =
  | { type: "idle" }
  | { type: "loading" }
  | { type: "success"; message: string }
  | { type: "error"; message: string };

function App() {
  const [url, setUrl] = useState("");
  const [subtitle, setSubtitle] = useState("");
  const [status, setStatus] = useState<Status>({ type: "idle" });
  const [copied, setCopied] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const { settings, updateSettings } = useSettings();
  const { locale } = settings;

  useEffect(() => {
    const unlisten = listen<string>("extract-log", (event) => {
      setLogs((prev) => {
        const msg = event.payload;
        // "...중..." 류 진행 메시지는 같은 접두사면 교체 (예: "자막 다운로드 중... 50%" → "자막 다운로드 중... 100%")
        const baseMsg = msg.replace(/\d+%/, "").replace("완료!", "");
        const lastBase = prev.length > 0 ? prev[prev.length - 1].replace(/\d+%/, "").replace("완료!", "") : "";
        if (prev.length > 0 && baseMsg === lastBase) {
          return [...prev.slice(0, -1), msg];
        }
        return [...prev, msg];
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const handleExtract = useCallback(async () => {
    const trimmed = url.trim();
    if (!trimmed) return;

    setStatus({ type: "loading" });
    setSubtitle("");
    setLogs([]);
    setCopied(false);

    try {
      const result = await invoke<string>("extract_subtitle", {
        url: trimmed,
        langPriority: settings.subtitleLangPriority,
        locale,
      });
      setSubtitle(result);

      if (settings.autoCopy) {
        await writeText(result);
        setStatus({ type: "success", message: t(locale, "successWithCopy") });
        setCopied(true);
        setTimeout(() => setCopied(false), 3000);
      } else {
        setStatus({ type: "success", message: t(locale, "successWithoutCopy") });
      }
    } catch (err) {
      const message =
        typeof err === "string" ? err : (err as Error).message || "Unknown error";
      setStatus({ type: "error", message });
    }
  }, [url, settings, locale]);

  const handleCopy = useCallback(async () => {
    if (!subtitle) return;
    await writeText(subtitle);
    setCopied(true);
    setTimeout(() => setCopied(false), 3000);
  }, [subtitle]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && status.type !== "loading") {
        handleExtract();
      }
    },
    [handleExtract, status],
  );

  return (
    <div className="container">
      <div className="header">
        <h1>
          <span>{t(locale, "youtube")}</span> {locale === "ko" ? "자막 추출기" : "Subtitle Extractor"}
        </h1>
        <button className="btn-settings" onClick={() => setShowSettings(true)}>
          &#9881;
        </button>
      </div>

      <div className="input-group">
        <input
          type="text"
          placeholder={t(locale, "inputPlaceholder")}
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={status.type === "loading"}
        />
        <button
          className="btn-extract"
          onClick={handleExtract}
          disabled={status.type === "loading" || !url.trim()}
        >
          {status.type === "loading" ? t(locale, "extracting") : t(locale, "extract")}
        </button>
      </div>

      <div className="subtitle-area">
        <div className="subtitle-header">
          <span>
            {subtitle ? `${subtitle.split("\n").length} ${t(locale, "lines")}` : ""}
          </span>
          {subtitle && (
            <button
              className={`btn-copy ${copied ? "copied" : ""}`}
              onClick={handleCopy}
            >
              {copied ? t(locale, "copied") : t(locale, "copyToClipboard")}
            </button>
          )}
        </div>
        <div className={`subtitle-content ${!subtitle && status.type !== "loading" ? "empty" : ""} ${status.type === "loading" ? "log-terminal" : ""}`}>
          {status.type === "loading" ? (
            <>
              {logs.map((log, i) => (
                <div key={i} className={`log-line ${i === logs.length - 1 ? "log-current" : ""}`}>
                  {log}
                </div>
              ))}
              <div className="log-cursor" />
              <div ref={logsEndRef} />
            </>
          ) : (
            subtitle || t(locale, "emptyMessage")
          )}
        </div>
      </div>

      {status.type !== "idle" && status.type !== "loading" && (
        <div className={`status-bar ${status.type}`}>
          {status.type === "success" && status.message}
          {status.type === "error" && status.message}
        </div>
      )}

      {showSettings && (
        <Settings
          settings={settings}
          onUpdate={updateSettings}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}

export default App;
