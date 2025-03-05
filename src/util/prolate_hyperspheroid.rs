use crate::error::ProlateHyperspheroidError;
use nalgebra::{DMatrix, DVector};

pub struct ProlateHyperspheroid {
    dimension: usize,
    focus1: DVector<f64>,
    focus2: DVector<f64>,
    transverse_diameter: f64,
    min_transverse_diameter: f64,
    is_transform_up_to_date: bool,
    rotation_world_from_ellipse: DMatrix<f64>,
    transformation_world_from_ellipse: DMatrix<f64>,
    phs_measure: f64,
}

impl ProlateHyperspheroid {
    pub fn new(dimension: usize, focus1: &[f64], focus2: &[f64]) -> Self {
        panic!("Not implemented correctly.");
        assert_eq!(focus1.len(), dimension);
        assert_eq!(focus2.len(), dimension);
        let focus1 = DVector::from_column_slice(focus1);
        let focus2 = DVector::from_column_slice(focus2);
        let min_transverse_diameter = (focus1.clone() - focus2.clone()).norm();
        let rotation_world_from_ellipse = DMatrix::zeros(dimension, dimension);
        let transformation_world_from_ellipse = DMatrix::zeros(dimension, dimension);
        Self {
            dimension,
            focus1,
            focus2,
            transverse_diameter: 0.0,
            min_transverse_diameter,
            is_transform_up_to_date: false,
            rotation_world_from_ellipse,
            transformation_world_from_ellipse,
            phs_measure: 0.0,
        }
    }

    pub fn set_transverse_diameter(
        &mut self,
        transverse_diameter: f64,
    ) -> Result<(), ProlateHyperspheroidError> {
        if transverse_diameter < self.min_transverse_diameter {
            return Err(ProlateHyperspheroidError::InvalidTransverseDiameter);
        }
        if self.transverse_diameter != transverse_diameter {
            self.is_transform_up_to_date = false;
            self.transverse_diameter = transverse_diameter;
            self.update_transformation();
        }
        Ok(())
    }

    pub fn transform(
        &self,
        sphere: &[f64],
        phs: &mut [f64],
    ) -> Result<(), ProlateHyperspheroidError> {
        if !self.is_transform_up_to_date {
            return Err(ProlateHyperspheroidError::TransformationNotUpToDate);
        }
        let sphere = DVector::from_column_slice(sphere);
        let mut phs_vec = DVector::from_column_slice(phs);
        phs_vec =
            &self.transformation_world_from_ellipse * sphere + (&self.focus1 + &self.focus2) / 2.0;
        phs.copy_from_slice(phs_vec.as_slice());
        Ok(())
    }

    pub fn is_in_phs(&self, point: &[f64]) -> Result<bool, ProlateHyperspheroidError> {
        if !self.is_transform_up_to_date {
            return Err(ProlateHyperspheroidError::TransformationNotUpToDate);
        }
        Ok(self.get_path_length(point) < self.transverse_diameter)
    }

    pub fn is_on_phs(&self, point: &[f64]) -> Result<bool, ProlateHyperspheroidError> {
        if !self.is_transform_up_to_date {
            return Err(ProlateHyperspheroidError::TransformationNotUpToDate);
        }
        Ok(self.get_path_length(point) == self.transverse_diameter)
    }

    pub fn get_phs_dimension(&self) -> usize {
        self.dimension
    }

    pub fn get_phs_measure(&self) -> f64 {
        if !self.is_transform_up_to_date {
            return f64::INFINITY;
        }
        self.phs_measure
    }

    pub fn get_phs_measure_with_diameter(&self, tran_diam: f64) -> f64 {
        // ...measure logic...
        0.0
    }

    pub fn get_min_transverse_diameter(&self) -> f64 {
        self.min_transverse_diameter
    }

    pub fn get_path_length(&self, point: &[f64]) -> f64 {
        let point = DVector::from_column_slice(point);
        (point.clone() - self.focus1.clone()).norm() + (point - self.focus2.clone()).norm()
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    fn update_rotation(&mut self) {
        self.is_transform_up_to_date = false;
        let circle_tol = 1e-9;
        if self.min_transverse_diameter < circle_tol {
            self.rotation_world_from_ellipse.fill_with_identity();
        } else {
            // ...rotation update logic using nalgebra equivalent...
        }
    }

    fn update_transformation(&mut self) {
        let conjugate_diameter =
            (self.transverse_diameter.powi(2) - self.min_transverse_diameter.powi(2)).sqrt();
        for i in 0..self.dimension {
            for j in 0..self.dimension {
                self.transformation_world_from_ellipse[(i, j)] = if i == 0 {
                    self.transverse_diameter / 2.0
                } else {
                    conjugate_diameter / 2.0
                };
            }
        }
        self.phs_measure = 0.0; // Update with actual measure calculation
        self.is_transform_up_to_date = true;
    }
}
