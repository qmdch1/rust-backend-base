# Rust Backend Base

Production-ready Rust 백엔드 보일러플레이트. 업계 표준 기술 스택으로 구성되어 있어 클론 후 바로 백엔드 개발을 시작할 수 있습니다.

## Deploy

> **Ubuntu 22.04 LTS / 24.04 LTS** 기준 | PostgreSQL, Valkey(Redis)는 별도 구성 전제

배포는 **2단계**로 나뉩니다:

| 단계 | 스크립트 | 실행 빈도 | 역할 |
|------|----------|-----------|------|
| 1 | `deploy.sh` | 서버 초기 세팅 시 **1회** | Docker, k3s, NGINX Ingress 설치 |
| 2 | `scripts/app-deploy.sh` | 코드 변경 시 **반복** | 이미지 빌드 → .env → K8s Secret → Kustomize 배포 |

### Step 1. 인프라 설치 (1회)

```bash
sudo ./deploy.sh
```

### Step 2. 환경변수 설정

```bash
cp .env.example .env
vi .env  # 실제 값으로 수정
```

### Step 3. 앱 배포 (반복)

```bash
# 기본 (prod 환경)
sudo ./scripts/app-deploy.sh

# 환경 지정
sudo DEPLOY_ENV=dev ./scripts/app-deploy.sh       # dev | staging | prod
```

## Tech Stack

