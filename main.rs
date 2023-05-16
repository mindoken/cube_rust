#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate rusqlite;
extern crate serde_json;

use rocket::State;
use rocket_contrib::templates::Template;
use rocket_contrib::serve::StaticFiles;
use std::sync::Mutex;
use rusqlite::{params, Connection};
use serde_json::json;

const CUBE_SIZE: f64 = 100.0;


impl Cube {
    fn update(&mut self, dt: f64) {
        self.rotation += 60.0 * dt;
    }
}

#[get("/")]
fn index() -> Template {
    let context = ();
    Template::render("index", &context)
}

#[get("/cube")]
fn cube(db: State<Mutex<Connection>>) -> Template {
    let cube = Cube {vertices: vec![
        -CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE,
        CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE,
        -CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE,
        CUBE_SIZE, CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, CUBE_SIZE,
    ],
    indices: vec![
        0, 1, 2, 0, 2, 3,
        1, 5, 6, 1, 6, 2,
        5, 4, 7, 5, 7, 6,
        4, 0, 3, 4, 3, 7,
        3, 2, 6, 3, 6, 7,
        4, 5, 1, 4, 1, 0,
    ],
    rotation: 0.0,
};
    let conn = db.lock().unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cube (
             id INTEGER PRIMARY KEY,
             rotation FLOAT NOT NULL
         )",
        params![],
    ).unwrap();

    let mut stmt = conn.prepare("INSERT INTO cube (rotation) VALUES (?1)").unwrap();
    stmt.execute(params![cube.rotation]).unwrap();

    let mut binding = conn.prepare("SELECT * FROM cube").unwrap();
    let mut rows = binding.query(params![]).unwrap();

    let row = rows.next().unwrap().unwrap();
    let id: i32 = row.get(0).unwrap();
    let rotation: f64 = row.get(1).unwrap();

    let context = json!({
        "vertices": cube.vertices,
        "indices": cube.indices,
        "rotation": rotation,
    });

    Template::render("cube", &context)
}




#[post("/cube/update?<dt>")]
fn update_cube(dt: f64, db: State<Mutex<Connection>>) -> String {
    let mut cube = Cube {vertices: vec![
        -CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE,
        CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE,
        -CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE,
        CUBE_SIZE, CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE, CUBE_SIZE,
    ],
    indices: vec![
        0, 1, 2, 0, 2, 3,
        1, 5, 6, 1, 6, 2,
        5, 4, 7, 5, 7, 6,
        4, 0, 3, 4, 3, 7,
        3, 2, 6, 3, 6, 7,
        4, 5, 1, 4, 1, 0,
    ],
    rotation: 11.0,
};
    cube.update(dt);

    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE cube SET rotation = ?1 WHERE id = 1",
        params![cube.rotation],
    ).unwrap();

    format!("{{\"rotation\": {}}}", cube.rotation)
}

fn main() {
    let conn = Connection::open("cube.db").unwrap();
    let db = Mutex::new(conn);

    rocket::ignite()
        .mount("/", routes![index, cube, update_cube])
        .mount("/static", StaticFiles::from("static"))
        .attach(Template::fairing())
        .manage(db)
        .launch();
}

