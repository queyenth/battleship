#![feature(globs)]
#![feature(negate_unsigned)]

extern crate ncurses;
extern crate rand;

use ncurses::*;
use rand::Rng;

#[derive(Copy, Clone)]
enum CellType {
    EMPTY = 0,
    SHOT = 1,
    SHIP = 2,
    COLLISION_SHIP = 3,
    HIDE_SHIP = 4,
    SHOT_POS = 5,
}

#[derive(Copy, Clone)]
enum Color {
    BACKGROUND = 15,
    GREEN = 16,
    PINK = 17,
    RED = 18,
    WHITE = 19,
    BLUE = 20,
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
struct Cell {
    Type: CellType,
    Color: Color
}

#[derive(Copy, Clone)]
struct Coord {
    y: u32,
    x: u32
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

type Field = [[Cell; 10]; 10];

impl Cell {
   fn print(&self) {
       let mut symbol = " ";
       match self.Type {
           CellType::HIDE_SHIP => {
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
           CellType::EMPTY => {
               attron(COLOR_PAIR(COLOR_PAIR_EMPTY));
               printw(".");
               attroff(COLOR_PAIR(COLOR_PAIR_EMPTY));
           }
           CellType::SHOT => {
               attron(COLOR_PAIR(COLOR_PAIR_SHOT));
               printw("o");
               attroff(COLOR_PAIR(COLOR_PAIR_SHOT));
           }
           CellType::SHIP => {
               attron(COLOR_PAIR(COLOR_PAIR_SHIP));
               printw("x");
               attroff(COLOR_PAIR(COLOR_PAIR_SHIP));
           }
           CellType::COLLISION_SHIP => {
               attron(COLOR_PAIR(COLOR_PAIR_COLLISION_SHIP));
               printw("x");
               attroff(COLOR_PAIR(COLOR_PAIR_COLLISION_SHIP));
           }
           CellType::SHOT_POS => {
               attron(COLOR_PAIR(COLOR_PAIR_SHOT_POS));
               printw("o");
               attroff(COLOR_PAIR(COLOR_PAIR_SHOT_POS));
           }
       }
   }
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

    bkgd(' ' as u64 | COLOR_PAIR(COLOR_PAIR_EMPTY) as u64);
}

fn initialize() {
    initscr();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

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
    for i in 1..11 {
        attron(COLOR_PAIR(COLOR_PAIR_NUMBER));
        println!(" {}", i);
        attroff(COLOR_PAIR(COLOR_PAIR_NUMBER));
    }
    for i in 1..11 {
        if i != 10 {
            mv(y+(i as i32), x+1i32);
        }
        else {
            mv(y+(i as i32), x);
        }
        attron(COLOR_PAIR(COLOR_PAIR_NUMBER));
        println!("{}", i);
        attroff(COLOR_PAIR(COLOR_PAIR_NUMBER));
    }
    for i in 0..10 {
        mv(y+(i as i32)+1i32, x+2i32);
        for j in 0..10 {
            printw(" ");
            field[i][j].print();
        }
        println!("");
    }
}

fn add_ship(field: &mut Field, ship: &Ship) {
    let y = ship.coord.y as usize;
    let x = ship.coord.x as usize;
    for i in ship.cells.iter() {
        let cy = i.coord.y as usize;
        let cx = i.coord.x as usize;
        field[y+cy][x+cx] = Cell { Type : i.cell, Color : Color::GREEN };
    }
}

fn remove_ship(field: &mut Field, ship: &Ship) {
    let y = ship.coord.y as usize;
    let x = ship.coord.x as usize;
    for i in ship.cells.iter() {
        let cy = i.coord.y as usize;
        let cx = i.coord.x as usize;
        field[y+cy][x+cx] = Cell { Type : CellType::EMPTY, Color : Color::WHITE };
    }
}

fn rotate_ship(ship: &mut Ship) {
    for i in 0..ship.cells.len() {
        let temp = ship.cells[i].coord.y;
        ship.cells[i].coord.y = ship.cells[i].coord.x;
        ship.cells[i].coord.x = temp;
    }
}

fn have_neighbours(field: Field, y: usize, x: usize) -> bool {
    let mx : i32 = x as i32+1;
    let lx : i32 = x as i32-1;
    let my : i32 = y as i32+1;
    let ly : i32 = y as i32-1;

    let mxx : usize = x as usize - 1;
    let lxx : usize = x as usize + 1;
    let myy : usize = y as usize - 1;
    let lyy : usize = y as usize + 1;

    if field[y as usize][x as usize].Type as i32 == CellType::SHIP as i32 { return true; }
    if (lx >= 0) && (field[y as usize][mxx].Type as i32 == CellType::SHIP as i32) { return true; }
    if (mx < 10) && (field[y as usize][lxx].Type as i32 == CellType::SHIP as i32) { return true; }
    if (ly >= 0) && (field[myy][x as usize].Type as i32 == CellType::SHIP as i32) { return true; }
    if (my < 10) && (field[lyy][x as usize].Type as i32 == CellType::SHIP as i32) { return true; }
    if (ly >= 0) && (lx >= 0) && (field[myy][mxx].Type as i32 == CellType::SHIP as i32)  { return true; }
    if (ly >= 0) && (mx < 10) && (field[myy][lxx].Type as i32 == CellType::SHIP as i32)  { return true; }
    if (my < 10) && (lx >= 0) && (field[lyy][mxx].Type as i32 == CellType::SHIP as i32)  { return true; }
    if (my < 10) && (mx < 10) && (field[lyy][lxx].Type as i32 == CellType::SHIP as i32)  { return true; }

    return false;
}

fn collision(field: &Field, ship: &mut Ship) -> bool {
    let y = ship.coord.y as usize;
    let x = ship.coord.x as usize;
    println!("{} {}", y, x);
    let mut canbeplaced = true;
    for i in 0..ship.cells.len() {
        let cy = ship.cells[i].coord.y as usize;
        let cx = ship.cells[i].coord.x as usize;
        println!("{} {}", cy, cx);
        if (y+cy == -1) || (y+cy == 10) || (x+cx == -1) || (x+cx == 10) {
            return true;
        }

        // Check not only y+cy;x+cx, but and around this cell
        if have_neighbours(*field, y+cy, x+cx) {
            ship.can_be_placed = false;
            canbeplaced = false;
            ship.cells[i].cell = CellType::COLLISION_SHIP;
        }
        else if field[y+cy][x+cx].Type as i32 == CellType::EMPTY as i32 {
            ship.cells[i].cell = CellType::SHIP;
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
    let cury = ship.coord.y as usize;
    let curx = ship.coord.x as usize;
    for i in ship.cells.iter() {
        let y = cury + i.coord.y as usize;
        let x = curx + i.coord.x as usize;
        before.push(ShipCell { coord : Coord { y: y as u32, x: x as u32}, cell : field[y][x].Type });
    }
}

fn place_before(field: &mut Field, before: &Vec<ShipCell>) {
    for i in before.iter() {
        field[i.coord.y as usize][i.coord.x as usize].Type = i.cell;
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
        coord : Coord {y : 0, x : 1},
        can_be_placed: true,
        cells : vec![ ShipCell { coord : Coord {y : -1, x : 0}, cell : CellType::SHIP },
                      ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP},
                      ShipCell { coord : Coord {y : 1, x : 0}, cell : CellType::SHIP},
                      ShipCell { coord : Coord {y : 2, x : 0}, cell : CellType::SHIP}]
    };
    let mut count : u32 = 0;
    while count != 10 {
        let x = (rand::thread_rng().gen_range(0, 10));
        let y = (rand::thread_rng().gen_range(0, 10));
        let r = (rand::thread_rng().gen_range(0, 2));
        if count == 0 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : -1, x : 0}, cell : CellType::SHIP },
                              ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP},
                              ShipCell { coord : Coord {y : 1, x : 0}, cell : CellType::SHIP},
                              ShipCell { coord : Coord {y : 2, x : 0}, cell : CellType::SHIP}]
            };
        }
        else if count <= 2 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : -1, x : 0}, cell : CellType::SHIP},
                              ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP},
                              ShipCell { coord : Coord {y : 1, x : 0}, cell : CellType::SHIP}]
            };
        }
        else if count <= 5 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP},
                              ShipCell { coord : Coord {y : 1, x : 0}, cell : CellType::SHIP}]
            };
        }
        else if count <= 9 {
            curShip = Ship {
                coord : Coord {y : y, x : x},
                can_be_placed: true,
                cells : vec![ ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP}]
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
    for i in 0..10 {
        for j in 0..10 {
            if field[i][j].Type as i32 == CellType::SHIP as i32 {
                field[i][j].Type = CellType::HIDE_SHIP;
            }
        }
    }
}