| 영역 | 기술 | 설명 |
|------|------|------|
| **Web Framework** | [Axum](https://github.com/tokio-rs/axum) | Tokio 기반 업계 표준 웹 프레임워크 |
| **Runtime** | [Tokio](https://tokio.rs/) | 비동기 런타임 |
| **Database** | [SQLx](https://github.com/launchbadge/sqlx) + PostgreSQL | 컴파일 타임 체크 SQL, 마이그레이션 내장 |
| **Cache** | [Redis](https://github.com/redis-rs/redis-rs) | 세션/캐시용 인메모리 스토어 |
| **Authentication** | [jsonwebtoken](https://github.com/Keats/jsonwebtoken) | JWT (Access + Refresh Token) |
| **Password** | [Argon2](https://github.com/RustCrypto/password-hashes) | 업계 표준 패스워드 해싱 (OWASP 권장) |
| **Serialization** | [Serde](https://serde.rs/) | JSON 직렬화/역직렬화 |
| **Validation** | [Validator](https://github.com/Keats/validator) | 요청 데이터 유효성 검증 |
| **Error Handling** | [thiserror](https://github.com/dtolnay/thiserror) + [anyhow](https://github.com/dtolnay/anyhow) | 타입 안전 에러 처리 |
| **Logging** | [tracing](https://github.com/tokio-rs/tracing) | 구조화된 로깅 + 분산 트레이싱 |
| **Config** | [dotenvy](https://github.com/allan2/dotenvy) | 환경 변수 기반 설정 |
| **Middleware** | [Tower](https://github.com/tower-rs/tower) + [tower-http](https://github.com/tower-rs/tower-http) | CORS, Compression, Rate Limiting, Tracing |
| **UUID** | [uuid](https://github.com/uuid-rs/uuid) | v4 UUID 생성 |
| **DateTime** | [chrono](https://github.com/chronotope/chrono) | 날짜/시간 처리 |

| **Deployment** | [Kubernetes](https://kubernetes.io/) + [Kustomize](https://kustomize.io/) | 환경별 K8s 배포 (dev/staging/prod) |

## Project Structure

```
rust-backend-base/
├── Cargo.toml                  # 의존성 관리
├── Dockerfile                  # 멀티스테이지 Docker 빌드
├── docker-compose.yml          # 로컬 개발용 (PostgreSQL + Redis)
├── .env                        # 환경 변수 (git 제외, 서버에 직접 관리)
├── .env.example                # 환경 변수 템플릿
├── deploy.sh                   # 인프라 설치 (Docker, k3s, Ingress) - 1회성
├── scripts/
│   └── app-deploy.sh           # 앱 배포 (.env → Secret, 이미지 빌드, K8s 배포) - 반복
├── migrations/                 # SQLx 데이터베이스 마이그레이션
│   └── 20240101000000_create_users_table.sql
├── k8s/                        # Kubernetes 매니페스트
│   ├── base/                   # 공통 리소스
│   │   ├── kustomization.yaml
│   │   ├── namespace.yaml
│   │   ├── deployment.yaml     # App (envFrom: Secret, probes, resources)
│   │   ├── service.yaml        # ClusterIP Service
│   │   ├── ingress.yaml        # Nginx Ingress + TLS
│   │   ├── hpa.yaml            # HorizontalPodAutoscaler
│   │   ├── postgres.yaml       # PostgreSQL (Deployment + PVC + Service)
│   │   └── valkey.yaml         # Redis (Deployment + Service)
│   └── overlays/               # 환경별 오버라이드
│       ├── dev/                # 개발 (1 replica)
│       ├── staging/            # 스테이징 (2 replicas)
│       └── prod/               # 프로덕션 (3+ replicas, HPA 20까지)
└── src/
    ├── main.rs                 # 엔트리포인트 (서버 초기화)
    ├── config/                 # 환경 변수 → Config 구조체
    │   └── mod.rs
    ├── db/                     # 데이터베이스 연결
    │   ├── mod.rs
    │   ├── postgres.rs         # PostgreSQL 풀 + 마이그레이션
    │   └── redis.rs            # Redis 연결 매니저
    ├── errors/                 # 통합 에러 타입 (AppError → HTTP 응답)
    │   └── mod.rs
    ├── auth/                   # 인증 유틸리티
    │   ├── mod.rs
    │   ├── jwt.rs              # JWT 생성/검증
    │   └── password.rs         # Argon2 해싱/검증
    ├── models/                 # 데이터 모델 + DTO
    │   ├── mod.rs
    │   └── user.rs             # User 모델, Request/Response DTO
    ├── services/               # 비즈니스 로직
    │   ├── mod.rs
    │   └── user_service.rs     # CRUD 작업
    ├── middleware/              # Axum 미들웨어
    │   ├── mod.rs              # CORS, Compression, Tracing, Body Limit
    │   └── auth.rs             # JWT 인증 미들웨어
    └── routes/                 # API 라우터
        ├── mod.rs              # 라우터 조립 (public + protected)
        └── handlers/           # 핸들러 (컨트롤러)
            ├── mod.rs
            ├── health_handler.rs
            ├── auth_handler.rs
            └── user_handler.rs
```

## Quick Start

### 1. 사전 요구사항

- [Rust](https://rustup.rs/) (1.75+)
- [Docker](https://www.docker.com/) & Docker Compose

### 2. 인프라 실행

```bash
# PostgreSQL + Redis 실행
docker compose up -d
```

### 3. 환경 설정

```bash
# .env.example을 참고하여 .env 수정
cp .env.example .env
# 필요시 값 수정
```

### 4. 서버 실행

```bash
# 개발 모드 실행
cargo run

# 또는 watch 모드 (cargo-watch 설치 필요)
cargo watch -x run
```

서버가 `http://localhost:8080`에서 시작됩니다.

## API Endpoints

### Public (인증 불필요)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/health` | 헬스 체크 (DB 상태 포함) |
| `POST` | `/api/v1/auth/register` | 회원가입 |
| `POST` | `/api/v1/auth/login` | 로그인 (JWT 발급) |
| `POST` | `/api/v1/auth/refresh` | 토큰 갱신 |

### Protected (Bearer Token 필요)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/users/me` | 내 정보 조회 |
| `PUT` | `/api/v1/users/me` | 내 정보 수정 |
| `GET` | `/api/v1/users` | 유저 목록 (페이지네이션) |
| `GET` | `/api/v1/users/{id}` | 유저 조회 |
| `DELETE` | `/api/v1/users/{id}` | 유저 삭제 (본인/관리자) |

## API Usage Examples

### 회원가입

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123", "name": "John"}'
```

### 로그인

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password123"}'
```

### 인증된 요청

```bash
curl http://localhost:8080/api/v1/users/me \
  -H "Authorization: Bearer <access_token>"
```

### 토큰 갱신

```bash
curl -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "<refresh_token>"}'
```

## Commands

### 자주 쓰는 명령어 요약

```bash
# Rust 설치 직후 PATH 등록 (최초 1회)
source "$HOME/.cargo/env"

# PATH 영구 등록 (다음 터미널부터 자동 적용)
echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
```

| 명령어 | 설명 |
|--------|------|
| `cargo run` | 서버 실행 (개발 모드) |
| `cargo build` | 디버그 빌드 |
| `cargo build --release` | 릴리즈 빌드 (LTO + strip 최적화) |
| `cargo test` | 전체 테스트 실행 |
| `cargo test --test hello_test` | 특정 테스트 파일만 실행 |
| `cargo check` | 컴파일 체크만 (빌드 없이 에러 확인) |
| `cargo clippy` | 린트 (코드 품질 검사) |
| `cargo fmt` | 코드 포맷팅 |
| `cargo watch -x run` | 파일 변경 시 자동 재시작 (cargo-watch 필요) |
| `cargo watch -x test` | 파일 변경 시 자동 테스트 (cargo-watch 필요) |

### 테스트

테스트 파일은 `tests/` 디렉토리에 있습니다. DB/Redis 없이 순수 HTTP 핸들러를 검증합니다.

```bash
# 전체 테스트 실행
cargo test

# 특정 테스트 파일만 실행
cargo test --test hello_test

# 테스트 이름으로 필터링
cargo test test_hello_world

# 테스트 출력(println 등) 보기
cargo test -- --nocapture

# 테스트 목록만 확인 (실행하지 않음)
cargo test -- --list
```

테스트 파일 구조:
```
tests/
└── hello_test.rs    → GET /api/v1/hello 응답 검증
```

새 테스트 추가 시 `tests/` 폴더에 `_test.rs` 파일을 만들면 `cargo test`가 자동으로 인식합니다.

### 마이그레이션

```bash
# SQLx CLI 설치
cargo install sqlx-cli --no-default-features --features postgres

# 마이그레이션 생성
sqlx migrate add <migration_name>

# 마이그레이션 수동 실행 (서버 시작 시 자동 실행됨)
sqlx migrate run

# 마이그레이션 롤백
sqlx migrate revert
```

### Docker

```bash
# 이미지 빌드
docker build -t rust-backend .

# 컨테이너 실행
docker run -p 8080:8080 --env-file .env rust-backend
```

## Kubernetes Deployment

### 구조

```
docker-compose.yml  → 로컬 개발 전용 (DB + Redis만 띄움)
deploy.sh           → 인프라 설치 (Docker, k3s, Ingress) - 1회
scripts/app-deploy.sh → 앱 배포 (.env → Secret, 빌드, K8s 배포) - 반복
k8s/                → Kustomize 기반 환경 분리
  base/             → 공통 매니페스트
  overlays/
    dev/            → 1 replica, 낮은 리소스
    staging/        → 2 replicas
    prod/           → 3 replicas, HPA 최대 20, 높은 리소스
```

### 환경변수 관리

모든 환경변수는 `.env` 파일 하나로 관리합니다.

```
.env.example  → 템플릿 (git 포함)
.env          → 실제 값 (git 제외, 서버에 직접 관리)
```

`app-deploy.sh` 실행 시 `.env` → K8s Secret으로 자동 변환되어 Pod에 주입됩니다.

### 환경별 차이

| 설정 | Dev | Staging | Prod |
|------|-----|---------|------|
| Replicas | 1 | 2 | 3 |
| HPA Max | 3 | 5 | 20 |
| CPU Request | 50m | 100m | 200m |
| Memory Request | 64Mi | 128Mi | 256Mi |

### 주요 포함 리소스

- **Deployment**: Readiness/Liveness probe (`/api/v1/health`), resource limits
- **Service**: ClusterIP (내부 통신)
- **Ingress**: Nginx Ingress Controller + TLS 종료
- **HPA**: CPU 70% / Memory 80% 기준 오토스케일링
- **PostgreSQL**: PVC 10Gi 영구 볼륨
- **Redis**: 인메모리 캐시

## Architecture Decisions

- **Axum over Actix-web**: Tokio 에코시스템과의 네이티브 통합, Tower 미들웨어 호환성
- **SQLx over Diesel**: 런타임 오버헤드 없이 컴파일 타임 SQL 검증, async 네이티브 지원
- **Argon2 over bcrypt**: OWASP 권장 최신 패스워드 해싱 알고리즘
- **tracing over log**: 구조화된 로깅, span 기반 분산 트레이싱 지원
- **thiserror + anyhow**: 라이브러리/앱 레벨 에러 처리 분리

## 확장 가이드

이 베이스에 추가로 구축할 수 있는 기능:

- **WebSocket**: `axum::extract::ws` 사용
- **Background Jobs**: `tokio::spawn` 또는 `apalis` 크레이트
- **File Upload**: `axum::extract::Multipart`
- **API Documentation**: `utoipa` (OpenAPI/Swagger)
- **Rate Limiting**: `tower_governor` 크레이트
- **Email**: `lettre` 크레이트
- **Testing**: `reqwest` + `sqlx::test` (dev-dependencies에 포함됨)
- **gRPC**: `tonic` 크레이트
- **Message Queue**: `lapin` (RabbitMQ) 또는 `rdkafka` (Kafka)
