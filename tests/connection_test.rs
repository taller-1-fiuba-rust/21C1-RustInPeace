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
use redis::Commands;

use std::{
    error::Error,
    fmt,
    sync::{mpsc, Arc, Mutex},
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
//use crate::src::domain::entities::key_value_item::{ValueTimeItem, ValueType};

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

    let added_item_1 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("value_key_1")),
        KeyAccessTime::Volatile(1925487534),
    );
    database.add(String::from("key_1"), added_item_1);

    let added_item_2 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("value_key_2")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("key_2"), added_item_2);

    let added_item_3 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("value_key_3")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("key_3"), added_item_3);

    let added_item_4 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("value_key_4")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("key_4"), added_item_4);

    let added_item_5 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("Hello")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("mykey"), added_item_5);

    let added_item_6 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("10")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("key_to_decr"), added_item_6);

    let added_item_7 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("10")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("key_to_incr"), added_item_7);

    let added_item_8 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("Hello")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("key_getdel"), added_item_8);

    let added_item_9 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("OldValue")),
        KeyAccessTime::Volatile(1635597186),
    );
    database.add(String::from("key_getset"), added_item_9);
    let added_item_10 = ValueTimeItem::new_now(
        ValueType::StringType("hola".to_string()),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("mget_1"), added_item_10);
    let added_item_11 = ValueTimeItem::new_now(
        ValueType::StringType("chau".to_string()),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("mget_2"), added_item_11);

    let added_item_12 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "15".to_string(),
            "18".to_string(),
            "12".to_string(),
            "54".to_string(),
            "22".to_string(),
            "45".to_string(),
        ]),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("edades_amigos"), added_item_12);

    let added_item_13 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("10")),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("edad_maria"), added_item_13);

    let added_item_14 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("11")),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("edad_clara"), added_item_14);

    let added_item_15 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("12")),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("edad_josefina"), added_item_15);

    let added_item_16 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("13")),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("edad_luz"), added_item_16);

    let added_item_17 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "clara".to_string(),
            "maria".to_string(),
            "luz".to_string(),
            "josefina".to_string(),
        ]),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("grupo_amigas"), added_item_17);

    // let added_item_18 = ValueTimeItem::new_now(
    //     ValueType::StringType(String::from("63")),
    //     KeyAccessTime::Volatile(1635595186),
    // );
    // database.add(String::from("edad_mariana"), added_item_18);

    let added_item_18 = ValueTimeItem::new_now(
        ValueType::StringType(String::from("55")),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("edad_mariana"), added_item_18);

    let added_item_list_1 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "pomelo".to_string(),
            "sandia".to_string(),
            "kiwi".to_string(),
            "mandarina".to_string(),
        ]),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("frutas"), added_item_list_1);

    let added_persistent = ValueTimeItem::new_now(
        ValueType::StringType("persistente".to_string()),
        KeyAccessTime::Persistent,
    );
    database.add(String::from("persistente"), added_persistent);

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
        // match &mut Server::new(String::from("8080"), log_file, String::from("0")) {
        //     Ok(server) => server_service::init(server, database, config),
        //     Err(e) => println!("Error on server: {:?}", e),
        // }
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

    if let Ok(err) = receiver.recv_timeout(Duration::from_secs(20)) {
        panic!("{}", err);
    }

    pool.spawn(shutdown);
    let _ = handle.join().expect("Couldnt join");
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
        name: "keys command: expire",
        func: test_keys_expire,
    },
    Test {
        name: "keys command: expireat",
        func: test_keys_expireat,
    },
    Test {
        name: "keys command: ttl",
        func: test_keys_ttl,
    },
    Test {
        name: "keys command: touch",
        func: test_keys_touch,
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
        name: "keys command: sort ascending",
        func: test_keys_sort_ascending,
    },
    Test {
        name: "keys command: sort descending",
        func: test_keys_sort_descending,
    },
    Test {
        name: "keys command: sort ascending first four elements",
        func: test_keys_sort_ascending_first_four_elements,
    },
    Test {
        name: "keys command: sort descending first four elements",
        func: test_keys_sort_descending_first_four_elements,
    },
    Test {
        name: "keys command: sort by external key ascending",
        func: test_sort_by_external_key_value_using_pattern_ascending,
    },
    Test {
        name: "keys command: sort by external key descending",
        func: test_sort_by_external_key_value_using_pattern_descending,
    },
    Test {
        name: "keys command: get value type list",
        func: test_gets_value_type_list,
    },
    Test {
        name: "keys command: get value type list",
        func: test_gets_value_type_string,
    },
    Test {
        name: "string command: get only string value else nil",
        func: test_se_obtienen_solo_las_claves_que_tienen_value_tipo_string_sino_nil,
    },
    Test {
        name: "string command: set multiple keys never fails",
        func: test_se_setean_multiples_claves_nunca_falla,
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
        name: "string command: mget key_1 mykey",
        func: test_string_mget,
    },
    Test {
        name: "list command: push values into key - list type",
        func: test_se_guardan_valores_en_una_lista_que_no_existe_previamente,
    },
    Test {
        name: "list command: push values into existing key - list type",
        func: test_se_guardan_valores_en_una_lista_ya_existente,
    },
    Test {
        name: "list command: cannot push values into existing non-list type key",
        func: test_no_se_guardan_valores_en_un_value_cuyo_tipo_no_es_una_lista,
    },
    Test {
        name: "list command: get lenght of existing list",
        func: test_se_obtiene_la_longitud_de_la_lista_en_value,
    },
    Test {
        name: "list command: get 0 as lenght of unexisting list",
        func: test_se_obtiene_cero_como_la_longitud_de_key_inexistente,
    },
    Test {
        name: "list command: cannot get len of non-list type key",
        func: test_no_se_obtiene_len_de_value_cuyo_tipo_no_es_una_lista,
    },
    // Test {
    //     name: "pubsub command: subscribe channel_1 channel_2 ",
    //     func: test_pubsub,
    // },
    Test {
        name: "list command: lindex",
        func: test_list_index,
    },
    Test {
        name: "set command: sadd",
        func: test_set_add,
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

fn test_keys_expire() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXPIRE").arg("key_1").arg(15).query(&mut con)?;

    return if ret == 1 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }))
    };
}

