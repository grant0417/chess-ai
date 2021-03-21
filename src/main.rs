#![allow(dead_code, unused_imports)]

use std::{error::Error, str::from_utf8};

use crate::interface::*;
use crate::uci::{ResponseType, UCIDriver};
use bitboard::{generate_moves, perft, BitBoardMove, BitBoardState};
use board::{Board, Color};
use clap::{App, Arg};
use tokio::{
    fs::File,
    io::{self, stdin, stdout, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
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

pub const APPLICATION_VERSION: &'static str = "0.0.1";
pub const APPLICATION_NAME: &'static str = "Grants's AI";
pub const APPLICATION_AUTHOR: &'static str = "Grant";
pub const APPLICATION_ABOUT: &'static str = "This is a chess AI";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(APPLICATION_NAME)
        .version(APPLICATION_VERSION)
        .author(APPLICATION_AUTHOR)
        .about(APPLICATION_ABOUT)
        .arg(
            Arg::with_name("cli")
                .short("c")
                .long("cli")
                .help("This uses the cli interface instead of UCI"),
        )
        .arg(
            Arg::with_name("fen")
                .short("f")
                .long("fen")
                .help("This suplies a custom fen for the cli interface")
                .takes_value(true),
        )
        .get_matches();

    let cli_inteface = matches.occurrences_of("cli");
    let fen = matches.value_of("fen");

    let mut reader = BufReader::new(stdin());
    let mut writer = BufWriter::new(stdout());
    let mut buffer = String::new();
    let mut log = tokio::fs::File::create("log.txt").await?;

    if cli_inteface == 0 {
        let mut uci_driver = UCIDriver::new();

        loop {
            reader.read_line(&mut buffer).await?;
            log.write(b"> ").await?;
            log.write(buffer.as_bytes()).await?;

            match uci_driver.parse_command(&buffer) {
                ResponseType::Response(response) => {
                    writer.write(response.as_bytes()).await?;
                    writer.write(b"\n").await?;
                    writer.flush().await?;
                    log.write(response.as_bytes()).await?;
                    log.write(b"\n").await?;
                    log.flush().await?;
                }
                ResponseType::Log(s) => {
                    log.write(b"!").await?;
                    log.write(s.as_bytes()).await?;
                    log.write(b"\n").await?;
                    log.flush().await?;
                }
                ResponseType::ResponseLog(response, s) => {
                    writer.write(response.as_bytes()).await?;
                    writer.write(b"\n").await?;
                    writer.flush().await?;
                    log.write(b"!").await?;
                    log.write(s.as_bytes()).await?;
                    log.write(b"\n").await?;
                    log.write(response.as_bytes()).await?;
                    log.write(b"\n").await?;
                    log.flush().await?;
                }
                ResponseType::Nothing => {
                    log.write(b"!Nothing\n").await?;
                }
                ResponseType::Quit => {
                    log.write(b"!Quit\n").await?;
                    break;
                }
            };

            buffer.clear();
        }
    } else {
        let mut bit_board = if let Some(f) = fen {
            BitBoardState::from_fen(f)?
        } else {
            BitBoardState::new()
        };

        'main_loop: loop {
            let moves = generate_moves(&bit_board);

            let (start, end) = 'start_end_loop: loop {
                println!("\x1B[2J");
                bit_board.bitboard.print_board(None, None);

                writer
                    .write(format!("({}) From: ", &bit_board.active_color).as_bytes())
                    .await?;
                writer.flush().await?;

                buffer.clear();
                reader.read_line(&mut buffer).await?;

                match buffer.trim().as_bytes() {
                    b"q" => break 'main_loop,
                    [_, _] => {
                        if let Ok(start) = algebraic_to_index(&buffer.trim()[0..2].as_bytes()) {
                            break (
                                start,
                                loop {
                                    println!("\x1B[2J");
                                    bit_board.bitboard.print_board(Some(start), Some(&moves));

                                    let algebraic = index_to_algebraic(start);
                                    let algebraic_str = from_utf8(&algebraic)?;

                                    writer
                                        .write(
                                            format!(
                                                "({}) From: {} End: ",
                                                &bit_board.active_color, algebraic_str
                                            )
                                            .as_bytes(),
                                        )
                                        .await?;
                                    writer.flush().await?;

                                    buffer.clear();
                                    reader.read_line(&mut buffer).await?;

                                    match buffer.trim().as_bytes() {
                                        b"b" => continue 'start_end_loop,
                                        b"q" => break 'main_loop,
                                        [_, _] => {
                                            if let Ok(end) =
                                                algebraic_to_index(&buffer.trim()[0..2].as_bytes())
                                            {
                                                if moves.contains(&BitBoardMove::new(
                                                    start as u16,
                                                    end as u16,
                                                    0,
                                                )) {
                                                    break end;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                },
                            );
                        }
                    }
                    _ => {}
                }
            };

            let pos = moves
                .iter()
                .position(|m| m == &BitBoardMove::new(start as u16, end as u16, 0))
                .unwrap();

            bit_board.apply_move(&moves[pos]);
            bit_board.mirror_board();
            //bit_board.change_side();
        }
    }

    Ok(())
}

// if false {
//     let mut siv = Cursive::new();
//     siv.load_toml(include_str!("theme.toml")).unwrap();
//
//     let mut board = Board::default();
//     let moves = generate_legal_moves(&mut board);
//
//     let game_state = GameState {
//         board,
//         first_selected: None,
//         moves,
//         reverse_board: false,
//     };
//
//     let board = Panel::new(
//         Canvas::new(game_state)
//             .with_draw(|game_state: &GameState, printer| {
//                 for rank in 0..8 {
//                     let rank_str = format!(" {} ", rank + 1);
//                     if game_state.reverse_board {
//                         printer.print((0, rank), &*rank_str);
//                     } else {
//                         printer.print((0, 7 - rank), &*rank_str);
//                     }
//                     for file in 0..8 {
//                         let light_square = (file + rank) % 2 != 0;
//
//                         let background = match game_state.first_selected {
//                             Some(index) if index == rank * 8 + file => {
//                                 theme::Color::Rgb(239, 138, 50)
//                             }
//                             Some(index)
//                                 if game_state
//                                     .moves
//                                     .contains(&Move::new(index, rank * 8 + file)) =>
//                             {
//                                 if light_square {
//                                     theme::Color::Rgb(255, 158, 158)
//                                 } else {
//                                     theme::Color::Rgb(209, 71, 71)
//                                 }
//                             }
//                             _ => {
//                                 if light_square {
//                                     theme::Color::Rgb(255, 206, 158)
//                                 } else {
//                                     theme::Color::Rgb(209, 139, 71)
//                                 }
//                             }
//                         };
//
//                         let foreground = theme::Color::Rgb(0, 0, 0);
//
//                         let v = match game_state.board.pieces[rank * 8 + file] {
//                             Some((color, piece)) => {
//                                 format!(" {} ", UNICODE_PIECES[color as usize][piece as usize])
//                             }
//                             None => format!("   "),
//                         };
//
//                         if game_state.reverse_board {
//                             printer.with_color(
//                                 ColorStyle::new(foreground, background),
//                                 |printer| printer.print((3 + file * 3, rank), v.as_str()),
//                             );
//                         } else {
//                             printer.with_color(
//                                 ColorStyle::new(foreground, background),
//                                 |printer| printer.print((3 + file * 3, 7 - rank), v.as_str()),
//                             );
//                         }
//                     }
//                 }
//                 for file in 0..8 {
//                     let file_str = format!(" {} ", (b'a' + file) as char);
//                     if game_state.reverse_board {
//                         printer.print(((7 - file) * 3 + 3, 8), &*file_str);
//                     } else {
//                         printer.print((file * 3 + 3, 8), &*file_str);
//                     }
//                 }
//
//                 let debug_str = format!("{:?}", game_state.board.en_passant);
//                 printer.print((0, 9), &debug_str);
//             })
//             .with_on_event(|game_state: &mut GameState, event| match event {
//                 Event::Mouse {
//                     offset,
//                     position,
//                     event: MouseEvent::Press(MouseButton::Left),
//                 } => {
//                     if position.y >= offset.y && position.x >= offset.x {
//                         let pos = position - offset;
//                         if pos.x < 3 {
//                             return EventResult::Ignored;
//                         }
//                         let pos_x = (pos.x - 3) / 3;
//                         let pos_y = if game_state.reverse_board {
//                             7 - pos.y
//                         } else {
//                             pos.y
//                         };
//
//                         if !(pos_x > 7 || pos_y > 7) {
//                             let index = ((7 - pos_y) * 8) + pos_x;
//                             if let Some(first) = game_state.first_selected {
//                                 if first != index {
//                                     if let Some(m) = game_state
//                                         .moves
//                                         .iter()
//                                         .position(|&m| &m == &Move::new(first, index))
//                                     {
//                                         game_state.board.move_piece(&game_state.moves[m]);
//                                         game_state.moves =
//                                             generate_legal_moves(&mut game_state.board);
//                                     }
//                                 }
//                                 game_state.first_selected = None;
//                             } else {
//                                 if let Some((color, _)) = game_state.board.pieces[index] {
//                                     if color == game_state.board.active_color {
//                                         game_state.first_selected = Some(index);
//                                     }
//                                 }
//                             }
//                             return EventResult::Consumed(None);
//                         }
//                     }
//                     EventResult::Ignored
//                 }
//                 Event::Mouse {
//                     event: MouseEvent::Press(MouseButton::Right),
//                     ..
//                 } => {
//                     game_state.first_selected = None;
//                     EventResult::Consumed(None)
//                 }
//
//                 _ => EventResult::Ignored,
//             })
//             .with_required_size(|_text, _constraints| (4 + 8 * 3, 8 + 1 + 1).into()),
//     );
//
//     siv.add_layer(board);
//
//     siv.add_global_callback('q', |s| s.quit());
//
//     siv.run();
// }
