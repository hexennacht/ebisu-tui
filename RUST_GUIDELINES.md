# Rust Programming Best Practices & Guidelines

This document summarizes key best practices for writing idiomatic, performant, and maintainable Rust code. It is based on the "Rust Programming Best Practices Handbook".

> **Reference Materials:**
> - [Rust Official API Guidelines](https://rust-lang.github.io/api-guidelines/about.html)
> - [Rust Analyzer Style Guide](https://rust-analyzer.github.io/book/contributing/style.html)

---

## 1. Coding Styles and Idioms

### 1.1 Borrowing over Cloning

Rust's ownership system encourages **borrow** (`&T`) instead of **cloning** (`T.clone()`).

> ❗ Performance recommendation

#### ✅ Prefer Borrowing Alternatives
* `&[T]` instead of `Vec<T>` or `&Vec<T>`.
* `&str` instead of `String`.
* `&T` instead of `T`.

```rust
fn process(name: &str) {
    println!("Hello {name}");
}

let user = String::from("foo");
process(&user);
```

#### ✅ When to Clone
* You need to change the object AND preserve the original object (immutable snapshots).
* When you have `Arc` or `Rc` pointers.
* When data is shared across threads (usually `Arc`).
* When the underlying API expects owned data.
* When caching results:
  ```rust
  fn get_config(&self) -> Config {
      self.cached_config.clone()
  }
  ```

#### 🚨 Clone Traps to Avoid
* Auto-cloning inside loops `.map(|x| x.clone())` — prefer `.cloned()` or `.copied()` at the end of the iterator.
* Cloning large data structures like `Vec<T>` or `HashMap<K, V>`.
* Cloning because of bad API design instead of adjusting lifetimes.
* Cloning a reference argument — if you need ownership, make it explicit in the function signature:
  ```rust
  // ❌ BAD: The caller should have passed ownership instead
  fn take_a_borrow(thing: &Thing) {
      let thing_cloned = thing.clone();
  }
  ```

---

### 1.2 Pass by Value (Copy Trait)

Not all types should be passed by reference (`&T`). If a type is **small** and **cheap to copy**, it is often better to **pass it by value**.

#### ✅ When to Pass by Value
* The type **implements `Copy`** (`u32`, `bool`, `f32`, small structs).
* The type is small (up to ~24 bytes / 3 words).
* The cost of moving the value is negligible.

```rust
fn increment(x: u32) -> u32 {
    x + 1
}

let num = 1;
let new_num = increment(num); // `num` still usable after this point
```

#### ✅ When to Derive `Copy`
* All fields are `Copy` themselves.
* The struct is small (up to 24 bytes).
* The struct represents "plain data" without heap allocations.

```rust
#[derive(Debug, Copy, Clone)]
struct Point {
    x: f32,
    y: f32,
    z: f32
}
```

#### ✅ Enums Should Be `Copy` When
* They act like tags/atoms.
* All payloads are `Copy`.
* ⚠️ **Enum size is based on their largest variant.**

```rust
#[derive(Debug, Copy, Clone)]
enum Direction {
    North,
    South,
    East,
    West,
}
```

#### ❌ Do NOT Derive `Copy` For
* Types involving heap allocation (`String`, `Vec`).
* Large structs.

```rust
#[derive(Debug, Clone)]
struct BadIdea {
    age: i32,
    name: String, // String is not `Copy`
}
```

> ⚠️ **Rust arrays are stack allocated.** They can be copied if their underlying type is `Copy`, but large arrays can cause stack overflow.

#### Primitive Type Sizes Reference

| Type | Size |
|------|------|
| `i8`, `u8` | 1 byte |
| `i16`, `u16` | 2 bytes |
| `i32`, `u32`, `f32` | 4 bytes |
| `i64`, `u64`, `f64` | 8 bytes |
| `i128`, `u128` | 16 bytes |
| `isize`, `usize` | Architecture dependent |
| `bool` | 1 byte |
| `char` | 4 bytes |

---

### 1.3 Handling `Option<T>` and `Result<T, E>`

Rust 1.65 introduced `let Some(x) = … else { … }` and `let Ok(x) = … else { … }` patterns for safe unpacking with early returns.

#### ✅ When to Use Each Pattern

**Use `let PATTERN = EXPRESSION else { DIVERGING_CODE }`** when:
* The divergent code doesn't need to know about the failed pattern.
* You want to break, continue, or return early.

```rust
let Some(&Direction::North) = self.direction.as_ref() else {
    return Err(DirectionNotAvailable(self.direction));
};

for x in items {
    let Some(x) = x else {
        continue;
    };
}
```

**Use `if let PATTERN = EXPRESSION { ... } else { ... }`** when:
* The else branch needs extra computation.

```rust
if let Some(x) = self.next() {
    // computation with x
} else {
    // computation when None/Err
}
```

**Use `match`** when:
* You want to pattern match against inner types.
* Your type transforms into something more complex.
* Multiple error variants need different handling.

```rust
match self {
    Ok(Direction::South) => { … },
    Ok(Direction::North) => { … },
    Err(E::One) => { … },
    Err(E::Two) => { … },
}

// Transforming Result<T, E> to Result<Option<U>, E>
match self {
    Ok(t) => Ok(Some(t)),
    Err(E::Empty) => Ok(None),
    Err(err) => Err(err),
}
```

**Use `?`** to propagate errors when you don't care about the `Err` value.

#### Mapping Errors

Use `inspect_err` and `map_err` for logging and transforming errors:

```rust
x
    .inspect_err(|err| tracing::error!("function_name: {err}"))
    .map_err(|err| GeneralError::from(("function_name", err)))?;
```

#### ❌ Bad Patterns
* Conversion between `Result` and `Option` via match — prefer `.ok()`, `.ok_or()`, `.ok_or_else()`.
* Using `unwrap()` or `expect()` outside tests.

---

### 1.4 Prevent Early Allocation

When dealing with functions like `ok_or`, `map_or`, `unwrap_or`, consider using their lazy `_else` counterparts to prevent unnecessary allocations:

#### ✅ Good Cases

```rust
// Lazy - only allocates on error
x.ok_or_else(|| ParseError::ValueAbsent(format!("value {x}")))

// Lazy - only creates Vec on None
x.unwrap_or_else(Vec::new)

// Lazy - only formats on error
x.map_or_else(|e| format!("Error: {e}"), |v| v.len())
```

#### ❌ Bad Cases

```rust
// Allocates format string even on success
x.map_or(format!("Error with content"), |v| v.len())

// Creates Vec even when x has value
x.unwrap_or(Vec::new()) // Use unwrap_or_default() instead

// Allocates format string even when Some
x.ok_or(ParseError::ValueAbsent(format!("value {x}")))
```

---

### 1.5 Iterators vs For Loops

Both `for` loops and iterators are idiomatic Rust. Each shines in different contexts.

#### ✅ When to Prefer `for` Loops
* When you need **early exits** (`break`, `continue`, `return`).
* **Simple iteration** with side-effects (though `inspect` works for logging).
* When readability matters more than chaining.

```rust
for value in &mut values {
    if *value == 0 {
        break;
    }
    *value += fancy_equation();
}
```

#### ✅ When to Prefer Iterators (`.iter()`, `.into_iter()`)
* When **transforming collections** or `Option/Result`.
* You can **compose multiple steps** elegantly.
* No need for early exits.
* You need indexed values with `.enumerate()`.
* You need collection methods like `.windows()` or `.chunks()`.
* You need to combine data from multiple sources without allocating.

```rust
let values: Vec<_> = vec.into_iter()
    .enumerate()
    .filter(|(_index, value)| value % 2 == 0)
    .map(|(index, value)| value % index)
    .collect();
```

Iterators can be combined with `for` loops:
```rust
for value in vec.iter()
    .enumerate()
    .filter(|(index, value)| value % index == 0) 
{
    // ...
}
```

> #### ❗ REMEMBER: Iterators are Lazy
> * `.iter()`, `.map()`, `.filter()` don't do anything until consumed (`.collect()`, `.sum()`, `.for_each()`).
> * **Lazy Evaluation** means iterator chains are fused into one loop at compile time.

#### 🚨 Anti-patterns to AVOID
* Don't chain without formatting — each chained function on its own line.
* Don't chain if it makes code unreadable.
* Avoid needless `collect()` just to iterate again.
* Prefer `.iter()` over `.into_iter()` unless ownership transfer is required.
* Prefer `.iter()` over `.into_iter()` for `Copy` types.
* Prefer `.sum()` over `.fold()` for summing numbers.

---

### 1.6 Use Declarations (Imports)

Group imports in this order:
1. `std` (also `core`, `alloc`)
2. External crates (from `Cargo.toml` `[dependencies]`)
3. Workspace crates
4. `super::` imports
5. `crate::` imports

```rust
// std
use std::sync::Arc;

// external crates
use chrono::Utc;
use juniper::{FieldError, FieldResult};
use uuid::Uuid;

// workspace crates
use broker::database::PooledConnection;

// super:: / crate::
use super::schema::{Context, Payload};
use crate::models::Event;
```

#### Rustfmt Configuration

```toml
reorder_imports = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

> ⚠️ As of Rust 1.88, nightly is required for correct reordering: `cargo +nightly fmt`

---

## 2. Clippy and Linting Discipline

### 2.1 Why Care About Linting?

Rust compiler is powerful, but Clippy provides deeper analysis:
* Performance pitfalls
* Style issues
* Redundant code
* Potential bugs
* Non-idiomatic Rust

### 2.2 Always Run Clippy

Add this to your daily workflow:

```shell
cargo clippy --all-targets --all-features --locked -- -D warnings
```

| Flag | Purpose |
|------|---------|
| `--all-targets` | Checks library, tests, benches, examples |
| `--all-features` | Checks code with all features enabled |
| `--locked` | Requires up-to-date `Cargo.lock` |
| `-D warnings` | Treats warnings as errors |

Optional additions:
* `-- -W clippy::pedantic`: Stricter lints
* `-- -W clippy::nursery`: New lints under development

> ❗ Add this to your Makefile, Justfile, xtask, or CI pipeline.

### 2.3 Important Clippy Lints to Respect

| Lint Name | Why | Category |
|-----------|-----|----------|
| `redundant_clone` | Detects unnecessary clones, performance impact | perf |
| `needless_borrow` | Removes redundant `&` borrowing | style |
| `large_enum_variant` | Warns about large enum variants, suggests `Box` | perf |
| `map_unwrap_or` | Simplifies nested Option/Result handling | style |
| `manual_ok_or` | Suggest using `.ok_or_else` instead of `match` | style |
| `unnecessary_wraps` | Remove unnecessary Option/Result wrapping | pedantic |
| `clone_on_copy` | Catches `.clone()` on Copy types | complexity |
| `needless_collect` | Prevents unnecessary iterator collection | nursery |

### 2.4 Fix Warnings, Don't Silence Them!

**NEVER** just `#[allow(clippy::lint_something)]` unless:
* You **truly understand** why the warning happens.
* You have a documented reason why it's better that way.
* ❗ Use `#[expect(...)]` instead of `#[allow(...)]` — it warns when the lint is fixed!

```rust
// Faster matching is preferred over size efficiency
#[expect(clippy::large_enum_variant)]
enum Message {
    Code(u8),
    Content([u8; 1024]),
}
```

#### Handling False Positives
1. Try to refactor the code to improve the warning.
2. **Locally** override with `#[expect(clippy::lint_name)]` and a comment.
3. Avoid global overrides unless it's a core crate issue.

### 2.5 Configure Workspace/Package Lints

In `Cargo.toml`:

```toml
[lints.rust]
future-incompatible = "warn"
nonstandard_style = "deny"

[lints.clippy]
all = { level = "deny", priority = 10 }
redundant_clone = { level = "deny", priority = 9 }
manual_while_let_some = { level = "deny", priority = 4 }
pedantic = { level = "warn", priority = 3 }
```

For workspaces:

```toml
[workspace.lints.rust]
future-incompatible = "warn"
nonstandard_style = "deny"

[workspace.lints.clippy]
all = { level = "deny", priority = 10 }
pedantic = { level = "warn", priority = 3 }
```

---

## 3. Performance Mindset

The **golden rule** of performance work:

> **Don't guess, measure.**

Rust code is often already fast — don't "optimize" without evidence.

### 3.1 Quick Performance Tips
* Build with `--release` for meaningful performance tests.
* `cargo clippy -- -D clippy::perf` for performance hints.
* `cargo bench` for micro-benchmarks.
* `cargo flamegraph` for profiling.

### 3.2 Flamegraph

Visualize how much time CPU spent on each task:

```shell
# Install
cargo install flamegraph

# Profile (defaults to --release)
cargo flamegraph

# Profile specific binary
cargo flamegraph --bin=stress2

# Profile unit tests
cargo flamegraph --unit-test -- test::name

# Profile integration tests
cargo flamegraph --test test_name
```

> ❗ Always profile with `--release` — `--dev` isn't realistic.

**Reading Flamegraphs:**
* **Y-axis**: Stack depth (main at bottom, called functions stacked on top)
* **Width**: Total CPU time (wider = more time)
* **Color**: Random, not significant

**Remember:**
* Thick stacks = heavy CPU usage
* Thin stacks = low intensity (cheap)

### 3.3 Avoid Redundant Cloning

> Cloning is cheap... **until it isn't**

* 🚨 If you really need to clone, leave it to the last moment.

#### When to Pass Ownership
* Crate API requires owned data.
* You have reference-counted pointers (`Arc`, `Rc`).
* Small structs too big to `Copy` but cheap to clone.
* Ownership models business logic/state:
  ```rust
  let validated = Validate::try_from(not_validated)?;
  ```

#### When NOT to Pass Ownership
* Prefer APIs that take references (`fn process(values: &[T])`).
* If you only need read access, prefer `.iter()` or slices.
* For mutation, use `&mut T`.

#### Use `Cow` for Maybe-Owned Data

```rust
use std::borrow::Cow;

fn hello_greet(name: Cow<'_, str>) {
    println!("Hello {name}");
}

hello_greet(Cow::Borrowed("Julia"));
hello_greet(Cow::Owned("Naomi".to_string()));
```

### 3.4 Stack vs Heap: Be Size-Smart

#### ✅ Good Practices
* Keep small types (`impl Copy`, `usize`, `bool`) on the stack.
* Avoid passing huge types (>512 bytes) by value — use references.
* Heap allocate recursive data structures:
  ```rust
  enum OctreeNode<T> {
      Node(T),
      Children(Box<[Node<T>; 8]>),
  }
  ```
* Return small types by value.

#### ❗ Be Mindful
* Only use `#[inline]` when benchmarks prove beneficial.
* Avoid massive stack allocations — box them.
* For large allocations: `vec![0; size].into_boxed_slice()`
* Consider `smallvec` crate for large const arrays.

### 3.5 Iterators and Zero-Cost Abstractions

Rust iterators are lazy but compile into efficient tight loops.

* Prefer iterators over manual `for` loops for collections.
* Calling `.iter()` only creates a **reference** to the original collection.

#### ❗ Avoid Creating Intermediate Collections

```rust
// ❌ BAD - useless intermediate collection
let doubled: Vec<_> = items.iter().map(|x| x * 2).collect();
process(doubled);

// ✅ GOOD - pass the iterator
let doubled_iter = items.iter().map(|x| x * 2);
process(doubled_iter); // fn process(arg: impl Iterator<Item = u32>)
```

---

## 4. Error Handling

Rust enforces strict error handling, but *how* you handle them defines whether your code feels ergonomic or painful.

> Even if you decide to crash with `unwrap` or `expect`, Rust forces you to declare that intentionally.

### 4.1 Prefer `Result`, Avoid Panic

If your function can fail, return a `Result`:

```rust
fn divide(x: f64, y: f64) -> Result<f64, DivisionError> {
    if y == 0.0 {
        Err(DivisionError::DividedByZero)
    } else {
        Ok(x / y)
    }
}
```

Use `panic!` only in unrecoverable conditions — typically tests, assertions, or bugs.

#### Alternative Macros
* `todo!()`: Alerts compiler that code is missing.
* `unreachable!()`: You're sure this condition is impossible.
* `unimplemented!()`: Block is not yet implemented.

### 4.2 Avoid `unwrap`/`expect` in Production

Although `expect` is preferred (it has context), both should be avoided in production.

#### ✅ Use When
* In tests, assertions, or test helper functions.
* When failure is truly impossible.
* When smarter options can't handle the case.

#### 🚨 Alternative Patterns

**Early return with `let else`:**
```rust
let Ok(json) = serde_json::from_str(&input) else {
    return Err(MyError::InvalidJson);
};
```

**Error recovery with `if let`:**
```rust
if let Ok(json) = serde_json::from_str(&input) {
    // use json
} else {
    Err(do_something_with_input(&input))
}
```

**Default values:**
```rust
x.unwrap_or(default)
x.unwrap_or_else(|| compute_default())
x.unwrap_or_default()
```

### 4.3 `thiserror` for Library/Crate Errors

Use `thiserror` for precise, typed errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Network Timeout")]
    Timeout,
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid request. Header: {headers}, Metadata: {metadata}")]
    InvalidRequest {
        headers: Headers,
        metadata: Metadata
    }
}
```

#### Error Hierarchies
For layered systems, use nested errors with `#[from]`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Db(#[from] DbError),
    
    #[error("External services error: {0}")]
    ExternalServices(#[from] ExternalHttpError)
}
```

#### Custom Error Structs
When you only have one error type:

```rust
#[derive(Debug, thiserror::Error, PartialEq)]
#[error("Request failed with code `{code}`: {message}")]
struct HttpError {
    code: u16,
    message: String
}
```

### 4.4 Reserve `anyhow` for Binaries

`anyhow` is great for **binaries** where ergonomic error handling matters and specific types don't:

```rust
use anyhow::{Context, Result, anyhow};

fn main() -> Result<Config> {
    let content = std::fs::read_to_string("config.json")
        .context("Failed to read config file")?;
    Config::from_str(&content)
        .map_err(|err| anyhow!("Config parsing error: {err}"))
}
```

#### 🚨 Anyhow Gotchas
* Context strings are harder to maintain than `thiserror` messages.
* `anyhow::Result` erases context callers might need — avoid in libraries.
* Test helper functions can use `anyhow` safely.

### 4.5 Use `?` to Bubble Errors

Prefer `?` over verbose `match` chains:

```rust
fn handle_request(req: &Request) -> Result<ValidatedRequest, RequestValidationError> {
    validate_headers(req)?;
    validate_body_format(req)?;
    validate_credentials(req)?;
    let body = Body::try_from(req)?;
    Ok(ValidatedRequest::try_from((req, body))?)
}
```

> For error recovery, use `or_else`, `map_err`, `if let`. To inspect/log errors, use `inspect_err`.

### 4.6 Unit Tests Should Exercise Errors

Test your error messages:

```rust
#[test]
fn error_message_is_correct() {
    let err = divide(10., 0.0).unwrap_err();
    assert_eq!(err.to_string(), "division by zero");
}

#[test]
fn error_type_matches() {
    let err = process(my_value).unwrap_err();
    assert_eq!(err, MyError::InvalidInput { ... });
}
```

### 4.7 Async Errors

When using async runtimes (Tokio), ensure errors implement `Send + Sync + 'static`:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // ...
    Ok(())
}
```

> Avoid `Box<dyn std::error::Error>` in libraries unless truly needed.

---

## 5. Automated Testing

> Tests are not just for correctness. They are the first place people look to understand how your code works.

### 5.1 Tests as Living Documentation

Tests show how functions are meant to be used. If clear and targeted, they're often more helpful than reading the function body.

#### Use Descriptive Names

Names should describe: **unit_of_work** → **expected_behavior** → **state**

```rust
// Option 1: Full name
#[test]
fn process_should_return_blob_when_larger_than_b() { ... }

