.PHONY: help build run test clean docker-up docker-down docker-logs api-test

help: ## 도움말 표시
	@echo "사용 가능한 명령어:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## 프로젝트 빌드
	cargo build

build-release: ## Release 모드로 빌드
	cargo build --release

run: ## 서버 실행 (개발 모드)
	cargo run

test: ## 유닛 테스트 실행
	cargo test

test-verbose: ## 테스트 실행 (상세 로그)
	cargo test -- --nocapture

clean: ## 빌드 아티팩트 정리
	cargo clean

docker-up: ## MongoDB & Redis 실행
	docker-compose up -d
	@echo "✅ MongoDB: mongodb://localhost:27017"
	@echo "✅ Redis: redis://localhost:6379"

docker-down: ## Docker 컨테이너 중지
	docker-compose down

docker-logs: ## Docker 로그 확인
	docker-compose logs -f

docker-clean: ## Docker 볼륨까지 삭제
	docker-compose down -v

api-test: ## API 테스트 실행
	chmod +x test-api.sh
	./test-api.sh

dev: docker-up ## 개발 환경 시작 (Docker + 서버)
	@echo "⏳ MongoDB & Redis 시작 대기..."
	@sleep 3
	cargo run

check: ## 코드 검사 (컴파일 + 린트)
	cargo check
	cargo clippy

fmt: ## 코드 포맷팅
	cargo fmt

watch: ## 파일 변경 감지 및 자동 재시작 (cargo-watch 필요)
	cargo watch -x run

install-tools: ## 개발 도구 설치
	cargo install cargo-watch
	@echo "✅ cargo-watch 설치 완료"