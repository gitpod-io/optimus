use regex::Regex;
use serenity::{client::Context, framework::standard::Args, model::channel::Message};

pub struct Parse;

impl Parse {
    pub async fn user(_ctx: &Context, _message: &Message, _arguments: &Args) -> u64 {
        if _arguments.rest().is_empty() {
            *_message.author.id.as_u64()
        } else {
            let re = Regex::new(r#"\W"#).unwrap();
            let pstr = &_arguments.rest().to_string();
            let to_return = re.replace_all(pstr.as_str(), "");

            if Regex::new("[0-9]{18}+")
                .unwrap()
                .is_match(&to_return.to_string().as_str())
            {
                to_return.parse::<u64>().unwrap()
            } else {
                let userid_byname = _ctx
                    .cache
                    .guild(*_message.guild_id.unwrap().as_u64())
                    .await
                    .unwrap()
                    .member_named(&to_return)
                    .unwrap()
                    .user
                    .id;
                *userid_byname.as_u64()
            }

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