// Option 2: Use modules
mod process {
    #[test]
    fn should_return_blob_when_larger_than_b() { ... }
}
```

#### Use Modules for Organization

```rust
#[cfg(test)]
mod test {
    mod process {
        #[test]
        fn returns_error_xyz_when_b_is_negative() { ... }

        #[test]
        fn returns_invalid_input_error_when_a_and_b_not_present() { ... }
    }
}
```

#### Only Test One Behavior Per Function

```rust
// ❌ BAD: Testing multiple things
#[test]
fn test_thing_parser() {
    assert!(Thing::parse("abcd").is_ok());
    assert!(Thing::parse("ABCD").is_err());
}

// ✅ GOOD: One thing per test
#[test]
fn lowercase_letters_are_valid() {
    assert!(Thing::parse("abcd").is_ok());
}

#[test]
fn uppercase_letters_are_invalid() {
    assert!(Thing::parse("ABCD").is_err());
}
```

#### Use Few Assertions Per Test

Consider `rstest` for parameterized tests:

```rust
#[rstest]
#[case::single("a")]
#[case::first_letter("ab")]
#[case::last_letter("ba")]
fn the_function_accepts_strings_with_a(#[case] input: &str) {
    assert!(the_function(input).is_ok());
}
```

### 5.2 Add Test Examples to Your Docs

Doc tests run with `cargo test` and serve as both documentation and correctness checks:

```rust
/// Helper function that adds two numeric values.
/// 
/// # Examples
/// 
/// ```rust
/// # use crate_name::generic_add;
/// use num::numeric;
/// 
/// # assert_eq!(
/// generic_add(5.2, 4) // => 9.2
/// # , 9.2);
/// ```
```

> ⚠️ `cargo nextest` doesn't run doc tests — use `cargo t --doc` separately.

### 5.3 Unit Tests vs Integration Tests vs Doc Tests

#### Unit Tests
* Go in the **same module** as the tested unit.
* Can access private functions.
* Test implementation and edge cases.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_state_behavior() {
        let expected = ...;
        let result = ...;
        assert_eq!(result, expected);
    }
}
```

