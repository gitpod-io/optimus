use regex::Regex;
use serenity::{framework::standard::Args, model::channel::Message};

pub struct Parse;

impl Parse {
    pub fn user(_message: &Message, _arguments: &Args) -> u64 {
        if _arguments.rest().is_empty() {
            *_message.author.id.as_u64()
        } else {
            let re = Regex::new(r#"\W"#).unwrap();
            re.replace_all(_arguments.rest().to_string().as_str(), "")
                .parse::<u64>()
                .unwrap()
            // _arguments.rest().to_string().replace("<@!", "").replace(">", "")
        }
    }

    // pub fn avatar(_user_data: &Member) -> String {
    //     let user = &_user_data.user;
    //     if user.avatar_url().is_some() {
    //         user.avatar_url().unwrap()
    //     } else {
    //         user.default_avatar_url()
    //     }
    // }
}
