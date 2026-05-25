//! Tests for sequences of operations: loops, counters, iterator-style
//! indexing, and interactions between multiple crement calls.

use crement::{post_dec, post_inc, pre_dec, pre_inc};

// ── Counting loops ────────────────────────────────────────────────────────────

#[test]
fn postfix_inc_drives_loop_index() {
    let data = [10i32, 20, 30, 40, 50];
    let mut idx: usize = 0;
    let mut sum = 0i32;
    while idx < data.len() {
        sum += data[post_inc!(idx)];
    }
    assert_eq!(sum, 150);
    assert_eq!(idx, 5);
}

#[test]
fn prefix_inc_drives_loop_index() {
    let data = [0i32, 10, 20, 30, 40];
    let mut idx: usize = 0;
    let mut collected: Vec<i32> = Vec::new();
    while idx < data.len() - 1 {
        // ++idx advances first, so we skip index 0.
        collected.push(data[pre_inc!(idx)]);
    }
    assert_eq!(collected, [10, 20, 30, 40]);
}

#[test]
fn postfix_dec_countdown() {
    let mut count: i32 = 5;
    let mut log: Vec<i32> = Vec::new();
    while count > 0 {
        log.push(post_dec!(count));
    }
    assert_eq!(log, [5, 4, 3, 2, 1]);
    assert_eq!(count, 0);
}

#[test]
fn prefix_dec_countdown_off_by_one() {
    let mut count: i32 = 5;
    let mut log: Vec<i32> = Vec::new();
    // pre_dec logs the value AFTER decrement
    while count > 0 {
        let v = pre_dec!(count);
        log.push(v);
    }
    assert_eq!(log, [4, 3, 2, 1, 0]);
}

// ── Multiple variables in the same expression ────────────────────────────────

#[test]
fn two_postfix_incs_are_independent() {
    let mut a: i32 = 1;
    let mut b: i32 = 10;
    let old_a = post_inc!(a);
    let old_b = post_inc!(b);
    assert_eq!(old_a, 1);
    assert_eq!(old_b, 10);
    assert_eq!(a, 2);
    assert_eq!(b, 11);
}

#[test]
fn prefix_and_postfix_on_separate_vars() {
    let mut x: i32 = 5;
    let mut y: i32 = 5;
    let new_x = pre_inc!(x);  // 6
    let old_y = post_inc!(y); // 5, y becomes 6
    assert_eq!(new_x, 6);
    assert_eq!(old_y, 5);
    assert_eq!(x, 6);
    assert_eq!(y, 6);
}

// ── Interleaved increment and decrement ───────────────────────────────────────

#[test]
fn inc_then_dec_returns_to_original() {
    let mut x: i32 = 42;
    let _ = pre_inc!(x);
    let _ = pre_dec!(x);
    assert_eq!(x, 42);
}

#[test]
fn alternating_postfix_preserves_snapshot_independence() {
    let mut x: i32 = 0;
    let s0 = post_inc!(x); // 0
    let s1 = post_inc!(x); // 1
    let s2 = post_dec!(x); // 2
    let s3 = post_dec!(x); // 1
    assert_eq!([s0, s1, s2, s3], [0, 1, 2, 1]);
    assert_eq!(x, 0);
}

// ── Array fill patterns ───────────────────────────────────────────────────────

#[test]
fn fill_array_sequential_postfix() {
    let mut buf = [0i32; 8];
    let mut w: usize = 0;
    for val in 0..8 {
        buf[post_inc!(w)] = val;
    }
    assert_eq!(buf, [0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(w, 8);
}

#[test]
fn reverse_fill_with_predec() {
    let mut buf = [0i32; 5];
    let mut w: usize = buf.len();
    for val in 0..5i32 {
        buf[pre_dec!(w)] = val;
    }
    // Writing at indices 4,3,2,1,0 with values 0,1,2,3,4
    assert_eq!(buf, [4, 3, 2, 1, 0]);
    assert_eq!(w, 0);
}

// ── Interaction with conditional expressions ─────────────────────────────────

#[test]
fn postfix_inc_in_if_condition() {
    let mut x: i32 = 0;
    // Uses old value (0) in condition; x becomes 1 regardless.
    let branch = if post_inc!(x) == 0 { "zero" } else { "nonzero" };
    assert_eq!(branch, "zero");
    assert_eq!(x, 1);

    let branch2 = if post_inc!(x) == 0 { "zero" } else { "nonzero" };
    assert_eq!(branch2, "nonzero");
    assert_eq!(x, 2);
}

#[test]
fn prefix_inc_in_if_condition() {
    let mut x: i32 = 0;
    // Uses new value (1) in condition.
    let branch = if pre_inc!(x) == 1 { "one" } else { "other" };
    assert_eq!(branch, "one");
    assert_eq!(x, 1);
}

// ── match on returned value ───────────────────────────────────────────────────

#[test]
fn postfix_result_in_match() {
    let mut x: i32 = 2;
    let label = match post_dec!(x) {
        3 => "three",
        2 => "two",
        1 => "one",
        _ => "other",
    };
    assert_eq!(label, "two");
    assert_eq!(x, 1);
}

// ── Accumulator pattern ───────────────────────────────────────────────────────

#[test]
fn sum_via_postfix_inc_index() {
    let numbers = vec![3i32, 1, 4, 1, 5, 9, 2, 6];
    let mut idx: usize = 0;
    let mut total: i32 = 0;
    while idx < numbers.len() {
        total += numbers[post_inc!(idx)];
    }
    assert_eq!(total, numbers.iter().sum::<i32>());
}

// ── Nested postfix in function arguments ─────────────────────────────────────

fn takes_three(a: i32, b: i32, c: i32) -> (i32, i32, i32) {
    (a, b, c)
}

#[test]
fn postfix_as_function_arguments() {
    let mut x: i32 = 10;
    // Each post_inc evaluates left-to-right in Rust.
    let result = takes_three(post_inc!(x), post_inc!(x), post_inc!(x));
    assert_eq!(result, (10, 11, 12));
    assert_eq!(x, 13);
}
