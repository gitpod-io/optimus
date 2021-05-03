use serenity::model::id::MessageId;

use std::{fmt, env, path};
use tokio::fs;

pub mod db;
pub mod substr;
pub mod misc;