#### Integration Tests
* Go in the `tests/` directory.
* Only test the **public API**.
* Test that parts work together correctly.

```
├── src/lib.rs
└── tests/
    ├── common/mod.rs
    └── integration_test.rs
```

#### Doc Tests
* In `///` documentation comments.
* Test happy paths and public API usage.

**Doc Test Attributes:**
* `ignore`: Skip the code.
* `should_panic`: Block will panic.
* `no_run`: Compile but don't execute.
* `compile_fail`: Test that compilation fails.

### 5.4 How to `assert!`

```rust
// Boolean assertions
assert!(value.is_ok(), "'value' is not Ok: {value:?}");

// Equality assertions
assert_eq!(result, expected, "'result' differs: {}", result.diff(expected));

// Pattern matching
assert!(matches!(error, MyError::BadInput(_)), "Expected BadInput, found {error}");
```

**Test Attributes:**
* `#[ignore = "message"]`: Skip test.
* `#[should_panic]`: Test expects panic.

**Useful Crates:**
* `rstest`: Fixture-based test framework.
* `pretty_assertions`: Colorful diffs.

### 5.5 Snapshot Testing with `cargo insta`

> When correctness is visual or structural, snapshots tell the story better than asserts.

```toml
insta = { version = "1.42.2", features = ["yaml"] }
```

