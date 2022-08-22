use std::time::Duration;

// pub trait DurationExtended {
//     fn div_duration_f32(&self, rhs: Duration) -> f32;
// }

// impl DurationExtended for Duration {
//     fn div_duration_f32(&self, rhs: Duration) -> f32 {
//         self.as_secs_f32() / rhs.as_secs_f32()
//     }
// }

pub trait BoolExtended {
    /// Sets value to the opposite of current and returns new value
    fn flip(&mut self) -> Self;
}

impl BoolExtended for bool {
    fn flip(&mut self) -> Self {
        *self = !*self;
        *self
    }
}

//

pub trait RandomSelect<T> {
    fn random_select(self) -> T;
    fn get_random_select(self) -> Option<T>;
}

impl<Iter: ExactSizeIterator + Clone> RandomSelect<Iter::Item> for Iter {
    fn random_select(self) -> Iter::Item {
        self.get_random_select().unwrap()
    }

    fn get_random_select(self) -> Option<Iter::Item> {
        use rand::*;
        let len = self.len();
        if len == 0 {
            None
        } else {
            let i = thread_rng().gen_range(0..len);
            self.skip(i).next()
        }
    }
}
