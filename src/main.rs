#![feature(globs)]

extern crate ncurses;

use ncurses::*;
use std::rand;

enum CellType {
    EMPTY = 0,
    SHOT = 1,
    SHIP = 2,
    COLLISION_SHIP = 3,
    HIDE_SHIP = 4,
    SHOT_POS = 5,
}

enum Color {
    BACKGROUND = 15,
    GREEN = 16,
    PINK = 17,
    RED = 18,
    WHITE = 19,
    BLUE = 20,
}

enum Status {
    START = 0,
    PLACE_SHIP = 1,
    PLAYER_TURN = 2,
    AI_TURN = 3,
    QUIT = 4,
}

static COLOR_PAIR_NUMBER: i16 = 1;
static COLOR_PAIR_EMPTY: i16 = 2;
static COLOR_PAIR_SHIP: i16 = 3;
static COLOR_PAIR_COLLISION_SHIP: i16 = 4;
static COLOR_PAIR_SHOT: i16 = 5;
static COLOR_PAIR_SHOT_POS: i16 = 6;

static DEBUG: bool = false;

struct Cell {
    Type: CellType,
    Color: Color
}

impl Cell {
   fn print(&self) {
       let mut symbol = " ";
       match self.Type {
           HIDE_SHIP => {
               if DEBUG {
               attron(COLOR_PAIR(COLOR_PAIR_SHIP));
               printw(".");
               attroff(COLOR_PAIR(COLOR_PAIR_SHIP));
               }
               else {
               attron(COLOR_PAIR(COLOR_PAIR_EMPTY));
               printw(".");
               attroff(COLOR_PAIR(COLOR_PAIR_EMPTY));
               }
           }
           EMPTY => {
               attron(COLOR_PAIR(COLOR_PAIR_EMPTY));
               printw(".");
               attroff(COLOR_PAIR(COLOR_PAIR_EMPTY));
           }
           SHOT => {
               attron(COLOR_PAIR(COLOR_PAIR_SHOT));
               printw("o");
               attroff(COLOR_PAIR(COLOR_PAIR_SHOT));
           }
           SHIP => {
               attron(COLOR_PAIR(COLOR_PAIR_SHIP));
               printw("x");
               attroff(COLOR_PAIR(COLOR_PAIR_SHIP));
           }
           COLLISION_SHIP => {
               attron(COLOR_PAIR(COLOR_PAIR_COLLISION_SHIP));
               printw("x");
               attroff(COLOR_PAIR(COLOR_PAIR_COLLISION_SHIP));
           }
           SHOT_POS => {
               attron(COLOR_PAIR(COLOR_PAIR_SHOT_POS));
               printw("o");
               attroff(COLOR_PAIR(COLOR_PAIR_SHOT_POS));
           }
       }
   }
}

type Field = [[Cell, ..10], ..10];

struct Coord {
    y: uint,
    x: uint
}

struct ShipCell {
    coord: Coord,
    cell: CellType
}

struct Ship {
    coord: Coord,
    cells: Vec<ShipCell>,
    can_be_placed: bool
}

