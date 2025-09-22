use ordered_float::OrderedFloat;
use std::collections::{BinaryHeap, HashMap, VecDeque};

// Optimized data structure for depth-ordered processing
// Uses a heap for depth ordering and queues for cells at the same depth
pub struct DepthOrderedQueue {
    // Maps depth to queue of cells at that depth
    depth_queues: HashMap<OrderedFloat<f64>, VecDeque<(usize, usize, usize)>>,
    // Min-heap of depths (using Reverse for min-heap behavior)
    depth_heap: BinaryHeap<std::cmp::Reverse<OrderedFloat<f64>>>,
}

impl DepthOrderedQueue {
    pub fn new() -> Self {
        DepthOrderedQueue {
            depth_queues: HashMap::new(),
            depth_heap: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, depth: f64, x: usize, y: usize, z: usize) {
        let depth_key = OrderedFloat(depth);

        // Add to depth queue
        self.depth_queues
            .entry(depth_key)
            .or_default()
            .push_back((x, y, z));

        // Add depth to heap if not already present
        let reverse_depth = std::cmp::Reverse(depth_key);
        if !self.depth_heap.iter().any(|&d| d == reverse_depth) {
            self.depth_heap.push(reverse_depth);
        }
    }

    pub fn pop(&mut self) -> Option<(usize, usize, usize)> {
        while let Some(&std::cmp::Reverse(depth_key)) = self.depth_heap.peek() {
            if let Some(queue) = self.depth_queues.get_mut(&depth_key) {
                if let Some(cell) = queue.pop_front() {
                    return Some(cell);
                } else {
                    // Queue is empty, remove this depth
                    self.depth_queues.remove(&depth_key);
                    self.depth_heap.pop();
                }
            } else {
                // Shouldn't happen, but handle gracefully
                self.depth_heap.pop();
            }
        }
        None
    }
}
