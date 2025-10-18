use crate::domain::tries;

pub mod domain;

fn main() {
    let mut t = tries::Trie::new();
    for w in [
        "hankook", "hankook", "handy", "hangul", "hangul", "hangul", "hancom", "hanyoung",
    ] {
        t.insert(w);
    }
    let suggestions = t.suggest_top_k("han", 4);
    println!("Top-4 for 'han' => {:?}", suggestions);
}
