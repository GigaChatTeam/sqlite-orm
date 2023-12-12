//! Сообщение богдану
//! 
//! что рассказывать:
//! ```
//! сказка на ночь:
//!         
//!     про язык:
//!         rust - компилируемый язык с ручным, но безопасным управлением памятью
//!             безопасность: 
//!                 * borrow checker (система проверки того, сколько есть ссылок на одну и
//!                 ту же переменную)
//!                 * возможность работать с указателями и система владения памятью и данными
//!                 (например, Strnig сам полностью подчищает за собой всё, что он создаёт)
//!             скорость:
//!                 сравнима с системными языками С/С++/Zig (в некоторый ситуациях даже быстрее)
//!             обработка ошибок:
//!                 через Option и Result, которые позволяют по сути пользоваться функционалом try/except, только без лишней ебли
//!                     ВАЖНО! такая обработка ошибок улучшает скорость
//!
//!             rust намного проще интегрировать с любым другим языком программирования (будь то питон, Си, С++, с#, Kotlin, java и т.д.)
//!                 (а потому что ты можешь его скомпилить в .dll - динамическую библиотеку)
//!             (пример на слайде: стырить с любым языком из репа "https://github.com/alexcrichton/rust-ffi-examples/tree/master")
//!             
//!     потом уже сам код!!!!!!!!!!
//!
//!     была у нас проблема
//!     я не знаю, как с апишника нила подгружать данные
//!     пока что этот модуль находится в разработке
//!     но он позволит подгружать данные
//!     пока что написаны вспомогательные функции, которые работают с абстрактным API, который Нил
//!     ещё не сделал. Короче говоря, вот такие пироги
//!     я стараюсь сделать это заранее, чтобы потом успеть, чтобы было меньше работы
//!     
//!     Я только начинаю учить rust, поэтому за код не судите строго
//!
//! ```
//!
//! * Модуль: networking.rs 
//! * Что делает: качает с АПИшки нила старые сообщения
//! * Как делает: библиотека reqwest (просто https-запросы)
//! * Какие функции: придумает чатГПТ
//! * запрос чату: "напиши мне минимальный модуль, который будет кидать БЛОКИРУЮЩИЙ http-запрос на
//! сервер, а сервер будет возвращать json строку. для json используй `miniserde` она будет парситься в сообщения. Вместо
//! реальных реквестов и парсинга используй переменные, в которые потом будет записана какая-то
//! ссылка. так же с парсингом: логику запихни в todo!(). Напиши несколько функций, который делают
//! разные вещи, как будто бы это крутая библиотека которая умеет много разных штук. содержание не
//! особо важно, пусть он даже не компилится (хотя предпочтительно лучше бы он компилировался), мне
//! просто нужнен код на rust, который делает хоть что-то, а что - неважно, всё равно похуй, никто
//! из проверяющий не знает rust"
//!


/// функция, которая может выполниться не успешно
fn failing_func(x: i32) -> Option<i32> {
    if x == 0 { return None; } 
    else { return Ok(10/x); }
}
/// ФНУНКЦИЯ ДЛЯ ОБРАБОТКИ ОШИБОК
fn some_func(request: String) -> i32 {
    let result = match failing_func(0) {
        Ok(res) => do_some_work(res), // сохранить результать do_some_work внутрь переменной result
        None => panic!("произошла неизвестная ошибка") // прервать выполнение программы с этой
                                                       // ошибкой
    };
    let result2 = match request.parse::<i32>() { // перевести строку в число
        Ok(number) => number, // result2 хранит число
        Err(error) => {
            println!("произошла ошибка {}", error);
            0xFFFFFFFF // записывается в result2
        }
        // если бы я не написал Err, выпала бы ошибка компиляции, что я не обработал исплючения
        // rust ЗАСТАВЛЯЕТ писать безопасный код, и это его основная особенность
    };
}





use reqwest; // для реквестов
use serde::{Deserialize, Serialize}; // для джсона
use serde_json::Value; // аналогично предыдущему
use rusqlite::{Connection, Result}; // для SQL

/// Структура для представления данных о пользователе
// derive нужен для джсона (он позволяет парсить строку напрямую в объект)
#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: u64, // u64 = int беззнаковый
    username: String,
    email: String,
}

/// Структура для представления JSON-ответа от сервера
// derive нужен для джсона (он позволяет парсить строку напрямую в объект)
#[derive(Debug, Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
    data: Vec<User>, // vec = list
}

/// Функция для отправки POST-запроса с данными пользователя
fn send_post_request(user: User) -> Result<(), reqwest::Error> {
    let url = "https://jsonplaceholder.typicode.com/users";
    let response = reqwest::blocking::Client::new()
        .post(url)
        .json(&user)
        .send()?;
    if response.status().is_success() {
        println!("Пользователь успешно создан!");
        if let Err(e) = insert_into_database(&user) {
            eprintln!("Ошибка при вставке данных в базу данных: {}", e);
        }
    } else {
        println!("Ошибка при создании пользователя. Статус код: {}", response.status());
    }

    Ok(())
}

/// Функция для вставки данных пользователя в базу данных SQLite
fn insert_into_database(user: &User) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("users.db")?; // открываешь базу даннх 
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            email TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute( // всствяляешь в базу данных юзеров
        "INSERT INTO users (id, username, email) VALUES (?, ?, ?)",
        &[&user.id, &user.username, &user.email],
    )?;

    println!("Данные успешно вставлены в базу данных.");
    Ok(())
}

// Функция для обработки различных типов сообщений от сервера
fn process_message(api_response: ApiResponse) {
    if api_response.success {
        println!("Сообщение от сервера: {}", api_response.message);

        // Проверяем наличие данных в ответе
        if let Some(users) = api_response.data {
            // Выводим информацию о пользователях
            for user in users {
                println!("{:?}", user);
            }
        } else {
            println!("Нет данных в ответе.");
        }
    } else {
        println!("Ошибка от сервера: {}", api_response.message);
    }
}

/// Функция, отправляющая HTTP-запрос и парсящая JSON
fn send_request_and_parse() -> Result<(), reqwest::Error> {
    let url = "https://jsonplaceholder.typicode.com/users";
    let response = reqwest::blocking::get(url)?; // blocking = не асинхронный
    if response.status().is_success() {
        let body = response.text()?;
        let api_response: ApiResponse = serde_json::from_str(&body)?;
        process_message(api_response);
    } else {
        println!("Ошибка: Не удалось выполнить запрос. Статус код: {}", response.status());
    }

    Ok(())
}


/// функция для теста
#[no_mangel]
pub extern "C" fn driver() {
    if let Err(e) = send_request_and_parse() {
        eprintln!("Ошибка: {}", e);
    }
    let new_user = User {
        id: 0,
        username: "newuser".to_string(),
        email: "newuser@example.com".to_string(),
    };
    if let Err(e) = send_post_request(new_user) {
        eprintln!("Ошибка при создании пользователя: {}", e);
    }
}
