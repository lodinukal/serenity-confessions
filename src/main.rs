use poise::serenity_prelude as serenity;
use shuttle_secrets::SecretStore;

mod commands;
mod router;
use router::build_router;
mod auth;
mod button;
mod database;
mod entity;
mod operations;
mod util;

pub struct Data {
    database: sea_orm::DatabaseConnection,
}
pub struct BotService {
    discord_bot: poise::FrameworkBuilder<
        Data,
        Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>,
    >,
    router: axum::Router,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let router = self.router;

        let serve_router = axum::Server::bind(&addr).serve(router.into_make_service());
        tokio::select!(
            _ = self.discord_bot.run() => {},
            _ = serve_router => {}
        );

        Ok(())
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> Result<BotService, shuttle_runtime::Error> {
    database::initialise(&secret_store);

    let should_use_test = secret_store.get("TEST").unwrap_or("false".into()) == "true";
    let token_index = if should_use_test {
        "TEST_DISCORD_TOKEN"
    } else {
        "DISCORD_TOKEN"
    };
    let discord_api_key = secret_store.get(token_index);
    if let None = discord_api_key {
        panic!("Error getting discord api key");
    }
    let discord_api_key = discord_api_key.unwrap();

    let discord_bot = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::commands(),
                commands::initialise(),
                commands::util::ping_vc(),
                //
                commands::channel::get_channels(),
                //
                commands::confessions::confess(),
                // TODO: Add autocomplete for this thing.
                // commands::confessions::confess_to(),
                commands::confessions::set_vetting(),
                commands::confessions::set_confessing(),
                commands::confessions::vote_reveal(),
                commands::confessions::shuffle(),
                commands::confessions::lock_shuffle(),
                //
                commands::guild::set_mod_role(),
                // subjects
                commands::subjects::add_subject(),
                commands::subjects::get_subjects(),
                commands::subjects::remove_subject(),
                commands::subjects::add_user_subjects(),
                commands::subjects::get_user_subjects(),
                commands::subjects::remove_user_subjects(),
                commands::subjects::get_users_with_subject()
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                edit_tracker: Some(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                )),
                case_insensitive_commands: true,
                ..Default::default()
            },
            event_handler: |ctx, ev, framework, data| {
                Box::pin(async move { commands::handle(ctx, ev, framework, data).await })
            },
            ..Default::default()
        })
        .token(discord_api_key)
        .intents(
            serenity::GatewayIntents::privileged().union(serenity::GatewayIntents::non_privileged()),
        )
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    database: database::connect().await.unwrap(),
                })
            })
        });

    let router = build_router();

    Ok(BotService {
        discord_bot,
        router,
    })
}
