# NooTube Subtitle Extractor

YouTube 영상의 자막을 **한 번의 클릭**으로 텍스트로 변환하는 데스크탑 앱입니다.

영어 강의, 해외 뉴스, 외국어 콘텐츠를 볼 때 — URL만 붙여넣으면 자막이 바로 텍스트로 나옵니다. 번역기에 붙여넣거나 노트에 정리하거나, 원하는 대로 활용하세요.

## 이런 분에게 추천합니다

- 영어 유튜브 강의를 텍스트로 정리하고 싶은 분
- 해외 뉴스/인터뷰 내용을 빠르게 파악하고 싶은 분
- 외국어 영상에서 한국어 번역 자막을 뽑고 싶은 분

## 주요 기능

### 1. URL만 붙여넣으면 끝
YouTube 영상 URL을 입력하고 추출 버튼을 누르면, 깨끗하게 정리된 자막 텍스트가 바로 나옵니다.

### 2. 자막 언어 우선순위
한국어, 영어, 일본어, 중국어 중 원하는 언어 순서를 정할 수 있습니다. 메인 화면에서 **클릭 한 번**으로 우선순위를 바꿀 수 있고, 1순위 자막이 없으면 자동으로 다음 언어를 시도합니다.

예) 한국어 우선이면 → 한국어 번역 자막을 먼저 가져오고, 없으면 영어 원문을 가져옵니다.

### 3. 자동 클립보드 복사
자막 추출이 완료되면 **자동으로 클립보드에 복사**됩니다. 바로 번역기나 메모장에 붙여넣기 하세요.

### 4. 추출 중지
시간이 오래 걸리거나 잘못된 영상을 넣었을 때, 추출 중지 버튼으로 즉시 멈출 수 있습니다.

### 5. 한국어/영어 UI
앱 인터페이스를 한국어 또는 영어로 전환할 수 있습니다.

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
