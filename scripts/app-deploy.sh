#!/usr/bin/env bash
# =============================================================================
# 앱 배포 스크립트 (반복 실행)
# .env 파일 기반으로 Docker 이미지 빌드 → K8s Secret 생성 → Kustomize 배포
# 사전 조건: deploy.sh 로 인프라(Docker, k3s, Ingress) 설치 완료
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
  err "root 권한이 필요합니다. sudo ./scripts/app-deploy.sh 로 실행하세요."
fi

# ── 설정 ──
IMAGE_NAME="rust-backend"
IMAGE_TAG="${IMAGE_TAG:-latest}"
NAMESPACE="rust-backend"
DEPLOY_ENV="${DEPLOY_ENV:-prod}"  # dev | staging | prod
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="${ENV_FILE:-${PROJECT_DIR}/.env}"
KUBECONFIG="${KUBECONFIG:-/etc/rancher/k3s/k3s.yaml}"
export KUBECONFIG

# ── .env 파일 확인 ──
if [[ ! -f "$ENV_FILE" ]]; then
  err ".env 파일이 없습니다: ${ENV_FILE}\n  .env.example 을 복사해서 값을 채워주세요:\n  cp .env.example .env"
fi

# ── 사전 조건 확인 ──
command -v docker &>/dev/null || err "Docker가 설치되어 있지 않습니다. deploy.sh를 먼저 실행하세요."
command -v kubectl &>/dev/null || err "kubectl이 설치되어 있지 않습니다. deploy.sh를 먼저 실행하세요."
kubectl cluster-info &>/dev/null 2>&1 || err "Kubernetes 클러스터에 연결할 수 없습니다. deploy.sh를 먼저 실행하세요."

echo ""
echo "  ┌────────────────────────────────────────┐"
echo "  │  앱 배포 스크립트                      │"
echo "  │  Env: ${DEPLOY_ENV}  │  Image: ${IMAGE_NAME}:${IMAGE_TAG}  │"
echo "  │  .env: ${ENV_FILE}"
echo "  └────────────────────────────────────────┘"
echo ""

# =============================================================================
# 1. Docker 이미지 빌드
# =============================================================================
step "1/4 Docker 이미지 빌드"
cd "$PROJECT_DIR"
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .
docker save "${IMAGE_NAME}:${IMAGE_TAG}" | k3s ctr images import -
log "이미지 빌드 & import 완료: ${IMAGE_NAME}:${IMAGE_TAG}"

# =============================================================================
# 2. Namespace 생성
# =============================================================================
step "2/4 Namespace 확인"
kubectl apply -f "${PROJECT_DIR}/k8s/base/namespace.yaml"
log "Namespace 준비 완료"

# =============================================================================
# 3. .env → K8s Secret 생성
# =============================================================================
step "3/4 .env → K8s Secret 생성"
kubectl -n "${NAMESPACE}" create secret generic rust-backend-secret \
  --from-env-file="${ENV_FILE}" \
  --dry-run=client -o yaml | kubectl apply -f -
log ".env → Secret 적용 완료"

# =============================================================================
# 4. Kustomize 배포
# =============================================================================
step "4/4 Kustomize 배포 (${DEPLOY_ENV})"
kubectl apply -k "${PROJECT_DIR}/k8s/overlays/${DEPLOY_ENV}"

# 기존 Pod 재시작 (.env 변경 반영)
kubectl -n "${NAMESPACE}" rollout restart deployment/rust-backend 2>/dev/null || true

echo -n "  배포 완료 대기 중..."
kubectl -n "${NAMESPACE}" rollout status deployment/rust-backend --timeout=120s 2>/dev/null || true
log "배포 완료"

# =============================================================================
# 결과 출력
# =============================================================================
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  앱 배포 완료!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  환경: ${DEPLOY_ENV}"
echo "  네임스페이스: ${NAMESPACE}"
echo ""
echo "  ── 확인 명령어 ──"
echo "  kubectl -n ${NAMESPACE} get pods"
echo "  kubectl -n ${NAMESPACE} get svc"
echo "  kubectl -n ${NAMESPACE} logs -l app.kubernetes.io/name=rust-backend -f"
echo ""
echo "  ── 로컬 테스트 ──"
NODE_PORT=$(kubectl -n "${NAMESPACE}" get svc rust-backend-service -o jsonpath='{.spec.ports[0].nodePort}' 2>/dev/null || echo "N/A")
if [[ "${NODE_PORT}" != "N/A" ]]; then
  echo "  curl http://localhost:${NODE_PORT}/api/v1/health"
else
  echo "  kubectl -n ${NAMESPACE} port-forward svc/rust-backend-service 8080:80 &"
  echo "  curl http://localhost:8080/api/v1/health"
fi
echo ""
