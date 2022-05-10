//! Sentry integration for the game engine Bevy
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_sentry::{release_name, ClientOptions, SentryConfig, SentryIntegration};
//! # use sentry::types::Dsn;
//! # use std::str::FromStr;
//! fn main() {
//!     let mut app = App::new();
//! # /*
//!     app.add_plugins(DefaultPlugins)
//! # */
//! #   app.add_plugins(MinimalPlugins)
//!         .insert_resource(SentryConfig::from_options((
//!             "https://examplePublicKey@o0.ingest.sentry.io/0",
//!             ClientOptions {
//!                 release: release_name!(),
//!                 ..Default::default()
//!             },
//!         ))
//!     );
//!     SentryIntegration::new()
//!         .build(&mut app);
//! #   app.set_runner(|mut app| app.schedule.run(&mut app.world));
//!     app.run();
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

/// Reexported sentry crate
pub use sentry::*;

use bevy::app::App;
use bevy::ecs::system::Resource;
use bevy::log::error;
use bevy::prelude::{Res, SystemSet};
use std::collections::BTreeMap;
use std::marker::PhantomData;

/// [Sentry.io](https://sentry.io) integration for Bevy applications
pub struct SentryIntegration {
    systems: SystemSet,
    initial_contexts: Vec<Box<dyn DynamicSentryContext>>,
}

impl SentryIntegration {
    /// Create a new sentry plugin instance
    pub fn new() -> Self {
        SentryIntegration {
            systems: SystemSet::new(),
            initial_contexts: vec![],
        }
    }

    /// Register a new Sentry context
    ///
    /// If you pass an initial value, it will be configures as soon as Sentry is initialized.
    /// You can later update the context by changing the resource `SentryContext<T>`
    pub fn register_context<T: Resource>(
        mut self,
        initial_value: Option<SentryContext<T>>,
    ) -> Self {
        self.systems = self.systems.with_system(set_sentry_context::<T>);
        if let Some(context) = initial_value {
            self.initial_contexts.push(Box::new(context));
        }

        self
    }

    /// Finish configuring the [`SentryIntegration`]
    ///
    /// Calling this function is required to set up the asset loading.
    pub fn build(self, app: &mut App) {
        if let Some(configuration) = app.world.remove_resource::<SentryConfig>() {
            app.insert_resource(Sentry {
                guard: init(configuration.options),
            });
            if !self.initial_contexts.is_empty() {
                configure_scope(|scope| {
                    for context in self.initial_contexts {
                        scope.set_context(
                            context.get_key(),
                            protocol::Context::Other(*context.get_context().clone()),
                        );
                    }
                });
            }
            app.add_system_set(self.systems);
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
