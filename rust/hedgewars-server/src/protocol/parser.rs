/** The parsers for the chat and multiplayer protocol. The main parser is `message`.
 * # Protocol
 * All messages consist of `\n`-separated strings. The end of a message is
 * indicated by a double newline - `\n\n`.
 *
 * For example, a nullary command like PING will be actually sent as `PING\n\n`.
 * A unary command, such as `START_GAME nick` will be actually sent as `START_GAME\nnick\n\n`.
 */
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while},
    character::complete::{newline, not_line_ending},
    combinator::peek,
    error::{ErrorKind, ParseError},
    multi::separated_list,
    sequence::{pairc, precededc, terminatedc},
    Err, IResult,
};

use std::{
    num::ParseIntError,
    ops::Range,
    str,
    str::{FromStr, Utf8Error},
};

use super::messages::{HwProtocolMessage, HwProtocolMessage::*};
use crate::core::types::{
    GameCfg, HedgehogInfo, ServerVar, TeamInfo, VoteType, MAX_HEDGEHOGS_PER_TEAM,
};

#[derive(Debug, PartialEq)]
pub struct HwProtocolError {}

impl HwProtocolError {
    fn new() -> Self {
        HwProtocolError {}
    }
}

impl<I> ParseError<I> for HwProtocolError {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        HwProtocolError::new()
    }

    fn append(input: I, kind: ErrorKind, other: Self) -> Self {
        HwProtocolError::new()
    }
}

impl From<Utf8Error> for HwProtocolError {
    fn from(_: Utf8Error) -> Self {
        HwProtocolError::new()
    }
}

impl From<ParseIntError> for HwProtocolError {
    fn from(_: ParseIntError) -> Self {
        HwProtocolError::new()
    }
}

pub type HwResult<'a, O> = IResult<&'a [u8], O, HwProtocolError>;

fn end_of_message(input: &[u8]) -> HwResult<&[u8]> {
    tag("\n\n")(input)
}

fn convert_utf8(input: &[u8]) -> HwResult<&str> {
    match str::from_utf8(input) {
        Ok(str) => Ok((b"", str)),
        Err(utf_err) => Result::Err(Err::Failure(utf_err.into())),
    }
}

fn convert_from_str<T>(str: &str) -> HwResult<T>
where
    T: FromStr<Err = ParseIntError>,
{
    match T::from_str(str) {
        Ok(x) => Ok((b"", x)),
        Err(format_err) => Result::Err(Err::Failure(format_err.into())),
    }
}

fn str_line(input: &[u8]) -> HwResult<&str> {
    let (i, text) = not_line_ending(input.clone())?;
    if i != input {
        Ok((i, convert_utf8(text)?.1))
    } else {
        Err(Err::Error(HwProtocolError::new()))
    }
}

fn a_line(input: &[u8]) -> HwResult<String> {
    let (i, str) = str_line(input)?;
    Ok((i, str.to_string()))
}

fn hw_tag<'a>(tag_str: &'a str) -> impl Fn(&'a [u8]) -> HwResult<'a, ()> {
    move |i| tag(tag_str)(i).map(|(i, _)| (i, ()))
}

fn hw_tag_no_case<'a>(tag_str: &'a str) -> impl Fn(&'a [u8]) -> HwResult<'a, ()> {
    move |i| tag_no_case(tag_str)(i).map(|(i, _)| (i, ()))
}

fn cmd_arg(input: &[u8]) -> HwResult<String> {
    let delimiters = b" \n";
    let (i, str) = take_while(move |c| !delimiters.contains(&c))(input.clone())?;
    if i != input {
        Ok((i, convert_utf8(str)?.1.to_string()))
    } else {
        Err(Err::Error(HwProtocolError::new()))
    }
}

fn u8_line(input: &[u8]) -> HwResult<u8> {
    let (i, str) = str_line(input)?;
    Ok((i, convert_from_str(str)?.1))
}

fn u16_line(input: &[u8]) -> HwResult<u16> {
    let (i, str) = str_line(input)?;
    Ok((i, convert_from_str(str)?.1))
}

fn u32_line(input: &[u8]) -> HwResult<u32> {
    let (i, str) = str_line(input)?;
    Ok((i, convert_from_str(str)?.1))
}

fn yes_no_line(input: &[u8]) -> HwResult<bool> {
    alt((
        |i| tag_no_case(b"YES")(i).map(|(i, _)| (i, true)),
        |i| tag_no_case(b"NO")(i).map(|(i, _)| (i, false)),
    ))(input)
}

fn opt_arg<'a>(input: &'a [u8]) -> HwResult<'a, Option<String>> {
    alt((
        |i| peek(end_of_message)(i).map(|(i, _)| (i, None)),
        |i| precededc(i, hw_tag("\n"), a_line).map(|(i, v)| (i, Some(v))),
    ))(input)
}

