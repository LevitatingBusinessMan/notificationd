use nom;
use nom::character::complete::none_of;
use nom::AsChar;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::character::complete::char;
use nom::character::complete::digit1;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::preceded;
use nom::sequence::terminated;

fn sign(input: &str) -> IResult<&str, bool> {
    map(alt((char('+'), char('-'))), |s| s == '+').parse(input)
}

fn command(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_alphanum() || c == '_')(input)
}

fn arguments(input: &str) -> IResult<&str, Vec<&str>> {
    many0(preceded(space1, is_not(" \t\r\n:"))).parse(input)
}

fn trailing(input: &str) -> IResult<&str, &str> {
    preceded(opt(space1), take_while(|c| c != '\r' || c != '\n')).parse(input)
    //preceded(space1, is_not("\r\n")).parse(input)
}

fn id(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse::<u32>).parse(input)
}

fn crlf(input: &str) -> IResult<&str, &str> {
    alt((tag("\r\n"), tag("\n"))).parse(input)
}

#[derive(Debug)]
pub struct Message {
    pub id: Option<u32>,
    pub sign: Option<bool>,
    pub command: String,
    pub arguments: Vec<String>,
    pub trailing: Option<String>,
}

pub fn line(input: &str, terminated_: bool) -> IResult<&str, Message> {
    let (input, id) = opt(terminated(id, space1)).parse(input)?;
    let (input, sign) = opt(sign).parse(input)?;
    let (input, command) = command(input)?;
    let (input, arguments) = arguments(input)?;
    let (input, trailing) = opt(preceded((opt(space1), char(':')), trailing)).parse(input)?;
    let (input, _) = if terminated_ {
        (crlf)(input)?
    } else {
        (input, "")
    };

    let msg = Message {
        id,
        sign,
        command: String::from(command),
        arguments: arguments.into_iter().map(String::from).collect(),
        trailing: trailing.map(String::from),
    };

    Ok((input, msg))
}

#[test]
fn empty_trailing() {
    let (res_, msg) = line("BODY:", false).unwrap();
    println!("{res_:?} {msg:?}");
}
