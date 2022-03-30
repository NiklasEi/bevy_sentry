//! Sentry integration for the game engine Bevy
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_sentry::{release_name, ClientOptions, SentryConfig, SentryPlugin};
//! # use sentry::types::Dsn;
//! # use std::str::FromStr;
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .insert_resource(SentryConfig::from_options((
//!             "https://examplePublicKey@o0.ingest.sentry.io/0",
//!             ClientOptions {
//!                 release: release_name!(),
//!                 ..Default::default()
//!             },
//!         )))
//!         .add_plugin(SentryPlugin)
//!         .add_system(cause_panic)
//!         .run();
//! }
//!
//! fn cause_panic(_not_a_resource: Res<NotAResource>) {
//!     // This system causes a panic that will be
//!     // reported to the configured Sentry project
//! }
//!
//! struct NotAResource;
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use bevy::app::{App, Plugin};
use bevy::log::error;
pub use sentry::*;

/// Bevy [`Plugin`] configuring [Sentry.io](https://sentry.io)
pub struct SentryPlugin;

impl Plugin for SentryPlugin {
    fn build(&self, app: &mut App) {
        if let Some(configuration) = app.world.remove_resource::<SentryConfig>() {
            app.insert_resource(Sentry {
                guard: init(configuration.options),
            });
        } else {
            error!("Please supply a `SentryConfig` as resource");
        }
    }
}

/// Configuration resource for `bevy_sentry`
///
/// This resource is removed from the app during plugin initialization!
pub struct SentryConfig {
    options: ClientOptions,
}

impl SentryConfig {
    /// Build options for `bevy_sentry` based on Sentry's [`ClientOptions`](sentry::ClientOptions)
    pub fn from_options<C>(options: C) -> Self
    where
        C: Into<ClientOptions>,
    {
        SentryConfig {
            options: options.into(),
        }
    }
}

/// Runtime data for Sentry integration
pub struct Sentry {
    #[allow(dead_code)]
    guard: ClientInitGuard,
}