fn spaces(input: &[u8]) -> HwResult<&[u8]> {
    precededc(input, hw_tag(" "), |i| take_while(|c| c == b' ')(i))
}

fn opt_space_arg<'a>(input: &'a [u8]) -> HwResult<'a, Option<String>> {
    alt((
        |i| peek(end_of_message)(i).map(|(i, _)| (i, None)),
        |i| precededc(i, spaces, a_line).map(|(i, v)| (i, Some(v))),
    ))(input)
}

fn hedgehog_array(input: &[u8]) -> HwResult<[HedgehogInfo; 8]> {
    fn hedgehog_line(input: &[u8]) -> HwResult<HedgehogInfo> {
        let (i, name) = terminatedc(input, a_line, newline)?;
        let (i, hat) = a_line(i)?;
        Ok((i, HedgehogInfo { name, hat }))
    }

    let (i, h1) = terminatedc(input, hedgehog_line, newline)?;
    let (i, h2) = terminatedc(i, hedgehog_line, newline)?;
    let (i, h3) = terminatedc(i, hedgehog_line, newline)?;
    let (i, h4) = terminatedc(i, hedgehog_line, newline)?;
    let (i, h5) = terminatedc(i, hedgehog_line, newline)?;
    let (i, h6) = terminatedc(i, hedgehog_line, newline)?;
    let (i, h7) = terminatedc(i, hedgehog_line, newline)?;
    let (i, h8) = hedgehog_line(i)?;

    Ok((i, [h1, h2, h3, h4, h5, h6, h7, h8]))
}

fn voting(input: &[u8]) -> HwResult<VoteType> {
    alt((
        |i| tag_no_case("PAUSE")(i).map(|(i, _)| (i, VoteType::Pause)),
        |i| tag_no_case("NEWSEED")(i).map(|(i, _)| (i, VoteType::NewSeed)),
        |i| {
            precededc(i, |i| precededc(i, hw_tag_no_case("KICK"), spaces), a_line)
                .map(|(i, s)| (i, VoteType::Kick(s)))
        },
        |i| {
            precededc(
                i,
                |i| precededc(i, hw_tag_no_case("HEDGEHOGS"), spaces),
                u8_line,
            )
            .map(|(i, n)| (i, VoteType::HedgehogsPerTeam(n)))
        },
        |i| precededc(i, hw_tag_no_case("MAP"), opt_space_arg).map(|(i, v)| (i, VoteType::Map(v))),
    ))(input)
}

fn no_arg_message(input: &[u8]) -> HwResult<HwProtocolMessage> {
    fn messagec<'a>(
        input: &'a [u8],
        name: &'a str,
        msg: HwProtocolMessage,
    ) -> HwResult<'a, HwProtocolMessage> {
        tag(name)(input).map(|(i, _)| (i, msg.clone()))
    }

    alt((
        |i| messagec(i, "PING", Ping),
        |i| messagec(i, "PONG", Pong),
        |i| messagec(i, "LIST", List),
        |i| messagec(i, "BANLIST", BanList),
        |i| messagec(i, "GET_SERVER_VAR", GetServerVar),
        |i| messagec(i, "TOGGLE_READY", ToggleReady),
        |i| messagec(i, "START_GAME", StartGame),
        |i| messagec(i, "TOGGLE_RESTRICT_JOINS", ToggleRestrictJoin),
        |i| messagec(i, "TOGGLE_RESTRICT_TEAMS", ToggleRestrictTeams),
        |i| messagec(i, "TOGGLE_REGISTERED_ONLY", ToggleRegisteredOnly),
    ))(input)
}

