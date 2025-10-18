# 🚀 빠른 시작 가이드

## 1단계: MongoDB & Redis 실행

```bash
docker-compose up -d
```

## 2단계: 서버 시작

```bash
cargo run
```

서버가 `http://localhost:3000`에서 실행됩니다.

## 3단계: API 테스트

### 새 터미널에서 테스트 실행

```bash
chmod +x test-api.sh
./test-api.sh
```

또는 수동으로 API 호출:

### 1. Health Check
```bash
curl http://localhost:3000/health
```

### 2. 검색어 등록
```bash
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{"term": "hankook"}'
```

### 3. 검색어 추천
```bash
curl "http://localhost:3000/suggest?q=han&limit=5"
```

### 4. 실시간 인기 검색어
```bash
curl "http://localhost:3000/trending?limit=10"
```

## Makefile 사용

```bash
# 도움말
make help

# MongoDB & Redis 시작 + 서버 실행
make dev

# API 테스트
make api-test

# 테스트 실행
make test

# Docker 중지
make docker-down
```

## 환경 변수 설정 (선택사항)

`.env` 파일 생성:
```bash
cp .env.example .env
```

수정:
```env
MONGO_URI=mongodb://localhost:27017
MONGO_DB=search_db
MONGO_COLLECTION=search_terms
REDIS_URI=redis://localhost:6379
CACHE_TTL=300
PORT=3000
```

## 기본 예제 실행

```bash
cargo run --example basic_usage
```

## 문제 해결

### MongoDB 연결 실패
```bash
docker ps | grep mongo
docker logs tries-suggest-mongo
```

### Redis 연결 실패
```bash
docker exec -it tries-suggest-redis redis-cli ping
```

### 상세 로그 보기
```bash
RUST_LOG=debug cargo run
```