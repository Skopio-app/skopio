#[cfg(all(
    not(feature = "desktop"),
    not(feature = "server"),
    not(any(test, doctest, clippy))
))]
compile_error!("You must enable either the 'desktop' or 'server' feature.");

pub mod connection;
mod utils;

pub use connection::DBContext;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "desktop")]
pub mod desktop;
