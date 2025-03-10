// use std::cmp::Ordering;
// use std::collections::{BinaryHeap, HashSet, VecDeque};
// use std::f64;
// use std::sync::Arc;

// use super::nearest_neighbours::{DistanceFunction, NearestNeighbors};

// pub struct GNAT<'a, T: Eq + std::hash::Hash> {
//     degree: usize,
//     min_degree: usize,
//     max_degree: usize,
//     max_num_pts_per_leaf: usize,
//     removed_cache_size: usize,
//     rebuild_size: usize,
//     size: usize,
//     tree: Option<Node<T>>,
//     removed: HashSet<T>,
//     dist_fn: DistanceFunction<T>,

//     near_queue: NearQueue<'a, T>,
//     /// \brief Nodes yet to be processed for possible nearest neighbors
//     node_queue: NearQueue<'a, T>,
// }

// struct Node<T> {
//     // scratch space to store the distance to the pivot
//     dist_to_pivot: f64,

//     degree: usize,
//     pivot: T,
//     min_radius: f64,
//     max_radius: f64,
//     min_range: Vec<f64>,
//     max_range: Vec<f64>,
//     data: Vec<T>,
//     children: Vec<Node<T>>,
// }

// struct Neighbour<'a, T> {
//     dist: f64,
//     key: &'a T,
// }
// type NearQueue<'a, T> =  BinaryHeap<Neighbour<'a, T>>;

// impl <'a, T>
// PartialEq for Neighbour<'a, T> {
//     fn eq(&self, other: &Self) -> bool {
//         self.dist == other.dist
//     }
// }

// impl <'a, T>
// PartialOrd for Neighbour<'a, T> {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         self.dist.partial_cmp(&other.dist)
//     }
// }

// impl <'a, T>
// Eq for Neighbour<'a, T> {}

// impl <'a, T>
// Ord for Neighbour<'a, T> {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.dist.partial_cmp(&other.dist).unwrap()
//     }
// }

// // impl<T> PartialEq for Node<T> {
// //     fn eq(&self, other: &Self) -> bool {
// //         self.pivot == other.pivot
// //     }
// // }

// impl<T> Node<T>
// where
//     T: Eq + std::hash::Hash,
// {
//     fn new(degree: usize, pivot: T) -> Self {
//         Self {
//             dist_to_pivot: f64::INFINITY,
//             degree,
//             pivot,
//             min_radius: f64::INFINITY,
//             max_radius: f64::NEG_INFINITY,
//             min_range: vec![f64::INFINITY; degree],
//             max_range: vec![f64::NEG_INFINITY; degree],
//             data: Vec::new(),
//             children: Vec::new(),
//         }
//     }

//     fn update_radius(&mut self, dist: f64) {
//         if self.min_radius > dist {
//             self.min_radius = dist;
//         }
//         if self.max_radius < dist {
//             self.max_radius = dist;
//         }
//     }

//     fn update_range(&mut self, i: usize, dist: f64) {
//         if self.min_range[i] > dist {
//             self.min_range[i] = dist;
//         }
//         if self.max_range[i] < dist {
//             self.max_range[i] = dist;
//         }
//     }

//     // fn add(&mut self, gnat: &mut GNAT<T>, data: T) {
//     //     if self.children.is_empty() {
//     //         self.data.push(data);
//     //         gnat.size += 1;
//     //         if self.data.len() > gnat.max_num_pts_per_leaf && self.data.len() > self.degree {
//     //             self.split(gnat);
//     //         }
//     //     } else {
//     //         let mut min_dist = (gnat.dist_fn)(&data, &self.children[0].pivot);
//     //         let mut min_ind = 0;
//     //         for (i, child) in self.children.iter_mut().enumerate().skip(1) {
//     //             let dist = (gnat.dist_fn)(&data, &child.pivot);
//     //             if dist < min_dist {
//     //                 min_dist = dist;
//     //                 min_ind = i;
//     //             }
//     //             child.update_range(min_ind, dist);
//     //         }
//     //         self.children[min_ind].update_radius(min_dist);
//     //         self.children[min_ind].add(gnat, data);
//     //     }
//     // }

//     fn need_to_split(&self, gnat: &GNAT<T>) -> bool {
//         self.data.len() > gnat.max_num_pts_per_leaf && self.data.len() > self.degree
//     }

//     fn split(&mut self, gnat: &mut GNAT<T>) {
//         // ...existing code for splitting the node...
//     }

//     fn nearest_k(&self, gnat: &GNAT<T>, data: &T, k: usize, nbh: &mut VecDeque<(f64, Arc<T>)>) {
//         // ...existing code for finding k nearest neighbors...
//     }

//     fn nearest_r(&self, gnat: &GNAT<T>, data: &T, radius: f64, nbh: &mut VecDeque<(f64, Arc<T>)>) {
//         // ...existing code for finding neighbors within radius...
//     }
// }

//     pub fn insert_neighbour_k<'a, T>(
//         // &self
//         nbh: &mut NearQueue<'a, T>,
//         k: usize,
//         data: &'a T,
//         key: &'a T,
//         dist: f64,
//     ) -> bool
//         where
//             T: Eq + std::hash::Hash
//     {
//         if nbh.len() < k {
//             nbh.push(Neighbour { dist, key });
//             return true;
//         }

//         match nbh.peek() {
//             Some(neighbour) => {
//                 if dist < neighbour.dist || dist < f64::EPSILON
//                 && data == key
//                 {
//                     nbh.pop();
//                     nbh.push(Neighbour { dist, key });
//                     return true;
//                 }
//             }
//             None => {
//                 return false;
//             }
//         }

