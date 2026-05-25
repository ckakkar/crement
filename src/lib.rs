//! # crement
//!
//! Safe, zero-`unsafe` prefix and postfix `++` / `--` for Rust.
//!
//! All four C-style variants are supported, compiled down to idiomatic
//! `+=`/`-=` with automatic Copy/Clone dispatch for the postfix
//! "return the old value" case.
//!
//! ## The interesting bit — Copy vs Clone
//!
//! Postfix `x++` must *capture the old value* before mutating `x`.  In C that
//! is a raw bit-copy; in Rust we need either a `Copy` bit-copy or a full
//! `Clone`.  Because every `Copy` type also implements `Clone` (and the
//! compiler erases the call entirely for `Copy` types), the macro always emits
//! `Clone::clone`, letting LLVM/MIR optimise it away for `Copy` types while
//! still working correctly for clone-only types such as `BigInt` or `Decimal`.
//!
//! ## Quick example
//!
//! ```rust
//! use crement::{crement, post_inc, pre_inc};
//!
//! let mut n: i32 = 5;
//!
//! // postfix — returns old value
//! assert_eq!(crement!(n++), 5);
//! assert_eq!(n, 6);
//!
//! // prefix — returns new value
//! assert_eq!(crement!(++n), 7);
//! assert_eq!(n, 7);
//!
//! // convenience aliases
//! assert_eq!(post_inc!(n), 7); // n goes 7 → 8, returns 7
//! assert_eq!(pre_inc!(n),  9); // n goes 8 → 9, returns 9
//! ```

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, Spacing, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{parse2, Error, Expr};

// ── Internal representation ───────────────────────────────────────────────────

enum Op {
    PreInc,
    PostInc,
    PreDec,
    PostDec,
}

struct Crement {
    op: Op,
    expr: Expr,
}

// ── Token-stream parser ───────────────────────────────────────────────────────

/// Returns `true` when the token is a `+` punctuation character.
fn is_plus(t: &TokenTree) -> bool {
    matches!(t, TokenTree::Punct(p) if p.as_char() == '+')
}

/// Returns `true` when the token is a `-` punctuation character.
fn is_minus(t: &TokenTree) -> bool {
    matches!(t, TokenTree::Punct(p) if p.as_char() == '-')
}

/// Returns `true` when the token is a punctuation character with `Joint`
/// spacing, i.e. it is immediately followed (no whitespace) by another
/// punctuation character in the source text.  This distinguishes `++` from
/// `+ +`.
fn is_joint(t: &TokenTree) -> bool {
    matches!(t, TokenTree::Punct(p) if p.spacing() == Spacing::Joint)
}

fn parse_crement_inner(input: TokenStream2) -> Result<Crement, Error> {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let n = tokens.len();

    if n == 0 {
        return Err(Error::new(
            Span::call_site(),
            "expected one of: `++expr`, `expr++`, `--expr`, `expr--`\n\
             hint: the macro requires an expression with a `++` or `--` operator",
        ));
    }

    // ── Prefix ++expr / --expr ────────────────────────────────────────────────
    if n >= 3 && is_joint(&tokens[0]) {
        if is_plus(&tokens[0]) && is_plus(&tokens[1]) {
            let rest: TokenStream2 = tokens[2..].iter().cloned().collect();
            return parse2::<Expr>(rest).map(|expr| Crement { op: Op::PreInc, expr });
        }
        if is_minus(&tokens[0]) && is_minus(&tokens[1]) {
            let rest: TokenStream2 = tokens[2..].iter().cloned().collect();
            return parse2::<Expr>(rest).map(|expr| Crement { op: Op::PreDec, expr });
        }
    }

    // ── Postfix expr++ / expr-- ───────────────────────────────────────────────
    if n >= 3 && is_joint(&tokens[n - 2]) {
        if is_plus(&tokens[n - 2]) && is_plus(&tokens[n - 1]) {
            let rest: TokenStream2 = tokens[..n - 2].iter().cloned().collect();
            return parse2::<Expr>(rest).map(|expr| Crement { op: Op::PostInc, expr });
        }
        if is_minus(&tokens[n - 2]) && is_minus(&tokens[n - 1]) {
            let rest: TokenStream2 = tokens[..n - 2].iter().cloned().collect();
            return parse2::<Expr>(rest).map(|expr| Crement { op: Op::PostDec, expr });
        }
    }

    Err(Error::new(
        Span::call_site(),
        "expected one of: `++expr`, `expr++`, `--expr`, `expr--`\n\
         hint: `++` and `--` must be written without spaces between the two characters",
    ))
}

// ── Code generator ────────────────────────────────────────────────────────────

fn generate(c: Crement) -> TokenStream2 {
    let expr = c.expr;
    // Both idents use `mixed_site` hygiene so they cannot shadow a user
    // variable that happens to share the same name.
    //
    // We always bind through a `let` before returning so that
    // `Clone::clone` (which is `#[must_use]`) is considered "used" the
    // moment it is assigned — the caller is then free to discard the block
    // value without a spurious must_use warning.
    let old = Ident::new("__crement_old", Span::mixed_site());
    let new = Ident::new("__crement_new", Span::mixed_site());

    match c.op {
        // { expr += 1; let __new = Clone::clone(&expr); __new }
        Op::PreInc => quote! {
            {
                #expr += 1;
                let #new = ::core::clone::Clone::clone(&#expr);
                #new
            }
        },
        // { let __old = Clone::clone(&expr); expr += 1; __old }
        Op::PostInc => quote! {
            {
                let #old = ::core::clone::Clone::clone(&#expr);
                #expr += 1;
                #old
            }
        },
        // { expr -= 1; let __new = Clone::clone(&expr); __new }
        Op::PreDec => quote! {
            {
                #expr -= 1;
                let #new = ::core::clone::Clone::clone(&#expr);
                #new
            }
        },
        // { let __old = Clone::clone(&expr); expr -= 1; __old }
        Op::PostDec => quote! {
            {
                let #old = ::core::clone::Clone::clone(&#expr);
                #expr -= 1;
                #old
            }
        },
    }
}

