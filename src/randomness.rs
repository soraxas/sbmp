use rand::distributions::{Distribution, Standard, Uniform};
use rand::rngs::{StdRng, ThreadRng};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng, SeedableRng};
use statrs::distribution::Normal;
use std::f64::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, warn};

use crate::util::prolate_hyperspheroid::ProlateHyperspheroid;

lazy_static::lazy_static! {
    static ref RANDOMNESS: Mutex<RNGSeedGenerator> = Mutex::new(RNGSeedGenerator::new());
}

/// We use a different random number generator for the seeds of the
/// other random generators. The root seed is from the number of
/// nano-seconds in the current time, or given by the user.
pub struct RNGSeedGenerator {
    first_seed: u64,
    s_gen: rand::rngs::StdRng,
    s_dist: Uniform<u64>,
    some_seeds_generated: bool,
}

impl RNGSeedGenerator {
    fn new() -> Self {
        let first_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        let s_gen = rand::rngs::StdRng::seed_from_u64(first_seed);
        let s_dist = Uniform::new(1, 1_000_000_000);
        Self {
            first_seed,
            s_gen,
            s_dist,
            some_seeds_generated: false,
        }
    }

    /// Get the first seed used to generate the other seeds.
    pub fn first_global_seed(&self) -> u64 {
        self.first_seed
    }

    pub fn set_global_seed(&mut self, seed: u64) {
        if seed > 0 {
            if self.some_seeds_generated {
                error!("Random number generation already started. Changing seed now will not lead to deterministic sampling.");
            } else {
                self.first_seed = seed;
            }
        } else {
            if self.some_seeds_generated {
                warn!("Random generator seed cannot be 0, and random number generation already started. Ignoring seed.");
                return;
            }
            warn!("Random generator seed cannot be 0. Using 1 instead.");
            self.first_seed = 1;
        }
        self.s_gen = rand::rngs::StdRng::seed_from_u64(self.first_seed);
    }

    fn next_seed(&mut self) -> u64 {
        // 1
        self.some_seeds_generated = true;
        self.s_dist.sample(&mut self.s_gen)
    }
}

pub struct RNG {
    rng: StdRng,
    normal: Normal,
    local_seed: u64,
}

impl Default for RNG {
    fn default() -> Self {
        Self::new()
    }
}

impl RNG {
    pub fn new() -> Self {
        let local_seed = RANDOMNESS.lock().unwrap().next_seed();
        Self::with_seed(local_seed)
    }

    pub fn get_local_seed(&self) -> u64 {
        self.local_seed
    }

    pub fn with_seed(local_seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(local_seed),
            normal: Normal::standard(),
            local_seed,
        }
    }

    pub fn uniform01(&mut self) -> f64 {
        self.rng.sample(Standard)
    }

    pub fn uniform_real(&mut self, lower_bound: f64, upper_bound: f64) -> f64 {
        self.rng.sample(Uniform::new(lower_bound, upper_bound))
    }

    pub fn uniform_int(&mut self, lower_bound: i32, upper_bound: i32) -> i32 {
        self.rng.sample(Uniform::new(lower_bound, upper_bound))
    }

    pub fn uniform_bool(&mut self) -> bool {
        self.uniform01() <= 0.5
    }

    pub fn gaussian01(&mut self) -> f64 {
        self.normal.sample(&mut self.rng)
    }

    pub fn gaussian(&mut self, mean: f64, stddev: f64) -> f64 {
        Normal::new(mean, stddev).unwrap().sample(&mut self.rng)
        // self.normal.sample(&mut self.rng) * stddev + mean
    }

    pub fn half_normal_real(&mut self, r_min: f64, r_max: f64, focus: f64) -> f64 {
        assert!(r_min <= r_max);

        let mean = r_max - r_min;
        let stddev = mean / focus;
        let mut value = self.gaussian(mean, stddev);

        if value > mean {
            value = 2.0 * mean - value;
        }
        value.clamp(r_min, r_max)
    }

    pub fn half_normal_int(&mut self, r_min: i32, r_max: i32, focus: f64) -> i32 {
        let r = self
            .half_normal_real(r_min as f64, r_max as f64 + 1., focus)
            .floor();
        (r as i32).clamp(r_min, r_max)
    }

    pub fn quaternion(&mut self, value: &mut [f64; 4]) {
        let x0 = self.uniform01();
        let r1 = (1. - x0).sqrt();
        let r2 = x0.sqrt();
        let t1 = 2. * PI * self.uniform01();
        let t2 = 2. * PI * self.uniform01();
        let c1 = t1.cos();
        let s1 = t1.sin();
        let c2 = t2.cos();
        let s2 = t2.sin();

        value[0] = r1 * s1;
        value[1] = r1 * c1;
        value[2] = r2 * s2;
        value[3] = r2 * c2;
    }

    pub fn euler_rpy(&mut self, value: &mut [f64; 3]) {
        value[0] = PI * (-2. * self.uniform01() + 1.);
        value[1] = (1. - 2. * self.uniform01()).acos() - PI / 2.;
        value[2] = PI * (-2. * self.uniform01() + 1.);
    }

    /// Sample a random unit vector in 3D space.
    /// We draw a normal distribution for each element of the vector, and then normalize the vector.
    pub fn uniform_normal_vector(&mut self, v: &mut [f64]) {
        let mut norm = 0.0;
        // sample a normal distribution for each element
        v.iter_mut().for_each(|x| {
            *x = self.normal.sample(&mut self.rng);
            norm += (*x) * (*x);
        });
        if norm <= 0. {
            // If the norm is zero, we cannot normalize the vector.
            // unlikely, but we'll just sample it again.
            return self.uniform_normal_vector(v);
        }
        norm = norm.sqrt();
        v.iter_mut().for_each(|x| *x /= norm);
    }

    pub fn uniform_in_ball(&mut self, r: f64, v: &mut [f64]) {
        self.uniform_normal_vector(v);

        // draw a random radius
        let radius = self.uniform01().powf(1.0 / v.len() as f64) * r;

        // scale the point on the unit sphere
        v.iter_mut().for_each(|x| *x *= radius);
    }

    pub fn uniform_prolate_hyperspheroid_surface(
        &mut self,
        phs: &ProlateHyperspheroid,
        value: &mut [f64],
    ) {
        let mut sphere = vec![0.0; phs.get_dimension()];

        // random point on the sphere
        self.uniform_normal_vector(&mut sphere);

        // transform to the prolate hyperspheroid
        phs.transform(&sphere, value).unwrap();
    }

    pub fn uniform_prolate_hyperspheroid(&mut self, phs: &ProlateHyperspheroid, value: &mut [f64]) {
        let mut sphere = vec![0.0; phs.get_dimension()];

        // get a random point in the sphere
        self.uniform_in_ball(1.0, &mut sphere);

        phs.transform(&sphere, value).unwrap();
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        slice.shuffle(&mut self.rng);
    }
}

// test RNGSeedGenerator with multi thread
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rng_seed_generator() {
        // let mut rng_seed_generator = RNGSeedGenerator::new();
        // let mut rng_seed_generator = rng_seed_generator;
        let mut handles = vec![];
        for _ in 0..10 {
            let handle = thread::spawn(move || {
                let seed = RANDOMNESS.lock().unwrap().next_seed();
                // sleep 1s
                // thread::sleep(std::time::Duration::from_secs(1));
                seed
            });
            handles.push(handle);
        }
        for handle in handles {
            let _seed = handle.join().unwrap();
            // println!("seed: {}", seed);
        }
    }
}