```rust
#[test]
fn test_split_words() {
    let words = split_words("hello from the other side");
    insta::assert_yaml_snapshot!(words);
}
```

```shell
cargo insta test      # Execute
cargo insta review    # Review conflicts
```

#### ✅ What to Snapshot
* Generated code
* Serialized complex data
* Rendered HTML
* CLI output

#### ❌ What NOT to Snapshot
* Simple primitives (use `assert_eq!`)
* Critical path logic (use precise unit tests)
* Flaky/random output (unless redacted)
* External resources (use mocks)

#### ✅ Snapshot Best Practices

```rust
// Use named snapshots
assert_snapshot!("this_is_a_named_snapshot", output);

// Keep snapshots small
assert_snapshot!("app_config/http", whole_app_config.http); // ✅
assert_snapshot!("app_config", whole_app_config);           // ❌ Huge

// Use redactions for unstable fields
assert_json_snapshot!(
    "endpoints/get_user",
    data,
    ".created_at" => "[timestamp]",
    ".id" => "[uuid]"
);
```

* Commit snapshots to git.
* Review changes carefully before accepting.

---

## 6. Generics, Dynamic Dispatch, and Static Dispatch

> Static where you can, dynamic where you must.

### 6.1 Generics

Generics are abstract stand-ins for concrete types. Rust performs **monomorphization** at compile time — generic code becomes specific code with zero runtime cost.

