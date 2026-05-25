use crement::crement;

fn main() {
    let mut x: i32 = 5;
    // `+ +` with a space is NOT the same as `++` — spacing check must reject.
    let _ = crement!(x + +);
}
