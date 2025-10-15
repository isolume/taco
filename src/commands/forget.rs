use serenity::all::{CommandOptionType, CreateCommandOption};
use serenity::builder::CreateCommand;
use serenity::model::application::{ResolvedOption, ResolvedValue};
use crate::HISTORY;

pub async fn run(options: &[ResolvedOption<'_>]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::Channel(channel), ..
                }) = options.first()
    {
        HISTORY.lock().await.remove(&channel.id.get());
        format!("Forgetting channel: {}", channel.id)
    } else {
        HISTORY.lock().await.clear();
        "Forgetting everything".to_string()
    }
}

pub fn register() -> CreateCommand {
    
    CreateCommand::new("forget")
        .description("forget everything, or just a little bit")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Channel, "channel", "The channel to forget")
                .required(false)
        )
}