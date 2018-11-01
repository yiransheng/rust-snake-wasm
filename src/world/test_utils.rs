use data::Coordinate;

pub fn chars_from_ascii_grid<'a>(
    string: &'a str,
) -> impl Iterator<Item = (Coordinate, char)> + 'a {
    string
        .lines()
        .filter(|line| !line.is_empty())
        .scan(0, |width, line| {
            let line_len = line.len();

            if *width == 0 || line_len == *width {
                *width = line_len;
                Some(line)
            } else {
                None
            }
        })
        .flat_map(|line| line.chars().enumerate())
        .enumerate()
        .map(|(line, (col, ch))| (Coordinate::from_usize(col, line), ch))
}
