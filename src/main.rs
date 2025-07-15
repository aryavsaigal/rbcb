use rand::{rng, seq::SliceRandom};
use std::{cmp::{max, min}, isize};

#[derive(Clone)]
struct Game {
    board: [[Pieces; 8]; 8],
    castle: [[bool; 2]; 2], // 0-W, 1-B
    promotion: char,
    turn: bool,
    en_passant: Vec<usize>,
    turn_count: usize,
    counter: usize
}

impl Game {
    fn new() -> Self {
        Self {
            board: [[Pieces::Empty; 8]; 8],
            castle: [[true; 2]; 2],
            promotion: 'q',
            turn: true,
            en_passant: vec![],
            turn_count: 0,
            counter: 1,
        }
    }

    fn parse_move(mov: &str) -> Result<[[usize; 2]; 2], String> {
        fn get_coords(m: String) -> Option<[usize; 2]> {
            Some([
                m.chars()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .parse::<usize>()
                    .unwrap_or(1)
                    - 1,
                match m.chars().nth(0).unwrap() {
                    'a' => 0,
                    'b' => 1,
                    'c' => 2,
                    'd' => 3,
                    'e' => 4,
                    'f' => 5,
                    'g' => 6,
                    'h' => 7,
                    _ => 0,
                },
            ])
        }
        if mov.len() != 4 {
            return Err("Invalid Length".to_string());
        }

        let mov = mov.to_lowercase();

        let (i, f) = mov.split_at(2);

        Ok([
            get_coords(i.to_string()).unwrap(),
            get_coords(f.to_string()).unwrap(),
        ])
    }

    fn init(&mut self) {
        self.board[0] = [
            Pieces::Rook(true),
            Pieces::Knight(true),
            Pieces::Bishop(true),
            Pieces::Queen(true),
            Pieces::King(true),
            Pieces::Bishop(true),
            Pieces::Knight(true),
            Pieces::Rook(true),
        ];
        self.board[1] = [Pieces::Pawn(true); 8];
        self.board[6] = [Pieces::Pawn(false); 8];
        self.board[7] = [
            Pieces::Rook(false),
            Pieces::Knight(false),
            Pieces::Bishop(false),
            Pieces::Queen(false),
            Pieces::King(false),
            Pieces::Bishop(false),
            Pieces::Knight(false),
            Pieces::Rook(false),
        ];
    }

    fn get_surrounding_cells(coords: [usize; 2]) -> Vec<[usize; 2]> {
        let [x, y] = coords;
        let mut ret = Vec::new();

        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for (dx, dy) in directions.iter() {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                ret.push([nx as usize, ny as usize]);
            }
        }

