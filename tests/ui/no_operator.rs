use crement::crement;

fn main() {
    let mut x: i32 = 5;
    // Missing ++ or -- — must produce a helpful compile error.
    let _ = crement!(x);
}
