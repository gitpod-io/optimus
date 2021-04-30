use super::*;

pub struct Note {
    note_name: String,
}

impl Note {
    pub async fn from(note_name: &String) -> Self {
        Self {
            note_name: note_name.to_string(),
        }
    }

    pub async fn get_contents(&self) -> String {
        let dbnode = Database::from("notes".to_string()).await;
        String::from(
            fs::read_to_string(format!("{}/{}", dbnode, &self.note_name))
                .await
                .unwrap(),
        )
    }

    pub async fn save_contents(&self, contents: String) {
        let dbnode = Database::from("notes".to_string()).await;
        fs::write(
            format!("{}/{}", &dbnode.to_string(), &self.note_name),
            &contents,
        )
        .await
        .unwrap();
    }
}

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn add(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let mut note_name = Args::new(_args.rest(), &[Delimiter::Single(' ')]);
    let first_arg = note_name.single_quoted::<String>().unwrap();

    Note::from(&first_arg)
        .await
        .save_contents(note_name.rest().to_string())
        .await;

    _msg.reply(&_ctx.http, format!("Note(**{}**) written", &first_arg))
        .await
        .unwrap();
    Ok(())
}

#[command]
#[aliases("rm")]
#[required_permissions(ADMINISTRATOR)]
async fn remove(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let mut note_name = Args::new(_args.rest(), &[Delimiter::Single(' ')]);
    let first_arg = note_name.single_quoted::<String>().unwrap();

    fs::remove_file(format!(
        "{}/{}",
        Database::from("notes".to_string()).await,
        &first_arg
    ))
    .await
    .unwrap();

    _msg.reply(&_ctx.http, format!("Note(**{}**) removed", &first_arg))
        .await
        .unwrap();

    Ok(())
}

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn link(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let mut note_name = Args::new(_args.rest(), &[Delimiter::Single(' ')]);
    let first_arg = note_name.single_quoted::<String>().unwrap();
    let second_arg = note_name.single_quoted::<String>().unwrap();
    let dbnode = Database::from("notes".to_string()).await;

    symlink(
        format!("{}/{}", &dbnode, &first_arg),
        format!("{}/{}", &dbnode, &second_arg),
    )
    .await
    .unwrap();

    _msg.reply(
        &_ctx.http,
        format!("Note(**{}**) linked As(**{}**)", &first_arg, &second_arg),
    )
    .await
    .unwrap();

    Ok(())
}

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn list(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let dbnode = Database::from("notes".to_string()).await;

    let mut grid = Grid::new(GridOptions {
        filling: Filling::Spaces(1),
        direction: Direction::TopToBottom,
    });

    for elem in glob::glob(format!("{}/*", &dbnode).as_str()).unwrap() {
        if let Ok(path) = elem {
            grid.add(Cell::from(String::from(format!(
                "⌘`{}`",
                path.file_name().unwrap().to_str().unwrap()
            ))));
        }
    }

    _msg.reply(&_ctx.http, grid.fit_into_columns(300))
        .await
        .unwrap();

    Ok(())
}
