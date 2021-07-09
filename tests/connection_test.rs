extern crate redis;

use proyecto_taller_1::{
    domain::{
        entities::{
            config::Config,
            key_value_item::{KeyAccessTime, ValueTimeItem, ValueType},
            server::Server,
        },
        implementations::database::Database,
    },
    services::{server_service, worker_service::ThreadPool},
};

use std::{
    error::Error,
    fmt,
    sync::{mpsc, Arc, Mutex, Barrier},
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
    let config_file = String::from("./src/dummy_config.txt");
    let db_file = String::from("./src/dummy_database.txt");
    let log_file = String::from("./src/dummy_log.txt");

    match std::fs::File::create(&config_file) {
        Ok(_) => {}
        Err(e) => println!("Error creating config {:?}", e),
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

    let added_item_1 = ValueTimeItem::new(
        ValueType::StringType(String::from("value_key_1")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_1"), added_item_1);

    let added_item_2 = ValueTimeItem::new(
        ValueType::StringType(String::from("value_key_2")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_2"), added_item_2);

    let added_item_3 = ValueTimeItem::new(
        ValueType::StringType(String::from("value_key_3")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_3"), added_item_3);

    let added_item_4 = ValueTimeItem::new(
        ValueType::StringType(String::from("value_key_4")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_4"), added_item_4);

    let added_item_5 = ValueTimeItem::new(
        ValueType::StringType(String::from("Hello")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("mykey"), added_item_5);

    let added_item_6 = ValueTimeItem::new(
        ValueType::StringType(String::from("10")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_to_decr"), added_item_6);

    let added_item_7 = ValueTimeItem::new(
        ValueType::StringType(String::from("10")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_to_incr"), added_item_7);

    let added_item_8 = ValueTimeItem::new(
        ValueType::StringType(String::from("Hello")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_getdel"), added_item_8);

    let added_item_9 = ValueTimeItem::new(
        ValueType::StringType(String::from("OldValue")),
        KeyAccessTime::Volatile(4234234),
    );
    database.add(String::from("key_getset"), added_item_9);

    let (server_sender, server_receiver) = mpsc::channel();
    let server_receiver = Arc::new(Mutex::new(server_receiver));
    let port = String::from("8080");
    let verbose = String::from("0");
    let port_2 = port.clone();
    let dir = String::from("127.0.0.1");

    let handle: thread::JoinHandle<()> = thread::spawn(|| {
        let h = thread::spawn(|| {
            let mut server = Server::new(port_2, log_file, verbose, server_receiver).unwrap();
            server.listen();
        });

        server_service::init(database, config, port, dir, server_sender);
        h.join().unwrap();
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

    // pool.spawn(shutdown);
    // println!("join test");
    // let _ = handle.join().expect("Couldnt join");
    std::fs::remove_file("./src/dummy_config.txt").unwrap();
    // std::fs::remove_file("./src/dummy_log.txt").unwrap();
    std::fs::remove_file("./src/dummy_database.txt").unwrap();
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
        name: "string command: incrby mykey 3",
        func: test_string_incrby,
    },
    Test {
        name: "string command: get key_1",
        func: test_string_get,
    },
    Test {
        name: "string command: getdel key_getdel",
        func: test_string_getdel,
    },
    Test {
        name: "string command: getset key_getset",
        func: test_string_getset,
    },
    Test {
        name: "string command: strlen key_1",
        func: test_string_strlen,
    },
    Test {
        name: "pubsub command: subscribe foo",
        func: test_pubsub,
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
            got: ret.to_string(),
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

fn test_string_incrby() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("INCRBY")
        .arg("key_to_incr")
        .arg(3)
        .query(&mut con)?;

    if ret == 13 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("13"),
            got: ret.to_string(),
        }));
    }
}

fn test_string_get() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("GET").arg("key_1").query(&mut con)?;

    if ret == String::from("value_key_1") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("value_key_1"),
            got: ret,
        }));
    }
}

fn test_string_strlen() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("STRLEN").arg("key_1").query(&mut con)?;

    if ret == 11 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("11"),
            got: ret.to_string(),
        }));
    }
}

fn test_string_getdel() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("GETDEL").arg("key_getdel").query(&mut con)?;

    if ret == String::from("Hello") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Hello"),
            got: ret,
        }));
    }
}

