use serenity::model::id::MessageId;

use std::{env, fmt, path};
use tokio::fs;

pub mod db;
pub mod index_threads;
pub mod misc;
pub mod parser;
pub mod substr;
