# Tries-Suggest 🔍

Rust로 구현한 고성능 검색어 추천 시스템

## 🎯 특징

- **Trie 자료구조**: 빠른 prefix 기반 검색
- **MongoDB**: 검색어 영구 저장 및 통계
- **Redis**: 고속 캐싱 및 실시간 인기 검색어
- **REST API**: Axum 기반 웹 API
- **비동기 처리**: Tokio 런타임

## 🏗️ 아키텍처

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────┐
│      API Layer (Axum)       │
│  /suggest, /search, etc.    │
└──────────┬──────────────────┘
           │
           ▼
┌──────────────────────────────┐
│    Service Layer             │
│  SuggestionService           │
│  (비즈니스 로직 + Trie)       │
└──┬───────────┬───────────────┘
   │           │
   ▼           ▼
┌─────────┐ ┌──────────┐
│ MongoDB │ │  Redis   │
│(영구저장)│ │ (캐싱)   │
└─────────┘ └──────────┘
```

## 📁 디렉토리 구조

```
src/
├── main.rs              # 앱 진입점
├── domain/              # 도메인 로직
│   ├── mod.rs
│   └── tries.rs         # Trie 자료구조
├── repository/          # 데이터 레이어
│   ├── mod.rs
│   ├── mongodb.rs       # MongoDB 연동
│   └── redis.rs         # Redis 연동
├── service/             # 비즈니스 로직
│   ├── mod.rs
│   └── suggestion.rs    # 추천 서비스
└── api/                 # API 레이어
    ├── mod.rs
    └── routes.rs        # HTTP 핸들러
```

## 🚀 시작하기

### 1. 사전 요구사항

- Rust (1.70+)
- Docker & Docker Compose

### 2. MongoDB & Redis 실행

```bash
docker-compose up -d
```

### 3. 환경 변수 설정

```bash
cp .env.example .env
# .env 파일 수정 (필요시)
```

### 4. 의존성 설치 및 실행

```bash
cargo build
cargo run
```

서버가 `http://localhost:3000`에서 실행됩니다.

## 📡 API 엔드포인트

### 1. Health Check
```bash
GET /health
```

### 2. 검색어 추천
```bash
GET /suggest?q=han&limit=5

# 응답 예시:
{
  "success": true,
  "data": [
    ["hangul", 3],
    ["hankook", 2],
    ["handy", 1]
  ],
  "error": null
}
```

### 3. 검색어 기록
```bash
POST /search
Content-Type: application/json

{
  "term": "hankook"
}
```

### 4. 실시간 인기 검색어 (Redis)
```bash
GET /trending?limit=10

# 응답 예시:
{
  "success": true,
  "data": [
    ["hankook", 15.0],
    ["hangul", 12.0]
  ]
}
```

### 5. 전체 인기 검색어 (MongoDB)
```bash
GET /top?limit=10
```

### 6. 검색어 존재 확인
```bash
GET /contains?q=hankook

# 응답:
{
  "success": true,
  "data": true
}
```

### 7. Trie 재구성 (관리자용)
```bash
POST /rebuild
```

## 🧪 테스트

```bash
# 유닛 테스트
cargo test

# 특정 모듈 테스트
cargo test --lib domain::tries
```

## 📊 성능

- **Trie 검색**: O(m) - m은 prefix 길이
- **Redis 캐시**: O(1) 조회
- **동시 요청 처리**: Tokio 비동기 런타임

## 💡 사용 시나리오

### 시나리오 1: 사용자 검색
```bash
# 1. 사용자가 "han" 입력
curl "http://localhost:3000/suggest?q=han&limit=5"

# 2. 사용자가 "hankook" 선택
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{"term": "hankook"}'
```

### 시나리오 2: 인기 검색어 조회
```bash
# 실시간 트렌딩
curl "http://localhost:3000/trending?limit=10"

# 전체 통계
curl "http://localhost:3000/top?limit=10"
```

## 🔧 설정 옵션

### 환경 변수

| 변수 | 기본값 | 설명 |
|------|--------|------|
| `MONGO_URI` | `mongodb://localhost:27017` | MongoDB 연결 URI |
| `MONGO_DB` | `search_db` | 데이터베이스 이름 |
| `MONGO_COLLECTION` | `search_terms` | 컬렉션 이름 |
| `REDIS_URI` | `redis://localhost:6379` | Redis 연결 URI |
| `CACHE_TTL` | `300` | 캐시 TTL (초) |
| `PORT` | `3000` | 서버 포트 |

## 🔍 작동 원리

### 1. 검색어 추가 플로우
```
사용자 입력
    ↓
Trie 업데이트 (메모리)
    ↓
MongoDB 저장 (영구)
    ↓
Redis 트렌딩 업데이트
    ↓
관련 캐시 무효화
```

### 2. 검색어 추천 플로우
```
사용자 쿼리
    ↓
Redis 캐시 확인 ──→ HIT → 결과 반환
    ↓ MISS
Trie 검색 (인메모리)
    ↓
결과를 Redis 캐싱
    ↓
결과 반환
```

## 🛠️ 트러블슈팅

### MongoDB 연결 실패
```bash
# MongoDB 상태 확인
docker ps | grep mongo
docker logs tries-suggest-mongo
```

### Redis 연결 실패
```bash
# Redis 상태 확인
docker ps | grep redis
docker exec -it tries-suggest-redis redis-cli ping
```

### 로그 레벨 조정
```bash
RUST_LOG=debug cargo run
```

## 📈 향후 개선 사항

- [ ] 한글 초성 검색 지원
- [ ] 오타 교정 (Levenshtein Distance)
- [ ] 사용자별 개인화 추천
- [ ] gRPC API 추가
- [ ] Prometheus 메트릭
- [ ] Kubernetes 배포 설정

## 📝 라이선스

MIT License

## 👨‍💻 개발자

Made with ❤️ by wj