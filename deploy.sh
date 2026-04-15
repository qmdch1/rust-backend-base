#!/usr/bin/env bash
# =============================================================================
# rust-backend-base 프로덕션 한방 배포 스크립트
# 대상 OS: Ubuntu LTS (22.04 / 24.04)
# 설치 항목: Docker, k3s(경량 Kubernetes), kubectl, NGINX Ingress, 앱 빌드 & 배포
# PostgreSQL / Valkey(Redis)는 별도 구성 전제 (관리형 DB 권장)
# =============================================================================
set -euo pipefail

# ── 색상 ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()  { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
err()  { echo -e "${RED}[✗]${NC} $1"; exit 1; }
step() { echo -e "\n${BLUE}━━━ $1 ━━━${NC}"; }

# ── root 체크 ──
if [[ $EUID -ne 0 ]]; then
  err "root 권한이 필요합니다. sudo ./deploy.sh 로 실행하세요."
fi

# ── 설정 (필요시 수정) ──
APP_NAME="rust-backend"
IMAGE_NAME="rust-backend"
IMAGE_TAG="latest"
NAMESPACE="rust-backend"
DOMAIN="${DOMAIN:-api.yourdomain.com}"
DB_PASSWORD="${DB_PASSWORD:-$(openssl rand -base64 16)}"
JWT_SECRET="${JWT_SECRET:-$(openssl rand -base64 32)}"
DEPLOY_ENV="${DEPLOY_ENV:-prod}"  # dev | staging | prod
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo ""
echo "  ┌────────────────────────────────────────┐"
echo "  │  rust-backend-base 배포 스크립트       │"
echo "  │  OS: Ubuntu LTS  │  Env: ${DEPLOY_ENV}          │"
echo "  │  DB/Redis: 별도 구성 (스크립트 미포함) │"
echo "  └────────────────────────────────────────┘"
echo ""

# =============================================================================
# 1. 시스템 패키지 업데이트
# =============================================================================
step "1/5 시스템 패키지 업데이트"
apt-get update -qq
apt-get upgrade -y -qq
apt-get install -y -qq \
  curl wget git ca-certificates gnupg lsb-release \
  openssl apt-transport-https software-properties-common
log "시스템 패키지 업데이트 완료"

# =============================================================================
# 2. Docker 설치
# =============================================================================
step "2/5 Docker 설치"
if command -v docker &>/dev/null; then
  log "Docker 이미 설치됨: $(docker --version)"
else
  # Docker 공식 GPG 키 & 저장소
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
    gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  chmod a+r /etc/apt/keyrings/docker.gpg

  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
    https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | \
    tee /etc/apt/sources.list.d/docker.list > /dev/null

  apt-get update -qq
  apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-buildx-plugin

  systemctl enable --now docker
  log "Docker 설치 완료: $(docker --version)"
fi

# =============================================================================
# 3. k3s (경량 Kubernetes) 설치
# =============================================================================
step "3/5 k3s 설치"
if command -v kubectl &>/dev/null && kubectl cluster-info &>/dev/null 2>&1; then
  log "Kubernetes 이미 동작 중"
else
  curl -sfL https://get.k3s.io | sh -s - \
    --write-kubeconfig-mode 644 \
    --disable traefik

  # kubectl 심볼릭 링크
  if ! command -v kubectl &>/dev/null; then
    ln -sf /usr/local/bin/k3s /usr/local/bin/kubectl
  fi

  # kubeconfig 설정
  export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
  mkdir -p /root/.kube
  cp /etc/rancher/k3s/k3s.yaml /root/.kube/config

  # k3s 준비 대기
  echo -n "  k3s 준비 대기 중..."
  for i in $(seq 1 30); do
    if kubectl get nodes &>/dev/null 2>&1; then
      echo ""
      break
    fi
    echo -n "."
    sleep 2
  done
  log "k3s 설치 완료: $(kubectl version --short 2>/dev/null || kubectl version --client)"
fi

export KUBECONFIG="${KUBECONFIG:-/etc/rancher/k3s/k3s.yaml}"

# =============================================================================
# 4. NGINX Ingress Controller 설치
# =============================================================================
step "4/5 NGINX Ingress Controller 설치"
if kubectl get ns ingress-nginx &>/dev/null 2>&1; then
  log "NGINX Ingress 이미 설치됨"
else
  kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.10.1/deploy/static/provider/cloud/deploy.yaml
  echo -n "  Ingress Controller 준비 대기 중..."
  for i in $(seq 1 60); do
    if kubectl -n ingress-nginx get pods -l app.kubernetes.io/component=controller \
       -o jsonpath='{.items[0].status.phase}' 2>/dev/null | grep -q Running; then
      echo ""
      break
    fi
    echo -n "."
    sleep 3
  done
  log "NGINX Ingress Controller 설치 완료"
