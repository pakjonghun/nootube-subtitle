use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

fn get_sidecar_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("Cannot find executable path")?;

    #[cfg(target_os = "windows")]
    let binary = dir.join("yt-dlp.exe");
    #[cfg(not(target_os = "windows"))]
    let binary = dir.join("yt-dlp");

    if binary.exists() {
        Ok(binary)
    } else {
        Err(format!("yt-dlp binary not found: {:?}", binary))
    }
}

fn emit_log(app: &AppHandle, msg: &str) {
    let _ = app.emit("extract-log", msg.to_string());
}

fn lang_name(code: &str, locale: &str) -> String {
    match (code, locale) {
        ("ko", "ko") => "한국어".into(),
        ("ko", _) => "Korean".into(),
        ("en", "ko") => "영어".into(),
        ("en", _) => "English".into(),
        ("ja", "ko") => "일본어".into(),
        ("ja", _) => "Japanese".into(),
        ("zh", "ko") => "중국어".into(),
        ("zh", _) => "Chinese".into(),
        (other, _) => other.to_string(),
    }
}

fn msg(locale: &str, key: &str) -> String {
    match (locale, key) {
        ("ko", "invalid_url") => "유효한 URL이 아닙니다.".into(),
        (_, "invalid_url") => "Invalid URL.".into(),
        ("ko", "no_subtitle") => "이 영상에서 자막을 찾을 수 없습니다.".into(),
        (_, "no_subtitle") => "No subtitles found for this video.".into(),
        ("ko", "download_failed") => "자막을 다운로드하지 못했습니다.".into(),
        (_, "download_failed") => "Failed to download subtitles.".into(),
        ("ko", "searching") => "자막 검색 중...".into(),
        (_, "searching") => "Searching subtitles...".into(),
        ("ko", "found") => "자막 발견! 처리 중...".into(),
        (_, "found") => "Subtitle found! Processing...".into(),
        _ => key.into(),
    }
}

#[tauri::command]
async fn extract_subtitle(
    app: AppHandle,
    url: String,
    lang_priority: Vec<String>,
    locale: String,
) -> Result<String, String> {
    let loc = locale.as_str();

    if !is_youtube_url(&url) {
        return Err(msg(loc, "invalid_url"));
    }

    let yt_dlp = get_sidecar_path()?;
    let yt_dlp_str = yt_dlp.to_string_lossy().to_string();

    let langs = if lang_priority.is_empty() {
        vec!["ko".to_string(), "en".to_string(), "ja".to_string(), "zh".to_string()]
    } else {
        lang_priority
    };

    let lang_display: Vec<String> = langs.iter().map(|l| lang_name(l, loc)).collect();

    let temp_dir = std::env::temp_dir().join("yt-subtitle-extractor");
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    // 기존 파일 정리
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    // 우선순위 순서대로 언어별 개별 시도
    // 쿠키 파일 탐색 (~/.yt-cookies.txt)
    let cookie_file = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(|h| PathBuf::from(h).join(".yt-cookies.txt"))
        .filter(|p| p.exists());

    if let Some(ref cf) = cookie_file {
        let cookie_msg = if loc == "ko" {
            format!("쿠키 사용: {}", cf.display())
        } else {
            format!("Using cookies: {}", cf.display())
        };
        emit_log(&app, &cookie_msg);
    }

    let searching_msg = if loc == "ko" {
        format!("{} 자막 검색 중...", lang_display.join(", "))
    } else {
        format!("Searching {} subtitles...", lang_display.join(", "))
    };
    emit_log(&app, &searching_msg);

    let mut best_lang = String::new();
    let mut best_path: Option<PathBuf> = None;

    for (i, lang) in langs.iter().enumerate() {
        // 기존 파일 정리
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }

        let try_msg = if loc == "ko" {
            format!("{} 자막 시도 중... ({}/{})", lang_name(lang, loc), i + 1, langs.len())
        } else {
            format!("Trying {} subtitle... ({}/{})", lang_name(lang, loc), i + 1, langs.len())
        };
        emit_log(&app, &try_msg);

        let mut cmd = Command::new(&yt_dlp_str);
        cmd.args([
                "--skip-download",
                "--write-auto-sub",
                "--sub-format", "vtt",
                "--sub-lang", lang.as_str(),
                "--output", "subtitle",
            ]);

        if let Some(ref cf) = cookie_file {
            cmd.args(["--cookies", &cf.to_string_lossy()]);
        }

        cmd.arg(&url)
            .current_dir(&temp_dir);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let result = cmd.output().await.map_err(|e| e.to_string())?;
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();
        eprintln!("[yt-dlp {}] {}", lang, stderr);

        // 다운로드된 vtt 파일 찾기
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "vtt" {
                        best_lang = lang.clone();
                        best_path = Some(path);
                        break;
                    }
                }
            }
        }

        if best_path.is_some() {
            break;
        }
    }

    match best_path {
        Some(path) => {
            let found_msg = if loc == "ko" {
                format!("{} 자막 발견! 처리 중...", lang_name(&best_lang, loc))
            } else {
                format!("{} subtitle found! Processing...", lang_name(&best_lang, loc))
            };
            emit_log(&app, &found_msg);

            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

            // 정리
            if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            Ok(clean_subtitle(&content))
        }
        None => Err(msg(loc, "no_subtitle")),
    }
}

fn is_youtube_url(url: &str) -> bool {
    url.contains("youtube.com") || url.contains("youtu.be")
}

fn clean_subtitle(raw: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut prev_line = String::new();

    for line in raw.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("WEBVTT")
            || trimmed.starts_with("Kind:")
            || trimmed.starts_with("Language:")
            || trimmed.starts_with("NOTE")
            || trimmed.is_empty()
        {
            continue;
        }
        if trimmed.contains("-->") {
            continue;
        }
        if trimmed.parse::<u32>().is_ok() {
            continue;
        }

        let clean = remove_tags(trimmed).trim().to_string();
        if clean.is_empty() {
            continue;
        }
        if clean != prev_line {
            lines.push(clean.clone());
            prev_line = clean;
        }
    }

    lines.join("\n")
}

fn remove_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![extract_subtitle])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
