use serenity::model::id::UserId;

use super::*;

pub struct Database {
    node_name: String,
}

impl Database {
    pub async fn from(node_name: String) -> Self {
        let botsource = env::current_exe().unwrap();
        let botsource_dir = path::Path::new(botsource.parent().unwrap());
        let db_root = format!(
            "{}{}{}",
            &botsource_dir.to_string_lossy(),
            path::Path::new("/").as_os_str().to_string_lossy(),
            "db"
        );

        let new_node = format!("{}/{}", &db_root, node_name);
        fs::create_dir_all(&new_node).await.unwrap();

        Self {
            node_name: new_node,
        }
    }

    pub async fn fetch_msg(&self, id: MessageId) -> String {
        let var = fs::read_to_string(format!("{}/{}", &self.node_name, id))
            .await
            .unwrap();
        var
    }

    pub async fn save_msg(&self, id: &MessageId, contents: String) {
        fs::write(format!("{}/{}", &self.node_name, &id), &contents)
            .await
            .unwrap();
    }

    pub async fn remove_msg(&self, id: &MessageId) {
        fs::remove_file(format!("{}/{}", &self.node_name, id))
            .await
            .unwrap();
    }

    pub async fn msg_exists(&self, id: &MessageId) -> bool {
        let _path = format!("{}/{}", &self.node_name, id);
        path::Path::new(&_path).exists()
    }

    pub async fn save_user_info(&self, id: &UserId, contents: String) {
        let path = format!("{}/{}", &self.node_name, &id);

        let new_contents = {
            if path::Path::new(&path).exists() {
                let old_content = fs::read_to_string(&path).await.unwrap();
                let ready_content = {
                    if !fs::read_to_string(&path).await.unwrap().contains(&contents) {
                        format!("{}\n{}", &old_content, &contents)
                    } else {
                        String::from(&old_content)
                    }
                };
                ready_content
            } else {
                String::from(&contents)
            }
        };
        // println!("{}", &new_contents);
        // if String::from(&new_contents) != String::from(&old_content) {
        fs::write(&path, &new_contents).await.unwrap();
        // }
    }

    pub async fn get_user_info(&self, id: &String) -> String {
        let path = format!("{}/{}", &self.node_name, &id);
        fs::read_to_string(&path).await.unwrap()
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_name)
    }
}
