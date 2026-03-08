import { useCallback } from "react";
import { t, LANGUAGES } from "./i18n";
import type { Locale } from "./i18n";
import type { Settings as SettingsType } from "./useSettings";

interface Props {
  settings: SettingsType;
  onUpdate: (partial: Partial<SettingsType>) => void;
  onClose: () => void;
}

export default function Settings({ settings, onUpdate, onClose }: Props) {
  const { locale, subtitleLangPriority, autoCopy } = settings;

  const moveLanguage = useCallback(
    (index: number, direction: -1 | 1) => {
      const newOrder = [...subtitleLangPriority];
      const targetIndex = index + direction;
      if (targetIndex < 0 || targetIndex >= newOrder.length) return;
      [newOrder[index], newOrder[targetIndex]] = [newOrder[targetIndex], newOrder[index]];
      onUpdate({ subtitleLangPriority: newOrder });
    },
    [subtitleLangPriority, onUpdate],
  );

  const getLangLabel = (code: string) => {
    const lang = LANGUAGES.find((l) => l.code === code);
    return lang ? t(locale, lang.labelKey) : code;
  };

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>{t(locale, "settingsTitle")}</h2>
          <button className="btn-close" onClick={onClose}>
            &times;
          </button>
        </div>

        <div className="settings-section">
          <label>{t(locale, "subtitleLanguagePriority")}</label>
          <div className="lang-list">
            {subtitleLangPriority.map((code, index) => (
              <div key={code} className="lang-item">
                <span className="lang-rank">{index + 1}</span>
                <span className="lang-name">{getLangLabel(code)}</span>
                <div className="lang-arrows">
                  <button
                    disabled={index === 0}
                    onClick={() => moveLanguage(index, -1)}
                    title="Up"
                  >
                    &#9650;
                  </button>
                  <button
                    disabled={index === subtitleLangPriority.length - 1}
                    onClick={() => moveLanguage(index, 1)}
                    title="Down"
                  >
                    &#9660;
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="settings-section">
          <label className="toggle-label">
            <span>{t(locale, "autoCopyToClipboard")}</span>
            <input
              type="checkbox"
              checked={autoCopy}
              onChange={(e) => onUpdate({ autoCopy: e.target.checked })}
            />
            <span className="toggle-slider" />
          </label>
        </div>

        <div className="settings-section">
          <label>{t(locale, "appLanguage")}</label>
          <div className="locale-selector">
            <button
              className={locale === "ko" ? "active" : ""}
              onClick={() => onUpdate({ locale: "ko" as Locale })}
            >
              {t(locale, "korean")}
            </button>
            <button
              className={locale === "en" ? "active" : ""}
              onClick={() => onUpdate({ locale: "en" as Locale })}
            >
              {t(locale, "english")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