### 6.2 Static Dispatch: `impl Trait` or `<T: Trait>`

#### ✅ Best When
* You want **zero runtime cost**.
* You need **tight loops or performance**.
* Types are **known at compile time**.
* You're working with **single-use implementations**.

```rust
fn specialized_sum<U: Sum + RandomMapping>(iter: impl Iterator<Item = U>) -> U {
    iter.map(|x| x.random_mapping()).sum()
}
```

### 6.3 Dynamic Dispatch: `dyn Trait`

Usually used with pointers: `Box<dyn Trait>`, `Arc<dyn Trait>`, `&dyn Trait`.

#### ✅ Best When
* You need **runtime polymorphism**.
* You need to **store different implementations** in one collection.
* You want to **abstract internals behind a stable interface**.
* You're writing **plugin-style architecture**.

```rust
trait Animal {
    fn greet(&self) -> String;
}

fn all_animals_greeting(animals: Vec<Box<dyn Animal>>) {
    for animal in animals {
        println!("{}", animal.greet());
    }
}
```

### 6.4 Trade-off Summary

|                   | Static Dispatch | Dynamic Dispatch |
|-------------------|-----------------|------------------|
| Performance       | ✅ Faster, inlined | ❌ Slower (vtable) |
| Compile time      | ❌ Slower (monomorphization) | ✅ Faster |
| Binary size       | ❌ Larger | ✅ Smaller |
| Flexibility       | ❌ One type at a time | ✅ Mix types |
| Errors            | ✅ Clearer | ❌ Type erasure confuses |