//         return false;
//     }

// impl<'a, T> GNAT<'a, T>
// where
//     T: Eq + std::hash::Hash,
// {
//     pub fn new(
//         degree: usize,
//         min_degree: usize,
//         max_degree: usize,
//         max_num_pts_per_leaf: usize,
//         removed_cache_size: usize,
//         dist_fn: DistanceFunction<T>,
//     ) -> Self {
//         Self {
//             node_queue: BinaryHeap::new(),
//             near_queue: BinaryHeap::new(),

//             degree,
//             min_degree,
//             max_degree,
//             max_num_pts_per_leaf,
//             removed_cache_size,
//             rebuild_size: max_num_pts_per_leaf * degree,
//             size: 0,
//             tree: None,
//             removed: HashSet::new(),
//             dist_fn,
//         }
//     }

//     fn rebuild_data_structure(&mut self) {
//         // ...existing code for rebuilding the data structure...
//     }

//     fn add_to_node(&mut self, node: &mut Node<T>, data: T) {
//         // TODO ifdef GNAT_SAMPLER
//         if node.children.is_empty() {
//             node.data.push(data);
//             self.size += 1;
//             if node.need_to_split(self) {
//                 if !self.removed.is_empty() {
//                     self.rebuild_data_structure();
//                 } else if self.size >= self.rebuild_size {
//                     let rebuild_size = self.rebuild_size << 1;
//                     self.rebuild_data_structure();
//                     self.rebuild_size = rebuild_size;
//                 } else {
//                     node.split(self);
//                 }
//             }
//         } else {
//             let mut min_dist = (self.dist_fn)(&data, &node.children[0].pivot);
//             node.children[0].dist_to_pivot = min_dist;

//             let mut min_ind = 0;
//             for (i, child) in node.children.iter_mut().enumerate().skip(1) {
//                 child.dist_to_pivot = (self.dist_fn)(&data, &child.pivot);
//                 if child.dist_to_pivot < min_dist {
//                     min_dist = child.dist_to_pivot;
//                     min_ind = i;
//                 }
//             }

//             for child in node.children.iter_mut() {
//                 child.update_range(min_ind, child.dist_to_pivot);
//             }

//             node.children[min_ind].update_radius(min_dist);
//             self.add_to_node(&mut node.children[min_ind], data);
//         }
//     }

//     fn nearest_k_internal(&self, data: &T, k: usize) -> bool {

//         let tree = if let Some(tree) = self.tree { tree
//         } else {
//             return false;
//         };

//         let mut is_pivot = false;

//         tree.dist_to_pivot = (self.dist_fn)(data, &tree.pivot);
//         is_pivot = insert_neighbour_k(
//             &mut self.near_queue,
//             k,
//             &tree.pivot,
//             data,
//             tree.dist_to_pivot,
//         );

//         if self.size == 0 {
//             return None;
//         }

//         let mut nbh = VecDeque::with_capacity(k);
//         if let Some(tree) = &self.tree {
//             tree.nearest_k(self, data, k, &mut nbh);
//         }

//         is_pivot
//     }
// }

// impl<T> NearestNeighbors<T> for GNAT<T>
// where
//     T: Eq + std::hash::Hash,
// {
//     fn set_distance_function(&mut self, dist_fn: DistanceFunction<T>) {
//         self.dist_fn = dist_fn;
//         if self.tree.is_some() {
//             self.rebuild_data_structure();
//         }
//     }

//     fn add(&mut self, data: T) {
//         // tempoarily remove the data from the tree (to avoid ownership)
//         let mut tree = self.tree.take();

//         if let Some(tree) = &mut tree {
//             if self.removed.contains(&data) {
//                 self.rebuild_data_structure();
//             }

//             self.add_to_node(tree, data);
//         } else {
//             tree = Some(Node::new(self.degree, data));
//             self.size = 1;
//         }

//         // insert the data back into the tree
//         self.tree = tree;
//     }

//     fn add_multiple(&mut self, mut data: Vec<T>) {
//         if self.tree.is_none() {
//             if data.is_empty() {
//                 return;
//             }
//             // data has length 1, rest has the rest of the data
//             let rest = data.split_off(1);
//             let mut new_root = Node::new(self.degree, data.remove(0));
//             // TODO: GNAT_SAMPLER tree_->subtreeSize_ = data.size();

//             new_root.data.extend(rest);
//             self.size = 1 + data.len();
//             if new_root.need_to_split(self) {
//                 new_root.split(self);
//             }

//             self.tree = Some(new_root);
//         }
//         for d in data {
//             self.add(d);
//         }
//     }

//     fn remove(&mut self, data: &T) -> bool {
//         if self.size == 0 {
//             return false;
//         }
//         // find data in tree
//         let is_pivor = self.nearest_k_internal(data, k)
//     }

//     fn nearest(&self, data: &T) -> Option<T> {
//         // ...existing code for finding the nearest neighbor...
//     }

//     fn nearest_k(&self, data: &T, k: usize) -> Vec<T> {
//         // ...existing code for finding k nearest neighbors...
//     }

//     fn nearest_r(&self, data: &T, radius: f64) -> Vec<T> {
//         // ...existing code for finding neighbors within radius...
//     }

//     fn clear(&mut self) {
//         self.tree = None;
//         self.size = 0;
//         self.removed.clear();
//         // TODO: rebuildSize_
//     }

//     fn size(&self) -> usize {
//         self.size
//     }
// }
