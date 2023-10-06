# thistermination

thistermination is a library crate inspired by [thiserror](https://crates.io/crates/thiserror) to add the [`std::process::Termination`](https://doc.rust-lang.org/std/process/trait.Termination.html) trait to error enums.

```toml
[dependencies]
thistermination = "1.0"
```

Compiler support: requires rustc 1.56+

## Why Implement The Termination Trait?

A struct or enum that implements the `std::process::Termination` and `std::fmt::Debug` traits can be returned by the `main` function, allowing developers to print a message on program exit and set the exit code.

## Usage

To add the `std::process::Termination` trait to an enum, you can use one of three possible derive macros:

- `#[derive(Termination)]`: is intended to be used in combination with thiserror, this macro implements the traits `std::process::Termination` and `std::fmt::Debug`. The `exit_code` defaults to `libc::EXIT_FAILURE`, and the Debug message is the same as the Display message unless explicitly set using `exit_code` and `msg`.

  ```rust
  use thistermination::{Termination};
  use thiserror::Error;
  
  #[derive(Error, Termination)]
  pub enum RequestError {
      #[error("request failed {0:?}")]
      RequestFailed(#[from] reqwest::Error),
      #[termination(msg("exiting wrong api key"))]
      #[error("wrong api key")]
      WrongAPIKey,
      #[termination(exit_code(3))]
      #[error("failed with status {0}")]
      RequestStatusError(u16),
      #[termination(exit_code(4), msg("exiting failed to load image {error:?}"))]
      #[error("failed to load image {error:?}")]
      ImageLoadError{#[from] error: image::ImageError},
  }
  
  fn main() -> Result<(), RequestError> {
      Err(RequestError::WrongAPIKey)
  }
  ```

- `#[derive(TerminationFull)]`: is intended to be used without thiserror, this macro implements the traits `std::process::Termination`, `std::fmt::Debug`, `std::fmt::Display`, and `std::error::Error`. The `exit_code` defaults to `libc::EXIT_FAILURE`, and `msg` is required and used for both Display and Debug.

  ```rust
  use thistermination::{TerminationFull};
  
  #[derive(TerminationFull)]
  pub enum RequestError {
      #[termination(exit_code(1), msg("request failed {0:?}"))]
      RequestFailed(#[from] reqwest::Error),
      #[termination(exit_code(2), msg("wrong api key"))]
      WrongAPIKey,
      #[termination(exit_code(3), msg("failed with status {0}"))]
      RequestStatusError(u16),
      #[termination(exit_code(4), msg("failed to load image {error:?}"))]
      ImageLoadError{#[from] error: image::ImageError},
  }
  
  fn main() -> Result<(), RequestError> {
      Err(RequestError::WrongAPIKey)
  }
  ```

- `#[derive(TerminationNoDebug)]`: is the most basic variant, implementing only the `std::process::Termination` trait. If no `exit_code` is provided, it defaults to `libc::EXIT_FAILURE`. However, the `std::fmt::Debug` trait is necessary for the enum to be returned by the `main` function and must be implemented manually or using the Debug macro.

  ```rust
  use thistermination::{TerminationNoDebug};
  
  #[derive(TerminationNoDebug, Debug)]
  pub enum RequestError {
      #[termination(exit_code(1))]
      RequestFailed(reqwest::Error),
      WrongAPIKey,
      #[termination(exit_code(3))]
      RequestStatusError(u16),
      ImageLoadError{error: image::ImageError},
  }
  
  fn main() -> Result<(), RequestError> {
      Err(RequestError::WrongAPIKey)
  }
  ```

## Details

- thistermination does not appear in your public API; the macros simply implement the the various traits.

- The macros can be derived for unit enums, enums with named fields, and enum tuples.

- `msg` supports accessing the fields of the enum in a format string manner

  - `#[termination(msg("{var}"))]`&ensp;⟶&ensp;`write!("{}", self.var)`
  - `#[termination(msg("{0}"))]`&ensp;⟶&ensp;`write!("{}", self.0)`
  - `#[termination(msg("{var:?}"))]`&ensp;⟶&ensp;`write!("{:?}", self.var)`
  - `#[termination(msg("{0:?}"))]`&ensp;⟶&ensp;`write!("{:?}", self.0)`

  You can also specify additional format string arguments for `msg`
  ```rust  
  #[derive(TerminationFull)]
  pub enum RequestError {
      #[termination(exit_code(4), msg("failed to load image {0:?}"))]
      ImageLoadError(#[from] image::ImageError),
  }
  ```

- Using `#[from]` will generate a `std::convert::From` implementation for the specific variant. A variant with `#[from]` is not allowed to contain any additional fields and can only be used in combination with `#[derive(TerminationFull)]`. 

  ```rust  
  #[derive(TerminationFull)]
  pub enum CLIError {
      #[termination(exit_code(4), msg("Invalid argument {0}, expected < {}", i16::MAX))]
      InvalidArgument(u16),
  }
  ```

## Comparison To thiserror

`#[derive(TerminationFull)]` can be used instead of thiserror as it offers many of the basic features of thiserror. However, it lacks some features like `#[source]`, `#[backtrace]`, the ability to automatically detect a backtrace, and `#[error(transparent)]`. If any of these features are required, you can use thiserror in combination with `#[derive(Termination)]`.