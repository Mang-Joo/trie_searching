use std::collections::HashMap;

pub struct Node {
    pub children: HashMap<char, Box<Node>>,
    pub is_end: bool,
}

impl Node {
    fn new() -> Self {
        Node {
            children: HashMap::new(),
            is_end: false,
        }
    }
}

pub struct Trie {
    root: Node,
}

impl Trie {
    pub fn new() -> Self {
        Self { root: Node::new() }
    }

    pub fn insert(&mut self, word: &str) {
        if word.is_empty() {
            return;
        }

        let mut cur = &mut self.root;

        for ch in word.chars() {
            cur = cur
                .children
                .entry(ch)
                .or_insert_with(|| Box::new(Node::new()))
        }

        cur.is_end = true;
    }

    pub fn contains(&self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }
        self.find_node(word).map(|n| n.is_end).unwrap_or(false)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        if prefix.is_empty() {
            return true;
        }
        self.find_node(prefix).is_some()
    }

    fn find_node(&self, s: &str) -> Option<&Node> {
        let mut cur = &self.root;

        for ch in s.chars() {
            cur = cur.children.get(&ch)?.as_ref();
        }

        Some(cur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ops() {
        let mut t = Trie::new();
        for w in ["han", "hanpass", "hangul", "handy"] {
            t.insert(w);
        }

        assert!(t.contains("han"));
        assert!(t.contains("hanpass"));
        assert!(!t.contains("hap"));
        assert!(t.starts_with("han"));
        assert!(!t.contains("hap"));
        assert!(t.contains(""));
    }

    #[test]
    fn empty_is_not_inserted() {
        let mut t = Trie::new();
        t.insert("");
        assert!(!t.contains(""));
        assert!(t.starts_with(""));
    }
}
