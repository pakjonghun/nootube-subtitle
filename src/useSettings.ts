import { useState, useEffect, useCallback } from "react";
import type { Locale } from "./i18n";

export interface Settings {
  subtitleLangPriority: string[];
  autoCopy: boolean;
  locale: Locale;
}

const STORAGE_KEY = "yt-subtitle-settings";

const DEFAULT_SETTINGS: Settings = {
  subtitleLangPriority: ["ko", "en", "ja", "zh"],
  autoCopy: true,
  locale: "ko",
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
      }
    } catch {
      // ignore
    }
    return DEFAULT_SETTINGS;
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }, [settings]);

  const updateSettings = useCallback((partial: Partial<Settings>) => {
    setSettings((prev) => ({ ...prev, ...partial }));
  }, []);

  return { settings, updateSettings };
}
