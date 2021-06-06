pub(crate) trait AsRawPtr<Target> {
    fn as_raw_ptr(&self) -> *const Target;
}

pub(crate) trait AsRawMutPtr<Target> {
    fn as_mut_raw_ptr(&mut self) -> *mut Target;
}

impl<T, A> AsRawMutPtr<A> for Option<T>
where
    T: AsRawMutPtr<A>,
{
    fn as_mut_raw_ptr(&mut self) -> *mut A {
        match self {
            Some(val) => val.as_mut_raw_ptr(),
            None => std::ptr::null_mut(),
        }
    }
}

impl<T, A> AsRawMutPtr<*mut A> for Vec<T>
where
    T: AsRawMutPtr<A>,
{
    fn as_mut_raw_ptr(&mut self) -> *mut *mut A {
        let mut collection: Vec<*mut A> =
            self.iter_mut().map(|item| item.as_mut_raw_ptr()).collect();

        collection.as_mut_ptr()
    }
}
