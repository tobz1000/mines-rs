// Mutably borrow two indices from a slice.
pub fn index_pair<T>(slice: &mut [T], ia: usize, ib: usize) -> (&mut T, &mut T) {
    if ia == ib || ia > slice.len() || ib > slice.len() {
        panic!(
            "Invalid index pair ({}, {}); slice.len() == {}",
            ia,
            ib,
            slice.len()
        );
    }

    let a;
    let b;

    unsafe {
        a = &mut *(slice.get_unchecked_mut(ia) as *mut _);
        b = &mut *(slice.get_unchecked_mut(ib) as *mut _);
    }

    (a, b)
}