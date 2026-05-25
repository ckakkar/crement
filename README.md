# crement

[![crates.io](https://img.shields.io/crates/v/crement.svg)](https://crates.io/crates/crement)
[![docs.rs](https://docs.rs/crement/badge.svg)](https://docs.rs/crement)
[![CI](https://github.com/ckakkar/crement/actions/workflows/ci.yml/badge.svg)](https://github.com/ckakkar/crement/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![MSRV: 1.71](https://img.shields.io/badge/MSRV-1.71-orange.svg)](https://blog.rust-lang.org/2023/11/16/Rust-1.71.1.html)

Safe, zero-`unsafe` prefix and postfix `++` / `--` for Rust.

All four C-style variants — compile-time macro transformation, no linker
hacks, no runtime overhead beyond a single `+=`/`-=`.  The postfix forms
capture the old value using either a bit-copy (for `Copy` types) or a
`Clone::clone` call (for clone-only types), selected automatically by the
type system.

---

## Table of contents

1. [Why?](#why)
2. [Quick start](#quick-start)
3. [Installation](#installation)
4. [All four forms](#all-four-forms)
5. [Convenience aliases](#convenience-aliases)
6. [How Copy / Clone dispatch works](#how-copy--clone-dispatch-works)
7. [Generated code](#generated-code)
8. [Complex lvalue expressions](#complex-lvalue-expressions)
9. [Clone-only types](#clone-only-types)
10. [Supported types](#supported-types)
11. [Hygiene](#hygiene)
12. [Limitations](#limitations)
13. [Comparison with alternatives](#comparison-with-alternatives)
14. [Minimum supported Rust version](#minimum-supported-rust-version)
15. [Contributing](#contributing)
16. [License](#license)

---

## Why?

Port C code, write tight index loops, or simply prefer the classic `i++`
idiom — without reaching for `unsafe`, without a custom operator-overloading
crate, and without paying any runtime cost that a hand-written `+=` wouldn't
also pay.

```rust
// Instead of:
let old = i;
i += 1;
buf[old] = next_byte();

// Write:
buf[post_inc!(i)] = next_byte();
```

---

## Quick start

```rust
use crement::{crement, post_inc, pre_inc, post_dec, pre_dec};

let mut n: i32 = 5;

assert_eq!(crement!(n++), 5);  // postfix: returns OLD value, n is now 6
assert_eq!(crement!(++n), 7);  // prefix:  returns NEW value, n is now 7
assert_eq!(crement!(n--), 7);  // postfix: returns OLD value, n is now 6
assert_eq!(crement!(--n), 5);  // prefix:  returns NEW value, n is now 5
```

---

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
crement = "0.1"
```

No feature flags, no optional dependencies, no `build.rs`.

---

## All four forms

The primary macro is `crement!`, which accepts C-style `++`/`--` syntax:

| Invocation | Effect | Returns |
|---|---|---|
| `crement!(++x)` | `x += 1` | **new** value of `x` |
| `crement!(x++)` | `x += 1` | **old** value of `x` |
| `crement!(--x)` | `x -= 1` | **new** value of `x` |
| `crement!(x--)` | `x -= 1` | **old** value of `x` |

The `++` and `--` operators must be written **without spaces** — this is how the
macro distinguishes them from separate arithmetic operators (`+ +` is parsed by
Rust as two `+` tokens with `Alone` spacing; `++` is a `+` token with `Joint`
spacing followed by another `+`).

### Prefix increment/decrement

Mutates first, then yields the **new** (post-mutation) value:

```rust
use crement::crement;

let mut score: i32 = 9;
assert_eq!(crement!(++score), 10);
assert_eq!(score, 10);

let mut lives: i32 = 3;
assert_eq!(crement!(--lives), 2);
assert_eq!(lives, 2);
```

### Postfix increment/decrement

Captures the **old** (pre-mutation) value first, then mutates:

```rust
use crement::crement;

let mut idx: usize = 0;
let data = [10, 20, 30, 40];

// Classic C idiom: advance index, return the one that was used.
assert_eq!(data[crement!(idx++)], 10);  // reads data[0], idx is now 1
assert_eq!(data[crement!(idx++)], 20);  // reads data[1], idx is now 2
assert_eq!(idx, 2);
```

---

## Convenience aliases

Five individual macros mirror the `crement!` forms for readability:

```rust
use crement::{pre_inc, post_inc, pre_dec, post_dec};

let mut x: i32 = 0;

assert_eq!(pre_inc!(x),  1); // ++x  →  new value
assert_eq!(post_inc!(x), 1); // x++  →  old value (x is now 2)
assert_eq!(pre_dec!(x),  1); // --x  →  new value
assert_eq!(post_dec!(x), 1); // x--  →  old value (x is now 0)
```

---

## How Copy / Clone dispatch works

This is the technically interesting part of the crate.

### The problem

Postfix `x++` must return the value that `x` held **before** the increment.
In C that is a raw bit-copy.  In Rust the language has no implicit copying
unless a type is `Copy`.

### The approach

The macro always emits:

```rust
let __old = ::core::clone::Clone::clone(&x);
x += 1;
__old
```

This requires `Clone`, not `Copy`.  Every `Copy` type automatically implements
`Clone`, and for `Copy` types the compiler (LLVM / MIR optimiser) erases
`Clone::clone` entirely — it becomes an identical bit-copy or a register
move.  No virtual dispatch, no heap allocation, no extra instructions.

For types that are `Clone` but **not** `Copy` (e.g. `BigInt`, `Decimal`,
custom counters), the actual `Clone` implementation is called.  The returned
snapshot is an independent value; mutating the original after the call cannot
alias or affect the snapshot.

### Why not specialise on Copy?

Stable Rust does not have specialisation.  A single blanket `impl<T: Clone>`
covers both cases, and the optimiser handles the rest.  The generated code is
identical regardless — one path just gets optimised away.

### Type-system enforcement

If a type implements neither `Copy` nor `Clone`, the compiler emits a clear
diagnostic:

```
error[E0277]: the trait bound `Foo: Clone` is not satisfied
  --> src/main.rs:8:13
   |
 8 |     let _ = post_inc!(x);
   |             ^^^^^^^^^^^^ the trait `Clone` is not implemented for `Foo`
   |
help: consider annotating `Foo` with `#[derive(Clone)]`
```

No cryptic proc-macro errors — the requirement surfaces exactly where the
type fails to satisfy the bound.

---

## Generated code

Given `let mut n: i32 = 5;`:

```rust
// crement!(++n)  expands to:
{
    n += 1;
    let __crement_new = ::core::clone::Clone::clone(&n);
    __crement_new
}

// crement!(n++)  expands to:
{
    let __crement_old = ::core::clone::Clone::clone(&n);
    n += 1;
    __crement_old
}

// crement!(--n)  expands to:
{
    n -= 1;
    let __crement_new = ::core::clone::Clone::clone(&n);
    __crement_new
}

// crement!(n--)  expands to:
{
    let __crement_old = ::core::clone::Clone::clone(&n);
    n -= 1;
    __crement_old
}
```

Both local variable names (`__crement_old`, `__crement_new`) use
`Span::mixed_site()` hygiene, so they are invisible to surrounding code and
cannot shadow a user variable with the same name.

---

## Complex lvalue expressions

The macro strips the `++`/`--` tokens and parses the remainder as a
`syn::Expr`, so any valid mutable place expression works:

```rust
use crement::{crement, post_inc, pre_inc};

// Struct field access
struct Frame { pc: usize }
let mut f = Frame { pc: 0 };
let old_pc = crement!(f.pc++);

// Nested field
struct Outer { inner: Frame }
let mut o = Outer { inner: Frame { pc: 0 } };
pre_inc!(o.inner.pc);

// Array/slice indexing
let mut arr = [0i32; 4];
crement!(arr[2]++);

// Vec element
let mut v = vec![10i32, 20, 30];
let old = post_inc!(v[1]);

// Box<T>
let mut b: Box<i64> = Box::new(41);
assert_eq!(crement!(++(*b)), 42);
```

> **Note:** `++`/`--` must appear at the **outermost** level of the
> expression, not nested inside another `crement!` call.  For compound
> indexing like `arr[i++][j++]` use two separate statements.

---

## Clone-only types

Any type that implements `Clone` (but not `Copy`) and the appropriate
`AddAssign`/`SubAssign` trait works out of the box:

```rust
use crement::crement;
use std::ops::AddAssign;

#[derive(Clone, Debug, PartialEq)]
struct BigUint(u64);

impl AddAssign<u64> for BigUint {
    fn add_assign(&mut self, rhs: u64) { self.0 += rhs; }
}

let mut counter = BigUint(0);
let snap_a = crement!(counter++); // BigUint(0) — deep clone
let snap_b = crement!(counter++); // BigUint(1) — deep clone
assert_eq!(snap_a, BigUint(0));
assert_eq!(snap_b, BigUint(1));
assert_eq!(counter, BigUint(2));

// Mutating `counter` does not alias the snapshots.
counter.0 = 999;
assert_eq!(snap_a.0, 0);
assert_eq!(snap_b.0, 1);
```

---

## Supported types

| Type | `Copy` | Works with `crement!` |
|---|---|---|
| `i8` / `i16` / `i32` / `i64` / `i128` / `isize` | ✓ | ✓ |
| `u8` / `u16` / `u32` / `u64` / `u128` / `usize` | ✓ | ✓ |
| `f32` / `f64` | ✓ | ✗ (see below) |
| Any type implementing `AddAssign` + `Clone` | varies | ✓ |

### Floating-point note

`f32` and `f64` implement `AddAssign<f32>` / `SubAssign<f64>` (same-type RHS)
but **not** `AddAssign<{integer}>`.  Because the macro generates `+= 1` with
an unsuffixed integer literal — matching C semantics where `++` is defined for
integral types — floats are out of scope.  For floats, write `x += 1.0`
directly; the one-liner is just as clear.

---

## Hygiene

Both internal temporaries (`__crement_old`, `__crement_new`) are created with
`Span::mixed_site()`, which gives them definition-site hygiene.  A user variable
named `__crement_old` in the surrounding scope will **not** be shadowed:

```rust
let mut __crement_old: i32 = 99;
let result = crement!(__crement_old++); // captures old value correctly
assert_eq!(result, 99);
assert_eq!(__crement_old, 100);
```

---

## Limitations

### Floats

Described [above](#floating-point-note).

### Step size is always 1

The macro always adds or subtracts exactly `1`.  For a different step, use
`+=`/`-=` directly.

### Labels at the outermost position only

The `++`/`--` operator must appear at the start or end of the entire macro
input.  These work:

```rust
crement!(++x)
crement!(x.field--)
crement!(arr[2]++)
```

This does **not** work (the `++` is nested inside another expression):

```rust
// ERROR — use two calls instead:
// let i = post_inc!(outer);
// let j = post_inc!(inner);
crement!(arr[crement!(i++)]++)
```

### async fn / closures

The macros expand to block expressions and are fully compatible with regular,
`unsafe`, and generic functions.  Using them inside `async` blocks is
supported.  Using the lvalue itself in a closure is subject to normal Rust
capture rules; `crement!` imposes no extra restrictions.

### No const context

`Clone::clone` is not yet callable in `const fn` on stable Rust, so `const`
usage is unsupported.

---

## Comparison with alternatives

| Approach | Syntax | Copy / Clone | Complexity |
|---|---|---|---|
| Hand-written `let old = x; x += 1; old` | verbose | explicit | none |
| `crement!` (this crate) | `x++` / `++x` | automatic | zero-cost |
| `num-traits` + custom impl | `x.increment()` | manual impl | external dep |
| Nightly `std::ops::Step` | not stabilised | — | nightly only |

---

## Minimum supported Rust version

**1.71** (stable, released 2023-07-13), dictated by `syn 2.x`.

Bumps to MSRV are treated as breaking changes and will be accompanied by a
semver minor version bump.

---

## Contributing

Bug reports and pull requests are welcome on
[GitHub](https://github.com/ckakkar/crement).  Before submitting:

1. Run `cargo test` — all tests must pass.
2. Run `cargo clippy -- -D warnings` — no warnings allowed.
3. Run `cargo fmt --check` — formatting must match `rustfmt` defaults.
4. If adding a new feature, add at least one integration test in `tests/`.

See [`RELEASING.md`](RELEASING.md) for the publish checklist.

---

## License

MIT — see [`LICENSE`](LICENSE).
