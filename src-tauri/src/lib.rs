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

    // Step 1: --list-subs로 사용 가능한 자막 목록 조회 (yt-dlp 1회 호출)
    let searching_msg = if loc == "ko" {
        format!("{} 자막 검색 중...", lang_display.join(", "))
    } else {
        format!("Searching {} subtitles...", lang_display.join(", "))
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
    let list_stderr = String::from_utf8_lossy(&list_output.stderr).to_string();
    eprintln!("[yt-dlp list-subs] {}", list_stderr);

    // --list-subs 출력 파싱: 사용 가능한 언어코드 수집
    let mut available_manual: Vec<String> = Vec::new();
    let mut available_auto: Vec<String> = Vec::new();
    let mut section: Option<&str> = None;

    for line in list_text.lines() {
        if line.contains("Available subtitles") {
            section = Some("manual");
            continue;
        }
        if line.contains("Available automatic captions") {
            section = Some("auto");
            continue;
        }
        if section.is_none() { continue; }

        // 언어코드 행: "ko       Korean       vtt, srt, ..."
        let trimmed = line.trim();
        if let Some(lang_code) = trimmed.split_whitespace().next() {
            if lang_code == "Language" || lang_code.is_empty() { continue; }
            // lang_code가 유효한 코드인지 확인 (알파벳 최소 1자 포함, 알파벳/숫자/하이픈만)
            if lang_code.chars().any(|c| c.is_ascii_alphabetic())
                && lang_code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                match section {
                    Some("manual") => available_manual.push(lang_code.to_string()),
                    Some("auto") => available_auto.push(lang_code.to_string()),
                    _ => {}
                }
            }
        }
    }

    eprintln!("[subtitle] manual: {:?}, auto count: {}", available_manual, available_auto.len());

    if available_manual.is_empty() && available_auto.is_empty() {
        return Err(msg(loc, "no_subtitle"));
    }

    // Step 2: 우선순위에 따라 후보 목록 생성
    // 정규화된 코드 비교 ("en-orig" → "en", "zh-Hans" → "zh")
    let normalize = |code: &str| -> String {
        code.split('-').next().unwrap_or(code).to_string()
    };

    // (언어코드, is_auto) 후보 리스트를 우선순위 순으로 생성
    let mut candidates: Vec<(String, bool)> = Vec::new();
    for lang in &langs {
        // 수동 자막 우선
        if let Some(found) = available_manual.iter().find(|c| normalize(c) == *lang) {
            candidates.push((found.clone(), false));
        }
        // 자동 자막
        if let Some(found) = available_auto.iter().find(|c| normalize(c) == *lang) {
            candidates.push((found.clone(), true));
        }
    }

    // 우선순위에 없으면 첫 번째 사용 가능한 것
    if candidates.is_empty() {
        if let Some(first) = available_manual.first() {
            candidates.push((first.clone(), false));
        } else if let Some(first) = available_auto.first() {
            candidates.push((first.clone(), true));
        }
    }

    if candidates.is_empty() {
        return Err(msg(loc, "no_subtitle"));
    }

    // Step 3: 후보 순서대로 다운로드 시도, 성공하면 즉시 반환
    for (selected_lang, is_auto) in &candidates {
        let display_lang = normalize(selected_lang);
        let dl_msg = if loc == "ko" {
            format!("{} 자막 다운로드 중...", lang_name(&display_lang, loc))
        } else {
            format!("Downloading {} subtitle...", lang_name(&display_lang, loc))
        };
        emit_log(&app, &dl_msg);

        // 기존 파일 정리
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }

        let sub_flag = if *is_auto { "--write-auto-sub" } else { "--write-sub" };

        let mut dl_cmd = Command::new(&yt_dlp_str);
        dl_cmd.args([
            "--skip-download",
            sub_flag,
            "--sub-format", "vtt",
            "--sub-lang", selected_lang.as_str(),
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
        eprintln!("[yt-dlp download {}] {}", selected_lang, dl_stderr);

        // 다운로드된 vtt 파일 찾기
        let mut subtitle_path: Option<PathBuf> = None;
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "vtt" {
                        subtitle_path = Some(path);
                        break;
                    }
                }
            }
        }

        if let Some(path) = subtitle_path {
            let done_msg = if loc == "ko" {
                format!("{} 자막 발견! 처리 중...", lang_name(&display_lang, loc))
            } else {
                format!("{} subtitle found! Processing...", lang_name(&display_lang, loc))
            };
            emit_log(&app, &done_msg);

            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

            // 정리
            if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            return Ok(clean_subtitle(&content));
        }

        // 실패 시 다음 후보로
        let fail_msg = if loc == "ko" {
            format!("{} 자막 실패, 다음 시도...", lang_name(&display_lang, loc))
        } else {
            format!("{} failed, trying next...", lang_name(&display_lang, loc))
        };
        emit_log(&app, &fail_msg);
    }

    Err(msg(loc, "download_failed"))
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