> Favor static dispatch until your trait needs to live behind a pointer.

### 6.5 Best Practices for Dynamic Dispatch

#### ✅ Use When
* Heterogeneous types in a collection.
* Runtime plugins or hot-swappable components.
* Abstracting internals from callers (library design).

#### ❌ Avoid When
* You control the concrete types.
* Performance-critical paths.
* Can express logic with generics while keeping simplicity.

### 6.6 Trait Objects Ergonomics

* Prefer `&dyn Trait` over `Box<dyn Trait>` when ownership not needed.
* Use `Arc<dyn Trait + Send + Sync>` for shared multi-threaded access.
* Don't use `dyn Trait` if the trait has methods returning `Self`.
* **Don't box too early** — use generics when possible, box at boundaries.

```rust
// ✅ Use generics when possible
struct Renderer<B: RenderBackend> {
    backend: B
}

// ❌ Premature Boxing
struct Renderer {
    backend: Box<dyn RenderBackend>
}
```

#### Object Safety Requirements
* No generic methods.
* No `Self: Sized`.
* Methods use `&self`, `&mut self`, or `self`.

```rust
// ✅ Object Safe
trait Runnable {
    fn run(&self);
}

// ❌ Not Object Safe
trait Factory {
    fn create<T>() -> T; // Generic methods not allowed
}
```

---

## 7. Type State Pattern

Encode state in the type system to prevent runtime errors.

> Invalid states become compile errors instead of runtime bugs.

### 7.1 What is Type State Pattern?

A design pattern where you encode different **states** of the system as **types**, not runtime flags or enums. The compiler enforces state transitions and prevents illegal actions at compile time.

### 7.2 Why Use It?

* Avoids runtime checks for state validity.
* Models state transitions as type transitions.
* Prevents data misuse (e.g., using uninitialized objects).
* Improves API safety and correctness.
* `PhantomData` is removed after compilation — no extra memory.

### 7.3 Simple Example: File State

```rust
use std::{io, path::{Path, PathBuf}};

struct FileNotOpened;
struct FileOpened;

#[derive(Debug)]
struct File<State> {
    path: PathBuf,
    handle: Option<std::fs::File>,
    _state: std::marker::PhantomData<State>
}

impl File<FileNotOpened> {
    fn open(path: &Path) -> io::Result<File<FileOpened>> {
        let file = std::fs::File::open(path)?;
        Ok(File {
            path: path.to_path_buf(),
            handle: Some(file),
            _state: std::marker::PhantomData::<FileOpened>
        })
    }
}

impl File<FileOpened> {
    fn read(&mut self) -> io::Result<String> {
        use io::Read;
        let mut content = String::new();
        self.handle.as_mut().unwrap().read_to_string(&mut content)?;
        Ok(content)
    }
}
```

### 7.4 Real-World Examples

#### Builder Pattern with Compile-Time Guarantees

Forces users to set required fields before calling `.build()`.

```rust
struct MissingName;
struct NameSet;
struct MissingAge;
struct AgeSet;

struct Builder<HasName, HasAge> {
    name: Option<String>,
    age: u8,
    _name: PhantomData<HasName>,
    _age: PhantomData<HasAge>,
}

impl Builder<MissingName, MissingAge> {
    fn new() -> Self { ... }
    fn name(self, name: String) -> Builder<NameSet, MissingAge> { ... }
    fn age(self, age: u8) -> Builder<MissingName, AgeSet> { ... }
}

impl Builder<NameSet, AgeSet> {
    fn build(self) -> Person { ... } // Only available when both set
}
```

```rust
// ✅ Valid
let person = Builder::new().name("Alice".into()).age(30).build();

// ❌ Compile error: Name required
let person = Builder::new().age(30).build();
```

### 7.5 Pros and Cons

#### ✅ Use When
* You want **compile-time state safety**.
* You need to enforce **API constraints**.
* Writing a library heavy on variants.
* Replacing runtime booleans/enums with type-safe paths.

#### ❌ Avoid When
* Writing trivial states (simple enums suffice).
* Don't need type-safety.
* Leads to overcomplicated generics.
* Runtime flexibility is required.

#### 🚨 Downsides
* More **verbose solutions**.
* **Complex type signatures**.
* May require **unsafe** for variant outputs.
* May require field duplication.
* `PhantomData` isn't intuitive for beginners.

