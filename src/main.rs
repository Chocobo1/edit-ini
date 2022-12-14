#[derive(Debug, PartialEq)]
enum Action {
    Set,
    Remove,
}

impl std::str::FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "-s" | "--set" => Ok(Action::Set),
            "-r" | "--remove" => Ok(Action::Remove),
            _ => Err(String::from("Expected valid action")),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Command {
    action: Action,
    section: String,
    key: String,
    value: String,
}

impl Command {
    fn parse(s: &[&str]) -> Result<Self, ()> {
        // examples:
        // ["--set", "Section \"A\"   ", "Key=Value"]
        // ["--remove", "Section"]

        if !(1..=3).contains(&s.len()) {
            return Err(());
        }

        let mut key = "";
        let mut value = "";

        if let Some(kv_pair) = s.get(2) {
            let mut iter = kv_pair.splitn(2, '=');
            if let Some(k) = iter.next() {
                key = k;
            }
            if let Some(v) = iter.next() {
                value = v;
            }
        }

        Ok(Command {
            action: s.first().ok_or(())?.parse::<Action>().map_err(|_| ())?,
            section: String::from(*s.get(1).unwrap_or(&"")),
            key: String::from(key),
            value: String::from(value),
        })
    }
}

fn into_commands<I>(args: I) -> Vec<Command>
where
    I: Iterator<Item = String>,
{
    fn into_refs(input: &[String]) -> Vec<&str> {
        input.iter().map(AsRef::as_ref).collect()
    }

    let mut ret = Vec::new();
    let mut buffer = Vec::with_capacity(3);

    for value in args {
        if value.starts_with('-') {
            if let Ok(c) = Command::parse(&into_refs(&buffer)) {
                ret.push(c);
            }

            buffer.clear();
        }

        buffer.push(value);
    }

    if let Ok(c) = Command::parse(&into_refs(&buffer)) {
        ret.push(c);
    }

    ret
}

