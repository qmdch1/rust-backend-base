pipeline {
    agent any

    environment {
        REGISTRY      = 'localhost:5000'
        IMAGE_NAME    = 'rust-backend'
        IMAGE_TAG     = "${env.BUILD_NUMBER}"
        IMAGE         = "${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"
        IMAGE_LATEST  = "${REGISTRY}/${IMAGE_NAME}:latest"
        NAMESPACE     = 'rust-backend'
        DEPLOY_ENV    = "${params.DEPLOY_ENV ?: 'prod'}"
        KUBECONFIG    = '/etc/rancher/k3s/k3s.yaml'
    }

    parameters {
        choice(name: 'DEPLOY_ENV', choices: ['prod', 'staging', 'dev'], description: '배포 환경')
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Test') {
            steps {
                sh 'cargo test'
            }
        }

        stage('Build & Push') {
            steps {
                sh """
                    docker build -t ${IMAGE} -t ${IMAGE_LATEST} .
                    docker push ${IMAGE}
                    docker push ${IMAGE_LATEST}
                """
            }
        }

        stage('Deploy') {
            steps {
                // Jenkins Credentials에서 .env 파일 가져오기
                withCredentials([file(credentialsId: 'rust-backend-env', variable: 'ENV_FILE')]) {
                    sh """
                        kubectl -n ${NAMESPACE} create secret generic rust-backend-secret \
                            --from-env-file=\${ENV_FILE} \
                            --dry-run=client -o yaml | kubectl apply -f -
                    """
                }

                // Kustomize 배포
                sh "kubectl apply -k k8s/overlays/${DEPLOY_ENV}"

                // 새 이미지로 롤링 업데이트
                sh """
                    kubectl -n ${NAMESPACE} set image deployment/rust-backend \
                        rust-backend=${IMAGE}
                """

                // 배포 완료 대기
                sh """
                    kubectl -n ${NAMESPACE} rollout status deployment/rust-backend \
                        --timeout=120s
                """
            }
        }
    }

    post {
        success {
            echo "배포 성공: ${IMAGE} → ${DEPLOY_ENV}"
        }
        failure {
            echo "배포 실패: ${DEPLOY_ENV}"
            // 실패 시 이전 버전으로 롤백
            sh "kubectl -n ${NAMESPACE} rollout undo deployment/rust-backend || true"
        }
    }
}
