use amaze::generators::RecursiveBacktracker4;
use amaze::renderers::{UnicodeRenderStyle, UnicodeRenderer};
use clap::{value_parser, Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("amaze-cli")
        .about("A Maze Generator")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("Markus Mayer")
        .subcommand(
            Command::new("gen")
                .short_flag('g')
                .long_flag("generate")
                .about("Generate a maze.")
                .arg(
                    Arg::new("algorithm")
                        .short('a')
                        .long("algorithm")
                        .help("selects the algorithm to use")
                        .display_order(0)
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("seed")
                        .short('s')
                        .long("seed")
                        .help("selects the seed to use")
                        .display_order(1)
                        .value_parser(value_parser!(u64))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("width")
                        .short('W')
                        .long("width")
                        .help("selects the width of the maze")
                        .display_order(2)
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("height")
                        .short('H')
                        .long("height")
                        .help("selects the height of the maze")
                        .display_order(3)
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("style")
                        .long("style")
                        .help("selects the style to use")
                        .display_order(4)
                        .action(ArgAction::Set),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("gen", gen_matches)) => {
            let seed: u64 = *gen_matches.get_one::<u64>("seed").unwrap_or(&0u64);
            let width = gen_matches.get_one::<usize>("width").unwrap_or(&8usize);
            let height = gen_matches.get_one::<usize>("height").unwrap_or(&8usize);

            let default_algorithm = String::from("recursive-backtracker");
            let default_style = String::from("heavy");
            let algorithm = gen_matches
                .get_one::<String>("algorithm")
                .unwrap_or(&default_algorithm);
            let style = gen_matches
                .get_one::<String>("style")
                .unwrap_or(&default_style);
            let generator = match algorithm.as_str() {
                "recursive-backtracker" => RecursiveBacktracker4::new_from_seed(seed),
                _ => unreachable!(),
            };
            let style = match style.as_str() {
                "heavy" => UnicodeRenderStyle::Heavy,
                "thin" => UnicodeRenderStyle::Thin,
                "double" => UnicodeRenderStyle::Double,
                "hex" => UnicodeRenderStyle::Hexadecimal,
                _ => UnicodeRenderStyle::Heavy,
            };
            let grid = generator.generate(*width, *height);
            let renderer = UnicodeRenderer::new(style, true);
            println!("{}", renderer.render(&grid));
            return;
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
}