fn test_keys_expireat() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXPIREAT")
        .arg("key_1")
        .arg(1725487534)
        .query(&mut con)?;

    return if ret == 1 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }))
    };
}
fn test_keys_ttl() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("TTL").arg("key_1").query(&mut con)?;

    return if ret > 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("Positive number"),
            got: ret.to_string(),
        }))
    };
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
fn test_keys_sort_ascending() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("SORT").arg("edades_amigos").query(&mut con)?;

    if &ret[0] == &String::from("12")
        && &ret[1] == &String::from("15")
        && &ret[2] == &String::from("18")
        && &ret[3] == &String::from("22")
        && &ret[4] == &String::from("45")
        && &ret[5] == &String::from("54")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("12 15 18 22 45 54"),
            got: format!(
                "{} {} {} {} {} {}",
                ret[0], ret[1], ret[2], ret[3], ret[4], ret[5]
            ),
        }));
    }
}

fn test_keys_sort_descending() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("SORT")
        .arg("edades_amigos")
        .arg("DESC")
        .query(&mut con)?;

    if &ret[0] == &String::from("54")
        && &ret[1] == &String::from("45")
        && &ret[2] == &String::from("22")
        && &ret[3] == &String::from("18")
        && &ret[4] == &String::from("15")
        && &ret[5] == &String::from("12")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("54 45 22 18 15 12"),
            got: format!(
                "{} {} {} {} {} {}",
                ret[0], ret[1], ret[2], ret[3], ret[4], ret[5]
            ),
        }));
    }
}

fn test_keys_sort_ascending_first_four_elements() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("SORT")
        .arg("edades_amigos")
        .arg("LIMIT")
        .arg("0")
        .arg("4")
        .query(&mut con)?;
    if ret.len() == 4 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("4"),
            got: ret.len().to_string(),
        }));
    }
}

fn test_keys_sort_descending_first_four_elements() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("SORT")
        .arg("edades_amigos")
        .arg("LIMIT")
        .arg("0")
        .arg("4")
        .arg("DESC")
        .query(&mut con)?;
    if ret.len() == 4 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("4"),
            got: ret.len().to_string(),
        }));
    }
}

fn test_sort_by_external_key_value_using_pattern_ascending() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("SORT")
        .arg("grupo_amigas")
        .arg("BY")
        .arg("edad_")
        .query(&mut con)?;
    if &ret[0] == &String::from("maria")
        && &ret[1] == &String::from("clara")
        && &ret[2] == &String::from("josefina")
        && &ret[3] == &String::from("luz")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("maria clara josefina luz"),
            got: format!("{} {} {} {}", ret[0], ret[1], ret[2], ret[3]),
        }));
    }
}

fn test_sort_by_external_key_value_using_pattern_descending() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("SORT")
        .arg("grupo_amigas")
        .arg("BY")
        .arg("edad_")
        .arg("DESC")
        .query(&mut con)?;
    if &ret[0] == &String::from("luz")
        && &ret[1] == &String::from("josefina")
        && &ret[2] == &String::from("clara")
        && &ret[3] == &String::from("maria")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("luz josefina clara maria"),
            got: format!("{} {} {} {}", ret[0], ret[1], ret[2], ret[3]),
        }));
    }
}

fn test_gets_value_type_list() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("TYPE").arg("edades_amigos").query(&mut con)?;
    if ret == "list" {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("list"),
            got: ret.to_string(),
        }));
    }
}

fn test_gets_value_type_string() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("TYPE").arg("edad_maria").query(&mut con)?;
    if ret == "string" {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("string"),
            got: ret.to_string(),
        }));
    }
}

