use std::f64;

pub type DistanceFunction<T> = Box<dyn Fn(&T, &T) -> f64 + Send + Sync>;

/// A trait for nearest neighbors search algorithms.
pub trait NearestNeighbors<T> {
    /// Sets the distance function to be used for nearest neighbors search.
    ///
    /// # Arguments
    ///
    /// * `dist_fn` - A function that calculates the distance between two data points.
    fn set_distance_function(&mut self, dist_fn: DistanceFunction<T>);

    /// Adds a data point to the data structure.
    ///
    /// # Arguments
    ///
    /// * `data` - The data point to be added.
    fn add(&mut self, data: T);

    /// Adds multiple data points to the data structure.
    ///
    /// # Arguments
    ///
    /// * `data` - A vector of data points to be added.
    fn add_multiple(&mut self, data: Vec<T>);

    /// Removes a data point from the data structure.
    ///
    /// # Arguments
    ///
    /// * `data` - The data point to be removed.
    ///
    /// # Returns
    ///
    /// `true` if the data point was successfully removed, `false` otherwise.
    fn remove(&mut self, data: &T) -> bool;

    /// Finds the nearest neighbor to a given data point.
    ///
    /// # Arguments
    ///
    /// * `data` - The data point to find the nearest neighbor for.
    ///
    /// # Returns
    ///
    /// An `Option` containing the nearest neighbor, or `None` if the data structure is empty.
    fn nearest(&self, data: &T) -> Option<T>;

    /// Finds the `k` nearest neighbors to a given data point.
    ///
    /// # Arguments
    ///
    /// * `data` - The data point to find the nearest neighbors for.
    /// * `k` - The number of nearest neighbors to find.
    ///
    /// # Returns
    ///
    /// A vector containing the `k` nearest neighbors.
    fn nearest_k(&self, data: &T, k: usize) -> Vec<T>;

    /// Finds all neighbors within a given radius of a data point.
    ///
    /// # Arguments
    ///
    /// * `data` - The data point to find the neighbors for.
    /// * `radius` - The radius within which to find neighbors.
    ///
    /// # Returns
    ///
    /// A vector containing all neighbors within the given radius.
    fn nearest_r(&self, data: &T, radius: f64) -> Vec<T>;

    /// Clears all data points from the data structure.
    fn clear(&mut self);

    /// Returns the number of data points in the data structure.
    ///
    /// # Returns
    ///
    /// The number of data points in the data structure.
    fn size(&self) -> usize;
}