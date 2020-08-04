pub fn with_ptr<Output, ValueType, F: FnOnce(&Box<ValueType>) -> Output>(
    ptr: *mut ValueType,
    func: F,
) -> Output {
    let val = unsafe { Box::from_raw(ptr) };
    let result = func(&val);
    Box::into_raw(val);
    result
}
