pub const ACCESS_DENIED: &str = "Access denied.";
pub const AUTHENTICATION_FAILED: &str = "Authentication failed";
pub const BAD_NUMBER: &str = "Bad number.";
pub const ILLEGAL_CLIENT_NAME: &str = "Illegal nickname! Nicknames must be between 1-40 characters long, must not have a trailing or leading space and must not have any of these characters: $()*+?[]^{|}";
pub const ILLEGAL_ROOM_NAME: &str = "Illegal room name! A room name must be between 1-40 characters long, must not have a trailing or leading space and must not have any of these characters: $()*+?[]^{|}";
pub const NICKNAME_PROVIDED: &str = "Nickname already provided.";
pub const NO_CHECKER_RIGHTS: &str = "No checker rights";
pub const NO_ROOM: &str = "No such room.";
pub const NO_TEAM: &str = "No such team.";
pub const NO_TEAM_TO_REMOVE: &str = "Error: The team you tried to remove does not exist.";
pub const NO_USER: &str = "No such user.";
pub const NOT_MASTER: &str = "You're not the room master!";
pub const PROTOCOL_PROVIDED: &str = "Protocol already known.";
pub const PROTOCOL_TOO_OLD: &str = "Protocol version is too old";
pub const REPLAY_LOAD_FAILED: &str = "Could't load the replay";
pub const REPLAY_NOT_SUPPORTED: &str = "This server does not support replays!";
pub const REGISTRATION_REQUIRED: &str = "This server only allows registered users to join.";
pub const REGISTERED_ONLY_ENABLED: &str =
    "This server no longer allows unregistered players to join.";
pub const REGISTERED_ONLY_DISABLED: &str = "This server now allows unregistered players to join.";
pub const ROOM_CONFIG_SAVE_FAILED: &str = "Unable to save the room configs.";
pub const ROOM_CONFIG_LOAD_FAILED: &str = "Unable to load the room configs.";
pub const ROOM_CONFIG_DESERIALIZE_FAILED: &str = "Unable to deserialize the room configs.";
pub const ROOM_CONFIG_LOADED: &str = "Room configs loaded successfully.";
pub const ROOM_CONFIG_SAVED: &str = "Room configs saved successfully.";
pub const ROOM_EXISTS: &str = "A room with the same name already exists.";
pub const ROOM_FULL: &str = "This room is already full.";
pub const ROOM_JOIN_RESTRICTED: &str = "Access denied. This room currently doesn't allow joining.";
pub const ROUND_IN_PROGRESS: &str = "Joining not possible: Round is in progress.";
pub const ROOM_REGISTRATION_REQUIRED: &str =
    "Access denied. This room is for registered users only.";
pub const SUPER_POWER: &str = "Super power activated.";
pub const TEAM_EXISTS: &str = "There's already a team with same name in the list.";
pub const TEAM_NOT_OWNED: &str = "You can't remove a team you don't own.";
pub const TEAM_ADD_RESTRICTED: &str = "This room currently does not allow adding new teams.";
pub const TOO_MANY_HEDGEHOGS: &str = "Too many hedgehogs!";
pub const TOO_MANY_TEAMS: &str = "Too many teams!";
pub const USER_OFFLINE: &str = "Player is not online.";
pub const VARIABLE_UPDATED: &str = "Server variable has been updated.";
pub const INCOMPATIBLE_ROOM_PROTOCOL: &str = "Room version incompatible to your Hedgewars version!";
