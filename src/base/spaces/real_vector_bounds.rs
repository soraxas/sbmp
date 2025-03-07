#[derive(Debug, Clone)]
pub struct RealVectorBounds {
    pub(crate) low: Vec<f64>,
    pub(crate) high: Vec<f64>,
}

impl RealVectorBounds {
    /// Constructor; `dim` represents the dimension of the space these bounds are for.
    pub fn new(dim: usize) -> Self {
        let mut bounds = RealVectorBounds {
            low: vec![0.0; dim],
            high: vec![0.0; dim],
        };
        bounds.resize(dim);
        bounds
    }

    /// Set the lower bound in each dimension to a specific value
    pub fn set_low(&mut self, value: f64) {
        for i in 0..self.low.len() {
            self.low[i] = value;
        }
    }

    /// Set the upper bound in each dimension to a specific value
    pub fn set_high(&mut self, value: f64) {
        for i in 0..self.high.len() {
            self.high[i] = value;
        }
    }

    /// Set the lower bound of a dimension to a specific value
    pub fn set_low_at(&mut self, index: usize, value: f64) {
        if index < self.low.len() {
            self.low[index] = value;
        }
    }

    /// Set the upper bound of a dimension to a specific value
    pub fn set_high_at(&mut self, index: usize, value: f64) {
        if index < self.high.len() {
            self.high[index] = value;
        }
    }

    /// Change the number of dimensions for the bounds
    pub fn resize(&mut self, size: usize) {
        self.low.resize(size, 0.0);
        self.high.resize(size, 0.0);
    }

    /// Compute the volume of the space enclosed by the bounds
    pub fn get_volume(&self) -> f64 {
        self.low
            .iter()
            .zip(&self.high)
            .map(|(l, h)| h - l)
            .product()
    }

    /// Get the difference between the high and low bounds for each dimension
    pub fn get_difference(&self) -> Vec<f64> {
        self.low
            .iter()
            .zip(&self.high)
            .map(|(l, h)| h - l)
            .collect()
    }

    /// Check if the bounds are valid (same length for low and high, high[i] > low[i])
    pub fn check(&self) {
        assert_eq!(
            self.low.len(),
            self.high.len(),
            "Low and high bounds must have the same length"
        );
        for i in 0..self.low.len() {
            assert!(
                self.high[i] > self.low[i],
                "High bound must be greater than low bound at index {}",
                i
            );
        }
    }
}
