CREATE TABLE IF NOT EXISTS server_config (
	getting_started_channel INTEGER,
	introduction_channel INTEGER,
	question_channels TEXT,
	subscriber_role INTEGER,
);

CREATE TABLE IF NOT EXISTS user_profile (
	user_id INTEGER,
	programming_langs TEXT,
	spoken_langs TEXT,
	subscribed INTEGER,
)

CREATE TABLE IF NOT EXISTS pending_questions (
	user_id INTEGER,
	channel_id INTEGER,
	message_contents TEXT,
);