// ── Public proc macros ────────────────────────────────────────────────────────

/// Unified C-style prefix/postfix increment and decrement macro.
///
/// # Syntax
///
/// | Form | Mutates | Returns |
/// |------|---------|---------|
/// | `crement!(++x)` | `x += 1` | new value of `x` |
/// | `crement!(x++)` | `x += 1` | **old** value of `x` (requires `Clone`) |
/// | `crement!(--x)` | `x -= 1` | new value of `x` |
/// | `crement!(x--)` | `x -= 1` | **old** value of `x` (requires `Clone`) |
///
/// The expression `x` must be a mutable place expression and its type must
/// implement `AddAssign` (for `++`) or `SubAssign` (for `--`).  Postfix forms
/// additionally require `Clone`; for `Copy` types the compiler optimises the
/// clone away to a zero-cost bit-copy.
///
/// # Examples
///
/// ```rust
/// use crement::crement;
///
/// let mut n: i32 = 10;
///
/// assert_eq!(crement!(n++), 10); // returns old
/// assert_eq!(n, 11);
///
/// assert_eq!(crement!(++n), 12); // returns new
/// assert_eq!(n, 12);
///
/// assert_eq!(crement!(n--), 12); // returns old
/// assert_eq!(n, 11);
///
/// assert_eq!(crement!(--n), 10); // returns new
/// assert_eq!(n, 10);
/// ```
///
/// # Complex lvalue expressions
///
/// ```rust
/// use crement::crement;
///
/// struct Counter { value: i32 }
/// let mut c = Counter { value: 0 };
/// crement!(++c.value);
/// assert_eq!(c.value, 1);
///
/// let mut arr = [0i32; 4];
/// crement!(arr[2]++);
/// assert_eq!(arr[2], 1);
/// ```
///
/// # Clone-only types
///
/// ```rust
/// use crement::crement;
/// use std::ops::AddAssign;
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Counter(u64);
///
/// impl AddAssign<u64> for Counter {
///     fn add_assign(&mut self, rhs: u64) { self.0 += rhs; }
/// }
///
/// let mut n = Counter(5);
/// let old = crement!(n++);
/// assert_eq!(old, Counter(5));
/// assert_eq!(n,   Counter(6));
/// ```
#[proc_macro]
pub fn crement(input: TokenStream) -> TokenStream {
    let ts: TokenStream2 = input.into();
    match parse_crement_inner(ts) {
        Ok(c) => generate(c).into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Prefix increment: increments `expr` by one and returns the **new** value.
///
/// Equivalent to `crement!(++expr)`.
///
/// The type of `expr` must implement `AddAssign` and `Clone`.
/// For `Copy` types the clone is a zero-cost bit-copy.
///
/// # Example
///
/// ```rust
/// use crement::pre_inc;
///
/// let mut x: i32 = 0;
/// assert_eq!(pre_inc!(x), 1);
/// assert_eq!(x, 1);
/// ```
#[proc_macro]
pub fn pre_inc(input: TokenStream) -> TokenStream {
    let ts: TokenStream2 = input.into();
    match parse2::<Expr>(ts) {
        Ok(expr) => generate(Crement { op: Op::PreInc, expr }).into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Postfix increment: increments `expr` by one and returns the **old** value.
///
/// Equivalent to `crement!(expr++)`.
///
/// The type of `expr` must implement `AddAssign` and `Clone`.
/// For `Copy` types (all numeric primitives) the clone is a zero-cost bit-copy.
///
/// # Example
///
/// ```rust
/// use crement::post_inc;
///
/// let mut x: i32 = 0;
/// assert_eq!(post_inc!(x), 0); // old value
/// assert_eq!(x, 1);
/// ```
#[proc_macro]
pub fn post_inc(input: TokenStream) -> TokenStream {
    let ts: TokenStream2 = input.into();
    match parse2::<Expr>(ts) {
        Ok(expr) => generate(Crement { op: Op::PostInc, expr }).into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Prefix decrement: decrements `expr` by one and returns the **new** value.
///
/// Equivalent to `crement!(--expr)`.
///
/// The type of `expr` must implement `SubAssign` and `Clone`.
///
/// # Example
///
/// ```rust
/// use crement::pre_dec;
///
/// let mut x: i32 = 5;
/// assert_eq!(pre_dec!(x), 4);
/// assert_eq!(x, 4);
/// ```
#[proc_macro]
pub fn pre_dec(input: TokenStream) -> TokenStream {
    let ts: TokenStream2 = input.into();
    match parse2::<Expr>(ts) {
        Ok(expr) => generate(Crement { op: Op::PreDec, expr }).into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Postfix decrement: decrements `expr` by one and returns the **old** value.
///
/// Equivalent to `crement!(expr--)`.
///
/// The type of `expr` must implement `SubAssign` and `Clone`.
/// For `Copy` types the clone is a zero-cost bit-copy.
///
/// # Example
///
/// ```rust
/// use crement::post_dec;
///
/// let mut x: i32 = 5;
/// assert_eq!(post_dec!(x), 5); // old value
/// assert_eq!(x, 4);
/// ```
#[proc_macro]
pub fn post_dec(input: TokenStream) -> TokenStream {
    let ts: TokenStream2 = input.into();
    match parse2::<Expr>(ts) {
        Ok(expr) => generate(Crement { op: Op::PostDec, expr }).into(),
        Err(e) => e.into_compile_error().into(),
    }
}
