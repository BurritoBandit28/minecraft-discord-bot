mod discord;
mod minecraft;
mod util;

use crate::discord::channel_id_val;
use crate::minecraft::{chat_send_queue, start_server};
use discord::MessageHandler;
use dotenv::dotenv;
use serenity::Client;
use serenity::all::{EventHandler, GatewayIntents};
use std::env;
use std::io::{BufRead, Read, Write};
use std::ops::Deref;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::join;
use tokio::process::Command;

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

    let mut argument_as_vec = args[1].split(util::get_dir_char()).collect::<Vec<&str>>();
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
        command = Command::new("sh");
        command.args([minecraft_server_launch_command]);
    }
    command.current_dir(&minecraft_server_directory);
    command.stdout(Stdio::piped());
    command.stdin(Stdio::piped());

    let (result_one, result_two, result_three) = join!(
        start_server(&mut command, &http),
        client.start(),
        handle_console_input()
    );
}
