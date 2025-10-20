use crate::HISTORY;
use serenity::all::{CommandOptionType, CreateCommandOption};
use serenity::builder::CreateCommand;
use serenity::model::application::{ResolvedOption, ResolvedValue};

pub async fn run(options: &[ResolvedOption<'_>]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Channel(channel),
        ..
    }) = options.first()
    {
        HISTORY.lock().await.remove(&channel.id.get());
        format!("forgetting channel <#{}>", channel.id)
    } else {
        HISTORY.lock().await.clear();
        "forgetting everything".to_string()
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("forget")
        .description("forget everything, or just a little bit")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "channel",
                "the channel to forget",
            )
            .required(false),
        )
}