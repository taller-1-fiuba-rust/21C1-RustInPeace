extern crate redis;


use proyecto_taller_1::{
    domain::{
        entities::{
            config::Config,
            key_value_item::{KeyValueItem, ValueType},
            server::Server,
        },
        implementations::database::Database,
    },
    services::{server_service, worker_service::ThreadPool},
};

use std::{
    error::Error,
    fmt,
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

const ADDR: &str = "redis://127.0.0.1:8080/";

type TestResult = Result<(), Box<dyn Error>>;
type TestFunction = fn() -> TestResult;

#[derive(Copy, Clone)]
struct Test {
    name: &'static str,
    func: TestFunction,
}

#[derive(Debug)]
struct ReturnError {
    expected: String,
    got: String,
}

impl fmt::Display for ReturnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expected: {}\n\
            Got:     {}",
            self.expected, self.got
        )
    }
}

impl Error for ReturnError {}

#[test]
fn test_main() {
    let pool = ThreadPool::new(4);

    let handle: thread::JoinHandle<()> = thread::spawn(|| {
        let config_file = String::from("./src/dummy_config.txt");
        let db_file = String::from("./src/dummy_database.txt");
        let log_file = String::from("./src/dummy_log.txt");

        match std::fs::File::create(&config_file) {
            Ok(_) => {}
            Err(e) => println!("Error creating config {:?}", e),
        }

        match std::fs::File::create(&db_file) {
            Ok(_) => {}
            Err(e) => println!("Error creating db {:?}", e),
        }

        match std::fs::File::create(&log_file) {
            Ok(_) => {}
            Err(e) => println!("Error creating log {:?}", e),
        }

        let mut config = Config::new(config_file);
        config
            .set_attribute(String::from("verbose"), String::from("1"))
            .unwrap();

        let mut database = Database::new(db_file);
        let added_item = KeyValueItem::new(
            String::from("key_1"),
            ValueType::StringType(String::from("value_key_1")),
        );
        database.add(added_item);
        let added_item = KeyValueItem::new(
            String::from("key_2"),
            ValueType::StringType(String::from("value_key_2")),
        );
        database.add(added_item);
        let added_item = KeyValueItem::new(
            String::from("key_3"),
            ValueType::StringType(String::from("value_key_3")),
        );
        database.add(added_item);
        let added_item = KeyValueItem::new(
            String::from("key_4"),
            ValueType::StringType(String::from("value_key_4")),
        );
        database.add(added_item);

        let added_item = KeyValueItem::new(
            String::from("mykey"),
            ValueType::StringType(String::from("Hello")),
        );
        database.add(added_item);

        let added_item = KeyValueItem::new(
            String::from("key_to_decr"),
            ValueType::StringType(String::from("10")),
        );
        database.add(added_item);

        let added_item = KeyValueItem::new(
            String::from("key_getdel"),
            ValueType::StringType(String::from("Hello")),
        );
        database.add(added_item);


        match &mut Server::new(String::from("8080"), log_file, String::from("0")) {
            Ok(server) => server_service::init(server, database, config),
            Err(e) => println!("Error on server: {:?}", e),
        }
        std::fs::remove_file("./src/dummy_config.txt").unwrap();
        std::fs::remove_file("./src/dummy_log.txt").unwrap();
        std::fs::remove_file("./src/dummy_database.txt").unwrap();
    });

    sleep(Duration::from_secs(5));
    let (sender, receiver) = mpsc::channel::<String>();

    for test in TESTS.iter().cloned() {
        let tx = sender.clone();

        pool.spawn(move || {
            let res = (test.func)();

            if let Err(e) = res {
                tx.send(format!("\n{}: \n{}\n", test.name, e.to_string()))
                    .unwrap();
            } else {
                println!("{} .. ok", test.name);
            }
        });

        if let Ok(err) = receiver.try_recv() {
            panic!("{}", err);
        }
    }

    if let Ok(err) = receiver.recv_timeout(Duration::from_secs(10)) {
        panic!("{}", err);
    }

    pool.spawn(shutdown);
    let _ = handle.join().expect("Couldnt join");
}

