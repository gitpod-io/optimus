use serenity::model::id::ChannelId;

pub const INTRODUCTION_CHANNEL: ChannelId = if cfg!(debug_assertions) {
    ChannelId(947769443516284939)
} else {
    ChannelId(816249489911185418)
};

pub const QUESTIONS_CHANNEL: ChannelId = if cfg!(debug_assertions) {
    ChannelId(1026115789721444384)
} else {
    ChannelId(1026792978854973460)
};

pub const SELFHOSTED_QUESTIONS_CHANNEL: ChannelId = if cfg!(debug_assertions) {
    ChannelId(1026800568989143051)
} else {
    ChannelId(1026800700002402336)
};

pub const GENERAL_CHANNEL: ChannelId = if cfg!(debug_assertions) {
    ChannelId(947769443516284943)
} else {
    ChannelId(839379835662368768)
};

pub const OFFTOPIC_CHANNEL: ChannelId = if cfg!(debug_assertions) {
    ChannelId(947769443793141769)
} else {
    ChannelId(972510491933032508)
};
