//! Sentry integration for the game engine Bevy
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_sentry::{release_name, ClientOptions, SentryConfig, SentryPlugin};
//! # use sentry::types::Dsn;
//! # use std::str::FromStr;
//! fn main() {
//!     let mut app = App::new();
//! #    /*
//!     app.add_plugins(DefaultPlugins)
//! #    */
//! #    app.add_plugins(MinimalPlugins)
//!     app.insert_resource(SentryConfig::from_options((
//!             "https://examplePublicKey@o0.ingest.sentry.io/0",
//!             ClientOptions {
//!                 release: release_name!(),
//!                 ..Default::default()
//!             },
//!         )))
//!         .add_plugin(SentryPlugin)
//! #        .set_runner(|mut app| app.schedule.run(&mut app.world))
//!         .run();
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

/// Reexported sentry crate
pub use sentry::*;

use bevy::app::{App, Plugin};
use bevy::ecs::system::Resource;
use bevy::log::error;
use bevy::prelude::{Res, SystemSet};
use sentry::protocol::Value;
use std::collections::BTreeMap;
use std::marker::PhantomData;

/// [Sentry.io](https://sentry.io) integration for Bevy applications
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

/// App extension trait for sentry integration
pub trait SentryApp {
    fn register_context<T: Resource>(
        &mut self,
        initial_value: Option<SentryContext<T>>,
    ) -> &mut Self;
}

impl SentryApp for App {
    /// Register a new Sentry context
    ///
    /// If you pass an initial value, it will be configures as soon as Sentry is initialized.
    /// You can later update the context by changing the resource `SentryContext<T>`
    fn register_context<T: Resource>(
        &mut self,
        initial_value: Option<SentryContext<T>>,
    ) -> &mut Self {
        self.add_system(set_sentry_context::<T>);
        if let Some(context) = initial_value {
            configure_scope(|scope| {
                scope.set_context(
                    context.get_key(),
                    protocol::Context::Other(*context.get_context().clone()),
                );
            });
        }

        self
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

/// A sentry context
///
/// The string value mapping will be passed to Sentry as a custom context with every event.
/// See https://docs.sentry.io/product/sentry-basics/enrich-data/#types-of-data
pub struct SentryContext<T> {
    key: &'static str,
    context: Box<BTreeMap<String, Value>>,
    _phantom_data: PhantomData<T>,
}

impl<T> DynamicSentryContext for SentryContext<T> {
    fn get_key(&self) -> &str {
        self.key
    }

    fn get_context(&self) -> Box<BTreeMap<String, Value>> {
        self.context.clone()
    }
}

impl<T> SentryContext<T> {
    /// Create a new sentry context
    pub fn new(key: &'static str, context: BTreeMap<String, Value>) -> Self {
        SentryContext {
            key,
            context: Box::new(context),
            _phantom_data: PhantomData::default(),
        }
    }
}

trait DynamicSentryContext {
    fn get_key(&self) -> &str;

    fn get_context(&self) -> Box<BTreeMap<String, Value>>;
}

fn set_sentry_context<T: Resource>(context: Option<Res<SentryContext<T>>>) {
    if let Some(context) = context {
        if context.is_changed() {
            configure_scope(|scope| {
                scope.set_context(
                    context.key,
                    sentry::protocol::Context::Other(*context.context.clone()),
                );
            });
        }
    }
}
