use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::Arc};

use crate::base::{state_validity_checker::StateValidityChecker, statespace::StateSpace};

use super::{MotionCheckStats, MotionValidator};

pub struct DiscreteMotionValidator {
    state_space: Rc<dyn StateSpace>,
    checker: Arc<dyn StateValidityChecker>,
    stats: RefCell<MotionCheckStats>,
}

impl MotionValidator for DiscreteMotionValidator {
    fn new(state_space: std::rc::Rc<dyn StateSpace>, checker: Arc<dyn StateValidityChecker>) -> Self
    where
        Self: Sized,
    {
        Self {
            state_space,
            checker,
            stats: RefCell::new(MotionCheckStats::default()),
        }
    }

    fn get_motion_check_stats(&self) -> &RefCell<super::MotionCheckStats> {
        &self.stats
    }

    fn check_motion(
        &self,
        s1: &crate::base::state_allocator::StateId,
        s2: &crate::base::state_allocator::StateId,
    ) -> bool {
        if (self.checker.is_valid(s2)) {
            self.stats.borrow_mut().invalid += 1;
            return false;
        }

        let mut result = true;
        let nd = self.state_space.valid_segment_count(s1, s2);

        let mut pos = VecDeque::new();
        if nd >= 2 {
            pos.push_back((1, nd - 1));

            let mut test = self.state_space.alloc_state();

            while let Some((first, second)) = pos.pop_front() {
                let mid = (first + second) / 2;
                self.state_space
                    .interpolate(s1, s2, mid as f64 / nd as f64, &mut test);

                if !self.checker.is_valid(&test) {
                    result = false;
                    break;
                }

                if first < mid {
                    pos.push_back((first, mid - 1));
                }
                if second > mid {
                    pos.push_back((mid + 1, second));
                }
            }

            self.state_space.free_state(&test);
        }

        if result {
            self.stats.borrow_mut().valid += 1;
        } else {
            self.stats.borrow_mut().invalid += 1;
        }
        result
    }

    fn check_motion_with_last_valid(
        &self,
        s1: &crate::base::state_allocator::StateId,
        s2: &crate::base::state_allocator::StateId,
        last_valid: &mut (Option<crate::base::state_allocator::StateId>, f64),
    ) -> bool {
        // assume motion starts in a valid configuration

        let mut result = true;
        let nd = self.state_space.valid_segment_count(s1, s2);

        if nd > 1 {
            let mut test = self.state_space.alloc_state();

            for j in 1..nd {
                self.state_space
                    .interpolate(s1, s2, j as f64 / nd as f64, &mut test);

                if !self.checker.is_valid(&test) {
                    last_valid.1 = (j - 1) as f64 / nd as f64;
                    if let Some(s) = &mut last_valid.0 {
                        self.state_space.interpolate(s1, s2, last_valid.1, s);
                    }

                    result = false;
                    break;
                }
            }

            self.state_space.free_state(&test);
        }

        if result && !self.checker.is_valid(s2) {
            last_valid.1 = (nd - 1) as f64 / nd as f64;
            if let Some(s) = &mut last_valid.0 {
                self.state_space.interpolate(s1, s2, last_valid.1, s);
            }
            result = false;
        }

        if result {
            self.stats.borrow_mut().valid += 1;
        } else {
            self.stats.borrow_mut().invalid += 1;
        }

        result
    }
}
