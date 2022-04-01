use bevy::prelude::*;
use bevy_sentry::{release_name, ClientOptions, SentryConfig, SentryContext, SentryIntegration};
use std::collections::BTreeMap;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(SentryConfig::from_options((
            include_str!("sentry_dsn.txt"),
            ClientOptions {
                release: release_name!(),
                ..Default::default()
            },
        )));
    SentryIntegration::new()
        .register_context(Some(SentryContext::<CharacterContext>::new("Character", {
            let mut context = BTreeMap::new();
            context.insert("name".to_owned(), "Nikl".into());
            context.insert("age".to_owned(), "38".into());

            context
        })))
        .build(&mut app);
    app.add_system(cause_panic).run();
}

struct CharacterContext;

fn cause_panic(_not_a_resource: Res<NotAResource>) {
    // This system causes a panic
}

struct NotAResource;
