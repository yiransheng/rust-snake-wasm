use std::io::{StdoutLock, Write};

use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor};

use snake_wasm::data::{Direction, SmallNat};
use snake_wasm::{Color as GameColor, DrawGrid, UnitInterval};

pub struct TermEnv<W: Write> {
    stdout: W,
    color: GameColor,
    width: SmallNat,
    height: SmallNat,
}
impl<W: Write> Drop for TermEnv<W> {
    fn drop(&mut self) {
        write!(
            self.stdout,
            "{}{}{}",
            cursor::Goto(1, 1),
            clear::All,
            cursor::Show,
        );
    }
}
impl<'a> TermEnv<RawTerminal<StdoutLock<'a>>> {
    pub fn wrap(stdout: StdoutLock<'a>) -> Self {
        let stdout = stdout.into_raw_mode().unwrap();

        TermEnv {
            stdout,
            color: GameColor::Black,
            width: 10,
            height: 10,
        }
    }
}

impl<W: Write> DrawGrid for TermEnv<W> {
    fn setup(
        &mut self,
        _tile_size: SmallNat,
        width: SmallNat,
        height: SmallNat,
    ) {
        self.width = width;
        self.height = height;

        self.clear();
        write!(self.stdout, "{}", cursor::Hide);

        writeln!(
            self.stdout,
            "{}Movements: h,j,k,l Quit: q Restart: Space",
            cursor::Goto(1, self.height + 2),
        )
        .unwrap();
    }

    fn clear(&mut self) {
        write!(self.stdout, "{}", clear::All);

        for x in 0..self.width {
            for y in 0..self.height {
                write!(self.stdout, "{}.", cursor::Goto(x + 1, y + 1)).unwrap();
            }
        }
    }

    fn set_fill_color(&mut self, color: GameColor) -> GameColor {
        let prev_color = self.color;
        self.color = color;

        prev_color
    }

    fn circle(&mut self, x: SmallNat, y: SmallNat, _radius: UnitInterval) {
        write!(self.stdout, "{}*", cursor::Goto(x + 1, y + 1)).unwrap();
    }

    fn fill_tile(
        &mut self,
        x: SmallNat,
        y: SmallNat,
        dir: Direction,
        size: UnitInterval,
    ) {
        let c = if size.scale(10.0) > 7.5 {
            'o'
        } else {
            match dir {
                Direction::East | Direction::West => '-',
                _ => '|',
            }
        };

        write!(self.stdout, "{}{}\n", cursor::Goto(x + 1, y + 1), c,).unwrap();
    }

    fn clear_tile(
        &mut self,
        x: SmallNat,
        y: SmallNat,
        _dir: Direction,
        _size: UnitInterval,
    ) {
        write!(self.stdout, "{}.", cursor::Goto(x + 1, y + 1)).unwrap();
    }

    fn show_game_over(&mut self) {}
}
