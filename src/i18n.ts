export type Locale = "ko" | "en";

const translations = {
  ko: {
    appTitle: "너튜브 자막 추출기",
    youtube: "너튜브",
    inputPlaceholder: "영상 URL을 입력하세요...",
    extract: "자막 추출",
    extracting: "추출 중...",
    stopExtract: "추출 중지",
    stopped: "추출이 중지되었습니다.",
    copyToClipboard: "클립보드에 복사",
    copied: "복사됨!",
    lines: "줄",
    emptyMessage: "영상 URL을 입력하고 자막을 추출하세요.",
    loading: "자막을 추출하고 있습니다...",
    successWithCopy: "자막 추출 완료! 클립보드에 복사되었습니다.",
    successWithoutCopy: "자막 추출 완료!",
    settings: "설정",
    settingsTitle: "설정",
    subtitleLanguagePriority: "자막 언어 우선순위",
    dragToReorder: "드래그하여 순서 변경",
    autoCopyToClipboard: "자동 클립보드 복사",
    appLanguage: "앱 언어",
    close: "닫기",
    priorityLabel: "우선순위",
    korean: "한국어",
    english: "영어",
    japanese: "일본어",
    chinese: "중국어",
  },
  en: {
    appTitle: "NooTube Subtitle Extractor",
    youtube: "NooTube",
    inputPlaceholder: "Enter video URL...",
    extract: "Extract",
    extracting: "Extracting...",
    stopExtract: "Stop",
    stopped: "Extraction stopped.",
    copyToClipboard: "Copy to clipboard",
    copied: "Copied!",
    lines: "lines",
    emptyMessage: "Enter a video URL and extract subtitles.",
    loading: "Extracting subtitles...",
    successWithCopy: "Subtitles extracted! Copied to clipboard.",
    successWithoutCopy: "Subtitles extracted!",
    settings: "Settings",
    settingsTitle: "Settings",
    subtitleLanguagePriority: "Subtitle language priority",
    dragToReorder: "Drag to reorder",
    autoCopyToClipboard: "Auto copy to clipboard",
    appLanguage: "App language",
    close: "Close",
    priorityLabel: "Priority",
    korean: "Korean",
    english: "English",
    japanese: "Japanese",
    chinese: "Chinese",
  },
} as const;

export type TranslationKey = keyof typeof translations.ko;

export function t(locale: Locale, key: TranslationKey): string {
  return translations[locale][key];
}

export const LANGUAGES = [
  { code: "ko", labelKey: "korean" as const },
  { code: "en", labelKey: "english" as const },
  { code: "ja", labelKey: "japanese" as const },
  { code: "zh", labelKey: "chinese" as const },
] as const;
