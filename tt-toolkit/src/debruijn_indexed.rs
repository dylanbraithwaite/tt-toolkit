/// Syntax nodes which have free variables represented by DeBruijn indices
pub trait DeBruijnIndexed: Sized {
    fn map_indices_from<F>(&self, start: usize, map_fn: F) -> Self
    where
        F: Fn(usize) -> usize + Clone;

    fn get_var(&self) -> Option<usize>;

    fn map_indices<F>(&self, map_fn: F) -> Self
    where
        F: Fn(usize) -> usize + Clone,
    {
        self.map_indices_from(0, map_fn)
    }

    // type VarType: std::cmp::Ord;
    /// Increment de Bruijn indices in the term which are at least equal to `start`
    /// by `amount`.
    /// This is required for example, when adjusting de Bruijn indices in an
    /// expression where new variables are bound.
    fn increment_indices_from_by(&self, start: usize, amount: usize) -> Self {
        self.map_indices_from(start, |i| i + amount)
    }

    /// Increment de Bruijn indices in the term by `amount`.
    fn increment_indices_by(&self, amount: usize) -> Self {
        self.increment_indices_from_by(0, amount)
    }

    /// Increment de Bruijn indices in the term by one.
    fn increment_indices(&self) -> Self {
        self.increment_indices_by(1)
    }
}

impl<T: DeBruijnIndexed> DeBruijnIndexed for Box<T> {
    fn map_indices_from<F>(&self, start: usize, map_fn: F) -> Self
    where
        F: Fn(usize) -> usize + Clone,
    {
        (**self).map_indices_from(start, map_fn).into()
    }

    fn get_var(&self) -> Option<usize> {
        DeBruijnIndexed::get_var(&**self)
    }
}

impl<T: DeBruijnIndexed> DeBruijnIndexed for std::rc::Rc<T> {
    fn map_indices_from<F>(&self, start: usize, map_fn: F) -> Self
    where
        F: Fn(usize) -> usize + Clone,
    {
        (**self).map_indices_from(start, map_fn).into()
    }

    fn get_var(&self) -> Option<usize> {
        DeBruijnIndexed::get_var(&**self)
    }
}

impl<T: DeBruijnIndexed> DeBruijnIndexed for Option<T> {
    fn map_indices_from<F>(&self, start: usize, map_fn: F) -> Self
    where
        F: Fn(usize) -> usize + Clone,
    {
        self.as_ref()
            .map(|expr| expr.map_indices_from(start, map_fn))
    }

    fn get_var(&self) -> Option<usize> {
        self.as_ref().and_then(|expr| expr.get_var())
    }
}
