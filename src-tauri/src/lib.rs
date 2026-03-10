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
        ("ko", "ja") => "韓国語".into(),
        ("ko", "zh") => "韩语".into(),
        ("ko", _) => "Korean".into(),
        ("en", "ko") => "영어".into(),
        ("en", "ja") => "英語".into(),
        ("en", "zh") => "英语".into(),
        ("en", _) => "English".into(),
        ("ja", "ko") => "일본어".into(),
        ("ja", "ja") => "日本語".into(),
        ("ja", "zh") => "日语".into(),
        ("ja", _) => "Japanese".into(),
        ("zh", "ko") => "중국어".into(),
        ("zh", "ja") => "中国語".into(),
        ("zh", "zh") => "中文".into(),
        ("zh", _) => "Chinese".into(),
        (other, _) => other.to_string(),
    }
}

fn msg(locale: &str, key: &str) -> String {
    match (locale, key) {
        ("ko", "invalid_url") => "유효한 URL이 아닙니다.".into(),
        ("ja", "invalid_url") => "有効なURLではありません。".into(),
        ("zh", "invalid_url") => "无效的URL。".into(),
        (_, "invalid_url") => "Invalid URL.".into(),
        ("ko", "no_subtitle") => "이 영상에서 자막을 찾을 수 없습니다.".into(),
        ("ja", "no_subtitle") => "この動画に字幕が見つかりません。".into(),
        ("zh", "no_subtitle") => "未找到该视频的字幕。".into(),
        (_, "no_subtitle") => "No subtitles found for this video.".into(),
        ("ko", "download_failed") => "자막을 다운로드하지 못했습니다.".into(),
        ("ja", "download_failed") => "字幕のダウンロードに失敗しました。".into(),
        ("zh", "download_failed") => "字幕下载失败。".into(),
        (_, "download_failed") => "Failed to download subtitles.".into(),
        ("ko", "searching") => "자막 검색 중...".into(),
        ("ja", "searching") => "字幕を検索中...".into(),
        ("zh", "searching") => "正在搜索字幕...".into(),
        (_, "searching") => "Searching subtitles...".into(),
        ("ko", "found") => "자막 발견! 처리 중...".into(),
        ("ja", "found") => "字幕発見！処理中...".into(),
        ("zh", "found") => "找到字幕！处理中...".into(),
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

    // 쿠키 파일 탐색 (~/.yt-cookies.txt)
    let cookie_file = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(|h| PathBuf::from(h).join(".yt-cookies.txt"))
        .filter(|p| p.exists());

    if let Some(ref cf) = cookie_file {
        let cookie_msg = match loc {
            "ko" => format!("쿠키 사용: {}", cf.display()),
            "ja" => format!("Cookie使用: {}", cf.display()),
            "zh" => format!("使用Cookie: {}", cf.display()),
            _ => format!("Using cookies: {}", cf.display()),
        };
        emit_log(&app, &cookie_msg);
    }

    let normalize = |code: &str| -> String {
        code.split('-').next().unwrap_or(code).to_string()
    };

    // Step 1: --list-subs로 실제 언어코드 확인 (yt-dlp 1회차)
    let searching_msg = match loc {
        "ko" => format!("{} 자막 검색 중...", lang_display.join(", ")),
        "ja" => format!("{} 字幕検索中...", lang_display.join(", ")),
        "zh" => format!("{} 字幕搜索中...", lang_display.join(", ")),
        _ => format!("Searching {} subtitles...", lang_display.join(", ")),
    };
    emit_log(&app, &searching_msg);

    let mut list_cmd = Command::new(&yt_dlp_str);
    list_cmd.args(["--list-subs", "--skip-download"]);
    if let Some(ref cf) = cookie_file {
        list_cmd.args(["--cookies", &cf.to_string_lossy()]);
    }
    list_cmd.args(["--", &url]);

    #[cfg(target_os = "windows")]
    list_cmd.creation_flags(0x08000000);

    let list_output = list_cmd.output().await.map_err(|e| e.to_string())?;
    let list_text = String::from_utf8_lossy(&list_output.stdout).to_string();

    // 사용 가능한 자막 코드 수집
    let mut available_auto: Vec<String> = Vec::new();
    let mut section_is_auto = false;

    for line in list_text.lines() {
        if line.contains("Available subtitles") {
            section_is_auto = false;
            continue;
        }
        if line.contains("Available automatic captions") {
            section_is_auto = true;
            continue;
        }
        if !section_is_auto { continue; }

        let trimmed = line.trim();
        if let Some(lang_code) = trimmed.split_whitespace().next() {
            if lang_code == "Language" || lang_code.is_empty() { continue; }
            if lang_code.chars().any(|c| c.is_ascii_alphabetic())
                && lang_code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                available_auto.push(lang_code.to_string());
            }
        }
    }

    if available_auto.is_empty() {
        return Err(msg(loc, "no_subtitle"));
    }

    // 우선순위에 맞는 실제 코드 매핑 (zh → zh-Hans 등)
    let mut download_codes: Vec<String> = Vec::new();
    for lang in &langs {
        if let Some(found) = available_auto.iter().find(|c| normalize(c) == *lang) {
            if !download_codes.contains(found) {
                download_codes.push(found.clone());
            }
        }
    }

    if download_codes.is_empty() {
        return Err(msg(loc, "no_subtitle"));
    }

    // Step 2: 실제 코드로 한 번에 다운로드 (yt-dlp 2회차)
    let dl_csv = download_codes.join(",");
    let dl_msg = match loc {
        "ko" => "자막 다운로드 중...".to_string(),
        "ja" => "字幕ダウンロード中...".to_string(),
        "zh" => "正在下载字幕...".to_string(),
        _ => "Downloading subtitles...".to_string(),
    };
    emit_log(&app, &dl_msg);

    let mut dl_cmd = Command::new(&yt_dlp_str);
    dl_cmd.args([
        "--skip-download",
        "--write-auto-sub",
        "--sub-format", "vtt",
        "--sub-lang", &dl_csv,
        "--output", "subtitle",
    ]);
    if let Some(ref cf) = cookie_file {
        dl_cmd.args(["--cookies", &cf.to_string_lossy()]);
    }
    dl_cmd.args(["--", &url]).current_dir(&temp_dir);

    #[cfg(target_os = "windows")]
    dl_cmd.creation_flags(0x08000000);

    let dl_result = dl_cmd.output().await.map_err(|e| e.to_string())?;
    let dl_stderr = String::from_utf8_lossy(&dl_result.stderr).to_string();
    eprintln!("[yt-dlp download] {}", dl_stderr);

    // 다운로드된 vtt 파일에서 우선순위 높은 것 선택
    let mut found_files: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "vtt" {
                    let parts: Vec<&str> = filename.split('.').collect();
                    if parts.len() >= 3 {
                        let raw_lang = parts[parts.len() - 2].to_string();
                        let lang_code = normalize(&raw_lang);
                        found_files.push((lang_code, path));
                    }
                }
            }
        }
    }

    if found_files.is_empty() {
        return Err(msg(loc, "download_failed"));
    }

    // 우선순위 순 선택
    let mut best: Option<&(String, PathBuf)> = None;
    for lang in &langs {
        if let Some(entry) = found_files.iter().find(|(l, _)| l == lang) {
            best = Some(entry);
            break;
        }
    }
    if best.is_none() {
        best = found_files.first();
    }

    match best {
        Some((lang_code, path)) => {
            let done_msg = match loc {
                "ko" => format!("{} 자막 발견! 처리 중...", lang_name(lang_code, loc)),
                "ja" => format!("{} 字幕発見！処理中...", lang_name(lang_code, loc)),
                "zh" => format!("{} 字幕已找到！处理中...", lang_name(lang_code, loc)),
                _ => format!("{} subtitle found! Processing...", lang_name(lang_code, loc)),
            };
            emit_log(&app, &done_msg);

            let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

            // 정리
            if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            Ok(clean_subtitle(&content))
        }
        None => Err(msg(loc, "download_failed")),
    }
}

fn is_youtube_url(url: &str) -> bool {
    let url = url.trim();
    // https://www.youtube.com/watch?v=... 또는 https://youtu.be/... 형식만 허용
    if let Some(host_start) = url.find("://") {
        let after_scheme = &url[host_start + 3..];
        let host = after_scheme.split('/').next().unwrap_or("");
        let host = host.split(':').next().unwrap_or(""); // 포트 제거
        matches!(host,
            "youtube.com" | "www.youtube.com" | "m.youtube.com"
            | "youtu.be" | "www.youtu.be"
        )
    } else {
        false
    }
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
