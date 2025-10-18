#!/bin/bash

# API 테스트 스크립트

BASE_URL="http://localhost:3000"

echo "🧪 Tries-Suggest API 테스트"
echo "================================"
echo ""

# Health Check
echo "1️⃣ Health Check"
curl -s "${BASE_URL}/health" | jq '.'
echo -e "\n"

# 검색어 등록
echo "2️⃣ 검색어 등록"
for term in "hankook" "hankook" "hangul" "hangul" "hangul" "handy" "hancom" "hanyoung"; do
    echo "  - Registering: $term"
    curl -s -X POST "${BASE_URL}/search" \
        -H "Content-Type: application/json" \
        -d "{\"term\": \"$term\"}" | jq -r '.data'
done
echo ""

# 대기
echo "⏳ 잠시 대기 중..."
sleep 2
echo ""

# 검색어 추천
echo "3️⃣ 검색어 추천 (prefix: 'han')"
curl -s "${BASE_URL}/suggest?q=han&limit=5" | jq '.'
echo -e "\n"

# 존재 여부 확인
echo "4️⃣ 검색어 존재 확인"
echo "  - 'hangul' 존재?"
curl -s "${BASE_URL}/contains?q=hangul" | jq '.data'
echo "  - 'hanbok' 존재?"
curl -s "${BASE_URL}/contains?q=hanbok" | jq '.data'
echo ""

# 실시간 인기 검색어
echo "5️⃣ 실시간 인기 검색어 (Redis)"
curl -s "${BASE_URL}/trending?limit=5" | jq '.'
echo -e "\n"

# 전체 인기 검색어
echo "6️⃣ 전체 인기 검색어 (MongoDB)"
curl -s "${BASE_URL}/top?limit=5" | jq '.'
echo -e "\n"

echo "✅ 테스트 완료!"