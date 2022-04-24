mod direction4;
mod grid_coord_2d;
mod maze_generator;
mod recursive_backtracker4;
mod render_unicode;
mod room4;
mod room4_list;
mod visit_map_2d;
mod wall4_grid;

pub fn double(x: i64) -> i64 {
    x * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(double(2), 4);
    }
}
