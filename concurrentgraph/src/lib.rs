mod marking;

use crate::marking::Marking;
use guarded::atomic::Atomic;
use guarded::seize::{Collector, Guard};
use guarded::shared::Shared;
use magnitude::Magnitude;
use std::ops::Deref;
use std::ptr::null;
use std::sync::atomic::Ordering;

pub struct VertexNode {
    key: Magnitude<usize>,
    first_edge: Atomic<Marking<EdgeNode>>,
    next_vertex: Atomic<Marking<VertexNode>>,
}

pub struct EdgeNode {
    key: Magnitude<usize>,
    next_edge: Atomic<Marking<EdgeNode>>,
    target: Atomic<Marking<VertexNode>>,
}

pub struct ConcurrentGraph {
    vertex_head: Atomic<Marking<VertexNode>>,
    collector: Collector,
}

impl ConcurrentGraph {
    pub fn pin(&self) -> GraphRef {
        GraphRef {
            graph: self,
            guard: self.collector.enter(),
        }
    }

    pub fn new() -> Self {
        let collector = Collector::new();
        Self {
            vertex_head: Atomic::from(Shared::boxed(
                Marking::Marked(VertexNode {
                    key: Magnitude::NegInfinite,
                    first_edge: Atomic::null(),
                    next_vertex: Atomic::null(),
                }),
                &collector,
            )),
            collector,
        }
    }

    pub unsafe fn add_node<'guard>(&self, key: usize, guard: &'guard Guard<'_>) {}

    pub unsafe fn locate_vertex<'guard>(
        &self,
        start_from: Shared<Marking<VertexNode>>,
        key: usize,
        guard: &'guard Guard<'_>,
    ) -> (Atomic<Marking<VertexNode>>, Atomic<Marking<VertexNode>>) {
        let mut parent = start_from;

        'outer: while !parent.is_null() {
            let parent_deref = parent.deref();

            let mut current = parent_deref.next_vertex.load(Ordering::SeqCst, guard);

            if current.is_null() {
                break;
            }

            let current_deref = current.deref();
            let mut next = current_deref.next_vertex.load(Ordering::SeqCst, guard);

            if next.is_null() {
                break;
            }

            // Help other threads by deleting marked nodes
            let next_deref = next.deref();
            while next_deref.is_marked() && current_deref.key.unwrap() < key {
                if parent_deref
                    .next_vertex
                    .compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst, guard)
                    .is_err()
                {
                    // the parent node's next pointer has changed so we start from the beginning
                    continue 'outer;
                }
                current = next;
                next = next_deref.next_vertex.load(Ordering::SeqCst, guard);
            }

            // Current might have changed so deref again
            let current_deref = current.deref();
            if current_deref.key.unwrap() >= key {
                return (Atomic::from(parent), Atomic::from(current));
            }

            parent = current;
            current = next;
        }

        (Atomic::from(parent), Atomic::null())
    }
}

pub struct GraphRef<'graph> {
    pub(crate) graph: &'graph ConcurrentGraph,
    guard: Guard<'graph>,
}

impl GraphRef<'_> {
    pub unsafe fn add_node(&self, key: usize) -> bool {
        self.graph.add_node(key, &self.guard)
    }

    pub unsafe fn add_edge(&self, source: usize, target: usize) {
        self.graph.add_edge(source, target, &self.guard)
    }
}
