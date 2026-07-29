mod log_parser;

use crate::log_parser::{LogParser, INSTANCE};
use dotenv::dotenv;
use lazy_static::lazy_static;
use serenity::all::{
    ChannelId, Context, CreateEmbed, CreateEmbedAuthor, CreateMessage, EventHandler,
    GatewayIntents, Http, Message, Ready,
};
use tokio::process::{ChildStdout, Command};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use serenity::{async_trait, Client};
use std::io::{stdout, BufRead, BufReader, Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::process::Stdio;
use tokio::sync::Mutex;
use std::sync::{Arc};
use std::{env, thread};
use std::time::Duration;
use tokio::{join};

lazy_static! {
    pub static ref chat_send_queue: Mutex<Vec<String>> = Mutex::new(vec![]);
    pub static ref channel_id_val: Mutex<ChannelIdValue> = Mutex::new(ChannelIdValue { val: 0 });
}

pub struct ChannelIdValue {
    val: u64,
}

impl ChannelIdValue {
    pub fn set(&mut self, val: u64) {
        self.val = val;
    }
    pub fn get(&self) -> u64 {
        self.val
    }
}

#[derive(Clone, Debug)]
enum MessageType {
    JOIN,
    LEAVE,
    DEATH,
    CHAT,
    ADVANCEMENT,
    CHALLENGE,
}

#[derive(Debug)]
struct ChatMessage {
    message_type: MessageType,
    message_text: String,
    player_name: String,
}
impl ChatMessage {
    pub fn get_player(&self) -> String {
        self.player_name.clone()
    }
    pub fn get_message_text(&self) -> String {
        self.message_text.clone()
    }
    pub fn get_message_type(&self) -> MessageType {
        self.message_type.clone()
    }
}

async fn handle_console_input() {
    let stdin = io::stdin();
    let mut reader = TokioBufReader::new(stdin);

    loop {
        let mut buf = String::new();

        match reader.read_line(&mut buf).await {
            Ok(0) => {
                println!("Console input closed.");
                break;
            }
            Ok(_) => {
                if let Ok(mut queue_guard) = chat_send_queue.try_lock() {
                    queue_guard.push(buf);
                }
            }
            Err(e) => {
                eprintln!("Failed to read from console: {}", e);
                break;
            }
        }
    }
}

async fn start_server(command: &mut Command, http: &Arc<Http>) {
    let mut handle = command.spawn().unwrap();
    let mut stdout = handle.stdout.take().ok_or("handle present").unwrap();
    let mut stdin = handle.stdin.take().ok_or("handle present").unwrap();
    let reader : TokioBufReader<ChildStdout> = TokioBufReader::new(stdout);
    let mut lines = reader.lines();

    tokio::spawn(async move {
        loop {
            if let Ok(mut queue_guard) = chat_send_queue.try_lock() {
                for message in queue_guard.drain(..) {
                    if let Err(e) = stdin.write_all(message.as_bytes()).await {
                        eprintln!("Failed to write to stdin: {}", e);
                        break;
                    }
                    if let Err(e) = stdin.flush().await {
                        eprintln!("Failed to flush stdin: {}", e);
                        break;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });


    while let Ok(Some(line)) = lines.next_line().await {
        let chat_message = INSTANCE.try_parse_line(line.clone());
        println!("{}", line);

        if let Some(msg) = chat_message {
            send_player_message_embed(http, msg).await;
        }
    }

    let status = handle.wait().await.unwrap();
    if !status.success() {
        eprintln!("Process exited with status: {}", status);
    }
}

async fn send_player_message_embed(http: &Arc<Http>, message: ChatMessage) {
    let embed = CreateEmbed::new().title(message.get_message_text());

    let embed_author = CreateEmbedAuthor::new(&message.get_player()).icon_url(&format!(
        "https://mc-api.io/render/face/{}/java",
        message.get_player()
    ));

    let embed = embed.author(embed_author);

    // Create the message builder and add the embed
    let builder = CreateMessage::new().embed(embed);
    if let Err(why) = ChannelId::new(channel_id_val.lock().await.get())
        .send_message(&http, builder)
        .await
    {
        println!("Error sending message: {why:?}");
    }
}

struct MessageHandler;

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
               if let Ok(mut channel_id_guard) = channel_id_val.try_lock() && msg.channel_id == channel_id_guard.get() {

                   let mut message_to_send = &msg.content;
                   let mut mc_chat_message = String::new();

                   if msg.attachments.len()>0 {

                       mc_chat_message = format!("tellraw @a [\"\",{{\"text\":\"<\",\"color\":\"blue\"}},{{\"text\":\"{}\"}},{{\"text\":\">\",\"color\":\"blue\"}},{{\"text\":\" {}{}\"}},{{\"text\":\"[\",\"color\":\"blue\"}},{{\"text\":\"see Discord for attachments\",\"clickEvent\":{{\"action\":\"open_url\",\"value\":\"{}\"}}}},{{\"text\":\"]\",\"color\":\"blue\"}}]\n", msg.author.name, message_to_send,if message_to_send.len()>1 {" "} else {""},msg.link());

                       //message_to_send = format!("{}{}[see Discord for attachments]",message_to_send, if message_to_send.len()>1 {" "} else {""} )
                   }
                    else {
                        mc_chat_message = format!("tellraw @a [\"\",{{\"text\":\"<\",\"color\":\"blue\"}},{{\"text\":\"{}\"}},{{\"text\":\">\",\"color\":\"blue\"}},{{\"text\":\" {}\"}}]\n", msg.author.name, message_to_send);

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

fn get_dir_char() -> char {
    if env::consts::OS.eq("windows") {
        return '\\';
    }
    '/'
}
fn get_chain_char() -> char {
    if env::consts::OS.eq("windows") {
        return '&';
    }
    ';'
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN");
    channel_id_val
        .lock()
        .await
        .set(env::var("CHANNEL_ID").unwrap().parse::<u64>().unwrap());

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No launch command specified");
        return;
    }

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token.expect("Failed to read Discord Token"), intents)
        .event_handler(MessageHandler)
        .await
        .expect("Err creating client");

    let http = Arc::clone(&client.http);

    let mut argument_as_vec = args[1].split(get_dir_char()).collect::<Vec<&str>>();
    let minecraft_server_launch_command = /*&args[2];*/ <&str>::clone(argument_as_vec.last().unwrap());
    argument_as_vec.remove(argument_as_vec.len() - 1);
    let minecraft_server_directory = /*args[1].split(get_dir_char()).collect::<Vec<&str>>().join("/");*/ argument_as_vec.join("/");

    println!("{}", minecraft_server_launch_command);

    //let mut command = Command::new(format!("./{}",minecraft_server_launch_command));
    let mut command: Command;

    if env::consts::OS.eq("windows") {
        command = Command::new("cmd");
        command.args(["/C", &format!("{}", minecraft_server_launch_command)]);
    } else {
        command = Command::new(minecraft_server_launch_command);
    }
    command.current_dir(&minecraft_server_directory);
    command.stdout(Stdio::piped());
    command.stdin(Stdio::piped());

    let (result_one, result_two, result_three) = join!(client.start(), start_server(&mut command, &http), handle_console_input());


}
