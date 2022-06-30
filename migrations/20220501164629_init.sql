CREATE TABLE IF NOT EXISTS server_config (
	getting_started_channel INTEGER,
	introduction_channel INTEGER,
	feedback_channel INTEGER,
	showcase_channel INTEGER,
	question_channels INTEGER NOT NULL,
	subscriber_role INTEGER
);

CREATE TABLE IF NOT EXISTS user_profile (
	user_id INTEGER PRIMARY KEY,
	roles TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pending_questions (
	user_id INTEGER NOT NULL,
	channel_id INTEGER NOT NULL,
	message_contents TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS message_storage (
	message_id INTEGER PRIMARY KEY,
	message_contents TEXT,
	edit_history TEXT

);