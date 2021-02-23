#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate bitflags;

use crate::uci::{ResponseType, UCIDriver};
use tokio::{
    fs::File,
    io::{self, stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
};

mod bitboard;
mod board;
mod evaluation;
mod interface;
mod magic_bitboard;
mod move_gen;
mod search;
mod uci;
mod util;

// struct GameState {
//     board: Board,
//     first_selected: Option<usize>,
//     moves: Vec<Move>,
//     reverse_board: bool,
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stdin());
    let mut writer = BufWriter::new(stdout());
    let mut buffer = String::new();

    let mut log = tokio::fs::File::create("log.txt").await?;

    let mut uci_driver = UCIDriver::new();

    loop {
        reader.read_line(&mut buffer).await?;
        log.write(buffer.as_bytes()).await?;

        match uci_driver.parse_command(&buffer) {
            ResponseType::Print(response) => {
                writer.write(response.as_bytes()).await?;
                writer.write(b"\n").await?;
                writer.flush().await?;
            }
            ResponseType::Nothing => {}
            ResponseType::Quit => break,
        };

        buffer.clear();
    }

    Ok(())
}

// fn main() -> Result<(), Box<dyn Error>> {
//
//     // if false {
//     //     let mut siv = Cursive::new();
//     //     siv.load_toml(include_str!("theme.toml")).unwrap();
//     //
//     //     let mut board = Board::default();
//     //     let moves = generate_legal_moves(&mut board);
//     //
//     //     let game_state = GameState {
//     //         board,
//     //         first_selected: None,
//     //         moves,
//     //         reverse_board: false,
//     //     };
//     //
//     //     let board = Panel::new(
//     //         Canvas::new(game_state)
//     //             .with_draw(|game_state: &GameState, printer| {
//     //                 for rank in 0..8 {
//     //                     let rank_str = format!(" {} ", rank + 1);
//     //                     if game_state.reverse_board {
//     //                         printer.print((0, rank), &*rank_str);
//     //                     } else {
//     //                         printer.print((0, 7 - rank), &*rank_str);
//     //                     }
//     //                     for file in 0..8 {
//     //                         let light_square = (file + rank) % 2 != 0;
//     //
//     //                         let background = match game_state.first_selected {
//     //                             Some(index) if index == rank * 8 + file => {
//     //                                 theme::Color::Rgb(239, 138, 50)
//     //                             }
//     //                             Some(index)
//     //                                 if game_state
//     //                                     .moves
//     //                                     .contains(&Move::new(index, rank * 8 + file)) =>
//     //                             {
//     //                                 if light_square {
//     //                                     theme::Color::Rgb(255, 158, 158)
//     //                                 } else {
//     //                                     theme::Color::Rgb(209, 71, 71)
//     //                                 }
//     //                             }
//     //                             _ => {
//     //                                 if light_square {
//     //                                     theme::Color::Rgb(255, 206, 158)
//     //                                 } else {
//     //                                     theme::Color::Rgb(209, 139, 71)
//     //                                 }
//     //                             }
//     //                         };
//     //
//     //                         let foreground = theme::Color::Rgb(0, 0, 0);
//     //
//     //                         let v = match game_state.board.pieces[rank * 8 + file] {
//     //                             Some((color, piece)) => {
//     //                                 format!(" {} ", UNICODE_PIECES[color as usize][piece as usize])
//     //                             }
//     //                             None => format!("   "),
//     //                         };
//     //
//     //                         if game_state.reverse_board {
//     //                             printer.with_color(
//     //                                 ColorStyle::new(foreground, background),
//     //                                 |printer| printer.print((3 + file * 3, rank), v.as_str()),
//     //                             );
//     //                         } else {
//     //                             printer.with_color(
//     //                                 ColorStyle::new(foreground, background),
//     //                                 |printer| printer.print((3 + file * 3, 7 - rank), v.as_str()),
//     //                             );
//     //                         }
//     //                     }
//     //                 }
//     //                 for file in 0..8 {
//     //                     let file_str = format!(" {} ", (b'a' + file) as char);
//     //                     if game_state.reverse_board {
//     //                         printer.print(((7 - file) * 3 + 3, 8), &*file_str);
//     //                     } else {
//     //                         printer.print((file * 3 + 3, 8), &*file_str);
//     //                     }
//     //                 }
//     //
//     //                 let debug_str = format!("{:?}", game_state.board.en_passant);
//     //                 printer.print((0, 9), &debug_str);
//     //             })
//     //             .with_on_event(|game_state: &mut GameState, event| match event {
//     //                 Event::Mouse {
//     //                     offset,
//     //                     position,
//     //                     event: MouseEvent::Press(MouseButton::Left),
//     //                 } => {
//     //                     if position.y >= offset.y && position.x >= offset.x {
//     //                         let pos = position - offset;
//     //                         if pos.x < 3 {
//     //                             return EventResult::Ignored;
//     //                         }
//     //                         let pos_x = (pos.x - 3) / 3;
//     //                         let pos_y = if game_state.reverse_board {
//     //                             7 - pos.y
//     //                         } else {
//     //                             pos.y
//     //                         };
//     //
//     //                         if !(pos_x > 7 || pos_y > 7) {
//     //                             let index = ((7 - pos_y) * 8) + pos_x;
//     //                             if let Some(first) = game_state.first_selected {
//     //                                 if first != index {
//     //                                     if let Some(m) = game_state
//     //                                         .moves
//     //                                         .iter()
//     //                                         .position(|&m| &m == &Move::new(first, index))
//     //                                     {
//     //                                         game_state.board.move_piece(&game_state.moves[m]);
//     //                                         game_state.moves =
//     //                                             generate_legal_moves(&mut game_state.board);
//     //                                     }
//     //                                 }
//     //                                 game_state.first_selected = None;
//     //                             } else {
//     //                                 if let Some((color, _)) = game_state.board.pieces[index] {
//     //                                     if color == game_state.board.active_color {
//     //                                         game_state.first_selected = Some(index);
//     //                                     }
//     //                                 }
//     //                             }
//     //                             return EventResult::Consumed(None);
//     //                         }
//     //                     }
//     //                     EventResult::Ignored
//     //                 }
//     //                 Event::Mouse {
//     //                     event: MouseEvent::Press(MouseButton::Right),
//     //                     ..
//     //                 } => {
//     //                     game_state.first_selected = None;
//     //                     EventResult::Consumed(None)
//     //                 }
//     //
//     //                 _ => EventResult::Ignored,
//     //             })
//     //             .with_required_size(|_text, _constraints| (4 + 8 * 3, 8 + 1 + 1).into()),
//     //     );
//     //
//     //     siv.add_layer(board);
//     //
//     //     siv.add_global_callback('q', |s| s.quit());
//     //
//     //     siv.run();
//     // }
//
//     // let mut board = Board::default();
//     //
//     // loop {
//     //     let moves = gen_moves(&board);
//     //
//     //     let (start, end) = 'outer_loop: loop {
//     //         println!("\x1B[2J");
//     //         print_board(&board);
//     //
//     //         let mut input_start = String::new();
//     //         print!("({}) From: ", &board.active_color);
//     //         stdout().flush()?;
//     //         stdin().read_line(&mut input_start)?;
//     //
//     //         match input_start.trim().as_bytes() {
//     //             b"q" => return Ok(()),
//     //             [_, _] => {
//     //                 if let Ok(start) = algebraic_to_index(&input_start.trim()[0..2]) {
//     //                     break (
//     //                         start,
//     //                         loop {
//     //                             println!("\x1B[2J");
//     //                             print_board(&board);
//     //
//     //                             let mut input_end = String::new();
//     //                             print!(
//     //                                 "({}) From: {} End: ",
//     //                                 &board.active_color,
//     //                                 index_to_algebraic(start)
//     //                             );
//     //                             stdout().flush()?;
//     //                             stdin().read_line(&mut input_end)?;
//     //
//     //                             match input_end.trim().as_bytes() {
//     //                                 b"b" => continue 'outer_loop,
//     //                                 b"q" => return Ok(()),
//     //                                 [_, _] => {
//     //                                     if let Ok(end) = algebraic_to_index(&input_end.trim()[0..2])
//     //                                     {
//     //                                         break end;
//     //                                     }
//     //                                 }
//     //                                 _ => {}
//     //                             }
//     //                         },
//     //                     );
//     //                 }
//     //             }
//     //             _ => {}
//     //         }
//     //     };
//     //
//     //     board.pieces[end] = board.pieces[start];
//     //     board.pieces[start] = None;
//     //     board.active_color = if board.active_color == Color::White {
//     //         Color::Black
//     //     } else {
//     //         Color::White
//     //     };
//     // }
//
//     Ok(())
// }