        ret
    }

    fn move_piece(&mut self, i: [usize; 2], f: [usize; 2]) -> Result<(), String> {
        if self.board[i[0]][i[1]] == Pieces::Empty {
            return Err("Invalid move".to_string());
        }

        if self.turn_count == 0 {
            self.en_passant = vec![];
        }

        let piece_i = self.board[i[0]][i[1]];
        let piece_f = self.board[f[0]][f[1]];

        if piece_i.colour().unwrap() != self.turn {
            return Err("Invalid turn".to_string());
        }

        if piece_i.colour() == piece_f.colour() {
            return Err("Illegal move".to_string());
        }

        if !match piece_i {
            Pieces::Pawn(c) => match Game::distance(i, f) {
                1 => {
                    i[1] == f[1]
                        && (((f[0] > i[0]) && c) || ((f[0] < i[0]) && !c))
                        && (piece_f == Pieces::Empty)
                }
                2 => {
                    let result = (((c && i[0] == 1) || (!c && i[0] == 6))
                        && i[1] == f[1]
                        && (((f[0] > i[0]) && c) || ((f[0] < i[0]) && !c))
                        && !self.pieces_between(i, f))
                        && (piece_f == Pieces::Empty)
                        || (((i[1] as isize - f[1] as isize).abs() == 1)
                            && (piece_f != Pieces::Empty)
                            || (self.en_passant
                                == vec![
                                    (f[0] as isize + (if c { -1 as isize } else { 1 as isize }))
                                        as usize,
                                    f[1],
                                ]
                                && ({
                                    self.board[self.en_passant[0]][self.en_passant[1]] =
                                        Pieces::Empty;
                                    true
                                })));
                    if ((c && i[0] == 1) || (!c && i[0] == 6))
                        && i[1] == f[1]
                        && (((f[0] > i[0]) && c) || ((f[0] < i[0]) && !c))
                        && !self.pieces_between(i, f)
                    {
                        self.en_passant = vec![f[0], f[1]];
                        self.turn_count = 0;
                    }
                    result
                }
                _ => false,
            },
            Pieces::King(c) => {
                let ret = match Game::distance(i, f) {
                    1 => true,
                    2 => {
                        if i[0] != f[0] && i[1] != f[1] {
                            true
                        } else {
                            if c && !self.check(self.find(Pieces::King(c)).unwrap(), c) {
                                if self.castle[0][0]
                                    && f == [0, 6]
                                    && self.board[0][7] == Pieces::Rook(c)
                                    && !self.pieces_between(i, f)
                                    && !self.check([0, 5], c)
                                {
                                    self.board[i[0]][i[1]] = Pieces::Empty;
                                    self.board[0][7] = Pieces::Empty;
                                    self.board[0][5] = Pieces::Rook(c);
                                    self.board[f[0]][f[1]] = piece_i;
                                    self.turn_count += 1;
                                    self.turn_count %= 2;
                                    self.turn = !self.turn;
                                    self.counter += 1;
                                    return Ok(());
                                } else if self.castle[0][1]
                                    && f == [0, 2]
                                    && self.board[0][0] == Pieces::Rook(c)
                                    && !self.pieces_between(i, [0, 1])
                                    && !self.check([0, 3], c)
                                {
                                    self.board[i[0]][i[1]] = Pieces::Empty;
                                    self.board[0][0] = Pieces::Empty;
                                    self.board[0][3] = Pieces::Rook(c);
                                    self.board[f[0]][f[1]] = piece_i;
                                    self.turn_count += 1;
                                    self.turn_count %= 2;
                                    self.turn = !self.turn;
                                    self.counter += 1;
                                    return Ok(());
                                }
                            } else if !self.check(self.find(Pieces::King(c)).unwrap(), c) {
                                if self.castle[1][0]
                                    && f == [7, 6]
                                    && self.board[7][7] == Pieces::Rook(c)
                                    && !self.pieces_between(i, f)
                                    && !self.check([7, 5], c)
                                {
                                    self.board[i[0]][i[1]] = Pieces::Empty;
                                    self.board[7][7] = Pieces::Empty;
                                    self.board[7][5] = Pieces::Rook(c);
                                    self.board[f[0]][f[1]] = piece_i;
                                    self.turn_count += 1;
                                    self.turn_count %= 2;
                                    self.turn = !self.turn;
                                    self.counter += 1;
                                    return Ok(());
                                } else if self.castle[1][1]
                                    && f == [7, 2]
                                    && self.board[7][0] == Pieces::Rook(c)
                                    && !self.pieces_between(i, [7, 1])
                                    && !self.check([7, 3], c)
                                {
                                    self.board[i[0]][i[1]] = Pieces::Empty;
                                    self.board[7][0] = Pieces::Empty;
                                    self.board[7][3] = Pieces::Rook(c);
                                    self.board[f[0]][f[1]] = piece_i;
                                    self.turn_count += 1;
                                    self.turn_count %= 2;
                                    self.turn = !self.turn;
                                    self.counter += 1;
                                    return Ok(());
                                }
                            }
                            false
                        }
                    }
                    _ => false,
                };

                if c {
                    self.castle[0][0] = false;
                    self.castle[0][1] = false;
                } else {
                    self.castle[1][0] = false;
                    self.castle[1][1] = false;
                }

                ret
            }
            Pieces::Rook(c) => {
                if c {
                    if i == [0, 7] {
                        self.castle[0][0] = false;
                    } else if i == [0, 0] {
                        self.castle[0][1] = false;
                    }
                } else {
                    if i == [7, 7] {
                        self.castle[1][0] = false;
                    } else if i == [7, 0] {
                        self.castle[1][1] = false;
                    }
                }
                !self.pieces_between(i, f) && (i[0] == f[0] || i[1] == f[1])
            }
            Pieces::Bishop(_c) => {
                !self.pieces_between(i, f)
                    && ((i[0] as isize - f[0] as isize).abs()
                        == (i[1] as isize - f[1] as isize).abs())
            }
            Pieces::Queen(_c) => {
                !self.pieces_between(i, f)
                    && ((i[0] == f[0] || i[1] == f[1])
                        || ((i[0] as isize - f[0] as isize).abs()
                            == (i[1] as isize - f[1] as isize).abs()))
            }
            Pieces::Knight(_c) => {
                ((i[0] as isize - f[0] as isize).abs() == 2
                    && (i[1] as isize - f[1] as isize).abs() == 1)
                    || ((i[0] as isize - f[0] as isize).abs() == 1
                        && (i[1] as isize - f[1] as isize).abs() == 2)
            }
            _ => true,
        } {
            return Err("Illegal move".to_string());
        }

        let c = piece_i.colour().unwrap();

        let mut game_clone = self.clone();
        game_clone.board[i[0]][i[1]] = Pieces::Empty;
        game_clone.board[f[0]][f[1]] = piece_i;

        if game_clone.check(game_clone.find(Pieces::King(c)).unwrap(), c) {
            return Err("Illegal move; Places King in check".to_string());
        }

        self.board[i[0]][i[1]] = Pieces::Empty;
        self.board[f[0]][f[1]] = piece_i;

        if let Pieces::Pawn(_) = piece_i {
            if (c && f[0] == 7) || (!c && f[0] == 0) {
                self.board[f[0]][f[1]] = match self.promotion {
                    'q' => Pieces::Queen(c),
                    'r' => Pieces::Rook(c),
                    'b' => Pieces::Bishop(c),
                    'n' => Pieces::Knight(c),
                    _ => return Err("Invalid piece for promotion".to_string()),
                };
            }
        }

        self.turn = !self.turn;
        self.turn_count += 1;
        self.turn_count %= 2;
        self.counter += 1;

        Ok(())
    }

    fn check(&mut self, i: [usize; 2], c: bool) -> bool {
        let opp = !c;

        for pos in i[0] + 1..=7 {
            let piece = self.board[pos][i[1]];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Rook(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [pos, i[1]]) == 1 {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for pos in (0..i[0]).rev() {
            let piece = self.board[pos][i[1]];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Rook(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [pos, i[1]]) == 1 {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for pos in i[1] + 1..=7 {
            let piece = self.board[i[0]][pos];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Rook(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [i[0], pos]) == 1 {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for pos in (0..i[1]).rev() {
            let piece = self.board[i[0]][pos];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Rook(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [i[0], pos]) == 1 {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for increment in 1..=7 {
            if i[0] + increment >= 8 || i[1] + increment >= 8 {
                break;
            }

            let piece = self.board[i[0] + increment][i[1] + increment];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Bishop(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::Pawn(x) => {
                        if x == opp
                            && Game::distance(i, [i[0] + increment, i[1] + increment]) == 2
                            && c
                        {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [i[0] + increment, i[1] + increment]) == 2
                        {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for increment in 1..=7 {
            if i[0] as isize - increment as isize <= -1 || i[1] as isize - increment as isize <= -1
            {
                break;
            }

            let piece = self.board[i[0] - increment][i[1] - increment];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Bishop(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::Pawn(x) => {
                        if x == opp
                            && Game::distance(i, [i[0] - increment, i[1] - increment]) == 2
                            && !c
                        {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [i[0] - increment, i[1] - increment]) == 2
                        {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for increment in 1..=7 {
            if (i[0] + increment) as isize >= 8 || i[1] as isize - increment as isize <= -1 {
                break;
            }

            let piece = self.board[i[0] + increment][i[1] - increment];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Bishop(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::Pawn(x) => {
                        if x == opp
                            && Game::distance(i, [i[0] + increment, i[1] - increment]) == 2
                            && c
                        {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [i[0] + increment, i[1] - increment]) == 2
                        {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for increment in 1..=7 {
            if i[0] as isize - increment as isize <= -1 || (i[1] + increment) as isize >= 8 {
                break;
            }

            let piece = self.board[i[0] - increment][i[1] + increment];

            if piece == Pieces::Empty {
                continue;
            }

            if piece.colour().unwrap() == c {
                break;
            } else {
                match piece {
                    Pieces::Queen(x) | Pieces::Bishop(x) => {
                        if x == opp {
                            return true;
                        }
                    }
                    Pieces::Pawn(x) => {
                        if x == opp
                            && Game::distance(i, [i[0] - increment, i[1] + increment]) == 2
                            && !c
                        {
                            return true;
                        }
                    }
                    Pieces::King(x) => {
                        if x == opp && Game::distance(i, [i[0] - increment, i[1] + increment]) == 2
                        {
                            return true;
                        }
                    }
                    _ => break,
                };
            }
        }

        for pos in [
            (2, 1),
            (2, -1),
            (-2, 1),
            (-2, -1),
            (1, -2),
            (-1, -2),
            (1, 2),
            (-1, 2),
        ]
        .iter()
        {
            let y = i[0] as isize + pos.0;
            let x = i[1] as isize + pos.1;
            if y >= 0 && y <= 7 && x >= 0 && x <= 7 {
                let piece = self.board[y as usize][x as usize];

                if piece == Pieces::Empty {
                    continue;
                }

                if let Pieces::Knight(x) = piece {
                    if x == opp {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn find(&self, piece: Pieces) -> Option<[usize; 2]> {
        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if cell == &piece {
                    return Some([y, x]);
                }
            }
        }
        None
    }

    fn distance(i: [usize; 2], f: [usize; 2]) -> usize {
        ((i[0] as isize - f[0] as isize).abs() + (i[1] as isize - f[1] as isize).abs()) as usize
    }

    fn pieces_between(&mut self, i: [usize; 2], f: [usize; 2]) -> bool {
        if i[0] == f[0] {
            for c in min(i[1], f[1]) + 1..max(i[1], f[1]) {
                if self.board[i[0]][c] != Pieces::Empty {
                    return true;
                }
            }
        } else if i[1] == f[1] {
            for c in min(i[0], f[0]) + 1..max(i[0], f[0]) {
                if self.board[c][i[1]] != Pieces::Empty {
                    return true;
                }
            }
        } else {
            let (min_c, max_c) = if i[0] > f[0] { (f, i) } else { (i, f) };
            for c in 1..(i[0] as isize - f[0] as isize).abs() {
                let c = c as usize;
                if min_c[1] < max_c[1] {
                    if self.board[min_c[0] + c][min_c[1] + c] != Pieces::Empty {
                        return true;
                    }
                } else {
                    if self.board[min_c[0] + c][min_c[1] - c] != Pieces::Empty {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn display(&self) {
        let mut board = self.board;
        board.reverse();
        for row in board {
            for piece in row {
                print!("{} ", piece.symbol());
            }
            print!("\n");
        }
    }

    fn check_remaining_pieces(&mut self) -> State {
        let mut pieces = Vec::new();
        for row in self.board.iter() {
            for piece in row {
                if *piece != Pieces::Empty {
                    pieces.push(*piece);
                }
            }
        }

        if pieces.len() == 2 {
            return State::Draw;
        }

        State::Continue
    }

    fn check_game_end(&mut self) -> State {
        let king_white = self.find(Pieces::King(true)).unwrap();
        let king_black = self.find(Pieces::King(false)).unwrap();

        let white_check = self.check(king_white, true);
        let black_check = self.check(king_black, false);

        let crp = self.check_remaining_pieces() ;

        match crp {
            State::Continue => {},
            _ => return crp
        };

        let moves_white = self
            .get_pieces(true)
            .iter()
            .any(|piece| self.find_valid_move(*piece));
        let moves_black = self.get_pieces(false).iter().any(|piece| {
            self.find_valid_move(*piece)
        });

        if !moves_white && self.turn {
            if white_check {
                return State::WhiteCheckmate;
            } else {
                return State::WhiteStalemate;
            }
        }

        if !moves_black && !self.turn {
            if black_check {
                return State::BlackCheckmate;
            } else {
                return State::BlackStalemate;
            }
        }

        if white_check {
            return State::WhiteCheck;
        }

        if black_check {
            return State::BlackCheck;
        }

        State::Continue
    }

    fn get_pieces(&mut self, c: bool) -> Vec<[usize; 2]> {
        let mut pieces = Vec::new();

        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if cell != &Pieces::Empty {
                    if cell.colour().unwrap() == c {
                        pieces.push([y, x]);
                    }
                }
            }
        }

        pieces
    }

    fn find_valid_move(&mut self, coords: [usize; 2]) -> bool {
        let piece = self.board[coords[0]][coords[1]];

        if piece == Pieces::Empty {
            panic!("Error in finding valid move: invalid piece");
        }

        let mut valid_move = false;

        match piece {
            Pieces::Knight(c) => {
                for pos in [
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, -2),
                    (-1, -2),
                    (1, 2),
                    (-1, 2),
                ]
                .iter()
                {
                    let y = coords[0] as isize + pos.0;
                    let x = coords[1] as isize + pos.1;
                    if y >= 0 && y <= 7 && x >= 0 && x <= 7 {
                        let mut game_copy = self.clone();

                        if let Ok(_) = game_copy.move_piece(coords, [y as usize, x as usize])
                            && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                        {
                            valid_move = true;
                            break;
                        };
                    }
                }
            }
            Pieces::King(c) => {
                for pos in Game::get_surrounding_cells(coords).iter().chain(
                    [
                        [coords[0], min(coords[1] + 2, 7)],
                        [coords[0], max(coords[1] as isize - 2, 0) as usize],
                    ]
                    .iter(),
                ) {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, *pos)
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_move = true;
                        break;
                    };
                }
            }
            Pieces::Pawn(c) => {
                let moves;
                if c {
                    moves = [[1, 0], [2, 0], [1, 1], [1, -1]];
                } else {
                    moves = [[-1, 0], [-2, 0], [-1, -1], [-1, 1]];
                }
                for pos in moves.iter() {
                    let x = coords[1] as isize + pos[1];
                    let y = coords[0] as isize + pos[0];

                    if y >= 0 && y <= 7 && x >= 0 && x <= 7 {
                        let mut game_copy = self.clone();
                        if let Ok(_) = game_copy.move_piece(coords, [y as usize, x as usize])
                            && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                        {
                            valid_move = true;
                            break;
                        };
                    }
                }
            }
            Pieces::Bishop(c) => {
                let mut valid_moves = vec![];
                for inc in 1..=7 {
                    let y = coords[0] as isize;
                    let x = coords[1] as isize;

                    if x + inc >= 0 && x + inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x - inc]);
                    }
                    if x + inc >= 0 && x + inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x - inc]);
                    }
                }

                for mov in valid_moves.iter() {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, [mov[0] as usize, mov[1] as usize])
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_move = true;
                        break;
                    };
                }
            }
            Pieces::Rook(c) => {
                let mut valid_moves = vec![];
                for inc in 1..=7 {
                    let y = coords[0] as isize;
                    let x = coords[1] as isize;

                    if x + inc >= 0 && x + inc <= 7 {
                        valid_moves.push([y, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 {
                        valid_moves.push([y, x - inc]);
                    }
                    if y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x]);
                    }
                    if y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x]);
                    }
                }

                for mov in valid_moves.iter() {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, [mov[0] as usize, mov[1] as usize])
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_move = true;
                        break;
                    };
                }
            }
            Pieces::Queen(c) => {
                let mut valid_moves = vec![];
                for inc in 1..=7 {
                    let y = coords[0] as isize;
                    let x = coords[1] as isize;

                    if x + inc >= 0 && x + inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x - inc]);
                    }
                    if x + inc >= 0 && x + inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x - inc]);
                    }

                    if x + inc >= 0 && x + inc <= 7 {
                        valid_moves.push([y, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 {
                        valid_moves.push([y, x - inc]);
                    }
                    if y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x]);
                    }
                    if y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x]);
                    }
                }

                for mov in valid_moves.iter() {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, [mov[0] as usize, mov[1] as usize])
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_move = true;
                        break;
                    };
                }
            }
            _ => {}
        }
        valid_move
    }

    fn get_valid_moves(&mut self, coords: [usize; 2]) -> Vec<[usize; 2]> {
        let piece = self.board[coords[0]][coords[1]];

        if piece == Pieces::Empty {
            panic!("Error in finding valid move: invalid piece");
        }

        let mut valid_moves_game: Vec<[usize; 2]> = Vec::new();

        match piece {
            Pieces::Knight(c) => {
                for pos in [
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, -2),
                    (-1, -2),
                    (1, 2),
                    (-1, 2),
                ]
                .iter()
                {
                    let y = coords[0] as isize + pos.0;
                    let x = coords[1] as isize + pos.1;
                    if y >= 0 && y <= 7 && x >= 0 && x <= 7 {
                        let mut game_copy = self.clone();

                        if let Ok(_) = game_copy.move_piece(coords, [y as usize, x as usize])
                            && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                        {
                            valid_moves_game.push([y as usize, x as usize]);
                        };
                    }
                }
            }
            Pieces::King(c) => {
                for pos in Game::get_surrounding_cells(coords).iter().chain(
                    [
                        [coords[0], min(coords[1] + 2, 7)],
                        [coords[0], max(coords[1] as isize - 2, 0) as usize],
                    ]
                    .iter(),
                ) {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, *pos)
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_moves_game.push(*pos);
                    };
                }
            }
            Pieces::Pawn(c) => {
                let moves;
                if c {
                    moves = [[1, 0], [2, 0], [1, 1], [1, -1]];
                } else {
                    moves = [[-1, 0], [-2, 0], [-1, -1], [-1, 1]];
                }
                for pos in moves.iter() {
                    let x = coords[1] as isize + pos[1];
                    let y = coords[0] as isize + pos[0];

                    if y >= 0 && y <= 7 && x >= 0 && x <= 7 {
                        let mut game_copy = self.clone();
                        if let Ok(_) = game_copy.move_piece(coords, [y as usize, x as usize])
                            && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                        {
                            valid_moves_game.push([y as usize, x as usize]);
                        };
                    }
                }
            }
            Pieces::Bishop(c) => {
                let mut valid_moves = vec![];
                for inc in 1..=7 {
                    let y = coords[0] as isize;
                    let x = coords[1] as isize;

                    if x + inc >= 0 && x + inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x - inc]);
                    }
                    if x + inc >= 0 && x + inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x - inc]);
                    }
                }

                for mov in valid_moves.iter() {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, [mov[0] as usize, mov[1] as usize])
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_moves_game.push([mov[0] as usize, mov[1] as usize]);
                    };
                }
            }
            Pieces::Rook(c) => {
                let mut valid_moves = vec![];
                for inc in 1..=7 {
                    let y = coords[0] as isize;
                    let x = coords[1] as isize;

                    if x + inc >= 0 && x + inc <= 7 {
                        valid_moves.push([y, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 {
                        valid_moves.push([y, x - inc]);
                    }
                    if y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x]);
                    }
                    if y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x]);
                    }
                }

                for mov in valid_moves.iter() {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, [mov[0] as usize, mov[1] as usize])
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_moves_game.push([mov[0] as usize, mov[1] as usize]);
                    };
                }
            }
            Pieces::Queen(c) => {
                let mut valid_moves = vec![];
                for inc in 1..=7 {
                    let y = coords[0] as isize;
                    let x = coords[1] as isize;

                    if x + inc >= 0 && x + inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x - inc]);
                    }
                    if x + inc >= 0 && x + inc <= 7 && y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 && y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x - inc]);
                    }

                    if x + inc >= 0 && x + inc <= 7 {
                        valid_moves.push([y, x + inc]);
                    }
                    if x - inc >= 0 && x - inc <= 7 {
                        valid_moves.push([y, x - inc]);
                    }
                    if y + inc >= 0 && y + inc <= 7 {
                        valid_moves.push([y + inc, x]);
                    }
                    if y - inc >= 0 && y - inc <= 7 {
                        valid_moves.push([y - inc, x]);
                    }
                }

                for mov in valid_moves.iter() {
                    let mut game_copy = self.clone();
                    if let Ok(_) = game_copy.move_piece(coords, [mov[0] as usize, mov[1] as usize])
                        && !game_copy.check(game_copy.find(Pieces::King(c)).unwrap(), c)
                    {
                        valid_moves_game.push([mov[0] as usize, mov[1] as usize]);
                    };
                }
            }
            _ => {}
        }
        valid_moves_game
    }
    fn play_ai(&mut self, c: bool) -> [[usize; 2]; 2] {
        let mut best_score = isize::MIN;
        let mut best_mov: Option<[[usize;2];2]> = None;

        let mut moves = Vec::new();

        for piece in self.get_pieces(c) {
            moves.extend(self.get_valid_moves(piece).iter().map(|p| [piece, *p]));
        }

        moves.shuffle(&mut rng());

        for [i, f] in &moves {
            let mut game_clone = self.clone();
            game_clone.move_piece(*i, *f).unwrap();
            let score = Game::minimax(&mut game_clone, 3, isize::MIN, isize::MAX, !c, c);

            if score > best_score {
                best_score = score;
                best_mov = Some([*i, *f]);
            }
        }

        best_mov.unwrap()
    }

    fn is_game_over(&mut self) -> bool {
        match self.check_game_end() {
            State::WhiteCheckmate | State::WhiteStalemate | State::BlackCheckmate | State::BlackStalemate | State::Draw => true,
            _ => false,
        }
    }

    fn count_board(&mut self, c: bool) -> isize {
        let mut score = 0;

        let mut pieces = self.get_pieces(c);
        pieces.append(&mut self.get_pieces(!c));

        for coord in pieces {
            let piece = self.board[coord[0]][coord[1]];
            score += (match piece {
                Pieces::King(_) => 900,
                Pieces::Queen(_) => 90,
                Pieces::Rook(_) => 50,
                Pieces::Bishop(_) => 30,
                Pieces::Knight(_) => 30,
                Pieces::Pawn(_) => 10,
                Pieces::Empty => 0
            }) * (if piece.colour().unwrap() == c { 1 } else { -1 })
        }

        score += match self.check_game_end() {
            State::WhiteCheckmate => if c { -10000 } else { 10000 },
            State::BlackCheckmate => if !c { -10000 } else { 10000 },
            State::BlackStalemate | State::WhiteStalemate => -50,
            State::WhiteCheck => if c { -80 } else { 80 },
            State::BlackCheck => if !c { -80 } else { 80 },
            _ => 0
        };

        score
    }

    fn minimax(game: &mut Game, depth: usize, mut alpha: isize, mut beta: isize, c: bool, maximising_player: bool) -> isize {
        if depth == 0 || game.is_game_over() {
            return game.count_board(maximising_player);
        }
        
        let mut moves = Vec::new();

        for piece in game.get_pieces(c) {
            moves.extend(game.get_valid_moves(piece).iter().map(|p| [piece, *p]));
        }

        moves.shuffle(&mut rng());

        if c == maximising_player {
            let mut max_eval = isize::MIN;
            for [i, f] in moves {
                let mut game_clone = game.clone();
                game_clone.move_piece(i, f).unwrap();
                let eval = Game::minimax(&mut game_clone, depth-1, alpha, beta, !c, maximising_player);
                max_eval = max(max_eval, eval);
                alpha = max(alpha, eval);
                if beta <= alpha {
                    break;
                }
            }
            return max_eval
        }
        else {
            let mut min_eval = isize::MAX;
            for [i, f] in moves {
                let mut game_clone = game.clone();
                game_clone.move_piece(i, f).unwrap();
                let eval = Game::minimax(&mut game_clone, depth-1, alpha, beta, !c, maximising_player);
                min_eval = min(min_eval, eval);
                beta = min(beta, eval);
                if beta <= alpha {
                    break;
                }
            }
            return min_eval
        }
    }
}

