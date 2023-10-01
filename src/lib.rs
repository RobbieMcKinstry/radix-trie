use std::mem::MaybeUninit;

const BRANCH_FACTOR: usize = 256;

/// Each array contains a list of items.
/// In our case, the items are nodes which point
/// to the next level.
type RadixArray<T> = [T; BRANCH_FACTOR];
/// Each level is the child of another level, expect
/// for the root.
type Level<T> = RadixArray<RadixNode<T>>;

#[allow(dead_code)]
pub struct RadixTrie<T> {
    root: RadixNode<T>,
    node_count: usize,
}

impl<T> RadixTrie<T> {
    pub fn new() -> Self {
        Self {
            root: RadixNode::default(),
            node_count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.node_count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, key: impl Into<Vec<u8>>, value: T) {
        let buffer: Vec<u8> = key.into();
        let mut iterator = buffer.into_iter();
        self.root.insert(&mut iterator, value);
        self.increment();
    }

    fn increment(&mut self) {
        self.node_count += 1;
    }
}

struct RadixNode<T> {
    /// If Some, a match occurs if there are no characters
    /// remaining in the buffer. T is the value provided
    /// during insertion.
    accept_state: Option<T>,

    /// children contains the collection of radix
    /// nodes for which the bytes read thus far are a prefix.
    /// This field is initialized lazily to conserve memory.
    children: Option<Box<Level<T>>>,
}

impl<T> RadixNode<T> {
    pub fn new() -> Self {
        Self {
            accept_state: None,
            children: None,
        }
    }

    /// returns the item already in this position if the key matches
    /// an existing key.
    pub fn insert(&mut self, key: &mut dyn Iterator<Item = u8>, value: T) -> Option<T> {
        match key.next() {
            // Degenerate Case: We've reached the end of the string
            // and can store the value in the accept state.
            None => self.set_value(value),
            // Recursive Case: We have at least one more byte to process.
            Some(byte) => self.handle_next_byte(byte, key, value),
        }
    }

    fn set_value(&mut self, value: T) -> Option<T> {
        let prev = self.accept_state.take();
        self.accept_state = Some(value);
        prev
    }

    fn handle_next_byte(
        &mut self,
        byte: u8,
        key: &mut dyn Iterator<Item = u8>,
        value: T,
    ) -> Option<T> {
        // • Check if the array has been initialized.
        if self.children.is_none() {
            // • If not, initialize it with a collection of empty cells.
            self.children = Some(Self::new_children());
        }

        // • Insert this item at the given position.
        match self.children.as_mut() {
            Some(children) => children[byte as usize].insert(key, value),
            None => {
                let mut children = Self::new_children();
                let found = children[byte as usize].insert(key, value);
                self.children = Some(children);
                found
            }
        }
    }

    /// allocates a new array of radix nodes.
    fn new_children() -> Box<Level<T>> {
        let mut children_vec = Vec::with_capacity(BRANCH_FACTOR);

        for _ in 0..BRANCH_FACTOR {
            children_vec.push(RadixNode::default());
        }
        let children: [RadixNode<T>; BRANCH_FACTOR] =
            children_vec.try_into().unwrap_or_else(|_| unreachable!());
        Box::new(children)
    }
}

impl<T> Default for RadixNode<T> {
    fn default() -> Self {
        Self::new()
    }
}
