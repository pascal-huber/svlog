use std::collections::VecDeque;

pub struct Cache<T> {
    list: VecDeque<T>,
    size: usize,
}

impl<T: PartialEq> Cache<T> {
    pub fn new(size: u8) -> Self {
        Cache {
            list: VecDeque::new(),
            size: usize::from(size),
        }
    }

    // Adds the item to the cache. If the item was not in cache already, the
    // return value is true. If the item was already present, it is moved to the
    // front (LRU) and the return value is false.
    pub fn push(&mut self, item: T) -> bool {
        let mut result: bool = true;
        if self.list.contains(&item) {
            let index = self.list.iter().position(|r| r == &item).unwrap();
            self.list.remove(index);
            result = false;
        }
        self.list.push_front(item);
        while self.list.len() > self.size {
            self.list.pop_back();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_item() {
        let mut stk: Cache<u32> = Cache::new(2);
        assert!(stk.push(1));
        assert_eq!(stk.list.len(), 1);
    }

    #[test]
    fn add_same_item() {
        let mut stk: Cache<u32> = Cache::new(2);
        assert!(stk.push(1));
        assert!(!stk.push(1));
        assert_eq!(stk.list.len(), 1);
    }

    #[test]
    fn size_limit() {
        let mut stk: Cache<u32> = Cache::new(2);
        stk.push(1);
        stk.push(2);
        stk.push(3);
        assert_eq!(stk.list.len(), 2);
        assert!(stk.push(1)) // i.e. 1 was not in the list
    }

    #[test]
    fn test_string() {
        let mut stk: Cache<String> = Cache::new(2);
        let item1 = "asdf".to_string();
        stk.push(item1);
        let item2 = "asdf".to_string();
        assert!(!stk.push(item2)); // item wasl already in list
    }
}
