use super::*;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, one_of, satisfy};
use nom::combinator::{map, opt, recognize, value};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use nom::{IResult, InputIter, Slice};
use nom_locate::LocatedSpan;

pub type Input<'a> = LocatedSpan<&'a str>;
pub type Result<'a, T> = IResult<Input<'a>, T>;

/// Parse the name of the core, which is always the first entry.
fn name(input: Input) -> Result<String> {
    map(recognize(many1(satisfy(|c| c != ';'))), |s: Input| {
        s.to_string()
    })(input)
}

/// Parse the core settings, which is always the second entry.
fn core_settings(input: Input) -> Result<settings::Settings> {
    // TODO: add a proper parser for the settings line.
    map(
        recognize(many0(satisfy(|c| c != ';' && c != '-' && c != 'C'))),
        |s: Input| settings::Settings::from_str(s.fragment()).unwrap(),
    )(input)
}

/// Parses a separator menu option, e.g. `-`, with optional text.
fn separator(input: Input) -> Result<ConfigMenu> {
    map(
        preceded(char('-'), opt(recognize(many1(satisfy(|c| c != ';'))))),
        |maybe_text| ConfigMenu::Empty(maybe_text.map(|s: Input| s.to_string())),
    )(input)
}

/// `C[,{Text}]` - Enables a cheat menu entry with the label {Text}.
fn cheat(input: Input) -> Result<ConfigMenu> {
    map(
        preceded(
            char('C'),
            opt(preceded(char(','), recognize(many1(satisfy(|c| c != ';'))))),
        ),
        |maybe_text| ConfigMenu::Cheat(maybe_text.map(|s: Input| s.to_string())),
    )(input)
}

/// DIP.
fn dip(input: Input) -> Result<ConfigMenu> {
    value(ConfigMenu::Dip, tag("DIP"))(input)
}

/// Parses an integer value in u32.
fn integer(input: Input) -> Result<u32> {
    map(recognize(digit1), |s: Input| s.fragment().parse().unwrap())(input)
}

/// Parses an hexadecimal integer. This is needed but fixed in nom's main branch, just
/// hasn't been released yet.
fn hex_u32(input: Input) -> Result<u32> {
    let (i, o) = nom::bytes::complete::is_a(&b"0123456789abcdefABCDEF"[..])(input)?;
    // Do not parse more than 8 characters for a u32
    let (parsed, remaining) = if o.len() <= 8 {
        (o, i)
    } else {
        (input.slice(..8), input.slice(8..))
    };

    let res = parsed
        .iter_elements()
        .rev()
        .enumerate()
        .map(|(k, v)| {
            let digit = v;
            digit.to_digit(16).unwrap_or(0) << (k * 4)
        })
        .sum();

    Ok((remaining, res))
}

/// Disable if.
fn disable_if(line: u8) -> impl FnMut(Input) -> Result<ConfigMenu> {
    move |input| {
        map(
            tuple((char('D'), integer, config_menu_line(line))),
            |(_, s, c)| ConfigMenu::DisableIf(s, Box::new(c)),
        )(input)
    }
}

/// Disable unless.
fn disable_unless(line: u8) -> impl FnMut(Input) -> Result<ConfigMenu> {
    move |input| {
        map(
            tuple((char('d'), integer, config_menu_line(line))),
            |(_, s, c)| ConfigMenu::DisableIf(s, Box::new(c)),
        )(input)
    }
}

/// Hide if.
fn hide_if(line: u8) -> impl FnMut(Input) -> Result<ConfigMenu> {
    move |input| {
        map(
            tuple((char('H'), integer, config_menu_line(line))),
            |(_, s, c)| ConfigMenu::HideIf(s, Box::new(c)),
        )(input)
    }
}

/// Hide unless.
fn hide_unless(line: u8) -> impl FnMut(Input) -> Result<ConfigMenu> {
    move |input| {
        map(
            tuple((char('h'), integer, config_menu_line(line))),
            |(_, s, c)| ConfigMenu::HideIf(s, Box::new(c)),
        )(input)
    }
}

