/// From https://github.com/opacous/vp-avl
/// but because the repo uses unstable features, this
/// file is a copy of the original file with the unstable
/// feature removed.
use std::{collections::BinaryHeap, marker::PhantomData, rc::Rc, vec};

use crate::base::{state_allocator::StateId, statespace::StateSpace};

pub trait Metric {
    type PointType;
    fn distance(&self, p1: &Self::PointType, p2: &Self::PointType) -> f64;
}

pub trait VpTreeObject: Sized {
    type PointType;
    fn location(&self) -> &Self::PointType;
}

impl VpTreeObject for Vec<f64> {
    type PointType = Self;
    fn location(&self) -> &Self {
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct VpAvl<Point, PointMetric> {
    root: usize,
    nodes: Vec<Node>,
    data: Vec<Point>,
    metric: PointMetric,
}

#[derive(Clone, Debug)]
struct Node {
    height: usize,
    center: usize,
    radius: f64,
    parent: Option<usize>,
    interior: Option<usize>,
    exterior: Option<usize>,
}

impl<Point, PointMetric> VpAvl<Point, PointMetric>
where
    PointMetric: Metric<PointType = Point::PointType>,
    Point: VpTreeObject,
{
    fn node_index_data(&self, node_index: usize) -> &Point {
        &self.data[self.nodes[node_index].center]
    }

    pub fn new(metric: PointMetric) -> Self {
        VpAvl {
            root: 0,
            nodes: vec![],
            data: vec![],
            metric,
        }
    }

    pub fn update_metric<NewMetric: Metric<PointType = Point::PointType>>(
        self,
        metric: NewMetric,
    ) -> VpAvl<Point, NewMetric> {
        VpAvl::bulk_insert(metric, self.data)
    }

    pub fn bulk_insert(metric: PointMetric, data: Vec<Point>) -> Self {
        let indices: Vec<usize> = (1..data.len()).collect();
        let nodes = (0..data.len())
            .map(|ind| Node::new_leaf(ind, None))
            .collect();
        let mut rv = VpAvl {
            root: 0,
            nodes,
            data,
            metric,
        };

        rv.bulk_build_indices(0, indices);

        rv
    }

    fn bulk_build_indices(&mut self, root: usize, mut indices: Vec<usize>) {
        if indices.len() < 2 {
            // simpler case
            match indices.len() {
                0 => {
                    // leaf node
                    self.nodes[root].height = 0;
                    self.nodes[root].interior = None;
                    self.nodes[root].exterior = None;
                }
                1 => {
                    // still has one child

                    let exterior = indices.pop().unwrap();

                    self.nodes[root].exterior = Some(exterior);
                    self.nodes[root].radius = self.metric.distance(
                        self.node_index_data(root).location(),
                        self.node_index_data(exterior).location(),
                    );
                    self.nodes[root].height = 1;
                    self.nodes[root].interior = None;
                    self.nodes[exterior].parent = Some(root);

                    self.bulk_build_indices(self.nodes[root].exterior.unwrap(), indices)
                }
                _ => unreachable!(),
            }
            return;
        }

        let mut distances = Vec::with_capacity(indices.len());
        for index in indices.iter() {
            distances.push((
                *index,
                self.metric.distance(
                    self.node_index_data(root).location(),
                    self.node_index_data(*index).location(),
                ),
            ));
        }
        // sort indices by distance from root
        distances.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let partitions: Vec<&[(usize, f64)]> = distances
            .chunks(distances.len() / 2 + distances.len() % 2)
            .collect();
        let mut interior_indices: Vec<usize> = partitions[0].iter().map(|x| x.0).collect();
        let mut exterior_indices: Vec<usize> = partitions[1].iter().map(|x| x.0).collect();

        let min_exterior_distance = partitions[1].first().unwrap().1;

        self.nodes[root].radius = min_exterior_distance;

        let interior_center = interior_indices.pop();
        let exterior_center = exterior_indices.pop();

        self.nodes[root].interior = interior_center;
        self.nodes[root].exterior = exterior_center;

        if let Some(interior) = interior_center {
            self.nodes[interior].parent = Some(root);
        }

        if let Some(exterior) = exterior_center {
            self.nodes[exterior].parent = Some(root);
        }

        let mut height = 0;
        // recurse
        if let Some(x) = interior_center {
            self.bulk_build_indices(x, interior_indices);
            height = height.max(self.nodes[x].height);
        }

        if let Some(x) = exterior_center {
            self.bulk_build_indices(x, exterior_indices);
            height = height.max(self.nodes[x].height);
        }

        self.nodes[root].height = height + 1;
    }

    fn set_height(&mut self, root: usize) {
        let interior_height = self.nodes[root]
            .interior
            .map(|i| self.nodes[i].height + 1)
            .unwrap_or(0);
        let exterior_height = self.nodes[root]
            .exterior
            .map(|i| self.nodes[i].height + 1)
            .unwrap_or(0);

        self.nodes[root].height = interior_height.max(exterior_height);
    }

    fn insert_root(&mut self, root: usize, value: Point) {
        let distance = self.metric.distance(
            self.node_index_data(self.nodes[root].center).location(),
            value.location(),
        );
        let root_radius = self.nodes[root].radius;

        if distance < root_radius {
            // in the interior
            if let Some(ind) = self.nodes[root].interior {
                // recurse
                self.insert_root(ind, value);
            } else {
                // new leaf node
                self.data.push(value);
                let new_index = self.data.len() - 1;

                self.nodes.push(Node::new_leaf(new_index, Some(root)));

                self.nodes[new_index].radius = distance.clamp(root_radius / 2.0, root_radius);

                self.nodes[root].interior = Some(new_index);
            }
        } else if let Some(ind) = self.nodes[root].exterior {
            // recurse
            self.insert_root(ind, value);
        } else {
            // new leaf node
            self.data.push(value);
            let new_index = self.data.len() - 1;

            self.nodes.push(Node::new_leaf(new_index, Some(root)));

            self.nodes[new_index].radius = distance.clamp(root_radius / 2.0, root_radius);

            self.nodes[root].exterior = Some(new_index);
        }
        // update the height
        self.set_height(root);
        // inserted!
        // rebalance?
        // will be called again at each successively higher level
        self.rebalance(root);
    }

    pub fn insert(&mut self, value: Point) {
        if self.data.len() > 1 {
            self.insert_root(self.root, value)
        } else if self.data.is_empty() {
            self.data.push(value);
            self.nodes.push(Node::new_leaf(0, None))
        } else {
            let root_dist = self.metric.distance(
                self.node_index_data(self.nodes[self.root].center)
                    .location(),
                value.location(),
            );
            self.nodes[self.root].radius = root_dist / 2.0;
            self.insert_root(self.root, value)
        }
    }

    // insert an orphaned node
    fn insert_existing(&mut self, root: usize, graft: usize) {
        let distance = self.metric.distance(
            self.node_index_data(self.nodes[root].center).location(),
            self.node_index_data(graft).location(),
        );
        let root_radius = self.nodes[root].radius;

        if distance < root_radius {
            // in the interior
            if let Some(ind) = self.nodes[root].interior {
                // recurse
                self.insert_existing(ind, graft)
            } else {
                // leaf node
                self.nodes[root].interior = Some(graft);
                self.nodes[graft].radius = distance.clamp(root_radius / 2.0, root_radius);
                self.nodes[graft].parent = Some(root);
            }
        } else if let Some(ind) = self.nodes[root].exterior {
            // recurse
            self.insert_existing(ind, graft)
        } else {
            // leaf node
            self.nodes[root].exterior = Some(graft);
            self.nodes[graft].radius = distance.clamp(root_radius / 2.0, root_radius);
            self.nodes[graft].parent = Some(root);
        }
        // update the height
        self.set_height(root);

        // inserted!
        // rebalance?
        // will be called again at each successively higher level
        self.rebalance(root)
    }

    fn rebalance(&mut self, root: usize) {
        let interior_height = self.nodes[root]
            .interior
            .map(|ind| self.nodes[ind].height)
            .unwrap_or(0);
        let exterior_height = self.nodes[root]
            .exterior
            .map(|ind| self.nodes[ind].height)
            .unwrap_or(0);

        if interior_height > (exterior_height + 1) {
            // interior is too big, it must be rebalanced
            self.rebalance_interior(root)
        } else if exterior_height > (interior_height + 1) {
            // exterior is too big, must be rebalanced
            self.rebalance_exterior(root)
        }
    }

    fn child_indices_impl(&self, root: usize, progress: &mut Vec<usize>) {
        if let Some(int) = self.nodes[root].interior {
            self.child_indices_impl(int, progress)
        }

        if let Some(ext) = self.nodes[root].exterior {
            self.child_indices_impl(ext, progress)
        }

        progress.push(root);
    }

    fn child_indices(&self, root: usize) -> Vec<usize> {
        let mut chillum = vec![];
        self.child_indices_impl(root, &mut chillum);

        chillum
    }

    // make the interior shorter
    fn rebalance_interior(&mut self, root: usize) {
        let mut children = self.child_indices(root);

        let root = children.pop().unwrap();
        self.bulk_build_indices(root, children);

        //
        // The following doesn't work, because it's possible, following a bulk reindex, for the radius of a child to be larger than that of its parent.
        // Consequently I haven't figured out materially more efficient way of grafting the subtrees here.
        // I keep this in place as an inspiration to figure out how to do this properly in the future
        //

        // // moves nodes as:
        // // interior -> root
        // // exterior -> new root exterior
        // // old root -> reinsert

        // // there must be an interior in this case, but maybe no exterior
        // let old_interior = self.nodes[root].interior.unwrap();
        // let old_exterior = self.nodes[root].exterior;

        // let old_root_data = self.nodes[root].center;

        // println!(
        //     "swapping {}: {:?} <> {}: {:?}",
        //     root, self.nodes[root], old_interior, self.nodes[old_interior]
        // );

        // // if there is no graft node, no children....
        // let mut old_exterior_children = old_exterior
        //     .map(|ind| self.child_indices(ind))
        //     .unwrap_or(vec![]);

        // // transplant the old interior to the root
        // self.nodes[root] = self.nodes[old_interior].clone();
        // self.nodes[old_interior].center = old_root_data;

        // let old_root_distance = self.metric.distance(
        //     self.node_index_data(root),
        //     self.node_index_data(old_interior),
        // );

        // let root_radius = self.nodes[root].radius;

        // // make the old root data located in the old interior node
        // self.nodes[old_interior].center = old_root_data;
        // self.nodes[old_interior].interior = None;
        // self.nodes[old_interior].exterior = None;
        // self.nodes[old_interior].height = 0;
        // self.nodes[old_interior].radius = old_root_distance.clamp(root_radius / 2.0, root_radius);

        // // collect the new exterior nodes
        // let new_exterior_node = self.nodes[root].exterior;
        // // this could be empty
        // let mut new_exterior_children = new_exterior_node
        //     .map(|ind| self.child_indices(ind))
        //     .unwrap_or(vec![]);

        // println!(
        //     "new exterior children {:?}: {:?}",
        //     new_exterior_node, new_exterior_children
        // );

        // println!(
        //     "old exterior children {:?}: {:?}",
        //     old_exterior, old_exterior_children
        // );

        // // aggregate all children...
        // // either or both could be empty
        // new_exterior_children.append(&mut old_exterior_children);

        // // check where the old root should go...
        // println!(
        //     "swapped {}: {:?} <> {}: {:?} distance: {}/{}",
        //     root,
        //     self.nodes[root],
        //     old_interior,
        //     self.nodes[old_interior],
        //     old_root_distance,
        //     self.nodes[root].radius
        // );

        // if old_root_distance < root_radius {
        //     println!(
        //         "old root in interior {} < {}",
        //         old_root_distance, root_radius
        //     );
        //     // old root is within the new root interior
        //     match self.nodes[root].interior {
        //         Some(interior) => self.insert_existing(interior, old_interior),
        //         None => {
        //             self.nodes[root].interior = Some(old_interior);
        //             self.nodes[root].radius =
        //                 old_root_distance.clamp(root_radius / 2.0, root_radius)
        //         }
        //     }
        // } else {
        //     println!("old root in exterior");
        //     // old root can be handled along with all of the other new exterior points
        //     new_exterior_children.push(old_interior)
        // }

        // let new_exterior_root = new_exterior_children.pop();
        // self.nodes[root].exterior = new_exterior_root;

        // // now reindex the new exterior nodes
        // if let Some(exterior) = new_exterior_root {
        //     println!(
        //         "new exterior nodes {}: {:?}",
        //         exterior, new_exterior_children
        //     );
        //     self.bulk_build_indices(exterior, new_exterior_children);
        // }

        // self.set_height(root);

        // println!(
        //     "finally {}: {:?} int: {:?} ext {:?}",
        //     root,
        //     self.nodes[root],
        //     self.nodes[root].interior.map(|i| &self.nodes[i]),
        //     self.nodes[root].exterior.map(|i| &self.nodes[i]),
        // );

        // self.check_validity_root(root);
    }

    // make the exterior shorter
    fn rebalance_exterior(&mut self, root: usize) {
        // honestly I don't see a way to be clever about this case yet.
        // rebuilding the whole dang thing
        // TODO: be good
        let mut children = self.child_indices(root);

        let root = children.pop().unwrap();
        self.bulk_build_indices(root, children)
    }

    pub fn nn_iter<'a>(
        &'a self,
        query_point: &'a Point::PointType,
    ) -> impl Iterator<Item = &'a Point> {
        KnnIterator::new(query_point, self).map(|(p, _d)| p)
    }

