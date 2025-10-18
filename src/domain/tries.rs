use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

pub struct Node {
    pub children: HashMap<char, Box<Node>>,
    pub is_end: bool,
    pub frequency: u64,
}

impl Node {
    fn new() -> Self {
        Node {
            children: HashMap::new(),
            is_end: false,
            frequency: 0,
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
        cur.frequency = cur.frequency.saturating_add(1);
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

    pub fn suggest_top_k(&self, prefix: &str, k: usize) -> Vec<(String, u64)> {
        if k == 0 {
            return vec![];
        }

        let start = match self.find_node(prefix) {
            Some(n) => n,
            None => return vec![],
        };

        let mut heap: BinaryHeap<(Reverse<u64>, String)> = BinaryHeap::new();

        let mut stack: Vec<(String, &Node)> = vec![(prefix.to_string(), start)];

        while let Some((acc, node)) = stack.pop() {
            if node.is_end {
                let key = (Reverse(node.frequency), acc.clone());
                heap.push(key);
                if heap.len() > k {
                    heap.pop();
                }
            }
            for (ch, child) in node.children.iter() {
                let mut next = acc.clone();
                next.push(*ch);
                stack.push((next, child));
            }
        }

        let mut items: Vec<(u64, String)> = heap
            .into_vec()
            .into_iter()
            .map(|(Reverse(freq), word)| (freq, word))
            .collect();

        items.sort_by(|(f1, w1), (f2, w2)| f2.cmp(f1).then_with(|| w1.cmp(w2)));

        items.into_iter().map(|(f, w)| (w, f)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ops() {
        let mut t = Trie::new();
        for w in ["han", "hankook", "hangul", "handy"] {
            t.insert(w);
        }
        assert!(t.contains("han"));
        assert!(t.contains("hankook"));
        assert!(!t.contains("hap"));
        assert!(t.starts_with("han"));
        assert!(!t.starts_with("hap"));
        assert!(t.starts_with(""));
    }

    #[test]
    fn frequency_and_topk() {
        let mut t = Trie::new();
        t.insert("hankook");
        t.insert("hankook");
        t.insert("handy");
        t.insert("hangul");
        t.insert("hangul");
        t.insert("hangul");

        // 현재 빈도: hangul=3, hankook=2, handy=1
        let got = t.suggest_top_k("han", 2);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].0, "hangul");
        assert_eq!(got[0].1, 3);
        assert_eq!(got[1].0, "hankook");
        assert_eq!(got[1].1, 2);
    }
}
