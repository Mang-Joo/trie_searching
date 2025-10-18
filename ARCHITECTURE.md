# 🏗️ 아키텍처 문서

## 전체 구조

```
┌─────────────────────────────────────────────────────────┐
│                        Client                            │
│                   (HTTP/JSON API)                        │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                    API Layer                             │
│              (Axum Web Framework)                        │
│                                                          │
│  Routes:                                                 │
│  • GET  /health              Health check               │
│  • GET  /suggest             검색어 추천                 │
│  • POST /search              검색어 기록                 │
│  • GET  /trending            실시간 인기 검색어          │
│  • GET  /top                 전체 인기 검색어            │
│  • GET  /contains            검색어 존재 확인            │
│  • POST /rebuild             Trie 재구성                │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                  Service Layer                           │
│             (Business Logic)                             │
│                                                          │
│  SuggestionService:                                      │
│  ┌────────────────────────────────────┐                │
│  │  In-Memory Trie (RwLock)           │                │
│  │  • O(m) prefix search              │                │
│  │  • Thread-safe with parking_lot    │                │
│  │  • Frequency-based ranking         │                │
│  └────────────────────────────────────┘                │
│                                                          │
│  기능:                                                   │
│  • 검색어 추가 (Trie + MongoDB + Redis)                │
│  • 추천 검색 (Cache -> Trie -> Cache)                  │
│  • 인기 검색어 조회                                     │
│  • Trie 재구성                                          │
└─────────────┬───────────────────┬───────────────────────┘
              │                   │
              ▼                   ▼
┌──────────────────────┐  ┌──────────────────────┐
│  Repository Layer    │  │  Repository Layer    │
│   (MongoDB)          │  │     (Redis)          │
│                      │  │                      │
│  • 검색어 영구 저장  │  │  • 빠른 캐싱         │
│  • 통계 데이터       │  │  • 실시간 트렌드     │
│  • 빈도수 관리       │  │  • TTL 관리          │
│                      │  │                      │
│  Document:           │  │  Keys:               │
│  {                   │  │  suggest:{prefix}:{k}│
│    term: String,     │  │  trending:searches   │
│    frequency: u64,   │  │  top:searches:{n}    │
│    last_searched,    │  │                      │
│    created_at        │  │                      │
│  }                   │  │                      │
└──────────────────────┘  └──────────────────────┘
```

## 계층별 책임

### 1. API Layer (`src/api/`)
- **책임**: HTTP 요청/응답 처리
- **기술**: Axum, Tower
- **파일**:
  - `routes.rs`: 엔드포인트 핸들러 구현

### 2. Service Layer (`src/service/`)
- **책임**: 비즈니스 로직, Trie/MongoDB/Redis 통합
- **기술**: parking_lot (RwLock)
- **파일**:
  - `suggestion.rs`: 추천 서비스 핵심 로직

### 3. Repository Layer (`src/repository/`)
- **책임**: 데이터 저장소 추상화
- **파일**:
  - `mongodb.rs`: MongoDB CRUD 연산
  - `redis.rs`: Redis 캐싱 연산

### 4. Domain Layer (`src/domain/`)
- **책임**: 핵심 도메인 모델 및 알고리즘
- **파일**:
  - `tries.rs`: Trie 자료구조 구현

## 데이터 플로우

### 검색어 추가 플로우

```
User Input
    │
    ├─► [1] Trie.insert()          (메모리, 즉시)
    │
    ├─► [2] MongoDB.upsert_term()  (영구 저장)
    │       • frequency += 1
    │       • last_searched 업데이트
    │
    ├─► [3] Redis.zincrby()        (실시간 트렌딩)
    │       sorted set 업데이트
    │
    └─► [4] Redis.invalidate()     (캐시 무효화)
            관련 prefix 캐시 삭제
```

### 검색어 추천 플로우

```
User Query (prefix)
    │
    ├─► [1] Redis 캐시 확인
    │       Key: suggest:{prefix}:{limit}
    │       └─► HIT: 결과 반환 (end)
    │
    ├─► [2] Trie 검색 (MISS 시)
    │       suggest_top_k(prefix, k)
    │       • BFS 탐색
    │       • Min-Heap (k개 유지)
    │       • 빈도수 + 사전순 정렬
    │
    ├─► [3] Redis 캐싱
    │       결과를 TTL과 함께 저장
    │
    └─► [4] 결과 반환
```

### 서버 시작 플로우

```
Server Start
    │
    ├─► [1] MongoDB 연결
    │       • 인덱스 생성 (term: unique)
    │
    ├─► [2] Redis 연결
    │       • ConnectionManager
    │
    ├─► [3] Trie 초기화
    │       • MongoDB에서 모든 검색어 로드
    │       • frequency 반영하여 insert
    │
    └─► [4] Axum 서버 시작
            0.0.0.0:3000 바인딩
```

## 성능 특성

### Trie 연산 복잡도
- **Insert**: O(m) - m은 단어 길이
- **Search**: O(m)
- **Prefix Match**: O(m + n) - n은 결과 개수
- **Space**: O(ALPHABET_SIZE * N * M)

### 캐싱 전략
1. **L1 Cache (Trie)**: 메모리, 마이크로초
2. **L2 Cache (Redis)**: 네트워크, 밀리초
3. **Persistent (MongoDB)**: 디스크, 10-100ms

### 동시성
- **Trie**: RwLock (읽기 병렬화)
- **Redis**: 비동기 ConnectionManager
- **MongoDB**: Connection Pool
- **런타임**: Tokio (멀티스레드)

## 확장성 고려사항

### 수평 확장
```
Load Balancer
    │
    ├─► App Server 1 ─┐
    ├─► App Server 2 ─┤
    └─► App Server 3 ─┤
                      │
    ┌─────────────────┴─────────────────┐
    │                                   │
    ▼                                   ▼
MongoDB Cluster                    Redis Cluster
(Replica Set)                      (Sentinel/Cluster)
```

### 최적화 방안
1. **Trie 공유 메모리**: 읽기 전용 mmap
2. **샤딩**: prefix 기반 분산
3. **캐시 워밍**: 인기 prefix 미리 캐싱
4. **비동기 쓰기**: 배치 MongoDB insert

## 모니터링 포인트

### 메트릭
- Trie 메모리 사용량
- Redis 캐시 히트율
- MongoDB 쿼리 레이턴시
- API 응답 시간 (p50, p95, p99)
- 동시 요청 수

### 알람
- Redis 연결 실패
- MongoDB 레플리카 지연
- Trie 메모리 임계치 초과
- 5xx 에러율 증가

## 보안 고려사항

### 현재
- 입력 검증 (빈 문자열 체크)
- MongoDB injection 방지 (regex escape)

### 추가 필요
- Rate limiting (per IP)
- API key 인증
- HTTPS/TLS
- 악성 입력 필터링
- CORS 설정 세분화