fn init_colors() {
    start_color();

    init_color((Color::BACKGROUND as i16), 27 * 4, 29 * 4, 32 * 4);
    init_color((Color::GREEN as i16), 135 * 4, 140 * 4, 73 * 4);
    init_color((Color::PINK as i16), 127, 58 * 4, 96 * 4);
    init_color((Color::RED as i16), 130 * 4, 68 * 4, 76 * 4);
    init_color((Color::WHITE as i16), 51 * 4, 95 * 4, 121 * 4);
    init_color((Color::BLUE as i16), 51 * 4, 95 * 4, 121 * 4);

    init_pair(COLOR_PAIR_NUMBER, Color::GREEN as i16, Color::BACKGROUND as i16);
    init_pair(COLOR_PAIR_EMPTY, Color::WHITE as i16, Color::BACKGROUND as i16);
    init_pair(COLOR_PAIR_SHIP, Color::GREEN as i16, Color::BACKGROUND as i16);
    init_pair(COLOR_PAIR_SHOT, Color::PINK as i16, Color::BACKGROUND as i16);
    init_pair(COLOR_PAIR_SHOT_POS, Color::BLUE as i16, Color::BACKGROUND as i16);
    init_pair(COLOR_PAIR_COLLISION_SHIP, Color::RED as i16, Color::BACKGROUND as i16);

    bkgd(' ' as u32 | COLOR_PAIR(COLOR_PAIR_EMPTY) as u32);
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

fn print_field(field: Field, y: i32, x: i32) {
    mv(y, x+2i32);
    for i in range(1u, 11u) {
        attron(COLOR_PAIR(COLOR_PAIR_NUMBER));
        printw(format!(" {}", i).as_slice());
        attroff(COLOR_PAIR(COLOR_PAIR_NUMBER));
    }
    for i in range(1u, 11u) {
        if i != 10u {
            mv(y+(i as i32), x+1i32);
        }
        else {
            mv(y+(i as i32), x);
        }
        attron(COLOR_PAIR(COLOR_PAIR_NUMBER));
        printw(format!("{}", i).as_slice());
        attroff(COLOR_PAIR(COLOR_PAIR_NUMBER));
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

fn add_ship(field: &mut Field, ship: &Ship) {
    let y = ship.coord.y;
    let x = ship.coord.x;
    for i in ship.cells.iter() {
        let cy = i.coord.y;
        let cx = i.coord.x;
        field[y+cy][x+cx] = Cell { Type : i.cell, Color : GREEN };
    }
}

fn remove_ship(field: &mut Field, ship: &Ship) {
    let y = ship.coord.y;
    let x = ship.coord.x;
    for i in ship.cells.iter() {
        let cy = i.coord.y;
        let cx = i.coord.x;
        field[y+cy][x+cx] = Cell { Type : EMPTY, Color : WHITE };
    }
}

fn rotate_ship(ship: &mut Ship) {
    for i in range(0u, ship.cells.len()) {
        let temp = ship.cells[i].coord.y;
        ship.cells[i].coord.y = ship.cells[i].coord.x;
        ship.cells[i].coord.x = temp;
    }
}

fn have_neighbours(field: Field, y: uint, x: uint) -> bool {
    let mx : int = x as int+1;
    let lx : int = x as int-1;
    let my : int = y as int+1;
    let ly : int = y as int-1;

    if field[y][x].Type as int == SHIP as int { return true; }
    if (lx >= 0) && (field[y][x-1].Type as int == SHIP as int) { return true; }
    if (mx < 10) && (field[y][x+1].Type as int == SHIP as int) { return true; }
    if (ly >= 0) && (field[y-1][x].Type as int == SHIP as int) { return true; }
    if (my < 10) && (field[y+1][x].Type as int == SHIP as int) { return true; }
    if (ly >= 0) && (lx >= 0) && (field[y-1][x-1].Type as int == SHIP as int)  { return true; }
    if (ly >= 0) && (mx < 10) && (field[y-1][x+1].Type as int == SHIP as int)  { return true; }
    if (my < 10) && (lx >= 0) && (field[y+1][x-1].Type as int == SHIP as int)  { return true; }
    if (my < 10) && (mx < 10) && (field[y+1][x+1].Type as int == SHIP as int)  { return true; }

    return false;
}

fn collision(field: &Field, ship: &mut Ship) -> bool {
    let y = ship.coord.y;
    let x = ship.coord.x;
    let mut canbeplaced = true;
    for i in range(0u, ship.cells.len()) {
        let cy = ship.cells[i].coord.y;
        let cx = ship.cells[i].coord.x;
        if (y+cy == -1) || (y+cy == 10) || (x+cx == -1) || (x+cx == 10) {
            return true;
        }

        // Check not only y+cy;x+cx, but and around this cell
        if have_neighbours(*field, y+cy, x+cx) {
            ship.can_be_placed = false;
            canbeplaced = false;
            ship.cells[i].cell = COLLISION_SHIP;
        }
        else if field[y+cy][x+cx].Type as int == EMPTY as int {
            ship.cells[i].cell = SHIP;
        }
    }

    if (canbeplaced) {
        ship.can_be_placed = true;
    }

    return false;
}

fn move_ship(field: &mut Field, ship: &mut Ship, input: i32) -> bool {
    let mut x = ship.coord.x;
    let mut y = ship.coord.y;

    match input {
        KEY_LEFT =>  {
            ship.coord.x-=1;
            if collision(&*field, ship) {
                ship.coord.x+=1;
            }
        }
        KEY_UP =>  {
            ship.coord.y-=1;
            if collision(&*field, ship) {
                ship.coord.y+=1;
            }
        }
        KEY_DOWN =>  {
            ship.coord.y+=1;
            if collision(&*field, ship) {
                ship.coord.y-=1;
            }
        }
        KEY_RIGHT =>  {
            ship.coord.x+=1;
            if collision(&*field, ship) {
                ship.coord.x-=1;
            }
        }
        KEY_F1 => {
            if (ship.can_be_placed) {
                return true;
            }
        }
        _ => {
            rotate_ship(ship);
            if collision(&*field, ship) {
                rotate_ship(ship);
            }
        }
    }
    return false;
}

fn remember_before(field: Field, ship: &Ship, before: &mut Vec<ShipCell>) {
    before.clear();
    let cury = ship.coord.y;
    let curx = ship.coord.x;
    for i in ship.cells.iter() {
        let y = cury + i.coord.y;
        let x = curx + i.coord.x;
        before.push(ShipCell { coord : Coord { y: y, x: x}, cell : field[y][x].Type });
    }
}

fn place_before(field: &mut Field, before: &Vec<ShipCell>) {
    for i in before.iter() {
        field[i.coord.y][i.coord.x].Type = i.cell;
    }
}

fn print_menu() {
    clear();
    let mut height : i32 = 0;
    let mut width : i32 = 0;
    getmaxyx(stdscr, &mut height, &mut width);
    mv(height/2-8, width/2-23);
    printw("______       _   _   _           _     _       \n");
    mv(height/2-7, width/2-23);
    printw("| ___ \\     | | | | | |         | |   (_)      \n");
    mv(height/2-6, width/2-23);
    printw("| |_/ / __ _| |_| |_| | ___  ___| |__  _ _ __  \n");
    mv(height/2-5, width/2-23);
    printw("| ___ \\/ _` | __| __| |/ _ \\/ __| '_ \\| | '_ \\ \n");
    mv(height/2-4, width/2-23);
    printw("| |_/ / (_| | |_| |_| |  __/\\__ \\ | | | | |_) |\n");
    mv(height/2-3, width/2-23);
    printw("\\____/ \\__,_|\\__|\\__|_|\\___||___/_| |_|_| .__/ \n");
    mv(height/2+3, width/2-8);
    printw("<F1>: Start game\n");
    mv(height/2+4, width/2-5);
    printw("<F2>: About\n");
    mv(height/2+5, width/2-5);
    printw("<Q>: Quit\n");
}

fn print_about() {
    clear();
    let mut height : i32 = 0;
    let mut width : i32 = 0;
    getmaxyx(stdscr, &mut height, &mut width);
    mv(height/2-2, width/2-10);
    printw("Battleship on rust.\n");
    mv(height/2, width/2-23);
    printw("Sources: https://github.com/queyenth/battleship\n");
}

fn ai_place_ship(field : &mut Field) {
    let mut curShip : Ship = Ship {
        coord : Coord {y : 0u, x : 1u},
        can_be_placed: true,
        cells : vec![ ShipCell { coord : Coord {y : -1u, x : 0u}, cell : SHIP },
                      ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP},
                      ShipCell { coord : Coord {y : 1u, x : 0u}, cell : SHIP},
                      ShipCell { coord : Coord {y : 2u, x : 0u}, cell : SHIP}]
    };
    let mut count : uint = 0u;
    while count != 10 {
        let x = (rand::random::<uint>() % 10u);
        let y = (rand::random::<uint>() % 10u);
        let r = (rand::random::<uint>() % 2u);
        if count == 0 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : -1u, x : 0u}, cell : SHIP },
                              ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP},
                              ShipCell { coord : Coord {y : 1u, x : 0u}, cell : SHIP},
                              ShipCell { coord : Coord {y : 2u, x : 0u}, cell : SHIP}]
            };
        }
        else if count <= 2 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : -1u, x : 0u}, cell : SHIP},
                              ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP},
                              ShipCell { coord : Coord {y : 1u, x : 0u}, cell : SHIP}]
            };
        }
        else if count <= 5 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP},
                              ShipCell { coord : Coord {y : 1u, x : 0u}, cell : SHIP}]
            };
        }
        else if count <= 9 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP}]
            };
        }
        if r == 1 {
            rotate_ship(&mut curShip);
        }
        if collision(field, &mut curShip) {
            continue;
        }
        else if curShip.can_be_placed {
            count+=1;
            add_ship(field, &curShip);
        }
    }
}