fi

# =============================================================================
# 5. Docker 이미지 빌드
# =============================================================================
step "5/5 Docker 이미지 빌드 & Kubernetes 배포"
cd "$PROJECT_DIR"

docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .

# k3s가 로컬 이미지를 사용할 수 있도록 import
docker save "${IMAGE_NAME}:${IMAGE_TAG}" | k3s ctr images import -
log "이미지 빌드 & import 완료: ${IMAGE_NAME}:${IMAGE_TAG}"

# ── Kubernetes 배포 ──
log "Kubernetes 배포 시작 (${DEPLOY_ENV})"

# Secret 값 주입 (base64)
DB_USER_B64=$(echo -n "postgres" | base64)
DB_PASS_B64=$(echo -n "${DB_PASSWORD}" | base64)
DB_URL_B64=$(echo -n "postgres://postgres:${DB_PASSWORD}@postgres-service:5432/rust_backend" | base64)
REDIS_URL_B64=$(echo -n "redis://valkey-service:6379" | base64)
JWT_SECRET_B64=$(echo -n "${JWT_SECRET}" | base64)
JWT_ACCESS_B64=$(echo -n "3600" | base64)
JWT_REFRESH_B64=$(echo -n "604800" | base64)

# namespace 먼저 생성
kubectl apply -f "${PROJECT_DIR}/k8s/base/namespace.yaml"

# 실제 Secret 적용 (템플릿 대신 직접 생성)
kubectl apply -f - <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: rust-backend-secret
  namespace: ${NAMESPACE}
type: Opaque
data:
  DATABASE_URL: ${DB_URL_B64}
  REDIS_URL: ${REDIS_URL_B64}
  JWT_SECRET: ${JWT_SECRET_B64}
  JWT_ACCESS_TOKEN_EXPIRY_SECS: ${JWT_ACCESS_B64}
  JWT_REFRESH_TOKEN_EXPIRY_SECS: ${JWT_REFRESH_B64}
---
apiVersion: v1
kind: Secret
metadata:
  name: postgres-secret
  namespace: ${NAMESPACE}
type: Opaque
data:
  POSTGRES_PASSWORD: ${DB_PASS_B64}
EOF

# kustomize로 나머지 리소스 배포
kubectl apply -k "${PROJECT_DIR}/k8s/overlays/${DEPLOY_ENV}"

# Secret은 kustomize가 덮어쓸 수 있으므로 다시 적용
kubectl apply -f - <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: rust-backend-secret
  namespace: ${NAMESPACE}
type: Opaque
data:
  DATABASE_URL: ${DB_URL_B64}
  REDIS_URL: ${REDIS_URL_B64}
  JWT_SECRET: ${JWT_SECRET_B64}
  JWT_ACCESS_TOKEN_EXPIRY_SECS: ${JWT_ACCESS_B64}
  JWT_REFRESH_TOKEN_EXPIRY_SECS: ${JWT_REFRESH_B64}
---
apiVersion: v1
kind: Secret
metadata:
  name: postgres-secret
  namespace: ${NAMESPACE}
type: Opaque
data:
  POSTGRES_PASSWORD: ${DB_PASS_B64}
EOF

# 배포 완료 대기
echo -n "  배포 완료 대기 중..."
kubectl -n "${NAMESPACE}" rollout status deployment/rust-backend --timeout=120s 2>/dev/null || true
log "Kubernetes 배포 완료"

# =============================================================================
# 결과 출력
# =============================================================================
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  배포 완료!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  환경: ${DEPLOY_ENV}"
echo "  네임스페이스: ${NAMESPACE}"
echo ""
echo "  ── 접속 정보 ──"
echo "  DB Password:  ${DB_PASSWORD}"
echo "  JWT Secret:   ${JWT_SECRET}"
echo ""
echo "  ── 확인 명령어 ──"
echo "  kubectl -n ${NAMESPACE} get pods"
echo "  kubectl -n ${NAMESPACE} get svc"
echo "  kubectl -n ${NAMESPACE} logs -l app.kubernetes.io/name=rust-backend -f"
echo ""
echo "  ── 로컬 테스트 (NodePort) ──"
NODE_PORT=$(kubectl -n "${NAMESPACE}" get svc rust-backend-service -o jsonpath='{.spec.ports[0].nodePort}' 2>/dev/null || echo "N/A")
if [[ "${NODE_PORT}" != "N/A" ]]; then
  echo "  curl http://localhost:${NODE_PORT}/api/v1/health"
else
  echo "  kubectl -n ${NAMESPACE} port-forward svc/rust-backend-service 8080:80 &"
  echo "  curl http://localhost:8080/api/v1/health"
fi
echo ""
echo -e "  ${YELLOW}위 DB Password / JWT Secret 은 안전한 곳에 보관하세요!${NC}"
echo ""