#[derive(Copy, Clone)]
enum State {
    WhiteCheckmate,
    BlackCheckmate,
    WhiteStalemate,
    BlackStalemate,
    WhiteCheck,
    BlackCheck,
    Continue,
    Draw
}

impl State {
    fn symbol(&self) -> String {
        (match self {
            State::WhiteCheckmate => "White is checkmated",
            State::BlackCheckmate => "Black is checkmated",
            State::WhiteStalemate => "White is stalemated",
            State::BlackStalemate => "Black is stalemated",
            State::WhiteCheck => "White is in check",
            State::BlackCheck => "Black is in check",
            State::Continue => "Game continues",
            State::Draw => "Draw"
        })
        .to_string()
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Pieces {
    // true-W, false-B
    Pawn(bool),
    Bishop(bool),
    Knight(bool),
    Rook(bool),
    Queen(bool),
    King(bool),
    Empty,
}

impl Pieces {
    fn symbol(&self) -> String {
        (match self {
            Pieces::Rook(true) => "R",
            Pieces::Rook(false) => "r",
            Pieces::Bishop(true) => "B",
            Pieces::Bishop(false) => "b",
            Pieces::Knight(true) => "N",
            Pieces::Knight(false) => "n",
            Pieces::Queen(true) => "Q",
            Pieces::Queen(false) => "q",
            Pieces::King(true) => "K",
            Pieces::King(false) => "k",
            Pieces::Pawn(true) => "P",
            Pieces::Pawn(false) => "p",
            Pieces::Empty => " ",
        })
        .to_string()
    }

    fn colour(&self) -> Option<bool> {
        match self {
            Pieces::Bishop(x)
            | Pieces::Knight(x)
            | Pieces::Pawn(x)
            | Pieces::Rook(x)
            | Pieces::King(x)
            | Pieces::Queen(x) => Some(*x),
            Pieces::Empty => None,
        }
    }
}

fn main() {
    let mut game = Game::new();
    let mut error = String::new();
    let mut game_state = String::new();
    let mut end = false;
    game.init();

    loop {
        match game.check_game_end() {
            State::Continue | State::WhiteCheck | State::BlackCheck => {}
            _ => end = true,
        }

        println!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        game.display();
        println!(
            "{}. Turn: {} | Status: {} | Game State: {}",
            game.counter,
            if game.turn { "White" } else { "Black" },
            error,
            game_state
        );

        error = String::new();

        if end {
            break;
        }

        if !game.turn {
            let [i, f] = game.play_ai(false);
            game.move_piece(i, f).unwrap_or_else(|e| {
                error = e;
            });
        } else {
            let mut mov = String::new();
            std::io::stdin().read_line(&mut mov).unwrap();
            if mov.trim().len() == 1 {
                game.promotion = mov.chars().nth(0).unwrap();
                error = format!("Updated promotion to '{}'", mov.trim());
                continue;
            }

            let [i, f] = match Game::parse_move(mov.trim()) {
                Ok(val) => val,
                Err(s) => {
                    error = s;
                    continue;
                }
            };
            // let [i, f] = game.play_ai(true);
            game.move_piece(i, f).unwrap_or_else(|e| {
                error = e;
            });
        }

        game_state = game.check_game_end().symbol();
    }
}
