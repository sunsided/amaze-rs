use amaze::dungeon::{DungeonGrid, DungeonType, DungeonWalkGenerator, TileType};
use amaze::generators::{
    BinaryTree4, Eller4, GrowingTree4, HuntAndKill4, Kruskal4, MazeGenerator2D, MixedCell,
    RecursiveBacktracker4, Sidewinder4, Wilson4,
};
use amaze::preamble::*;
use amaze::renderers::{ImageRenderer, RenderStyle, UnicodeRenderer};
use clap::{value_parser, Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("amaze-cli")
        .about("A Maze Generator")
        .version("0.2.0")
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
                        .default_value("recursive-backtracker")
                        .value_parser([
                            "recursive-backtracker",
                            "growing-tree",
                            "growing-tree-mixed",
                            "kruskal",
                            "eller",
                            "wilson",
                            "hunt-and-kill",
                            "sidewinder",
                            "binary-tree",
                        ])
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
                        .value_parser(value_parser!(RenderStyle))
                        .default_value("heavy")
                        .action(ArgAction::Set),
                ),
        )
        .subcommand(
            Command::new("gen-dungeon")
                .about("Generate a dungeon.")
                .arg(
                    Arg::new("type")
                        .short('t')
                        .long("type")
                        .help("selects the dungeon type")
                        .display_order(0)
                        .default_value("rooms")
                        .value_parser(["caverns", "rooms", "winding"])
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
                        .help("selects the width of the dungeon")
                        .display_order(2)
                        .default_value("40")
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("height")
                        .short('H')
                        .long("height")
                        .help("selects the height of the dungeon")
                        .display_order(3)
                        .default_value("30")
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("floor-count")
                        .short('f')
                        .long("floor-count")
                        .help("target number of floor tiles to generate")
                        .display_order(4)
                        .default_value("200")
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("winding-probability")
                        .short('p')
                        .long("winding-probability")
                        .help("winding probability (0-99), only affects winding type")
                        .display_order(5)
                        .default_value("50")
                        .value_parser(value_parser!(u8))
                        .action(ArgAction::Set),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("gen", gen_matches)) => {
            let seed: u64 = *gen_matches.get_one::<u64>("seed").unwrap_or(&0u64);
            let width = gen_matches.get_one::<usize>("width").unwrap_or(&8usize);
            let height = gen_matches.get_one::<usize>("height").unwrap_or(&8usize);

            let default_style = RenderStyle::default();
            let algorithm = gen_matches
                .get_one::<String>("algorithm")
                .expect("defaulted");
            let style = gen_matches
                .get_one::<RenderStyle>("style")
                .unwrap_or(&default_style);
            let grid =
                match algorithm.as_str() {
                    "recursive-backtracker" => {
                        RecursiveBacktracker4::new_from_seed(seed).generate(*width, *height)
                    }
                    "growing-tree" => <GrowingTree4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height),
                    "growing-tree-mixed" => GrowingTree4::new_from_seed_with_selector(
                        seed,
                        MixedCell {
                            newest_probability: 0.7,
                        },
                    )
                    .generate(*width, *height),
                    "kruskal" => {
                        <Kruskal4 as MazeGenerator2D>::new_from_seed(seed).generate(*width, *height)
                    }
                    "eller" => {
                        <Eller4 as MazeGenerator2D>::new_from_seed(seed).generate(*width, *height)
                    }
                    "wilson" => {
                        <Wilson4 as MazeGenerator2D>::new_from_seed(seed).generate(*width, *height)
                    }
                    "hunt-and-kill" => <HuntAndKill4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height),
                    "sidewinder" => <Sidewinder4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height),
                    "binary-tree" => <BinaryTree4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height),
                    _ => unreachable!(),
                };
            match style {
                RenderStyle::Unicode(style) => {
                    let renderer = UnicodeRenderer::new(*style, true);
                    println!("{}", renderer.render(&grid).trim_end());
                }
                RenderStyle::Image(style) => {
                    let renderer = ImageRenderer::new(*style);
                    println!("{}", renderer.render(&grid).trim_end());
                }
            }
        }
        Some(("gen-dungeon", dungeon_matches)) => {
            let seed: u64 = *dungeon_matches.get_one::<u64>("seed").unwrap_or(&0u64);
            let width = *dungeon_matches
                .get_one::<usize>("width")
                .expect("defaulted");
            let height = *dungeon_matches
                .get_one::<usize>("height")
                .expect("defaulted");
            let floor_count = *dungeon_matches
                .get_one::<usize>("floor-count")
                .expect("defaulted");
            let winding_probability = *dungeon_matches
                .get_one::<u8>("winding-probability")
                .expect("defaulted");

            let dungeon_type_str = dungeon_matches
                .get_one::<String>("type")
                .expect("defaulted");
            let dungeon_type = match dungeon_type_str.as_str() {
                "caverns" => DungeonType::Caverns,
                "rooms" => DungeonType::Rooms,
                "winding" => DungeonType::Winding,
                _ => unreachable!(),
            };

            let dungeon = DungeonWalkGenerator::new_from_seed(dungeon_type, seed)
                .with_winding_probability(winding_probability)
                .generate(width, height, floor_count);

            println!("{}", render_dungeon(&dungeon));
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable
    }
}

fn render_dungeon(dungeon: &DungeonGrid) -> String {
    let mut output = String::new();

    for y in 0..dungeon.height() {
        for x in 0..dungeon.width() {
            let coord = GridCoord2D::new(x, y);
            let tile = dungeon.get(coord).expect("coord in bounds");
            let is_exit = dungeon.exit() == Some(coord);

            let ch = match tile {
                TileType::Wall => '#',
                TileType::Floor => {
                    if is_exit {
                        'E'
                    } else {
                        '.'
                    }
                }
                TileType::Empty => ' ',
            };
            output.push(ch);
        }
        output.push('\n');
    }

    output
}