    pub fn nn_dist_iter<'a>(
        &'a self,
        query_point: &'a Point::PointType,
    ) -> KnnIterator<'a, Point, PointMetric> {
        KnnIterator::new(query_point, self)
    }

    pub fn nn_iter_mut<'a>(
        &'a mut self,
        query_point: &'a Point::PointType,
    ) -> impl Iterator<Item = &'a mut Point> {
        self.nn_dist_iter_mut(query_point).map(|(p, _d)| p)
    }

    pub fn nn_dist_iter_mut<'a>(
        &'a mut self,
        query_point: &'a Point::PointType,
    ) -> KnnMutIterator<'a, Point, PointMetric> {
        KnnMutIterator::new(query_point, self)
    }

    fn nn_index_iter<'a>(
        &'a self,
        query_point: &'a Point::PointType,
    ) -> KnnIndexIterator<'a, Point, PointMetric> {
        KnnIndexIterator::new(query_point, self)
    }

    fn check_validity_node(&self, root: usize) {
        if let Some(interior) = self.nodes[root].interior {
            let distance = self.metric.distance(
                self.node_index_data(root).location(),
                self.node_index_data(interior).location(),
            );

            assert!(
                distance < self.nodes[root].radius,
                "interior {} of {} not within radius: {} >= {}",
                interior,
                root,
                distance,
                self.nodes[root].radius
            );
        }

        if let Some(exterior) = self.nodes[root].exterior {
            let distance = self.metric.distance(
                self.node_index_data(root).location(),
                self.node_index_data(exterior).location(),
            );

            assert!(
                distance >= self.nodes[root].radius,
                "exterior {} of {} not outside radius: {} < {}",
                exterior,
                root,
                distance,
                self.nodes[root].radius
            );
        }
    }

    fn check_validity_root(&self, root: usize) {
        self.check_validity_node(root);

        if let Some(interior) = self.nodes[root].interior {
            self.check_validity_root(interior)
        }

        if let Some(exterior) = self.nodes[root].exterior {
            self.check_validity_root(exterior)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ Point> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut Point> {
        self.data.iter_mut()
    }

    fn check_validity(&self) {
        if self.size() > 0 {
            self.check_validity_root(self.root)
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl<Point, PointMetric> VpAvl<Point, PointMetric>
where
    PointMetric: Metric<PointType = Point::PointType>,
    Point: VpTreeObject<PointType: PartialEq>,
{
    pub fn remove(&mut self, value: &Point::PointType) -> Option<Point> {
        let to_remove = self
            .nn_index_iter(value)
            .take_while(|nn| nn.1 <= 0.0)
            .find(|nn| self.data[nn.0].location() == value);

        Some(self.remove_index(to_remove.unwrap().0))
    }

    // TODO: DONT BE DUM
    fn remove_index(&mut self, index: usize) -> Point {
        let end = self.data.pop().unwrap();
        self.nodes.pop();
        let old = if index == self.data.len() {
            end
        } else {
            std::mem::replace(&mut self.data[index], end)
        };

        let indices: Vec<usize> = (1..self.data.len()).collect();

        if !indices.is_empty() {
            self.bulk_build_indices(0, indices);
        } else if self.data.len() == 1 {
            self.nodes[0] = Node::new_leaf(0, None);
        }

        old
    }
    // fn remove_index(&mut self, index: usize)  -> Point {
    //     println!("ri");
    //
    //     if index == self.root {
    //         let end = self.data.pop().unwrap();
    //         self.nodes.pop();
    //         let old = std::mem::replace(&mut self.data[index], end);
    //         let indices: Vec<usize> = (1..self.data.len()).collect();
    //         self.bulk_build_indices(0, indices);
    //
    //         return old
    //     }
    //
    //     let parent = self.nodes[index].parent;
    //     if let Some(p) = parent
    //         && let Some(pi) = self.nodes[p].interior
    //         && pi ==index {
    //         self.nodes[p].interior = None;
    //     }
    //
    //     if let Some(p) = parent
    //         && let Some(pe) = self.nodes[p].exterior
    //         && pe ==index {
    //         self.nodes[p].exterior = None;
    //     }
    //
    //     let mut reinsert = vec![];
    //     if let Some(interior) = self.nodes[index].interior {
    //         reinsert.push(interior);
    //         self.nodes[interior].parent = None;
    //     }
    //
    //     if let Some(exterior) = self.nodes[index].exterior {
    //         reinsert.push(exterior);
    //         self.nodes[exterior].parent = None;
    //     }
    //
    //     let new_root = parent.unwrap_or(self.root);
    //     for ri in reinsert {
    //         println!("reinserting {}", ri);
    //         self.insert_existing(new_root, ri);
    //     }
    //
    //     if index == self.nodes.len() - 1 {
    //         self.nodes.pop();
    //         return self.data.pop().unwrap()
    //     } else {
    //         let end = self.remove_index(self.nodes.len() - 1);
    //         let old = std::mem::replace(&mut self.data[index], end);
    //         self.nodes[index].interior = None;
    //         self.nodes[index].exterior = None;
    //         self.insert_existing(self.root, index);
    //         return old
    //     }
    // }
}

struct NodeProspect {
    index: usize,
    min_distance: f64,
}

impl PartialEq for NodeProspect {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.min_distance == other.min_distance
    }
}

impl Eq for NodeProspect {}

impl PartialOrd for NodeProspect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // reverses comparison order to make small distances greater than large ones
        other.min_distance.partial_cmp(&self.min_distance)
    }
}

impl Ord for NodeProspect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // reverses comparison order to make small distances greater than large ones
        other.min_distance.partial_cmp(&self.min_distance).unwrap()
    }
}

