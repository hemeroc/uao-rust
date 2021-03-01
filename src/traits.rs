pub trait KotlinAny {
    #[inline(always)]
    fn also<ANY>(self, function: impl FnOnce(&Self) -> ANY) -> Self where Self: Sized {
        function(&self);
        self
    }

    #[inline(always)]
    fn take_if(self, predicate: impl FnOnce(&Self) -> bool) -> Option<Self> where Self: Sized {
        if predicate(&self) {
            return Some(self);
        }
        None
    }
}

impl<ANY> KotlinAny for ANY {}
