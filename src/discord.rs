use crate::minecraft::{ChatMessage, MessageType, chat_send_queue};
use crate::util::GetSetWrapper;
use lazy_static::lazy_static;
use serenity::all::{
    ChannelId, Colour, Context, CreateEmbed, CreateEmbedAuthor, CreateMessage, EventHandler, Http,
    Message, Ready,
};
use serenity::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    pub static ref channel_id_val: Mutex<GetSetWrapper<u64>> = Mutex::new(GetSetWrapper::new());
}

pub async fn send_minecraft_embed(http: &Arc<Http>, message: ChatMessage) {
    let mut embed = CreateEmbed::new().title(message.get_message_text());

    let mut embed_author = CreateEmbedAuthor::new(&message.get_player()).icon_url(&format!(
        "https://mc-api.io/render/face/{}/java",
        message.get_player()
    ));

    // base chat colour is discord blurple, all other colours keep the same vibrance with different hues
    // I may adjust saturation
    match message.get_message_type() {
        MessageType::JOIN => {
            embed = embed.colour(Colour::new(0x67f257));
            embed_author = embed_author.name("Server");
        }
        MessageType::LEAVE => {
            embed = embed.colour(Colour::new(0xf25a57));
            embed_author = embed_author.name("Server");
        }
        MessageType::DEATH => {
            embed = embed.colour(Colour::new(0x5f5f5f));
            embed_author = embed_author.name("Server");
        }
        MessageType::CHAT => {
            embed = embed.colour(Colour::new(0x5865f2));
        }
        MessageType::ADVANCEMENT => {
            embed = embed.colour(Colour::new(0xf2f257));
            embed_author = embed_author.name("Server");
        }
        MessageType::CHALLENGE => {
            embed = embed.colour(Colour::new(0xb257f2));
            embed_author = embed_author.name("Server");
        }
    }

    embed = embed.author(embed_author);

    // Create the message builder and add the embed
    let builder = CreateMessage::new().embed(embed);
    if let Err(why) = ChannelId::new(channel_id_val.lock().await.get())
        .send_message(&http, builder)
        .await
    {
        println!("Error sending message: {why:?}");
    }
}

pub struct MessageHandler;

#[async_trait]
impl EventHandler for MessageHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(".poop") {
            // Create the embed with your desired content
            let embed = CreateEmbed::new()
                .title("This is an embed")
                .description("With a description");

            let embed_author = CreateEmbedAuthor::new("Burrito_Bandit28").icon_url(format!(
                "https://mc-api.io/render/face/{}/java",
                "Burrito_Bandit28"
            ));

            let embed = embed.author(embed_author);

            // Create the message builder and add the embed
            let builder = CreateMessage::new().content("test").embed(embed);

            // Send the message
            msg.channel_id.send_message(&ctx.http, builder).await;
        } else {
            if !msg.author.bot {
                if let Ok(mut channel_id_guard) = channel_id_val.try_lock()
                    && msg.channel_id == channel_id_guard.get()
                {
                    let mut message_to_send = &msg.content;
                    let mut mc_chat_message = String::new();

                    if msg.attachments.len() > 0 {
                        mc_chat_message = format!(
                            "tellraw @a [\"\",{{\"text\":\"<\",\"color\":\"blue\"}},{{\"text\":\"{}\"}},{{\"text\":\">\",\"color\":\"blue\"}},{{\"text\":\" {}{}\"}},{{\"text\":\"[\",\"color\":\"blue\"}},{{\"text\":\"see Discord for attachments\",\"clickEvent\":{{\"action\":\"open_url\",\"value\":\"{}\"}}}},{{\"text\":\"]\",\"color\":\"blue\"}}]\n",
                            msg.author.name,
                            message_to_send,
                            if message_to_send.len() > 1 { " " } else { "" },
                            msg.link()
                        );

                        //message_to_send = format!("{}{}[see Discord for attachments]",message_to_send, if message_to_send.len()>1 {" "} else {""} )
                    } else {
                        mc_chat_message = format!(
                            "tellraw @a [\"\",{{\"text\":\"<\",\"color\":\"blue\"}},{{\"text\":\"{}\"}},{{\"text\":\">\",\"color\":\"blue\"}},{{\"text\":\" {}\"}}]\n",
                            msg.author.name, message_to_send
                        );
                    }
                    let mut queue = chat_send_queue.lock().await;
                    queue.push(mc_chat_message);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