fn test_string_getset() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("GETSET")
        .arg("key_getset")
        .arg("NewValue")
        .query(&mut con)?;

    if ret == String::from("OldValue") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("OldValue"),
            got: ret,
        }));
    }
}

fn test_pubsub() -> TestResult {
    // Connection for subscriber api
    let mut pubsub_con = connect().unwrap();

    // Barrier is used to make test thread wait to publish
    // until after the pubsub thread has subscribed.
    let barrier = Arc::new(Barrier::new(4));
    let close_barrier = Arc::new(Barrier::new(4));
    let pubsub_barrier = barrier.clone();
    let close_pubsub = close_barrier.clone();
    let thread = thread::spawn(move || {
        let mut pubsub = pubsub_con.as_pubsub();
        pubsub.subscribe("foo").unwrap();

        let _ = pubsub_barrier.wait();

        let msg = pubsub.get_message().unwrap();
        assert_eq!(msg.get_channel(), Ok("foo".to_string()));
        assert_eq!(msg.get_payload(), Ok(42));
        let _ = close_pubsub.wait();
    });

    let mut pubsub_con_2 = connect().unwrap();
    let pubsub_barrier_2 = barrier.clone();
    let close_pubsub = close_barrier.clone();
    let thread_2 = thread::spawn(move || {
        let mut pubsub_2 = pubsub_con_2.as_pubsub();
        pubsub_2.subscribe("foo").unwrap();

        let _ = pubsub_barrier_2.wait();
        let _ = close_pubsub.wait();
    });

    let mut pubsub_con_3 = connect().unwrap();
    let pubsub_barrier_3 = barrier.clone();
    let close_pubsub = close_barrier.clone();
    let thread_3 = thread::spawn(move || {
        let mut pubsub_3 = pubsub_con_3.as_pubsub();
        pubsub_3.subscribe("helloworld").unwrap();

        let _ = pubsub_barrier_3.wait();
        let _ = close_pubsub.wait();
    });

    let _ = barrier.wait();
    let mut con = connect().unwrap();
    let receivers: usize = redis::cmd("PUBLISH").arg("foo").arg(42).query(&mut con).unwrap();
    let subs: Vec<String> = redis::cmd("PUBSUB").arg("NUMSUB").arg("foo").query(&mut con).unwrap();
    let channels: Vec<String> = redis::cmd("PUBSUB").arg("CHANNELS").query(&mut con).unwrap();
    let channels_pattern: Vec<String> = redis::cmd("PUBSUB").arg("CHANNELS").arg("*d").query(&mut con).unwrap();
    let _ = close_barrier.wait();

    let mut pass = true;
    if receivers != 2 {
        pass = false;
    }

    if subs != vec![String::from("foo"), String::from("2")] {
        pass = false;
    }

    if channels != vec![String::from("foo"), String::from("helloworld")] && channels != vec![String::from("helloworld"), String::from("foo")] {
        pass = false;
    }

    if channels_pattern != vec![String::from("helloworld")] {
        pass = false;
    }

    thread.join().expect("Something went wrong");
    thread_2.join().expect("Something went wrong");
    thread_3.join().expect("Something went wrong");
    if pass {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: format!("publish: {}, numsub: {:?}, channels: {:?}, channels pattern: {:?}", 2, vec![String::from("foo"), String::from("2")], vec![String::from("foo"), String::from("helloworld")], vec![String::from("helloworld")]),
            got: format!("publish: {}, numsub: {:?}, channels: {:?}, channels pattern: {:?}", receivers, subs, channels, channels_pattern),
        }));
    }
}

//test unsubscribe -> falta funcionalidad para estado tal que no pueda mandar ningun otro comando que los de pubsub
