use std::mem::transmute;


pub unsafe fn extend_lifetime<'a, 'b, T: ?Sized>(value: &'a T) -> &'b T {
    unsafe { transmute(value) }
}