fn process_commands(
    commands: Vec<Command>,
    ini: &mut ini::Ini,
) -> Result<(), Box<dyn std::error::Error>> {
    for command in commands {
        let section = if command.section.is_empty() {
            None
        } else {
            Some(command.section)
        };

        match command.action {
            Action::Set => {
                if command.key.is_empty() {
                    continue;
                }

                ini.set_to(section, command.key, command.value);
            }

            Action::Remove => {
                if command.key.is_empty() {
                    ini.delete(section);
                } else if let Some(ini_section) = ini.section_mut(section.clone()) {
                    ini_section.remove(command.key);

                    if ini_section.is_empty() {
                        ini.delete(section);
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = clap::command!()
        .arg(
            clap::Arg::new("input")
                .short('i')
                .long("input")
                .value_names(["file"])
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Input file to read from. Use `-` to read from stdin.")
                .long_help(
                    "Input file to read from. Use `-` to read from stdin.
If this argument is omitted or any error occurred, a new ini data will be created.",
                ),
        )
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .value_names(["file"])
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Output file to write to")
                .long_help(
                    "Output file to write to.
If this argument is omitted, output will be sent to stdout.",
                ),
        )
        .arg(
            clap::Arg::new("set")
                .short('s')
                .long("set")
                .action(clap::ArgAction::Append)
                .num_args(1..=2)
                .value_names(["Section", "Key=Value"])
                .help("Set a section/key-value pair. Can be invoked multiple times.")
                .long_help(
                    "Set a section/key-value pair. Can be invoked multiple times.
If key already exists, its value will be overwritten.

Usage examples:
* Add a section: `-a House`
* Set a key-value pair: `-a House \"Cat=meow meow\"`",
                ),
        )
        .arg(
            clap::Arg::new("remove")
                .short('r')
                .long("remove")
                .action(clap::ArgAction::Append)
                .num_args(1..=2)
                .value_names(["Section", "Key=Value"])
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .help("Remove a section/key-value pair. Can be invoked multiple times.")
                .long_help(
                    "Remove a section/key-value pair. Can be invoked multiple times.

Usage examples:
* Remove a section: `-r House`
* Remove a key-value pair: `-r House Cat`
  This removes the key `Cat` (and its value) from `House` section.
  Note 1. `House` section will be removed if it doesn't contain any other key-value pairs.
  Note 2. If you only need to clear its value, use: `-a House Cat=`.",
                ),
        )
        .after_help("edit-ini homepage: <https://github.com/Chocobo1/edit-ini>");
    let matches = cmd.get_matches();

    let commands = into_commands(std::env::args().skip(1));
    //eprintln!("{commands:#?}");

    let mut ini = match matches.get_one::<String>("input") {
        Some(input) => {
            let parse_option = ini::ParseOption {
                enabled_quote: true,
                enabled_escape: false,
            };
            match input.as_str() {
                "-" => ini::Ini::read_from_opt(&mut std::io::stdin(), parse_option),
                _ => ini::Ini::load_from_file_opt(input, parse_option),
            }?
        }
        _ => ini::Ini::new(),
    };

    process_commands(commands, &mut ini)?;

    let output_policy = ini::EscapePolicy::Nothing;
    if let Some(output) = matches.get_one::<String>("output") {
        ini.write_to_file_policy(output, output_policy)?;
    } else {
        ini.write_to_policy(&mut std::io::stdout(), output_policy)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_action_from_str() -> Result<(), String> {
        assert!("".parse::<Action>().is_err());
        assert!("-xxx".parse::<Action>().is_err());
        assert!("set".parse::<Action>().is_err());
        assert_eq!("   -s ".parse::<Action>()?, Action::Set);
        assert_eq!(" --set   ".parse::<Action>()?, Action::Set);
        assert_eq!(" -r   ".parse::<Action>()?, Action::Remove);
        assert_eq!(" --remove   ".parse::<Action>()?, Action::Remove);

        Ok(())
    }

    #[test]
    fn test_command_parse() -> Result<(), ()> {
        assert!(Command::parse(&[]).is_err());
        assert!(Command::parse(&[""]).is_err());
        assert!(Command::parse(&["", ""]).is_err());
        assert!(Command::parse(&["", "", ""]).is_err());
        assert!(Command::parse(&["", "", "", ""]).is_err());
        assert!(Command::parse(&["-s", "", "", ""]).is_err());
        assert_eq!(
            Command::parse(&["  --set  ", ""])?,
            Command {
                action: Action::Set,
                section: String::from(""),
                key: String::from(""),
                value: String::from("")
            }
        );
        assert_eq!(
            Command::parse(&[
                "  --set  ",
                "Section \"A\"  ",
                "'Key1 vvv'   =   \"value1 x value2 x value3\""
            ])?,
            Command {
                action: Action::Set,
                section: String::from("Section \"A\"  "),
                key: String::from("'Key1 vvv'   "),
                value: String::from("   \"value1 x value2 x value3\"")
            }
        );
        assert_eq!(
            Command::parse(&["  --set  ", "s", "key1"])?,
            Command {
                action: Action::Set,
                section: String::from("s"),
                key: String::from("key1"),
                value: String::from("")
            }
        );
        assert_eq!(
            Command::parse(&["  --set  ", "   ", "key1======"])?,
            Command {
                action: Action::Set,
                section: String::from("   "),
                key: String::from("key1"),
                value: String::from("=====")
            }
        );

        assert_eq!(
            Command::parse(&["  --remove  "])?,
            Command {
                action: Action::Remove,
                section: String::from(""),
                key: String::from(""),
                value: String::from("")
            }
        );
        assert_eq!(
            Command::parse(&["  --remove  ", "Section"])?,
            Command {
                action: Action::Remove,
                section: String::from("Section"),
                key: String::from(""),
                value: String::from("")
            }
        );
        assert_eq!(
            Command::parse(&["  -r  ", ""])?,
            Command {
                action: Action::Remove,
                section: String::from(""),
                key: String::from(""),
                value: String::from("")
            }
        );
        assert_eq!(
            Command::parse(&["  -r  ", "sect", "aaa"])?,
            Command {
                action: Action::Remove,
                section: String::from("sect"),
                key: String::from("aaa"),
                value: String::from("")
            }
        );

        Ok(())
    }

    #[test]
    fn test_into_commands() -> Result<(), ()> {
        assert_eq!(into_commands(vec![].into_iter()), vec![]);
        assert_eq!(
            into_commands(
                vec![
                    String::from("-s"),
                    String::from("House"),
                    String::from("Cat=123"),
                    String::from("-s"),
                    String::from("House \"x\""),
                    String::from("Cat=456"),
                    String::from("-i"),
                    String::from("Ifile"),
                    String::from("-o"),
                    String::from("Ofile"),
                    String::from("-r"),
                    String::from("section1"),
                    String::from("-r"),
                    String::from("Section2"),
                    String::from("KeyX")
                ]
                .into_iter()
            ),
            vec![
                Command {
                    action: Action::Set,
                    section: String::from("House"),
                    key: String::from("Cat"),
                    value: String::from("123")
                },
                Command {
                    action: Action::Set,
                    section: String::from("House \"x\""),
                    key: String::from("Cat"),
                    value: String::from("456")
                },
                Command {
                    action: Action::Remove,
                    section: String::from("section1"),
                    key: String::from(""),
                    value: String::from("")
                },
                Command {
                    action: Action::Remove,
                    section: String::from("Section2"),
                    key: String::from("KeyX"),
                    value: String::from("")
                },
            ]
        );

        Ok(())
    }
}