fn single_arg_message(input: &[u8]) -> HwResult<HwProtocolMessage> {
    fn messagec<'a, T, F, G>(
        input: &'a [u8],
        name: &'a str,
        parser: F,
        constructor: G,
    ) -> HwResult<'a, HwProtocolMessage>
    where
        F: Fn(&[u8]) -> HwResult<T>,
        G: Fn(T) -> HwProtocolMessage,
    {
        precededc(input, hw_tag(name), parser).map(|(i, v)| (i, constructor(v)))
    }

    alt((
        |i| messagec(i, "NICK\n", a_line, Nick),
        |i| messagec(i, "INFO\n", a_line, Info),
        |i| messagec(i, "CHAT\n", a_line, Chat),
        |i| messagec(i, "PART", opt_arg, Part),
        |i| messagec(i, "FOLLOW\n", a_line, Follow),
        |i| messagec(i, "KICK\n", a_line, Kick),
        |i| messagec(i, "UNBAN\n", a_line, Unban),
        |i| messagec(i, "EM\n", a_line, EngineMessage),
        |i| messagec(i, "TEAMCHAT\n", a_line, TeamChat),
        |i| messagec(i, "ROOM_NAME\n", a_line, RoomName),
        |i| messagec(i, "REMOVE_TEAM\n", a_line, RemoveTeam),
        |i| messagec(i, "ROUNDFINISHED", opt_arg, |_| RoundFinished),
        |i| messagec(i, "PROTO\n", u16_line, Proto),
        |i| messagec(i, "QUIT", opt_arg, Quit),
    ))(input)
}

fn cmd_message<'a>(input: &'a [u8]) -> HwResult<'a, HwProtocolMessage> {
    fn cmdc_no_arg<'a>(
        input: &'a [u8],
        name: &'a str,
        msg: HwProtocolMessage,
    ) -> HwResult<'a, HwProtocolMessage> {
        tag_no_case(name)(input).map(|(i, _)| (i, msg.clone()))
    }

    fn cmdc_single_arg<'a, T, F, G>(
        input: &'a [u8],
        name: &'a str,
        parser: F,
        constructor: G,
    ) -> HwResult<'a, HwProtocolMessage>
    where
        F: Fn(&'a [u8]) -> HwResult<'a, T>,
        G: Fn(T) -> HwProtocolMessage,
    {
        precededc(input, |i| pairc(i, hw_tag_no_case(name), spaces), parser)
            .map(|(i, v)| (i, constructor(v)))
    }

    fn cmd_no_arg_message(input: &[u8]) -> HwResult<HwProtocolMessage> {
        alt((
            |i| cmdc_no_arg(i, "STATS", Stats),
            |i| cmdc_no_arg(i, "FIX", Fix),
            |i| cmdc_no_arg(i, "UNFIX", Unfix),
            |i| cmdc_no_arg(i, "REGISTERED_ONLY", ToggleServerRegisteredOnly),
            |i| cmdc_no_arg(i, "SUPER_POWER", SuperPower),
        ))(input)
    }

    fn cmd_single_arg_message(input: &[u8]) -> HwResult<HwProtocolMessage> {
        alt((
            |i| cmdc_single_arg(i, "RESTART_SERVER", |i| tag("YES")(i), |_| RestartServer),
            |i| cmdc_single_arg(i, "DELEGATE", a_line, Delegate),
            |i| cmdc_single_arg(i, "DELETE", a_line, Delete),
            |i| cmdc_single_arg(i, "SAVEROOM", a_line, SaveRoom),
            |i| cmdc_single_arg(i, "LOADROOM", a_line, LoadRoom),
            |i| cmdc_single_arg(i, "GLOBAL", a_line, Global),
            |i| cmdc_single_arg(i, "WATCH", u32_line, Watch),
            |i| cmdc_single_arg(i, "VOTE", yes_no_line, Vote),
            |i| cmdc_single_arg(i, "FORCE", yes_no_line, ForceVote),
            |i| cmdc_single_arg(i, "INFO", a_line, Info),
            |i| cmdc_single_arg(i, "MAXTEAMS", u8_line, MaxTeams),
            |i| cmdc_single_arg(i, "CALLVOTE", voting, |v| CallVote(Some(v))),
        ))(input)
    }

    precededc(
        input,
        hw_tag("CMD\n"),
        alt((
            cmd_no_arg_message,
            cmd_single_arg_message,
            |i| tag_no_case("CALLVOTE")(i).map(|(i, _)| (i, CallVote(None))),
            |i| {
                precededc(i, hw_tag_no_case("GREETING"), opt_space_arg)
                    .map(|(i, s)| (i, Greeting(s)))
            },
            |i| precededc(i, hw_tag_no_case("PART"), opt_space_arg).map(|(i, s)| (i, Part(s))),
            |i| precededc(i, hw_tag_no_case("QUIT"), opt_space_arg).map(|(i, s)| (i, Quit(s))),
            |i| {
                precededc(i, hw_tag_no_case("SAVE"), |i| {
                    pairc(
                        i,
                        |i| precededc(i, spaces, cmd_arg),
                        |i| precededc(i, spaces, cmd_arg),
                    )
                })
                .map(|(i, (n, l))| (i, Save(n, l)))
            },
            |i| {
                let (i, _) = tag_no_case("RND")(i)?;
                let (i, v) = alt((
                    |i| peek(end_of_message)(i).map(|(i, _)| (i, vec![])),
                    |i| {
                        let (i, _) = spaces(i)?;
                        let (i, v) = separated_list(spaces, cmd_arg)(i)?;
                        Ok((i, v))
                    },
                ))(i)?;
                Ok((i, Rnd(v)))
            },
        )),
    )
}

