#!/usr/bin/env bash
# =============================================================================
# 인프라 설치 스크립트 (1회성)
# 대상 OS: Ubuntu LTS (22.04 / 24.04)
# 설치 항목: Docker, k3s(경량 Kubernetes), NGINX Ingress Controller
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

echo ""
echo "  ┌────────────────────────────────────────┐"
echo "  │  인프라 설치 스크립트                  │"
echo "  │  Docker + k3s + NGINX Ingress          │"
echo "  └────────────────────────────────────────┘"
echo ""

# =============================================================================
# 1. 시스템 패키지 업데이트
# =============================================================================
step "1/4 시스템 패키지 업데이트"
apt-get update -qq
apt-get upgrade -y -qq
apt-get install -y -qq \
  curl wget git ca-certificates gnupg lsb-release \
  openssl apt-transport-https software-properties-common
log "시스템 패키지 업데이트 완료"

# =============================================================================
# 2. Docker 설치
# =============================================================================
step "2/4 Docker 설치"
if command -v docker &>/dev/null; then
  log "Docker 이미 설치됨: $(docker --version)"
else
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
step "3/4 k3s 설치"
if command -v kubectl &>/dev/null && kubectl cluster-info &>/dev/null 2>&1; then
  log "Kubernetes 이미 동작 중"
else
  curl -sfL https://get.k3s.io | sh -s - \
    --write-kubeconfig-mode 644 \
    --disable traefik

  if ! command -v kubectl &>/dev/null; then
    ln -sf /usr/local/bin/k3s /usr/local/bin/kubectl
  fi

  export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
  mkdir -p /root/.kube
  cp /etc/rancher/k3s/k3s.yaml /root/.kube/config

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
step "4/4 NGINX Ingress Controller 설치"
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
# 완료
# =============================================================================
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  인프라 설치 완료!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  설치된 항목:"
echo "    - Docker: $(docker --version 2>/dev/null || echo 'N/A')"
echo "    - k3s:    $(kubectl version --client --short 2>/dev/null || kubectl version --client 2>/dev/null || echo 'N/A')"
echo "    - NGINX Ingress Controller"
echo ""
echo "  다음 단계:"
echo "    1. .env.example → .env 복사 후 값 설정"
echo "    2. scripts/app-deploy.sh 로 앱 배포"
echo ""