> Use this pattern when it **saves bugs, increases safety, or simplifies logic** — not for cleverness.

---

## 8. Comments vs Documentation

> Clear code beats clear comments. However, when the why isn't obvious, say it plainly.

### 8.1 Comments vs Documentation: Know the Difference

| Purpose      | Use `// comment`              | Use `///` doc                        |
|--------------|-------------------------------|--------------------------------------|
| Describe Why | ✅ Tricky reasoning           | ❌ Not for docs                      |
| Describe API | ❌ Not useful                 | ✅ Public interfaces, usage, errors  |
| Maintainable | 🚨 Often becomes obsolete     | ✅ Tied to code, in generated docs   |
| Visibility   | Local development only        | Exported to users via `cargo doc`    |

### 8.2 When to Use Comments

Use `//` when something can't be expressed clearly in code:

* **Safety guarantees:** `// SAFETY: ptr is guaranteed non-null by caller`
* Workarounds or **optimizations**.
* **Platform-specific** behaviors.
* Links to **Design Docs** or **ADRs**.
* Assumptions or **gotchas** that aren't obvious.

> **Name your comments!** E.g., `// SAFETY: ...`, `// PERF: ...`, `// CONTEXT: ...`

```rust
// SAFETY: We have checked that the pointer is valid and non-null.
unsafe { std::ptr::copy_nonoverlapping(src, dst, len); }

// PERF: See ADR-123 for TLS startup latency on MacOS
let root_store = configuration.create_certificate_store()?;
```

### 8.3 When Comments Get in the Way

Avoid comments that:
* Restate obvious things (`// increment i by 1`)
* Can grow stale over time
* Are `TODO`s without linked issues
* Could be replaced by better naming or smaller functions

```rust
// ❌ BAD
fn compute(counter: &mut usize) {
    // increment by 1
    *counter += 1;
}
```

### 8.4 Don't Write Living Documentation

Comments as "living documentation" is a **dangerous myth**:
* They **rot** — nobody compiles comments.
* They **mislead** — readers assume they're true.
* They **go stale** — unless maintained with code.
* They're **noisy** — clutter your code.

If something deserves to live beyond a PR, put it in:
* An **ADR** (Architecture Decision Record)
* A Design Document
* **Doc comments** (`///`) with examples
* Tests that explain the behavior

> 🚨 If you find a comment, **read it in context**. If it's wrong or outdated, fix or remove it.

### 8.5 Replace Comments with Code

**Instead of:**
```rust
fn save_user(&self) -> Result<(), MyError> {
    // check if user is authenticated
    // serialize user data
    // write to file
    ...
}
```

**Extract for clarity:**
```rust
fn save_auth_user(&self) -> Result<PathBuf, MyError> {
    if self.is_authenticated() {
        let serialized = serde_json::to_string(self)?;
        std::fs::write(self.path(), serialized)?;
        Ok(self.path())
    } else {
        Err(MyError::UserNotAuthenticated)
    }
}
```

### 8.6 TODO Should Become Issues

Don't leave `// TODO:` scattered with no owner:

1. File a GitHub Issue or Jira Ticket.
2. Reference the issue in the code:

```rust
// TODO(#42): Remove workaround after bugfix
```

### 8.7 When to Use Doc Comments

Use `///` to document:
* All **public functions, structs, traits, enums**.
* Purpose, usage, and behaviors.
* `# Errors`, `# Panics`, `# Safety` sections.
* Examples.

```rust
/// Loads [`User`] profile from disk.
/// 
/// # Errors
/// - Returns [`MyError::FileNotFound`] if the file is missing.
/// - Returns [`MyError::InvalidJson`] if content is invalid.
fn load_user(path: &Path) -> Result<User, MyError> { ... }
```

**With examples:**
```rust
/// Returns the square of the integer part.
/// 
/// # Examples
/// 
/// ```rust
/// assert_eq!(square(4.3), 16);
/// ```
fn square(x: impl ToInt) -> u128 { ... }
```

### 8.8 `///` vs `//!`

| Style | Used for | Scope |
|-------|----------|-------|
| `///` | Item documentation | Functions, structs, enums |
| `//!` | Module/crate documentation | Top of `lib.rs` or `mod.rs` |

```rust
//! This module implements a custom chess engine.
//! 
//! # Example
//! ```
//! let board = chess::Board::default();
//! assert!(board.is_valid());
//! ```
```

### 8.9 Documentation Lints

| Lint | Description |
|------|-------------|
| `missing_docs` | Warns about undocumented public items |
| `broken_intra_doc_links` | Detects broken internal doc links |
| `empty_docs` | Disallow empty docs |
| `missing_panics_doc` | Warn if `# Panics` section missing |
| `missing_errors_doc` | Warn if `# Errors` section missing |
| `missing_safety_doc` | Warn if `# Safety` section missing |

### 8.10 Documentation Checklist

**📦 Crate-Level (`lib.rs`):**
- [ ] `//!` doc explains what the crate does and what problems it solves.
- [ ] Includes crate-level examples or pointers to modules.

**📁 Modules (`mod.rs`):**
- [ ] `//!` doc explains what the module is for and its exports.