fn tryToShot(field: &mut Field, y: u32, x: u32) -> bool {
    if field[y as usize][x as usize].Type as i32 == CellType::HIDE_SHIP as i32 {
        field[y as usize][x as usize].Type = CellType::COLLISION_SHIP;
        return true;
    }
    return false;
}

fn main() {
    initialize();

    let mut gamestatus : Status = Status::START;

    let mut userfield = [[Cell { Type : CellType::EMPTY, Color : Color::WHITE }; 10]; 10];
    let mut enemyfield = [[Cell { Type : CellType::EMPTY, Color : Color::WHITE }; 10]; 10];
    let mut height : i32 = 0;
    let mut width : i32 = 0;
    let mut shotPos = Coord {x : 0, y : 0};
    getmaxyx(stdscr, &mut height, &mut width);

    while gamestatus as i32 != Status::QUIT as i32 {
        match gamestatus {
            Status::START => {
                // draw menu here
                let qkey = 'q' as i32;
                while gamestatus as i32 != Status::PLACE_SHIP as i32 {
                    print_menu();
                    let mut ch = get_input();
                    match ch {
                        KEY_F1 => {
                            gamestatus = Status::PLACE_SHIP;
                        }
                        KEY_F2 => {
                            print_about();
                            ch = get_input();
                        }
                        KEY_F3 => {
                            panic!("");
                        }
                        _ => {}
                    }
                }
            }
            Status::PLACE_SHIP => {
                clear();
                ai_place_ship(&mut enemyfield);
                hide_ships(&mut enemyfield);
                if DEBUG {
                    ai_place_ship(&mut userfield);
                    gamestatus = Status::PLAYER_TURN;
                }
                else {
                let mut curShip : Ship = Ship {
                    coord : Coord {y : 1, x : 0},
                    can_be_placed: true,
                    cells : vec![ ShipCell { coord : Coord {y : -1, x : 0}, cell : CellType::SHIP },
                                  ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP},
                                  ShipCell { coord : Coord {y : 1, x : 0}, cell : CellType::SHIP},
                                  ShipCell { coord : Coord {y : 2, x : 0}, cell : CellType::SHIP}]
                };

                let mut before : Vec<ShipCell> = vec![];

                remember_before(userfield, &curShip, &mut before);
                add_ship(&mut userfield, &curShip);

                print_field(userfield, height/2-5, width/2-30);
                print_field(enemyfield, height/2-5, width/2+5);

                let mut count = 0;
                let mut ch = getch();

                // TODO: rewrite this, 'count' is bad solution
                loop {
                    if ch == ('q' as i32) {
                        gamestatus = Status::QUIT;
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
                                coord : Coord {y : 1, x : 0},
                                can_be_placed: true,
                                cells : vec![ ShipCell { coord : Coord {y : -1, x : 0}, cell : CellType::SHIP},
                                              ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP},
                                              ShipCell { coord : Coord {y : 1, x : 0}, cell : CellType::SHIP}]
                            };
                            remember_before(userfield, &curShip, &mut before);
                            collision(&userfield, &mut curShip);
                            add_ship(&mut userfield, &curShip);
                        }
                        else if count <= 5 {
                            curShip = Ship {
                                coord : Coord {y : 0, x : 0},
                                can_be_placed: true,
                                cells : vec![ ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP},
                                              ShipCell { coord : Coord {y : 1, x : 0}, cell : CellType::SHIP}]
                            };
                            remember_before(userfield, &curShip, &mut before);
                            collision(&userfield, &mut curShip);
                            add_ship(&mut userfield, &curShip);
                        }
                        else if count <= 9 {
                            curShip = Ship {
                                coord : Coord {y : 0, x : 0},
                                can_be_placed: true,
                                cells : vec![ ShipCell { coord : Coord {y : 0, x : 0}, cell : CellType::SHIP}]
                            };
                            remember_before(userfield, &curShip, &mut before);
                            collision(&userfield, &mut curShip);
                            add_ship(&mut userfield, &curShip);
                        }
                        else {
                            gamestatus = Status::PLAYER_TURN;
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
            Status::PLAYER_TURN => {
                clear();
                print_field(userfield, height/2-5, width/2-30);
                print_field(enemyfield, height/2-5, width/2+5);
                let mut ch = getch();
                match ch {
                    KEY_LEFT => {
                        if shotPos.x as i32 - 1 as i32 >= 0 {
                            shotPos.x-=1;
                        }
                    }
                    KEY_RIGHT => {
                        if shotPos.x as i32 + 1 as i32 <= 9 {
                            shotPos.x+=1;
                        }
                    }
                    KEY_UP => {
                        if shotPos.y as i32 - 1 as i32 >= 0 {
                            shotPos.y-=1;
                        }
                    }
                    KEY_DOWN => {
                        if shotPos.y as i32 + 1 as i32 >= 0 {
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
                enemyfield[shotPos.y as usize][shotPos.x as usize].Type = CellType::SHOT_POS;
            }
            Status::AI_TURN => {
                clear();
                printw("Ai turn!\n");
                gamestatus = Status::PLAYER_TURN;
            }
            _ => {}
        }
    }

    deinitialize();
}
