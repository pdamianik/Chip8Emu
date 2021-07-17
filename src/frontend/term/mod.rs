#[cfg(windows)]
mod winterm;

#[cfg(windows)]
pub use winterm::*;

#[cfg(unix)]
mod unixterm;

#[cfg(unix)]
pub use unixterm::*;