**🧱 Structs, Enums, Traits:**
- [ ] `///` doc explains the role, invariants, and example usage.

**🔧 Functions and Methods:**
- [ ] Doc covers what it does, parameters, return value, edge cases.
- [ ] Usage example in `# Examples`.

**📑 Traits:**
- [ ] Explain the purpose and when/why to implement it.

---

## 9. Understanding Pointers

### 9.1 Thread Safety

Rust tracks pointers using `Send` and `Sync` traits:
* **`Send`**: Data can move across threads.
* **`Sync`**: Data can be referenced from multiple threads.

> A pointer is thread-safe only if the data behind it is.

### 9.2 Pointer Types Reference

| Type | Description | Send + Sync? | Use Case |
|------|-------------|--------------|----------|
| `&T` | Shared Reference | Yes | Read-only, multiple readers |
| `&mut T` | Mutable Reference | No (not Send) | Exclusive write access |
| `Box<T>` | Heap Allocation | If T: Send+Sync | Recursive types, large data, ownership transfer |
| `Rc<T>` | Ref Counted | Neither | Shared ownership (single-thread) |
| `Arc<T>` | Atomic Ref Counted | Yes | Shared ownership (multi-thread) |
| `Cell<T>` | Interior Mutability (Copy) | No (not Sync) | Copy types mutation, no panic |
| `RefCell<T>` | Interior Mutability | No (not Sync) | Runtime borrow check, may panic |
| `Mutex<T>` | Thread-safe Mutex | Yes | Shared mutable (multi-thread) |
| `RwLock<T>` | Read-Write Lock | Yes | Many readers OR one writer |
| `OnceCell<T>` | One-time init | No (not Sync) | Lazy single-thread init |
| `OnceLock<T>` | Thread-safe OnceCell | Yes | Static values |
| `LazyCell<T>` | Lazy OnceCell | No (not Sync) | Complex lazy init |
| `LazyLock<T>` | Thread-safe LazyCell | Yes | Complex static init |
| `*const T/*mut T` | Raw Pointers | Manual | FFI, unsafe code |

### 9.3 When to Use Each Pointer

#### `&T` - Shared Borrow
Most common. Safe, no mutation, multiple readers.

```rust
fn print_len(s: &str) {
    println!("{}", s.len());
}
```

#### `&mut T` - Exclusive Borrow
Safe, but only one mutable borrow at a time.

```rust
fn mark_update(s: &mut String) {
    s.push_str("_updated");
}
```

#### `Box<T>` - Heap Allocated
Single-owner heap-allocated data. Great for recursive types.

```rust
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}
```

#### `Rc<T>` - Reference Counter (Single-Thread)
Multiple references to data in a single thread.

#### `Arc<T>` - Atomic Reference Counter (Multi-Thread)
Multiple references across threads. Often wrapped around `Mutex`:

```rust
let shared = Arc::new(Mutex::new(data));
```

#### `RefCell<T>` - Runtime Checked Interior Mutability
Shared access with runtime borrow checking. **May panic!**

```rust
let x = RefCell::new(42);
*x.borrow_mut() += 1;
```

#### `Cell<T>` - Copy-Only Interior Mutability
Faster, safer version of `RefCell` for `Copy` types. No panic.

```rust
let cell = Cell::new(1);
cell.set(100);
```

#### `Mutex<T>` - Thread-Safe Mutability
Exclusive access across threads. Usually wrapped in `Arc`.

#### `RwLock<T>` - Read-Write Lock
Multiple readers OR single writer. Usually wrapped in `Arc`.

#### `OnceCell<T>` / `OnceLock<T>` - One-Time Initialization

```rust
static CONFIG: OnceLock<Config> = OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| load_config())
}
```

#### `LazyLock<T>` - Thread-Safe Lazy Initialization

```rust
static CONFIG: LazyLock<HashMap<String, Value>> = LazyLock::new(|| {
    let mut config = HashMap::new();
    // ... complex initialization
    config
});
```

#### `*const T/*mut T` - Raw Pointers
Inherently **unsafe**. Necessary for FFI.

```rust
let x = 5;
let ptr = &x as *const i32;
unsafe {
    println!("Value: {}", *ptr);
}
```

---

## 10. General Advice

*   **Make illegal states unrepresentable.**
*   **Clippy is your friend.**
*   **Rust Analyzer is your friend.**
*   **Measure before optimizing.**
*   **The compiler is your best friend** — it catches mistakes early.
*   **Keep learning** — every Rust version brings improvements.

> Rust doesn't prevent mistakes; it makes it easier to catch them early.

### External Resources

*   [Rust Blog](https://blog.rust-lang.org/) — Version updates and announcements
*   [Rust Official API Guidelines](https://rust-lang.github.io/api-guidelines/about.html)
*   [Rust Analyzer Style Guide](https://rust-analyzer.github.io/book/contributing/style.html)
*   [Idiomatic Rust](https://github.com/mre/idiomatic-rust)

### References

*   [Mara Bos - Rust Atomics and Locks](https://marabos.nl/atomics/)
*   [Semicolon Video on Pointers](https://www.youtube.com/watch?v=Ag_6Q44PBNs)
