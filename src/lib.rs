use std::cell::UnsafeCell;
use std::marker::PhantomData;

struct InvariantLifetime<'s> {
    _marker: PhantomData<&'s mut &'s ()>,
}

pub struct GhostToken<'id> {
    _marker: InvariantLifetime<'id>,
}

unsafe impl<'id> Send for GhostToken<'id> {}

unsafe impl<'id> Sync for GhostToken<'id> {}

impl<'id> GhostToken<'id> {
    pub fn new<R, F: FnOnce(Self) -> R>(f: F) -> R {
        f(Self { _marker: InvariantLifetime { _marker: PhantomData } })
    }
}

pub struct GhostCell<'id, T: ?Sized> {
    _marker: InvariantLifetime<'id>,
    data: UnsafeCell<T>,
}

unsafe impl<'id, T: ?Sized + Send> Send for GhostCell<'id, T> {}

unsafe impl<'id, T: ?Sized + Send + Sync> Sync for GhostCell<'id, T> {}

impl<'id, T> GhostCell<'id, T> {
    pub fn new(t: T) -> Self {
        Self {
            data: UnsafeCell::new(t),
            _marker: InvariantLifetime { _marker: PhantomData },
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    pub fn borrow<'s>(&'s self, _: &'s GhostToken<'id>) -> &'s T {
        unsafe { &*self.data.get() }
    }

    pub fn borrow_mut<'s>(&'s self, _: &'s mut GhostToken<'id>) -> &'s mut T {
        unsafe { &mut *self.data.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let value = GhostCell::new(0);
        let t = std::array::from_fn::<_, 3, _>(|_| &value);
        let _ = GhostToken::new(|mut token| {
            *t[0].borrow_mut(&mut token) += 1;
        });
        assert_eq!(value.into_inner(), 1);
    }
}