fn hide_ships(field: &mut Field) {
    for i in range(0u, 10u) {
        for j in range(0u, 10u) {
            if field[i][j].Type as int == SHIP as int {
                field[i][j].Type = HIDE_SHIP;
            }
        }
    }
}

fn tryToShot(field: &mut Field, y: uint, x: uint) -> bool {
    if field[y][x].Type as int == HIDE_SHIP as int {
        field[y][x].Type = COLLISION_SHIP;
        return true;
    }
    return false;
}

fn main() {
    initialize();

    let mut gamestatus : Status = START;

    let mut userfield = [[Cell { Type : EMPTY, Color : WHITE }, ..10], ..10];
    let mut enemyfield = [[Cell { Type : EMPTY, Color : WHITE }, ..10], ..10];
    let mut height : i32 = 0;
    let mut width : i32 = 0;
    let mut shotPos = Coord {x : 0u, y : 0u};
    getmaxyx(stdscr, &mut height, &mut width);

    while gamestatus as int != QUIT as int {
        match gamestatus {
            START => {
                // draw menu here
                let qkey = 'q' as i32;
                while gamestatus as int != PLACE_SHIP as int {
                    print_menu();
                    let mut ch = get_input();
                    match ch {
                        KEY_F1 => {
                            gamestatus = PLACE_SHIP;
                        }
                        KEY_F2 => {
                            print_about();
                            ch = get_input();
                        }
                        _ => {}
                    }
                }
            }
            PLACE_SHIP => {
                clear();
                ai_place_ship(&mut enemyfield);
                hide_ships(&mut enemyfield);
                if DEBUG {
                    ai_place_ship(&mut userfield);
                    gamestatus = PLAYER_TURN;
                }
                else {
                let mut curShip : Ship = Ship {
                    coord : Coord {y : 1u, x : 0u},
                    can_be_placed: true,
                    cells : vec![ ShipCell { coord : Coord {y : -1u, x : 0u}, cell : SHIP },
                                  ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP},
                                  ShipCell { coord : Coord {y : 1u, x : 0u}, cell : SHIP},
                                  ShipCell { coord : Coord {y : 2u, x : 0u}, cell : SHIP}]
                };

                let mut before : Vec<ShipCell> = vec![];

                remember_before(userfield, &curShip, &mut before);
                add_ship(&mut userfield, &curShip);

                print_field(userfield, height/2-5, width/2-30);
                print_field(enemyfield, height/2-5, width/2+5);

                let mut count = 0u;
                let mut ch = getch();

                // TODO: rewrite this, 'count' is bad solution
                loop {
                    if ch == ('q' as i32) {
                        gamestatus = QUIT;
                        break;
                    }
                    remove_ship(&mut userfield, &curShip);
                    let shipWasPlaced = move_ship(&mut userfield, &mut curShip, ch);
                    place_before(&mut userfield, &before);
                    remember_before(userfield, &curShip, &mut before);
                    if (shipWasPlaced) {
                        add_ship(&mut userfield, &curShip);
                        count+=1;
                        if count <= 2 {
                            curShip = Ship {
                                coord : Coord {y : 1u, x : 0u},
                                can_be_placed: true,
                                cells : vec![ ShipCell { coord : Coord {y : -1u, x : 0u}, cell : SHIP},
                                              ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP},
                                              ShipCell { coord : Coord {y : 1u, x : 0u}, cell : SHIP}]
                            };
                            remember_before(userfield, &curShip, &mut before);
                            collision(&userfield, &mut curShip);
                            add_ship(&mut userfield, &curShip);
                        }
                        else if count <= 5 {
                            curShip = Ship {
                                coord : Coord {y : 0u, x : 0u},
                                can_be_placed: true,
                                cells : vec![ ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP},
                                              ShipCell { coord : Coord {y : 1u, x : 0u}, cell : SHIP}]
                            };
                            remember_before(userfield, &curShip, &mut before);
                            collision(&userfield, &mut curShip);
                            add_ship(&mut userfield, &curShip);
                        }
                        else if count <= 9 {
                            curShip = Ship {
                                coord : Coord {y : 0u, x : 0u},
                                can_be_placed: true,
                                cells : vec![ ShipCell { coord : Coord {y : 0u, x : 0u}, cell : SHIP}]
                            };
                            remember_before(userfield, &curShip, &mut before);
                            collision(&userfield, &mut curShip);
                            add_ship(&mut userfield, &curShip);
                        }
                        else {
                            gamestatus = PLAYER_TURN;
                            break;
                        }
                    }
                    else {
                        add_ship(&mut userfield, &curShip);
                    }

                    print_field(userfield, height/2-5, width/2-30);
                    print_field(enemyfield, height/2-5, width/2+5);

                    // Need refresh after each frame?
                    refresh();
                    ch = getch();
                }
                }
            }
            PLAYER_TURN => {
                clear();
                print_field(userfield, height/2-5, width/2-30);
                print_field(enemyfield, height/2-5, width/2+5);
                let mut ch = getch();
                match ch {
                    KEY_LEFT => {
                        if shotPos.x as int - 1 as int >= 0 {
                            shotPos.x-=1;
                        }
                    }
                    KEY_RIGHT => {
                        if shotPos.x as int + 1 as int <= 9 {
                            shotPos.x+=1;
                        }
                    }
                    KEY_UP => {
                        if shotPos.y as int - 1 as int >= 0 {
                            shotPos.y-=1;
                        }
                    }
                    KEY_DOWN => {
                        if shotPos.y as int + 1 as int >= 0 {
                            shotPos.y+=1;
                        }
                    }
                    KEY_F1 => {
                        if !tryToShot(&mut enemyfield, shotPos.y, shotPos.x) {
                            // gamestatus = AI_TURN;
                        }
                    }
                    _ => {}
                }
                enemyfield[shotPos.y][shotPos.x].Type = SHOT_POS;
            }
            AI_TURN => {
                clear();
                printw("Ai turn!\n");
                gamestatus = PLAYER_TURN;
            }
            _ => {}
        }
    }

    deinitialize();
}
