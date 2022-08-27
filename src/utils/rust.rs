pub trait BoolExtended {
    /// Sets value to the opposite of current one and returns new value
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
    /// Returns random element.
    ///
    /// Panics if there are no elements to select from.
    fn random(self) -> T;

    /// Returns random element or None if there are no elements to select from
    fn get_random(self) -> Option<T>;
}

impl<Iter: ExactSizeIterator + Clone> RandomSelect<Iter::Item> for Iter {
    fn random(self) -> Iter::Item {
        self.get_random().unwrap()
    }

    fn get_random(self) -> Option<Iter::Item> {
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

//

pub trait OptionExtended<T> {
    /// Folds on self
    fn while_some(self, f: impl FnMut(T) -> Self);
}

impl<T> OptionExtended<T> for Option<T> {
    fn while_some(self, mut f: impl FnMut(T) -> Self) {
        let mut next = self;
        while let Some(value) = next {
            next = f(value)
        }
    }
}
