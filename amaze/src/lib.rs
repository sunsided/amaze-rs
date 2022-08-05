pub mod direction4;
mod generators;
mod grid_coord_2d;
mod room4;
mod room4_list;
mod unicode_renderer;
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
