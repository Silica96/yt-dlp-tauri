# yt-dlp GUI

yt-dlp를 활용한 사용자 친화적인 비디오 다운로더 데스크톱 애플리케이션

## 프로젝트 개요

- **목표**: 기술에 익숙하지 않은 사용자도 쉽게 사용할 수 있는 영상 다운로더
- **플랫폼**: macOS, Windows
- **프레임워크**: Tauri 2.0

## 기술 스택

### Frontend
- Vue 3 + TypeScript
- Tailwind CSS v4
- Pinia (상태 관리)
- VueUse (유틸리티 컴포저블)

### Backend (Rust)
- Tauri 2.0
- tokio (비동기 런타임)
- serde (JSON 직렬화)
- reqwest (HTTP 클라이언트)

## 핵심 기능

1. **단일 영상 다운로드**: URL 입력 → 화질 선택 → 다운로드
2. **플레이리스트 다운로드**: 전체 또는 선택적 다운로드
3. **MP3 오디오 추출**: 영상에서 오디오만 추출
4. **yt-dlp 자동 업데이트**: GitHub API로 최신 버전 체크 및 업데이트

## 프로젝트 구조

```
yt-dlp-gui/
├── src/                      # Vue 프론트엔드
│   ├── components/           # Vue 컴포넌트
│   ├── composables/          # Vue 컴포저블 (hooks)
│   ├── stores/               # Pinia 스토어
│   ├── App.vue
│   └── main.ts
├── src-tauri/               # Rust 백엔드
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── ytdlp/           # yt-dlp 관련 모듈
│   │   └── commands.rs      # Tauri 커맨드
│   ├── Cargo.toml
│   └── tauri.conf.json
└── claude.md                # 이 파일
```

## 개발 명령어

```bash
# 의존성 설치
npm install

# 개발 서버 실행
npm run tauri dev

# 프로덕션 빌드
npm run tauri build
```

## yt-dlp 옵션 매핑

| GUI 옵션 | yt-dlp 명령어 |
|---------|--------------|
| 최고화질 | `-f "bv*+ba/b"` |
| 중간화질 | `-f "bv*[height<=720]+ba/b"` |
| 오디오만 (MP3) | `-x --audio-format mp3` |
| 플레이리스트 정보 | `--flat-playlist -J` |
| 특정 항목 | `--playlist-items 1,3,5` |

## 아키텍처 원칙

### yt-dlp 바이너리 관리
- yt-dlp와 ffmpeg 바이너리는 앱 데이터 폴더에 분리 저장
- 앱 시작 시 GitHub API로 최신 버전 확인
- 백그라운드에서 자동 업데이트 지원

### UI/UX 원칙
- 큰 글씨 (16px+ 기본)
- 단순한 인터페이스
- 명확한 진행률 표시
- 친절한 오류 메시지

## Tauri 커맨드 규칙

```rust
// 커맨드 정의
#[tauri::command]
async fn command_name(arg: String) -> Result<ReturnType, String> {
    // 구현
}

// lib.rs에서 등록
.invoke_handler(tauri::generate_handler![command_name])
```

## 참고 자료

- [yt-dlp GitHub](https://github.com/yt-dlp/yt-dlp)
- [Tauri 2.0 문서](https://v2.tauri.app/)
- [Vue 3 문서](https://vuejs.org/)
