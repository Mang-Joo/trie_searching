/// Trie 기본 사용 예제
///
/// 실행 방법:
/// cargo run --example basic_usage
use tries_suggest::domain::tries::Trie;

fn main() {
    println!("🔍 Trie 기본 사용 예제\n");

    let mut trie = Trie::new();

    // 검색어 추가
    let terms = vec![
        "apple",
        "apple",
        "apple", // 빈도: 3
        "application",
        "application", // 빈도: 2
        "apply",       // 빈도: 1
        "banana",
        "banana", // 빈도: 2
        "band",   // 빈도: 1
    ];

    println!("📝 검색어 추가 중...");
    for term in &terms {
        trie.insert(term);
        println!("  + {}", term);
    }

    println!("\n🔎 검색 테스트");

    // contains 테스트
    println!("\n1. contains() 테스트:");
    let test_terms = vec!["apple", "app", "banana", "cat"];
    for term in &test_terms {
        let exists = trie.contains(term);
        println!("  '{}' 존재? {}", term, if exists { "✅" } else { "❌" });
    }

    // starts_with 테스트
    println!("\n2. starts_with() 테스트:");
    let prefixes = vec!["app", "ban", "ca"];
    for prefix in &prefixes {
        let exists = trie.starts_with(prefix);
        println!(
            "  '{}' prefix 존재? {}",
            prefix,
            if exists { "✅" } else { "❌" }
        );
    }

    // suggest_top_k 테스트
    println!("\n3. suggest_top_k() 테스트:");

    let tests = vec![("ap", 3), ("app", 3), ("b", 3), ("ban", 2)];

    for (prefix, k) in tests {
        let suggestions = trie.suggest_top_k(prefix, k);
        println!("\n  prefix='{}', k={}:", prefix, k);
        for (i, (term, freq)) in suggestions.iter().enumerate() {
            println!("    {}. {} (빈도: {})", i + 1, term, freq);
        }
    }

    println!("\n✅ 예제 완료!");
}