pub struct KnnIterator<'a, Point: VpTreeObject, PointMetric> {
    query_point: &'a Point::PointType,
    tree: &'a VpAvl<Point, PointMetric>,
    prospects: BinaryHeap<NodeProspect>,
    yield_queue: BinaryHeap<NodeProspect>,
}

impl<'a, Point, PointMetric> KnnIterator<'a, Point, PointMetric>
where
    Point: VpTreeObject,
    PointMetric: Metric<PointType = Point::PointType>,
{
    fn new(query_point: &'a Point::PointType, tree: &'a VpAvl<Point, PointMetric>) -> Self {
        let mut prospects = BinaryHeap::new();
        if !tree.nodes.is_empty() {
            prospects.push(NodeProspect {
                index: tree.root,
                min_distance: 0.0,
            });
        }

        KnnIterator {
            query_point,
            tree,
            prospects,
            yield_queue: BinaryHeap::new(),
        }
    }
}

impl<'a, Point, PointMetric> Iterator for KnnIterator<'a, Point, PointMetric>
where
    Point: VpTreeObject,
    PointMetric: Metric<PointType = Point::PointType>,
{
    type Item = (&'a Point, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let top_choice = match self.prospects.pop() {
            Some(x) => x,
            None => {
                // nothing left to check
                return self
                    .yield_queue
                    .pop()
                    .map(|p| (&self.tree.data[p.index], p.min_distance));
            }
        };

        let center_dist = self.tree.metric.distance(
            self.query_point,
            self.tree
                .node_index_data(self.tree.nodes[top_choice.index].center)
                .location(),
        );

        // soft-yield the center
        self.yield_queue.push(NodeProspect {
            index: top_choice.index,
            min_distance: center_dist,
        });

        let diff = center_dist - self.tree.nodes[top_choice.index].radius;
        let min_interior_distance = diff.max(0.0);
        let min_exterior_distance = (-diff).max(0.0);

        if let Some(interior) = self.tree.nodes[top_choice.index].interior {
            self.prospects.push(NodeProspect {
                index: interior,
                min_distance: min_interior_distance,
            })
        }

        if let Some(exterior) = self.tree.nodes[top_choice.index].exterior {
            self.prospects.push(NodeProspect {
                index: exterior,
                min_distance: min_exterior_distance,
            })
        }

        let yield_now = if let Some(yv) = self.yield_queue.peek() {
            if let Some(pv) = self.prospects.peek() {
                if yv.min_distance <= pv.min_distance {
                    // we already have a point at least as good as any prospect
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if yield_now {
            let yv = self.yield_queue.pop().unwrap();

            Some((
                &self.tree.data[self.tree.nodes[yv.index].center],
                yv.min_distance,
            ))
        } else {
            // recurse
            self.next()
        }
    }
}

pub struct KnnMutIterator<'a, Point: VpTreeObject, PointMetric> {
    query_point: &'a Point::PointType,
    tree: &'a mut VpAvl<Point, PointMetric>,
    prospects: BinaryHeap<NodeProspect>,
    yield_queue: BinaryHeap<NodeProspect>,
}

impl<'a, Point, PointMetric> KnnMutIterator<'a, Point, PointMetric>
where
    Point: VpTreeObject,
    PointMetric: Metric<PointType = Point::PointType>,
{
    fn new(query_point: &'a Point::PointType, tree: &'a mut VpAvl<Point, PointMetric>) -> Self {
        let mut prospects = BinaryHeap::new();
        if !tree.nodes.is_empty() {
            prospects.push(NodeProspect {
                index: tree.root,
                min_distance: 0.0,
            });
        }

        KnnMutIterator {
            query_point,
            tree,
            prospects,
            yield_queue: BinaryHeap::new(),
        }
    }
}

impl<'a, Point, PointMetric> Iterator for KnnMutIterator<'a, Point, PointMetric>
where
    Point: VpTreeObject,
    PointMetric: Metric<PointType = Point::PointType>,
{
    type Item = (&'a mut Point, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let top_choice = match self.prospects.pop() {
            Some(x) => x,
            None => {
                // nothing left to check
                return self.yield_queue.pop().map(|p| {
                    let target_loc: *mut Point = &mut self.tree.data[p.index] as *mut Point;
                    let rv: &'a mut Point = unsafe { &mut *target_loc };

                    (rv, p.min_distance)
                });
            }
        };

        let center_dist = self.tree.metric.distance(
            self.query_point,
            self.tree
                .node_index_data(self.tree.nodes[top_choice.index].center)
                .location(),
        );

        // soft-yield the center
        self.yield_queue.push(NodeProspect {
            index: top_choice.index,
            min_distance: center_dist,
        });

        let diff = center_dist - self.tree.nodes[top_choice.index].radius;
        let min_interior_distance = diff.max(0.0);
        let min_exterior_distance = (-diff).max(0.0);

        if let Some(interior) = self.tree.nodes[top_choice.index].interior {
            self.prospects.push(NodeProspect {
                index: interior,
                min_distance: min_interior_distance,
            })
        }

        if let Some(exterior) = self.tree.nodes[top_choice.index].exterior {
            self.prospects.push(NodeProspect {
                index: exterior,
                min_distance: min_exterior_distance,
            })
        }

        let yield_now = if let Some(yv) = self.yield_queue.peek() {
            if let Some(pv) = self.prospects.peek() {
                if yv.min_distance <= pv.min_distance {
                    // we already have a point at least as good as any prospect
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if yield_now {
            let yv = self.yield_queue.pop().unwrap();

            let target_loc: *mut Point = &mut self.tree.data[yv.index] as *mut Point;
            let rv: &'a mut Point = unsafe { &mut *target_loc };

            Some((rv, yv.min_distance))
        } else {
            // recurse
            self.next()
        }
    }
}

struct KnnIndexIterator<'a, Point: VpTreeObject, PointMetric> {
    query_point: &'a Point::PointType,
    tree: &'a VpAvl<Point, PointMetric>,
    prospects: BinaryHeap<NodeProspect>,
    yield_queue: BinaryHeap<NodeProspect>,
}

impl<'a, Point, PointMetric> KnnIndexIterator<'a, Point, PointMetric>
where
    Point: VpTreeObject,
    PointMetric: Metric<PointType = Point::PointType>,
{
    fn new(query_point: &'a Point::PointType, tree: &'a VpAvl<Point, PointMetric>) -> Self {
        let mut prospects = BinaryHeap::new();
        if !tree.nodes.is_empty() {
            prospects.push(NodeProspect {
                index: tree.root,
                min_distance: 0.0,
            });
        }

        KnnIndexIterator {
            query_point,
            tree,
            prospects,
            yield_queue: BinaryHeap::new(),
        }
    }
}

impl<Point, PointMetric> Iterator for KnnIndexIterator<'_, Point, PointMetric>
where
    Point: VpTreeObject,
    PointMetric: Metric<PointType = Point::PointType>,
{
    type Item = (usize, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let top_choice = match self.prospects.pop() {
            Some(x) => x,
            None => {
                // println!("no prospect yq: {}", self.yield_queue.len());

                return self.yield_queue.pop().map(|p| (p.index, p.min_distance));
                // nothing left to check
                // return None;
            }
        };

        let center_dist = self.tree.metric.distance(
            self.query_point,
            self.tree
                .node_index_data(self.tree.nodes[top_choice.index].center)
                .location(),
        );

        // soft-yield the center
        self.yield_queue.push(NodeProspect {
            index: top_choice.index,
            min_distance: center_dist,
        });

        let diff = center_dist - self.tree.nodes[top_choice.index].radius;
        let min_interior_distance = diff.max(0.0);
        let min_exterior_distance = (-diff).max(0.0);

        if let Some(interior) = self.tree.nodes[top_choice.index].interior {
            self.prospects.push(NodeProspect {
                index: interior,
                min_distance: min_interior_distance,
            })
        }

        if let Some(exterior) = self.tree.nodes[top_choice.index].exterior {
            self.prospects.push(NodeProspect {
                index: exterior,
                min_distance: min_exterior_distance,
            })
        }

        let yield_now = if let Some(yv) = self.yield_queue.peek() {
            if let Some(pv) = self.prospects.peek() {
                if yv.min_distance <= pv.min_distance {
                    // we already have a point at least as good as any prospect
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if yield_now {
            let yv = self.yield_queue.pop().unwrap();
            // println!("yield: {} {} ", yv.index,
            //          yv.min_distance);
            Some((yv.index, yv.min_distance))
        } else {
            // recurse
            self.next()
        }
    }
}

impl Node {
    fn new_leaf(center: usize, parent: Option<usize>) -> Self {
        Node {
            height: 0,
            center,
            radius: 0.0,
            interior: None,
            exterior: None,
            parent,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct EuclideanMetric<T> {
    _phantom: PhantomData<T>,
}

impl<T> Metric for EuclideanMetric<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a f64>,
    T: 'static,
{
    type PointType = T;

    fn distance(&self, p1: &Self::PointType, p2: &Self::PointType) -> f64 {
        p1.into_iter()
            .zip(p2)
            .fold(0.0, |acc, (l, r)| acc + (l - r).powf(2.0))
            .sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct WeightedEuclideanMetric<T> {
    weights: T,
}

impl<T> WeightedEuclideanMetric<T> {
    pub fn new(weights: T) -> Self {
        Self { weights }
    }
}

impl<T> Metric for WeightedEuclideanMetric<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a f64>,
    T: 'static,
{
    type PointType = T;

    fn distance(&self, p1: &Self::PointType, p2: &Self::PointType) -> f64 {
        p1.into_iter()
            .zip(p2)
            .zip(&self.weights)
            .fold(0.0, |acc, ((l, r), w)| acc + w * (l - r).powf(2.0))
            .sqrt()
    }
}

///////////////////////////////////////////////////
///////////////////////////////////////////////////

// implementation of a metric for all state space
pub struct StateSpaceMetric {
    space: Rc<dyn StateSpace>,
}

impl StateSpaceMetric {
    pub fn new(space: Rc<dyn StateSpace>) -> Self {
        Self { space }
    }
}

impl Metric for StateSpaceMetric {
    type PointType = StateId;

    fn distance(&self, p1: &Self::PointType, p2: &Self::PointType) -> f64 {
        self.space.distance(p1, p2)
    }
}

impl VpTreeObject for StateId {
    type PointType = Self;

    fn location(&self) -> &Self::PointType {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::base::spaces::real_vector_state_space::{RealVectorState, RealVectorStateSpace};
    use nalgebra::DVector;
    use std::rc::Rc;

    use crate::prelude::CanStateAllocateTrait;

    #[test]
    fn test_distance_state_space() {
        let mut space = RealVectorStateSpace::new();

        space.add_dimension(None, 0.0, 1.0);
        space.add_dimension(None, 1.0, 1.9);
        space.add_dimension(None, 100.0, 100.9);

        let space = Rc::new(space);

        let state1 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![0.5, 1.5, 8.0]),
        });
        let state1_delta = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![0.5, 1.5, 9.0]),
        });
        let state2 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![1.0, 1.5, 100.0]),
        });
        let state2_delta = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![0.5, 1.5, 90.0]),
        });

        let mut tree = VpAvl::new(StateSpaceMetric::new(space.clone()));

        let v1 = space.clone_state_inner_value(&state1);
        let v2 = space.clone_state_inner_value(&state2);

        tree.insert(state1);
        tree.insert(state2);

        assert_eq!(
            v1.values,
            space
                .clone_state_inner_value(tree.nn_iter(&state1_delta).next().unwrap())
                .values,
        );
        assert_eq!(
            v2.values,
            space
                .clone_state_inner_value(tree.nn_iter(&state2_delta).next().unwrap())
                .values,
        );
    }

    #[test]
    fn test_distance() {
        let mut tree = VpAvl::new(EuclideanMetric::default());

        tree.insert(vec![0.0, 0.0]);
        tree.insert(vec![0.0, 1.0]);

        assert_eq!(
            tree.nn_iter(&vec![0.0, 0.49]).next().unwrap(),
            &vec![0.0, 0.0]
        );
        assert_eq!(
            tree.nn_iter(&vec![0.0, 0.51]).next().unwrap(),
            &vec![0.0, 1.0]
        );
    }
}