const TESTS: &[Test] = &[
    Test {
        name: "server command: config get verbose",
        func: test_config_get_verbose,
    },
    Test {
        name: "server command: config set maxmemory",
        func: test_config_set_maxmemory,
    },
    // Test {
    //     name: "server command: dbsize",
    //     func: test_dbsize,
    // },
    // Test {
    //     name: "server command: flushdb",
    //     func: test_flushdb,
    // },
    Test {
        name: "keys command: del",
        func: test_keys_del,
    },
    Test {
        name: "keys command: exists",
        func: test_keys_exists,
    },
    Test {
        name: "keys command: persist",
        func: test_keys_persist,
    },
    Test {
        name: "keys command: rename",
        func: test_keys_rename,
    },
    Test {
        name: "keys command: copy",
        func: test_keys_copy,
    },
    Test {
        name: "keys command: copy replace",
        func: test_keys_copy_with_replace,
    },
    Test {
        name: "string command: append mykey newvalue",
        func: test_string_append,
    },
    Test {
        name: "string command: decrby mykey 3",
        func: test_string_decrby,
    },
    Test {
        name: "string command: get key_1",
        func: test_string_get,
    },
    Test {
        name: "string command: getdel key_getdel",
        func: test_string_getdel,
    },
];

fn connect() -> Result<redis::Connection, Box<dyn Error>> {
    let client = redis::Client::open(ADDR)?;
    let con = client.get_connection()?;
    Ok(con)
}

fn shutdown() {
    let mut con = connect().unwrap();
    let _: redis::RedisResult<()> = redis::cmd("SHUTDOWN").query(&mut con);
}

fn test_config_get_verbose() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("CONFIG")
        .arg("get")
        .arg("verbose")
        .query(&mut con)?;

    if &ret[0] == &String::from("1") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: String::from(&ret[0]),
        }));
    }
}

fn test_config_set_maxmemory() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("CONFIG")
        .arg("set")
        .arg("maxmemory")
        .arg("2mb")
        .query(&mut con)?;

    if ret == String::from("ok") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("ok"),
            got: ret,
        }));
    }
}

// no lo testeo porque el resultado depende de si se ejecuta antes o despues de borrar una clave
fn _test_dbsize() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("DBSIZE").query(&mut con)?;

    if ret == 4 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("4"),
            got: ret.to_string(),
        }));
    }
}

// no lo testeo porque depende el orden en que se ejecuten podrian fallarme los otros tests
fn _test_flushdb() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("FLUSHDB").query(&mut con)?;

    if ret == String::from("Erased database") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Erased database"),
            got: ret,
        }));
    }
}

fn test_keys_del() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("DEL").arg("key_4").query(&mut con)?;

    if ret == 1 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }));
    }
}

fn test_keys_exists() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXISTS").arg("key_1").query(&mut con)?;

    if ret == 1 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }));
    }
}

fn test_keys_persist() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("PERSIST").arg("key_1").query(&mut con)?;

    if ret == 1 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }));
    }
}

fn test_keys_rename() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("RENAME")
        .arg("key_2")
        .arg("key_2_renamed")
        .query(&mut con)?;

    if ret == String::from("OK") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("OK"),
            got: ret.to_string(),
        }));
    }
}

fn test_keys_copy() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("COPY")
        .arg("key_2")
        .arg("key_3")
        .query(&mut con)?;

    if ret == 0 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("0"),
            got: ret.to_string(),
        }));
    }
}

fn test_keys_copy_with_replace() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("COPY")
        .arg("key_1")
        .arg("key_3")
        .arg("REPLACE")
        .query(&mut con)?;

    if ret == 1 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }));
    }
}

fn test_string_append() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("APPEND")
        .arg("mykey")
        .arg(" World")
        .query(&mut con)?;

    if ret == 11 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("11"),
            got: ret.to_string()
        }));
    }
}

fn test_string_decrby() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("DECRBY")
        .arg("key_to_decr")
        .arg(3)
        .query(&mut con)?;

    if ret == 7 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("7"),
            got: ret.to_string(),
        }));
    }
}

fn test_string_get() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("GET")
        .arg("key_1")
        .query(&mut con)?;

    if ret == String::from("value_key_1") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("value_key_1"),
            got: ret,
        }));
    }
}

fn test_string_getdel() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("GETDEL")
        .arg("key_getdel")
        .query(&mut con)?;

    if ret == String::from("Hello") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Hello"),
            got: ret,
        }));
    }
}

