#!/usr/bin/env bash
# =============================================================================
# 인프라 설치 스크립트 (1회성)
# 대상 OS: Ubuntu LTS (22.04 / 24.04)
# 설치 항목: Docker, 로컬 레지스트리, k3s, NGINX Ingress Controller
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

REGISTRY_PORT="${REGISTRY_PORT:-5000}"

echo ""
echo "  ┌────────────────────────────────────────┐"
echo "  │  인프라 설치 스크립트                  │"
echo "  │  Docker + Registry + k3s + Ingress     │"
echo "  └────────────────────────────────────────┘"
echo ""

# =============================================================================
# 1. 시스템 패키지 업데이트
# =============================================================================
step "1/6 시스템 패키지 업데이트"
apt-get update -qq
apt-get upgrade -y -qq
apt-get install -y -qq \
  curl wget git ca-certificates gnupg lsb-release \
  openssl apt-transport-https software-properties-common
log "시스템 패키지 업데이트 완료"

# =============================================================================
# 2. Docker 설치
# =============================================================================
step "2/6 Docker 설치"
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
# 3. 로컬 Docker 레지스트리 설치
# =============================================================================
step "3/6 로컬 Docker 레지스트리 설치 (포트 ${REGISTRY_PORT})"
if docker ps --format '{{.Names}}' | grep -q '^registry$'; then
  log "로컬 레지스트리 이미 실행 중"
else
  docker run -d \
    --name registry \
    --restart always \
    -p "${REGISTRY_PORT}:5000" \
    -v registry-data:/var/lib/registry \
    registry:2
  log "로컬 레지스트리 실행 완료: localhost:${REGISTRY_PORT}"
fi

# Docker 데몬이 로컬 레지스트리를 insecure로 허용
DAEMON_JSON="/etc/docker/daemon.json"
if [[ -f "$DAEMON_JSON" ]] && grep -q "localhost:${REGISTRY_PORT}" "$DAEMON_JSON"; then
  log "Docker insecure-registries 이미 설정됨"
else
  cat > "$DAEMON_JSON" <<EOF
{
  "insecure-registries": ["localhost:${REGISTRY_PORT}"]
}
EOF
  systemctl restart docker
  # 레지스트리 컨테이너 재시작 (docker restart로 인해 중지될 수 있음)
  docker start registry 2>/dev/null || true
  log "Docker insecure-registries 설정 완료"
fi

# =============================================================================
# 4. k3s (경량 Kubernetes) 설치
# =============================================================================
step "4/6 k3s 설치"
if command -v kubectl &>/dev/null && kubectl cluster-info &>/dev/null 2>&1; then
  log "Kubernetes 이미 동작 중"
else
  # k3s도 로컬 레지스트리를 insecure로 허용
  mkdir -p /etc/rancher/k3s
  cat > /etc/rancher/k3s/registries.yaml <<EOF
mirrors:
  "localhost:${REGISTRY_PORT}":
    endpoint:
      - "http://localhost:${REGISTRY_PORT}"
EOF

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
# 5. NGINX Ingress Controller 설치
# =============================================================================
step "5/6 NGINX Ingress Controller 설치"
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
# 6. Jenkins 권한 설정
# =============================================================================
step "6/6 Jenkins 권한 설정"
if id jenkins &>/dev/null 2>&1; then
  # jenkins 사용자를 docker 그룹에 추가
  if groups jenkins | grep -q docker; then
    log "Jenkins docker 그룹 이미 설정됨"
  else
    usermod -aG docker jenkins
    log "Jenkins → docker 그룹 추가 완료"
  fi

  # kubectl 권한 (kubeconfig 복사)
  JENKINS_HOME=$(eval echo ~jenkins)
  mkdir -p "${JENKINS_HOME}/.kube"
  cp /etc/rancher/k3s/k3s.yaml "${JENKINS_HOME}/.kube/config"
  chown -R jenkins:jenkins "${JENKINS_HOME}/.kube"
  log "Jenkins kubeconfig 설정 완료: ${JENKINS_HOME}/.kube/config"
else
  warn "Jenkins 사용자가 없습니다. Jenkins 설치 후 deploy.sh를 다시 실행하세요."
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
echo "    - Docker:    $(docker --version 2>/dev/null || echo 'N/A')"
echo "    - Registry:  localhost:${REGISTRY_PORT}"
echo "    - k3s:       $(kubectl version --client --short 2>/dev/null || kubectl version --client 2>/dev/null || echo 'N/A')"
echo "    - NGINX Ingress Controller"
echo ""
echo "  다음 단계:"
echo "    1. .env.example → .env 복사 후 값 설정"
echo "    2. Jenkins 설정 또는 scripts/app-deploy.sh 로 앱 배포"
echo ""
