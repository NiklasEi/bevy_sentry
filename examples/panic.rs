use bevy::prelude::*;
use bevy_sentry::{release_name, ClientOptions, SentryConfig, SentryPlugin};
use sentry::types::Dsn;
use std::str::FromStr;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SentryConfig::from_options((
            "https://examplePublicKey@o0.ingest.sentry.io/0",
            ClientOptions {
                release: release_name!(),
                ..Default::default()
            },
        )))
        .add_plugin(SentryPlugin)
        .add_system(cause_panic)
        .run();
}

fn cause_panic(_not_a_resource: Res<NotAResource>) {
    // This system causes a panic
}

struct NotAResource;