fn config_message<'a>(input: &'a [u8]) -> HwResult<'a, HwProtocolMessage> {
    fn cfgc_single_arg<'a, T, F, G>(
        input: &'a [u8],
        name: &'a str,
        parser: F,
        constructor: G,
    ) -> HwResult<'a, GameCfg>
    where
        F: Fn(&[u8]) -> HwResult<T>,
        G: Fn(T) -> GameCfg,
    {
        precededc(input, |i| terminatedc(i, hw_tag(name), newline), parser)
            .map(|(i, v)| (i, constructor(v)))
    }

    let (i, cfg) = precededc(
        input,
        hw_tag("CFG\n"),
        alt((
            |i| cfgc_single_arg(i, "THEME", a_line, GameCfg::Theme),
            |i| cfgc_single_arg(i, "SCRIPT", a_line, GameCfg::Script),
            |i| cfgc_single_arg(i, "MAP", a_line, GameCfg::MapType),
            |i| cfgc_single_arg(i, "MAPGEN", u32_line, GameCfg::MapGenerator),
            |i| cfgc_single_arg(i, "MAZE_SIZE", u32_line, GameCfg::MazeSize),
            |i| cfgc_single_arg(i, "TEMPLATE", u32_line, GameCfg::Template),
            |i| cfgc_single_arg(i, "FEATURE_SIZE", u32_line, GameCfg::FeatureSize),
            |i| cfgc_single_arg(i, "SEED", a_line, GameCfg::Seed),
            |i| cfgc_single_arg(i, "DRAWNMAP", a_line, GameCfg::DrawnMap),
            |i| {
                precededc(
                    i,
                    |i| terminatedc(i, hw_tag("AMMO"), newline),
                    |i| {
                        let (i, name) = a_line(i)?;
                        let (i, value) = opt_arg(i)?;
                        Ok((i, GameCfg::Ammo(name, value)))
                    },
                )
            },
            |i| {
                precededc(
                    i,
                    |i| terminatedc(i, hw_tag("SCHEME"), newline),
                    |i| {
                        let (i, name) = a_line(i)?;
                        let (i, values) = alt((
                            |i| peek(end_of_message)(i).map(|(i, _)| (i, None)),
                            |i| {
                                precededc(i, newline, |i| separated_list(newline, a_line)(i))
                                    .map(|(i, v)| (i, Some(v)))
                            },
                        ))(i)?;
                        Ok((i, GameCfg::Scheme(name, values.unwrap_or_default())))
                    },
                )
            },
        )),
    )?;
    Ok((i, Cfg(cfg)))
}

