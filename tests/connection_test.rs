// extern crate proyecto_taller_1;
extern crate redis;

use proyecto_taller_1::{
    domain::{
        entities::{config::Config, server::Server},
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
        let database = Database::new(db_file);

        match &mut Server::new(String::from("8080"), log_file, String::from("0")) {
            Ok(server) => server_service::init(server, database, config),
            Err(e) => println!("Error on server: {:?}", e),
        }
        // std::fs::remove_file("./src/dummy_config.txt").unwrap();
        // std::fs::remove_file("./src/dummy_log.txt").unwrap();
        // std::fs::remove_file("./src/dummy_database.txt").unwrap();
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
    Test {
        name: "server command: dbsize",
        func: test_dbsize,
    },
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

fn test_dbsize() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("DBSIZE")
        .query(&mut con)?;

    // OJO QUE AHORA ES 2 PORQUE ESTA HARCODEADO EL CONSTRUCTOR DE DATABASE
    if ret == 2 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("2"),
            got: ret.to_string(),
        }));
    }
}

fn test_flushdb() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("FLUSHDB")
        .query(&mut con)?;

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

    // OJO PORQUE SALE DEL HARDCODEO EN DATABASE NEW
    let ret: usize = redis::cmd("DEL")
        .arg("clave_2")
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

fn test_keys_exists() -> TestResult {
    let mut con = connect()?;

    // OJO PORQUE SALE DEL HARDCODEO EN DATABASE NEW
    let ret: usize = redis::cmd("EXISTS")
        .arg("clave_1")
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

fn test_keys_persist() -> TestResult {
    let mut con = connect()?;

    // OJO PORQUE SALE DEL HARDCODEO EN DATABASE NEW
    let ret: usize = redis::cmd("PERSIST")
        .arg("clave_1")
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
