use crate::{ChatMessage, MessageType};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref INSTANCE: LogParser = LogParser::new();
}

pub struct LogParser {
    regex: Regex,
}

impl LogParser {
    pub fn get_chat_message(line: String) -> String {
        let message = line.split_once(":").unwrap().1.split_once(" ").unwrap().1;
        message.to_string()
    }

    fn new() -> Self {
        let regex = Regex::new(r"<[A-Za-z0-9_-]+>").unwrap();

        Self { regex }
    }

    // This function will be used for parsing other events, right now its just for chat messages
    pub fn try_parse_line(&self, line: String) -> Option<ChatMessage> {
        if let Some(caps) = self.regex.captures(&line) {
            
            if line.split(":").collect::<Vec<&str>>().get(3).unwrap().replace(" ","").starts_with("/") {
                return None
            }

            let mut player_name = String::from(&caps[0]);
            player_name = player_name.replace("<", "").replace(">", "");
            let message_text = line.split_once("> ").unwrap().1.to_string();
            return Some(ChatMessage {
                message_type: MessageType::CHAT,
                message_text,
                player_name,
            });
        }
        /*
        for (_,name) in self.regex.captures_iter(&line).map(|c| c.extract::<0>()) {
            let mut player_name = name.to_vec().join("");
            println!("{}", player_name);
            player_name = player_name.replace("<","").replace(">","");
            println!("{}", player_name);
            let message_text = line.split_once("> ").unwrap().1.to_string();
            return Some(ChatMessage {
                message_type: MessageType::CHAT,
                message_text,
                player_name
            })
        }
         */
        None
    }
}