/// File.
fn file<'a>(line: u8) -> impl FnMut(Input<'a>) -> Result<'a, ConfigMenu> {
    fn is_valid_filename_char(c: char) -> bool {
        c.is_alphanumeric() || c == '.' || c == '_' || c == ' '
    }

    preceded(
        char('F'),
        map(
            tuple((
                opt(char('C')),
                opt(char('S')),
                opt(integer),
                preceded(
                    char(','),
                    many1(recognize(tuple((
                        satisfy(is_valid_filename_char),
                        opt(satisfy(is_valid_filename_char)),
                        opt(satisfy(is_valid_filename_char)),
                    )))),
                ),
                opt(preceded(
                    char(','),
                    recognize(many1(satisfy(|c| c != ';' && c != ','))),
                )),
                opt(preceded(char(','), hex_u32)),
            )),
            move |(remember, save, index, ext, text, address)| {
                let extensions: Vec<FileExtension> = ext
                    .iter()
                    .map(|i| FileExtension::from_str(i.fragment()).unwrap())
                    .collect();
                let address = address.map(|a| FpgaRamMemoryAddress::try_from(a).unwrap());
                let index = match index {
                    Some(i) => i as u8,
                    None => line,
                };
                let load_file_info = LoadFileInfo {
                    save_support: save.is_some(),
                    index,
                    extensions,
                    label: text.map(|s: Input| s.to_string()),
                    address,
                };
                if remember.is_some() {
                    ConfigMenu::LoadFileAndRemember(Box::new(load_file_info))
                } else {
                    ConfigMenu::LoadFile(Box::new(load_file_info))
                }
            },
        ),
    )
}

fn sd_card(input: Input) -> Result<ConfigMenu> {
    fn is_valid_filename_char(c: char) -> bool {
        c.is_alphanumeric() || c == '.' || c == '_' || c == ' '
    }

    preceded(
        char('S'),
        map(
            tuple((
                integer,
                preceded(
                    char::<Input, _>(','),
                    many1(recognize(tuple((
                        satisfy(is_valid_filename_char),
                        opt(satisfy(is_valid_filename_char)),
                        opt(satisfy(is_valid_filename_char)),
                    )))),
                ),
                opt(preceded(char(','), recognize(many1(satisfy(|c| c != ';'))))),
            )),
            |(slot, ext, text)| {
                let extensions: Vec<FileExtension> = ext
                    .iter()
                    .map(|i| FileExtension::from_str(i.fragment()).unwrap())
                    .collect();
                let slot = slot as u8;

                ConfigMenu::MountSdCard {
                    slot,
                    extensions,
                    label: text.map(|s: Input| s.to_string()),
                }
            },
        ),
    )(input)
}

fn single_char_bit_index(input: Input) -> Result<u8> {
    map(one_of("0123456789ABCDEFGHIJKLMNOPQRSTUV"), |i| match i {
        '0'..='9' => i as u8 - b'0',
        'A'..='V' => i as u8 - b'A' + 10,
        _ => unreachable!(),
    })(input)
}

fn status_bit_index(input: Input) -> Result<u8> {
    alt((
        map(delimited(char('['), integer, char(']')), |i| i as u8),
        single_char_bit_index,
    ))(input)
}

fn status_bit_range(input: Input) -> Result<Range<u8>> {
    alt((
        map(
            delimited(
                char('['),
                separated_pair(integer, char(':'), integer),
                char(']'),
            ),
            |(b, a)| (a as u8)..(b as u8 + 1), // Bit ranges are inclusive.
        ),
        // Single bits are also accepted in ranges.
        map(
            pair(single_char_bit_index, single_char_bit_index),
            |(a, b)| a..(b + 1), // Bit ranges are inclusive.
        ),
        map(status_bit_index, |a| a..(a + 1)),
    ))(input)
}

fn option(input: Input) -> Result<ConfigMenu> {
    map(
        tuple((
            alt((
                preceded(char('O'), status_bit_range),
                preceded(
                    char('o'),
                    map(status_bit_range, |i| (i.start + 32)..(i.end + 32)),
                ),
            )),
            char(','),
            recognize(many0(satisfy(|c| c != ';' && c != ','))),
            char(','),
            separated_list0(
                char(','),
                recognize(many1(satisfy(|c| c != ';' && c != ','))),
            ),
        )),
        |(bits, _, label, _, choices)| ConfigMenu::Option {
            bits,
            label: label.to_string(),
            choices: choices.into_iter().map(|s: Input| s.to_string()).collect(),
        },
    )(input)
}

fn trigger(input: Input) -> Result<ConfigMenu> {
    map(
        tuple((
            alt((
                preceded(char('T'), status_bit_index),
                preceded(char('t'), map(status_bit_index, |i| i + 32)),
            )),
            recognize(many0(satisfy(|c| c != ';'))),
        )),
        |(index, label)| ConfigMenu::Trigger {
            close_osd: false,
            index,
            label: label.to_string(),
        },
    )(input)
}

