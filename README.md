# NooTube Subtitle Extractor

YouTube 영상에서 자막을 추출하는 데스크탑 애플리케이션입니다.

> 이 프로젝트는 **개인 학습 목적**으로 제작되었습니다.

## 기능

- YouTube URL을 입력하면 자막을 자동 추출
- 자막 언어 우선순위 설정 (한국어, 영어, 일본어, 중국어)
- 추출된 자막 클립보드 자동 복사
- 한국어/영어 UI 지원

## 기술 스택

- **Frontend**: React, TypeScript, Vite
- **Backend**: Rust, Tauri 2
- **자막 추출**: yt-dlp

## 설치 및 실행

### 개발 모드

```bash
pnpm install
pnpm tauri dev
```

### 빌드

```bash
pnpm tauri build
```

## 다운로드

[Releases](../../releases) 페이지에서 플랫폼별 설치 파일을 다운로드할 수 있습니다.

- **macOS (Apple Silicon)**: `.dmg` (aarch64)
- **macOS (Intel)**: `.dmg` (x64)
- **Windows**: `.msi` / `.exe`

## 라이선스

이 프로젝트는 [MIT License](./LICENSE)로 배포됩니다.

## 오픈소스 고지

이 프로젝트는 아래의 오픈소스 소프트웨어를 사용합니다.

| 라이브러리 | 라이선스 |
|---|---|
| [Tauri](https://tauri.app/) | MIT / Apache-2.0 |
| [React](https://react.dev/) | MIT |
| [Vite](https://vite.dev/) | MIT |
| [TypeScript](https://www.typescriptlang.org/) | Apache-2.0 |
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | Unlicense |
| [Tokio](https://tokio.rs/) | MIT |
| [Serde](https://serde.rs/) | MIT / Apache-2.0 |
