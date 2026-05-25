use crement::post_inc;
use std::ops::AddAssign;

// Does NOT implement Clone.
struct NoClone(i32);

impl AddAssign<i32> for NoClone {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs;
    }
}

fn main() {
    let mut x = NoClone(5);
    // post_inc requires Clone to capture the old value.
    let _ = post_inc!(x);
}