fn reset(input: Input) -> Result<ConfigMenu> {
    map(
        tuple((
            alt((
                preceded(char('R'), status_bit_index),
                preceded(char('r'), map(status_bit_index, |i| i + 32)),
            )),
            char(','),
            recognize(many0(satisfy(|c| c != ';'))),
        )),
        |(index, _, label)| ConfigMenu::Trigger {
            close_osd: true,
            index,
            label: label.to_string(),
        },
    )(input)
}

fn info(input: Input) -> Result<ConfigMenu> {
    map(
        preceded(
            char::<Input, _>('I'),
            separated_list0(
                char(','),
                recognize(many0(satisfy(|i| i != ',' && i != ';'))),
            ),
        ),
        |lines| ConfigMenu::Info(lines.iter().map(|x| x.to_string()).collect()),
    )(input)
}

fn page(input: Input) -> Result<ConfigMenu> {
    map(
        preceded(
            char('P'),
            separated_pair(integer, char(','), recognize(many0(satisfy(|i| i != ';')))),
        ),
        |(index, label)| ConfigMenu::Page {
            index: index as u8,
            label: label.to_string(),
        },
    )(input)
}

fn page_item(line: u8) -> impl FnMut(Input) -> Result<ConfigMenu> {
    move |input| {
        map(
            preceded(char('P'), pair(integer, config_menu_line(line))),
            |(index, item)| ConfigMenu::PageItem(index as u8, Box::new(item)),
        )(input)
    }
}

fn joystick_buttons(input: Input) -> Result<ConfigMenu> {
    preceded(
        char('J'),
        map(
            separated_pair(
                opt(char::<Input, _>('1')),
                char(','),
                separated_list1(
                    char(','),
                    recognize(many0(satisfy(|i| i != ',' && i != ';'))),
                ),
            ),
            |(joy_emulation, buttons)| ConfigMenu::JoystickButtons {
                keyboard: joy_emulation.is_none(),
                buttons: buttons.iter().map(|x| x.to_string()).collect(),
            },
        ),
    )(input)
}

fn joystick_mapping_default(input: Input) -> Result<ConfigMenu> {
    preceded(
        tag("jn,"),
        map(
            separated_list1(
                char::<Input, _>(','),
                recognize(many0(satisfy(|i| i != ',' && i != ';'))),
            ),
            |buttons| ConfigMenu::SnesButtonDefaultList {
                buttons: buttons.iter().map(|x| x.to_string()).collect(),
            },
        ),
    )(input)
}

fn joystick_mapping_positional(input: Input) -> Result<ConfigMenu> {
    preceded(
        tag("jp,"),
        map(
            separated_list1(
                char::<Input, _>(','),
                recognize(many0(satisfy(|i| i != ',' && i != ';'))),
            ),
            |buttons| ConfigMenu::SnesButtonDefaultPositionalList {
                buttons: buttons.iter().map(|x| x.to_string()).collect(),
            },
        ),
    )(input)
}

fn version(input: Input) -> Result<ConfigMenu> {
    map(
        preceded(tag("V,"), recognize(many0(satisfy(|i| i != ';')))),
        |version: Input| ConfigMenu::Version(version.to_string()),
    )(input)
}

/// Parse a single line of the config menu. Separated by `;`.
fn config_menu_line(line: u8) -> impl FnMut(Input) -> Result<ConfigMenu> {
    move |input| {
        alt((
            separator,
            cheat,
            dip,
            disable_if(line),
            disable_unless(line),
            hide_if(line),
            hide_unless(line),
            file(line),
            sd_card,
            option,
            trigger,
            reset,
            info,
            page,
            page_item(line),
            joystick_buttons,
            joystick_mapping_default,
            joystick_mapping_positional,
            version,
        ))(input)
    }
}

pub fn parse_config_menu(input: Input) -> Result<(String, settings::Settings, Vec<ConfigMenu>)> {
    let mut line = 1;
    map(
        tuple((
            name,
            char(';'),
            core_settings,
            char(';'),
            separated_list0(char(';'), move |input| {
                match config_menu_line(line)(input) {
                    Ok((i, o)) => {
                        line += 1;
                        Ok((i, o))
                    }
                    Err(e) => Err(e),
                }
            }),
            opt(char(';')),
        )),
        |(a, _, c, _, e, _)| (a, c, e),
    )(input)
}
