use amaze::dungeon::{DungeonGrid, DungeonType, DungeonWalkGenerator, TileType};
#[cfg(feature = "generators-hex")]
use amaze::generators::{AldousBroder6, GrowingTree6, MazeGenerator6D, RecursiveBacktracker6};
use amaze::generators::{
    BinaryTree4, Eller4, GrowingTree4, HuntAndKill4, Kruskal4, MazeGenerator2D, MixedCell, Prim4,
    RecursiveBacktracker4, Sidewinder4, Wilson4,
};
use amaze::preamble::*;
use amaze::renderers::{ImageRenderer, RenderStyle, UnicodeRenderer};
use clap::{Arg, ArgAction, Command, value_parser};

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
                            "prim",
                            "hex-recursive-backtracker",
                            "hex-growing-tree",
                            "hex-aldous-broder",
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
                        .value_parser(clap::value_parser!(u8).range(0..=99))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("long-walk-min")
                        .long("long-walk-min")
                        .help("minimum long walk distance (only affects rooms/winding types)")
                        .display_order(6)
                        .default_value("9")
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("long-walk-max")
                        .long("long-walk-max")
                        .help("maximum long walk distance (exclusive, only affects rooms/winding types)")
                        .display_order(7)
                        .default_value("18")
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("dynamic")
                        .long("dynamic")
                        .help("enable dynamic grid resizing (starts small, expands as needed)")
                        .display_order(8)
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("initial-size")
                        .long("initial-size")
                        .help("initial grid size when dynamic resize is enabled")
                        .display_order(9)
                        .default_value("32")
                        .value_parser(value_parser!(usize))
                        .action(ArgAction::Set),
                )
                .arg(
                    Arg::new("trim-padding")
                        .long("trim-padding")
                        .help("padding around final trimmed dungeon")
                        .display_order(10)
                        .default_value("0")
                        .value_parser(value_parser!(usize))
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

            #[cfg(not(feature = "generators-hex"))]
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
                    "prim" => {
                        <Prim4 as MazeGenerator2D>::new_from_seed(seed).generate(*width, *height)
                    }
                    _ => unreachable!(),
                };

            #[cfg(not(feature = "generators-hex"))]
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

            #[cfg(feature = "generators-hex")]
            match algorithm.as_str() {
                "recursive-backtracker" => {
                    let grid = RecursiveBacktracker4::new_from_seed(seed).generate(*width, *height);
                    render_grid(&grid, style);
                }
                "growing-tree" => {
                    let grid = <GrowingTree4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height);
                    render_grid(&grid, style);
                }
                "growing-tree-mixed" => {
                    let grid = GrowingTree4::new_from_seed_with_selector(
                        seed,
                        MixedCell {
                            newest_probability: 0.7,
                        },
                    )
                    .generate(*width, *height);
                    render_grid(&grid, style);
                }
                "kruskal" => {
                    let grid = <Kruskal4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height);
                    render_grid(&grid, style);
                }
                "eller" => {
                    let grid =
                        <Eller4 as MazeGenerator2D>::new_from_seed(seed).generate(*width, *height);
                    render_grid(&grid, style);
                }
                "wilson" => {
                    let grid =
                        <Wilson4 as MazeGenerator2D>::new_from_seed(seed).generate(*width, *height);
                    render_grid(&grid, style);
                }
                "hunt-and-kill" => {
                    let grid = <HuntAndKill4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height);
                    render_grid(&grid, style);
                }
                "sidewinder" => {
                    let grid = <Sidewinder4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height);
                    render_grid(&grid, style);
                }
                "binary-tree" => {
                    let grid = <BinaryTree4 as MazeGenerator2D>::new_from_seed(seed)
                        .generate(*width, *height);
                    render_grid(&grid, style);
                }
                "prim" => {
                    let grid =
                        <Prim4 as MazeGenerator2D>::new_from_seed(seed).generate(*width, *height);
                    render_grid(&grid, style);
                }
                "hex-recursive-backtracker" => {
                    let grid = RecursiveBacktracker6::new_from_seed(seed).generate(*width, *height);
                    render_hex_grid(&grid);
                }
                "hex-growing-tree" => {
                    let grid = <GrowingTree6 as MazeGenerator6D>::new_from_seed(seed)
                        .generate(*width, *height);
                    render_hex_grid(&grid);
                }
                "hex-aldous-broder" => {
                    let grid = AldousBroder6::new_from_seed(seed).generate(*width, *height);
                    render_hex_grid(&grid);
                }
                _ => unreachable!(),
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
            let long_walk_min = *dungeon_matches
                .get_one::<usize>("long-walk-min")
                .expect("defaulted");
            let long_walk_max = *dungeon_matches
                .get_one::<usize>("long-walk-max")
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
                .with_long_walk_range(long_walk_min, long_walk_max)
                .with_dynamic_resize(*dungeon_matches.get_one::<bool>("dynamic").unwrap_or(&false))
                .with_initial_grid_size(
                    *dungeon_matches
                        .get_one::<usize>("initial-size")
                        .unwrap_or(&32),
                )
                .with_trim_padding(
                    *dungeon_matches
                        .get_one::<usize>("trim-padding")
                        .unwrap_or(&0),
                )
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

#[cfg(feature = "generators-hex")]
fn render_grid(grid: &Wall4Grid, style: &RenderStyle) {
    match style {
        RenderStyle::Unicode(style) => {
            let renderer = UnicodeRenderer::new(*style, true);
            println!("{}", renderer.render(grid).trim_end());
        }
        RenderStyle::Image(style) => {
            let renderer = ImageRenderer::new(*style);
            println!("{}", renderer.render(grid).trim_end());
        }
    }
}

#[cfg(feature = "generators-hex")]
fn render_hex_grid(grid: &Wall6Grid) {
    let mut output = String::new();
    for r in 0..grid.height() {
        if r % 2 == 1 {
            output.push(' ');
        }
        for q in 0..grid.width() {
            let coord = HexCoord::new(q as isize, r as isize);
            let walls = grid.get(coord).expect("coord in bounds");
            output.push_str(&format!("{:02X} ", { **walls }));
        }
        output.push('\n');
    }
    println!("{}", output.trim_end());
}
