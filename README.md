# Rust Backend Base

Production-ready Rust 백엔드 보일러플레이트. 업계 표준 기술 스택으로 구성되어 있어 클론 후 바로 백엔드 개발을 시작할 수 있습니다.

## Deploy

> **Ubuntu 22.04 LTS / 24.04 LTS** 기준 | PostgreSQL, Valkey(Redis)는 별도 구성 전제

### 배포 구조

```
[Git Push] → [Jenkins Pipeline]
              ├── cargo test
              ├── docker build → 로컬 레지스트리 push (localhost:5000)
              ├── .env → K8s Secret
              └── kubectl apply (Kustomize 배포)
```

| 파일 | 역할 | 실행 빈도 |
|------|------|-----------|
| [`deploy.sh`](deploy.sh) | 인프라 설치 (Docker, Registry, k3s, Ingress) | 서버 초기 세팅 시 **1회** |
| [`Jenkinsfile`](Jenkinsfile) | CI/CD 파이프라인 (테스트→빌드→배포) | Git push 시 **자동** |

### Step 1. 인프라 설치 (1회)

```bash
sudo ./deploy.sh
```

Docker, 로컬 레지스트리(`localhost:5000`), k3s, NGINX Ingress를 설치합니다.

### Step 2. Jenkins 설정

**2-1. 필수 플러그인**

- Pipeline
- Git
- Credentials Binding

**2-2. Credentials 등록**

```
Jenkins 관리 → Credentials → System → Global credentials → Add Credentials
  Kind: Secret file
  File: .env 파일 업로드
  ID: rust-backend-env
```

**2-3. Pipeline 프로젝트 생성**

```
새 Item → Pipeline 선택
  Pipeline → Definition: Pipeline script from SCM
  SCM: Git
  Repository URL: <이 저장소 URL>
  Script Path: Jenkinsfile
```

**2-4. Jenkins 사용자 권한**

`deploy.sh`가 자동으로 설정합니다 (docker 그룹 추가 + kubeconfig 복사).
Jenkins 설치 후 `sudo ./deploy.sh`를 다시 실행하면 적용됩니다.

### Step 3. 배포

Git push → Jenkins가 자동으로 빌드/테스트/배포합니다.

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

| **CI/CD** | [Jenkins](https://www.jenkins.io/) | 파이프라인 자동 배포 |
| **Deployment** | [Kubernetes](https://kubernetes.io/) + [Kustomize](https://kustomize.io/) | 환경별 K8s 배포 (dev/staging/prod) |
| **Registry** | Docker Registry (로컬) | Docker Hub 대신 자체 레지스트리 (localhost:5000) |

## Project Structure

```
rust-backend-base/
├── Cargo.toml                  # 의존성 관리
├── Dockerfile                  # 멀티스테이지 Docker 빌드
├── docker-compose.yml          # 로컬 개발용 (PostgreSQL + Redis)
├── .env                        # 환경 변수 (git 제외, 서버에 직접 관리)
├── .env.example                # 환경 변수 템플릿
├── Jenkinsfile                 # CI/CD 파이프라인 (테스트→빌드→배포)
├── deploy.sh                   # 인프라 설치 (Docker, Registry, k3s, Ingress) - 1회성
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

## Kubernetes Deployment

### 구조

```
docker-compose.yml    → 로컬 개발 전용 (DB + Redis만 띄움)
deploy.sh             → 인프라 설치 (Docker, Registry, k3s, Ingress) - 1회
Jenkinsfile           → CI/CD 파이프라인 (테스트→빌드→배포) - 자동
scripts/app-deploy.sh → 수동 배포 (Jenkins 없이 직접 배포할 때)
k8s/                  → Kustomize 기반 환경 분리
  base/               → 공통 매니페스트
  overlays/
    dev/              → 1 replica, 낮은 리소스
    staging/          → 2 replicas
    prod/             → 3 replicas, HPA 최대 20, 높은 리소스
```

### 이미지 레지스트리

Docker Hub 대신 서버에 **로컬 레지스트리**(`localhost:5000`)를 사용합니다.
`deploy.sh`가 자동으로 설치하며, Jenkins와 k3s 모두 이 레지스트리를 통해 이미지를 주고받습니다.

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
