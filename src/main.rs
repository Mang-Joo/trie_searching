use crate::domain::tries;

pub mod domain;

fn main() {
    let mut trie = tries::Trie::new();
    for w in ["han", "hanpass", "hangul", "handy"] {
        trie.insert(w);
    }

    println!("contains('han')    => {}", trie.contains("han"));
    println!("contains('hap')    => {}", trie.contains("hap"));
    println!("starts_with('han') => {}", trie.starts_with("han"));
    println!("starts_with('hap') => {}", trie.starts_with("hap"));
}
