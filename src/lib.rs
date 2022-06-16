#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

use guarded::atomic::Atomic;
use guarded::shared::Shared;
use seize::{Collector, Guard};
use std::cmp;
use std::ops::Deref;
use std::sync::atomic::Ordering;



pub struct ConcurrentGraph {

}

impl ConcurrentGraph {

}

impl ConcurrentGraph {



    pub unsafe fn remove_vertex<'guard>(&self, key: usize, guard: &'guard Guard<'_>) -> bool {
        loop {
            let (parent, current) = self.locate_insert_position(self.vertex_head.load(Ordering::SeqCst, guard), key, guard);

            let parent = parent.load(Ordering::SeqCst, guard);
            let parent_deref = parent.deref();

            let current = current.load(Ordering::SeqCst, guard);
            let current_deref = current.deref();

            if current_deref.key != key {
                panic!()
            }

            let next = current_deref.next_vertex.load(Ordering::SeqCst, guard);
            let next_deref = next.deref();

            if !next_deref.is_marked() {
                if current_deref.next_vertex.compare_exchange(
                    next.clone(),
                    Shared::boxed(Marking::Marked(*next), &self.collector),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                    guard
                ).is_ok() {
                    if parent_deref.next_vertex.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst, guard).is_ok() {
                        break;
                    }
                };
            }
        }
        true
    }

    pub unsafe fn add_node<'guard>(&self, key: usize, guard: &'guard Guard<'_>) -> bool {
        loop {
            let head = self.vertex_head.load(Ordering::SeqCst, guard);

            let (less, greater) = self.locate_insert_position(head, key, guard);

            let less = less.load(Ordering::Relaxed, guard);
            let less_deref = less.deref();

            let greater = greater.load(Ordering::Relaxed, guard);

            let new_node = VertexNode {
                key,
                next_vertex: Atomic::from(greater),
                first_edge: Atomic::null(),
            };

            if less_deref
                .next_vertex
                .compare_exchange(
                    greater,
                    Shared::boxed(Marking::Unmarked(new_node), &self.collector),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                    guard,
                )
                .is_ok()
            {
                return true;
            }
        }
    }

    pub unsafe fn add_edge<'guard>(&self, source: usize, target: usize, guard: &'guard Guard<'_>) {
        let (source, target) = self.locate_both(source, target, guard);
    }

    unsafe fn locate_edge_position<'guard>(
        &self,
        vertex: Atomic<VertexNode>,
        key: usize,
        guard: &'guard Guard<'_>,
    ) {
        let vertex = vertex.load(Ordering::SeqCst, guard);
        let vertex_deref = vertex.deref();

        let mut edge = vertex_deref.first_edge.load(Ordering::SeqCst, guard);
        while !edge.is_null() {
            let edge_deref = edge.deref();
            let next = edge_deref.next_edge.load(Ordering::SeqCst, guard);

            if next.is_null() {
                break;
            }

            let next_deref = next.deref();
        }
    }

    unsafe fn locate_insert_position<'guard>(
        &self,
        start_from: Shared<Marking<VertexNode>>,
        key: usize,
        guard: &'guard Guard<'_>,
    ) -> (Atomic<Marking<VertexNode>>, Atomic<Marking<VertexNode>>) {
        let mut current = start_from;

        'outer: while !current.is_null() {
            let current_deref = current.deref();
            let mut next = current_deref.next_vertex.load(Ordering::SeqCst, guard);

            if next.is_null() {
                break;
            }

            let next_deref = next.deref();

            while next_deref.is_marked() && next_deref.key < key {
                let next_next = next_deref.next_vertex.load(Ordering::SeqCst, guard);
                if current_deref.next_vertex.compare_exchange(next, next_next, Ordering::Relaxed, Ordering::Relaxed, guard).is_ok() {
                    current = next_next;
                    continue 'outer;
                }
                next = next_next;
            }

        }

        let mut current = start_from;
        while !current.is_null() {
            let current_deref = current.deref();
            let next = current_deref.next_vertex.load(Ordering::SeqCst, guard);

            if next.is_null() {
                break;
            }

            let next_deref = next.deref();

            while next_deref.is_marked() &&

            match &**next_deref {
                Marking::Marked(next_deref) if next_deref.key < key => {
                    current = next;
                    continue;
                }
                _ => {}
            }

            if matches!(current, Marking::Marked(_)) && next_deref.key < key {

            }
            return (Atomic::from(current), Atomic::from(next));
        }

        return (Atomic::from(current), Atomic::null());
    }

    unsafe fn locate_both<'guard>(
        &self,
        source: usize,
        target: usize,
        guard: &'guard Guard<'_>,
    ) -> (Atomic<Marking<VertexNode>>, Atomic<Marking<VertexNode>>) {
        let (less, greater) = if source < target {
            (source, target)
        } else {
            (target, source)
        };

        let head = self.vertex_head.load(Ordering::SeqCst, guard);

        let (_, lower) = self.locate_insert_position(head, less, guard);
        let lower = lower.load(Ordering::SeqCst, guard);
        let lower_deref = lower.deref();
        if lower_deref.key != less {
            return (Atomic::null(), Atomic::null());
        }

        let (_, upper) = self.locate_insert_position(lower, greater, guard);
        let upper = upper.load(Ordering::SeqCst, guard);
        let upper_deref = upper.deref();
        if upper_deref.key != greater {
            return (Atomic::null(), Atomic::null());
        }
        return if source < target {
            (Atomic::from(lower), Atomic::from(upper))
        } else {
            (Atomic::from(upper), Atomic::from(lower))
        };
    }
}



pub struct VertexNode {
    key: usize,
    next_vertex: Atomic<Marking<VertexNode>>,
    first_edge: Atomic<Marking<EdgeNode>>,
}

pub struct EdgeNode {
    key: usize,
    next_edge: Atomic<Marking<EdgeNode>>,
    target: Atomic<Marking<VertexNode>>,
}

#[test]
fn test() {
    let graph = ConcurrentGraph::new();
    unsafe {
        graph.pin().add_node(1);
        graph.pin().add_node(2);
        graph.pin().locate_both(1, 2);
    }
    dbg!("");
    return;
}
