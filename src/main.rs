#![feature(globs)]

extern crate ncurses;

use ncurses::*;

/*
 * So, how this working?
 * And what functions do we need?
 * First of all, we need 2 fields (user and enemy)
 * So, list of functions
 * initialize, deinit
 * init_field()
 * get_input() -> i32;
 * 
 */

enum CellType {
    EMPTY = 0, // Just empty space
    SHOT = 1, // o ?
    SHIP = 2, // x ?
}

enum Color {
    GREEN = 16, // For numbers and current ship while placing
    PINK = 17, // For shoted ships
    RED = 18, // For death ships
    WHITE = 19, // For shot in empty space
}

struct Cell {
    Type: CellType,
    Color: Color
}

impl Cell {
   fn print(&self) {
       let mut symbol = " ";
       match self.Type {
           EMPTY => symbol = ".",
           SHOT => symbol = "o",
           SHIP => symbol = "x",
       }
       printw(format!("{}", symbol).as_slice());
   } 
}

struct Coord {
    y: uint,
    x: uint
}

struct Ship {
    coord: Coord,
    cells: Vec<Coord> 
}

fn init_colors() {
    start_color();
    init_color((Color::GREEN as i16), 0, 150 * 4, 0);
    init_color((Color::PINK as i16), 0, 150 * 4, 0);
    init_color((Color::RED as i16), 0, 150 * 4, 0);
    init_color((Color::WHITE as i16), 0, 150 * 4, 0);
}

fn initialize() {
    initscr();

    curs_set(CURSOR_INVISIBLE);

    init_colors();

    keypad(stdscr, true);
    noecho();
}

fn deinitialize() {
    endwin();
}

fn get_input() -> i32 {
    getch()
}

fn print_field(field: &mut [[Cell, ..10], ..10], y: i32, x: i32) {
    mv(y, x+2i32);
    for i in range(1u, 11u) {
        printw(format!(" {}", i).as_slice());
    }
    for i in range(1u, 11u) {
        if i != 10u {
            mv(y+(i as i32), x+1i32);
        }
        else {
            mv(y+(i as i32), x);
        }
        printw(format!("{}", i).as_slice());
    }
    for i in range(0u, 10u) {
        mv(y+(i as i32)+1i32, x+2i32);
        for j in range(0u, 10u) {
            printw(" ");
            field[i][j].print();
        }
        printw("\n");
    }
}

fn add_ship(field: &mut [[Cell, ..10], ..10], ship: Ship) {
    let y = ship.coord.y;
    let x = ship.coord.x;
    for i in ship.cells.iter() {
        let cy = i.y;
        let cx = i.x;
        field[y+cy][x+cx] = Cell { Type : SHIP, Color : GREEN };
    }
}

fn remove_ship(field: &mut [[Cell, ..10], ..10], ship: Ship) {
    let y = ship.coord.y;
    let x = ship.coord.x;
    for i in ship.cells.iter() {
        let cy = i.y;
        let cx = i.x;
        field[y+cy][x+cx] = Cell { Type : EMPTY, Color : WHITE };
    }
}

fn rotate_ship(ship: Ship) {
    for i in range(0u, ship.cells.len()) {
        let temp = ship.cells[i].y;
        ship.cells[i].y = ship.cells[i].x;
        ship.cells[i].x = temp;
    }
}

fn place_ship(field: &mut [[Cell, ..10], ..10], ship: Ship) {
    add_ship(field, ship);
}

fn move_ship(field: &mut [[Cell, ..10], ..10], ship: Ship, input: i32) -> bool {
    remove_ship(field, ship);
    let KEY_SPACE = ' ' as i32;
    match input {
        KEY_LEFT =>  {
            ship.coord.x-=1;
        }
        KEY_UP =>  {
            ship.coord.y-=1;
        }
        KEY_DOWN =>  {
            ship.coord.y+=1;
        }
        KEY_RIGHT =>  {
            ship.coord.x+=1;
        }
        KEY_ENTER => {
            place_ship(field, ship);
            return true;
        }
        KEY_SPACE => {
            rotate_ship(ship);
        }
    }
    add_ship(field, ship);
    return false;
}

fn main() {
    initialize();

    let mut userfield = &mut [[Cell { Type : EMPTY, Color : WHITE }, ..10], ..10];
    let mut enemyfield = &mut [[Cell { Type : EMPTY, Color : WHITE }, ..10], ..10];

    let mut oneship = Ship {
        coord : Coord {y : 0u, x : 0u},
        cells : vec!{ Coord {y : 0u, x : 0u}}
    };

    let mut twoship = Ship {
        coord : Coord {y : 0u, x : 0u},
        cells : vec!{ Coord {y : 0u, x : 0u},
                      Coord {y : 1u, x : 0u}}
    };

    let mut threeship = Ship {
        coord : Coord {y : 1u, x : 0u},
        cells : vec!{ Coord {y : -1u, x : 0u},
                      Coord {y : 0u, x : 0u},
                      Coord {y : 1u, x : 0u}}
    };

    let mut fourship = Ship {
        coord : Coord {y : 1u, x : 0u},
        cells : vec!{ Coord {y : -1u, x : 0u},
                      Coord {y : 0u, x : 0u},
                      Coord {y : 1u, x : 0u},
                      Coord {y : 2u, x : 0u}}
    };


    // Which ships must be placed?
    // 1 - 4ship
    // 2 - 3ship
    // 3 - 2ship
    // 4 - 1ship
    // sum - 10

    let mut count = 0u;
    let mut ch = getch();
    let mut curShip : Ship = fourship;
    while ch != ('q' as i32) {
        if (move_ship(userfield, curShip, ch)) {
            count+=1;
            if count <= 2 {
                curShip = threeship;
            }
            else if count <= 5 {
                curShip = twoship;
            }
            else if count <= 9 {
                curShip = oneship;
            }
        }

        print_field(userfield, 0, 0);
        print_field(enemyfield, 0, 35);
        refresh();
        ch = getch();
    }

    // need on end of each frame?

    deinitialize();
}