fn server_var_message(input: &[u8]) -> HwResult<HwProtocolMessage> {
    precededc(
        input,
        hw_tag("SET_SERVER_VAR\n"),
        alt((
            |i| {
                precededc(i, hw_tag("MOTD_NEW\n"), a_line)
                    .map(|(i, s)| (i, SetServerVar(ServerVar::MOTDNew(s))))
            },
            |i| {
                precededc(i, hw_tag("MOTD_OLD\n"), a_line)
                    .map(|(i, s)| (i, SetServerVar(ServerVar::MOTDOld(s))))
            },
            |i| {
                precededc(i, hw_tag("LATEST_PROTO\n"), u16_line)
                    .map(|(i, n)| (i, SetServerVar(ServerVar::LatestProto(n))))
            },
        )),
    )
}

fn complex_message(input: &[u8]) -> HwResult<HwProtocolMessage> {
    alt((
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("PASSWORD"), newline),
                |i| {
                    let (i, pass) = terminatedc(i, a_line, newline)?;
                    let (i, salt) = a_line(i)?;
                    Ok((i, Password(pass, salt)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("CHECKER"), newline),
                |i| {
                    let (i, protocol) = terminatedc(i, u16_line, newline)?;
                    let (i, name) = terminatedc(i, a_line, newline)?;
                    let (i, pass) = a_line(i)?;
                    Ok((i, Checker(protocol, name, pass)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("CREATE_ROOM"), newline),
                |i| {
                    let (i, name) = a_line(i)?;
                    let (i, pass) = opt_arg(i)?;
                    Ok((i, CreateRoom(name, pass)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("JOIN_ROOM"), newline),
                |i| {
                    let (i, name) = a_line(i)?;
                    let (i, pass) = opt_arg(i)?;
                    Ok((i, JoinRoom(name, pass)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("ADD_TEAM"), newline),
                |i| {
                    let (i, name) = terminatedc(i, a_line, newline)?;
                    let (i, color) = terminatedc(i, u8_line, newline)?;
                    let (i, grave) = terminatedc(i, a_line, newline)?;
                    let (i, fort) = terminatedc(i, a_line, newline)?;
                    let (i, voice_pack) = terminatedc(i, a_line, newline)?;
                    let (i, flag) = terminatedc(i, a_line, newline)?;
                    let (i, difficulty) = terminatedc(i, u8_line, newline)?;
                    let (i, hedgehogs) = hedgehog_array(i)?;
                    Ok((
                        i,
                        AddTeam(Box::new(TeamInfo {
                            owner: String::new(),
                            name,
                            color,
                            grave,
                            fort,
                            voice_pack,
                            flag,
                            difficulty,
                            hedgehogs,
                            hedgehogs_number: 0,
                        })),
                    ))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("HH_NUM"), newline),
                |i| {
                    let (i, name) = terminatedc(i, a_line, newline)?;
                    let (i, count) = u8_line(i)?;
                    Ok((i, SetHedgehogsNumber(name, count)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("TEAM_COLOR"), newline),
                |i| {
                    let (i, name) = terminatedc(i, a_line, newline)?;
                    let (i, color) = u8_line(i)?;
                    Ok((i, SetTeamColor(name, color)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("BAN"), newline),
                |i| {
                    let (i, n) = terminatedc(i, a_line, newline)?;
                    let (i, r) = terminatedc(i, a_line, newline)?;
                    let (i, t) = u32_line(i)?;
                    Ok((i, Ban(n, r, t)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("BAN_IP"), newline),
                |i| {
                    let (i, n) = terminatedc(i, a_line, newline)?;
                    let (i, r) = terminatedc(i, a_line, newline)?;
                    let (i, t) = u32_line(i)?;
                    Ok((i, BanIP(n, r, t)))
                },
            )
        },
        |i| {
            precededc(
                i,
                |i| terminatedc(i, hw_tag("BAN_NICK"), newline),
                |i| {
                    let (i, n) = terminatedc(i, a_line, newline)?;
                    let (i, r) = terminatedc(i, a_line, newline)?;
                    let (i, t) = u32_line(i)?;
                    Ok((i, BanNick(n, r, t)))
                },
            )
        },
    ))(input)
}

pub fn malformed_message(input: &[u8]) -> HwResult<()> {
    let (i, _) = terminatedc(input, |i| take_until(&b"\n\n"[..])(i), end_of_message)?;
    Ok((i, ()))
}

pub fn message(input: &[u8]) -> HwResult<HwProtocolMessage> {
    precededc(
        input,
        |i| take_while(|c| c == b'\n')(i),
        |i| {
            terminatedc(
                i,
                alt((
                    no_arg_message,
                    single_arg_message,
                    cmd_message,
                    config_message,
                    server_var_message,
                    complex_message,
                )),
                end_of_message,
            )
        },
    )
}

#[cfg(test)]
mod test {
    use super::message;
    use crate::{
        core::types::GameCfg,
        protocol::{messages::HwProtocolMessage::*, parser::HwProtocolError, test::gen_proto_msg},
    };
    use proptest::{proptest, proptest_helper};

    #[cfg(test)]
    proptest! {
        #[test]
        fn is_parser_composition_idempotent(ref msg in gen_proto_msg()) {
            println!("!! Msg: {:?}, Bytes: {:?} !!", msg, msg.to_raw_protocol().as_bytes());
            assert_eq!(message(msg.to_raw_protocol().as_bytes()), Ok((&b""[..], msg.clone())))
        }
    }

    #[test]
    fn parse_test() {
        assert_eq!(message(b"PING\n\n"), Ok((&b""[..], Ping)));
        assert_eq!(message(b"START_GAME\n\n"), Ok((&b""[..], StartGame)));
        assert_eq!(
            message(b"NICK\nit's me\n\n"),
            Ok((&b""[..], Nick("it's me".to_string())))
        );
        assert_eq!(message(b"PROTO\n51\n\n"), Ok((&b""[..], Proto(51))));
        assert_eq!(
            message(b"QUIT\nbye-bye\n\n"),
            Ok((&b""[..], Quit(Some("bye-bye".to_string()))))
        );
        assert_eq!(message(b"QUIT\n\n"), Ok((&b""[..], Quit(None))));
        assert_eq!(
            message(b"CMD\nwatch 49471\n\n"),
            Ok((&b""[..], Watch(49471)))
        );
        assert_eq!(
            message(b"BAN\nme\nbad\n77\n\n"),
            Ok((&b""[..], Ban("me".to_string(), "bad".to_string(), 77)))
        );

        assert_eq!(message(b"CMD\nPART\n\n"), Ok((&b""[..], Part(None))));
        assert_eq!(
            message(b"CMD\nPART _msg_\n\n"),
            Ok((&b""[..], Part(Some("_msg_".to_string()))))
        );

        assert_eq!(message(b"CMD\nRND\n\n"), Ok((&b""[..], Rnd(vec![]))));
        assert_eq!(
            message(b"CMD\nRND A B\n\n"),
            Ok((&b""[..], Rnd(vec![String::from("A"), String::from("B")])))
        );

        assert_eq!(
            message(b"CFG\nSCHEME\na\nA\n\n"),
            Ok((
                &b""[..],
                Cfg(GameCfg::Scheme("a".to_string(), vec!["A".to_string()]))
            ))
        );

        assert_eq!(
            message(b"QUIT\n1\n2\n\n"),
            Err(nom::Err::Error(HwProtocolError::new()))
        );
    }
}