fn test_se_obtienen_solo_las_claves_que_tienen_value_tipo_string_sino_nil() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("MGET")
        .arg("edad_luz")
        .arg("edad_maria")
        .arg("edades_amigos")
        .arg("grupo_amigas")
        .query(&mut con)?;

    if &ret[0] == &String::from("13")
        && &ret[1] == &String::from("10")
        && &ret[2] == &String::from("(nil)")
        && &ret[3] == &String::from("(nil)")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("13 10 (nil) (nil)"),
            got: format!("{} {} {} {}", ret[0], ret[1], ret[2], ret[3]),
        }));
    }
}

fn test_se_setean_multiples_claves_nunca_falla() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("MSET")
        .arg("comandante_1")
        .arg("luciano")
        .arg("edad_mariana")
        .arg("34")
        .query(&mut con)?;

    if ret == "Ok" {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Ok"),
            got: ret.to_string(),
        }));
    }
}

fn test_se_guardan_valores_en_una_lista_que_no_existe_previamente() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LPUSH")
        .arg("bandada_de_caranchos")
        .arg("carancho_1")
        .arg("carancho_2")
        .arg("carancho_3")
        .arg("carancho_4")
        .arg("carancho_5")
        .query(&mut con)?;

    if ret == "5".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "5".to_string(),
            got: ret,
        }));
    }
}

fn test_no_se_guardan_valores_en_un_value_cuyo_tipo_no_es_una_lista() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LPUSH")
        .arg("edad_luz")
        .arg("jacinta")
        .arg("leonela")
        .arg("margarita")
        .arg("leonilda")
        .arg("murcia")
        .query(&mut con)?;
    if ret == "error - not list type".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "error - not list type".to_string(),
            got: ret,
        }));
    }
}

fn test_se_guardan_valores_en_una_lista_ya_existente() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LPUSH")
        .arg("grupo_amigas")
        .arg("jacinta")
        .arg("leonela")
        .arg("margarita")
        .arg("leonilda")
        .arg("murcia")
        .query(&mut con)?;
    if ret == "9".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "9".to_string(),
            got: ret,
        }));
    }
}

fn test_se_obtiene_la_longitud_de_la_lista_en_value() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LLEN").arg("edades_amigos").query(&mut con)?;
    if ret == "6".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "6".to_string(),
            got: ret,
        }));
    }
}

fn test_se_obtiene_cero_como_la_longitud_de_key_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LLEN")
        .arg("porotos_de_canasta")
        .query(&mut con)?;
    if ret == "0".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "0".to_string(),
            got: ret,
        }));
    }
}

fn test_no_se_obtiene_len_de_value_cuyo_tipo_no_es_una_lista() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LLEN").arg("edad_luz").query(&mut con)?;
    if ret == "error - not list type".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "error - not list type".to_string(),
            got: ret,
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

fn test_string_mget() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("MGET")
        .arg("mget_1")
        .arg("mget_2")
        .query(&mut con)?;
    println!("RES MGET: {:?}", ret);
    if &ret[0] == &String::from("hola") && &ret[1] == &String::from("chau") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("hola chau"),
            got: format!("{} {}", ret[0], ret[1]),
        }));
    }
}

fn _test_pubsub() -> TestResult {
    let h = thread::spawn(|| {
        let mut con = connect().unwrap();
        let mut pubsub = con.as_pubsub();
        pubsub.subscribe("channel_1").unwrap();

        let msg = pubsub.get_message().unwrap();
        let payload: String = msg.get_payload().unwrap();
        println!("CHANNEL '{}': {}", msg.get_channel_name(), payload);
    });

    thread::sleep(Duration::from_secs(1));
    let mut con = connect().unwrap();
    let receivers: usize = con.publish("channel_1", "Hello channel_1")?;
    println!("rec {}", receivers);

    thread::sleep(Duration::from_secs(10));
    let mut con = connect()?;
    let mut pubsub = con.as_pubsub();
    pubsub.unsubscribe("channel_1")?;

    h.join().unwrap();

    // if receivers == 1 {
    return Ok(());
    // } else {
    //     return Err(Box::new(ReturnError {
    //         expected: String::from("1"),
    //         got: receivers.to_string(),
    //     }));
    // }
}
pub fn test_list_index() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LINDEX")
        .arg("frutas")
        .arg("0")
        .query(&mut con)?;

    return if ret == String::from("pomelo") {
        let mut con = connect()?;
        let ret: String = redis::cmd("LINDEX")
            .arg("frutas")
            .arg("-1")
            .query(&mut con)?;

        return if ret == String::from("mandarina") {
            Ok(())
        } else {
            Err(Box::new(ReturnError {
                expected: String::from("mandarina"),
                got: ret,
            }))
        };
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("pomelo"),
            got: ret,
        }))
    };
}

pub fn test_keys_touch() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("TOUCH")
        .arg("frutas")
        .arg("persistente")
        .query(&mut con)?;

    return if ret == 2 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("2"),
            got: ret.to_string(),
        }))
    };

}
pub fn test_set_add() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SADD")
        .arg("frutas_2")
        .arg("persistente")
        .query(&mut con)?;

    return if ret == 1 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("2"),
            got: ret.to_string(),
        }))
    };
}