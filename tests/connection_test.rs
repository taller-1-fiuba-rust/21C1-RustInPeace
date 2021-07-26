extern crate redis;

use proyecto_taller_1::{
    domain::{
        entities::{config::Config, key_value_item::ValueType, server::Server},
        implementations::database::Database,
    },
    services::{server_service, worker_service::ThreadPool},
};
use redis::RedisError;

use proyecto_taller_1::domain::entities::key_value_item::ValueTimeItemBuilder;
use std::{
    collections::HashSet,
    error::Error,
    fmt,
    sync::{mpsc, Arc, Barrier, Mutex},
    thread::{self, sleep},
    time::Duration,
    usize,
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
    let config_path = config_file.clone();

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

    let added_item_1 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("value_key_1")))
            .with_timeout(1925487534)
            .build();

    database.add(String::from("key_1"), added_item_1);

    let added_item_2 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("value_key_2")))
            .with_timeout(1635597186)
            .build();

    database.add(String::from("key_2"), added_item_2);

    let added_item_3 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("value_key_3")))
            .with_timeout(1635597186)
            .build();

    database.add(String::from("key_3"), added_item_3);

    let added_item_4 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("value_key_4")))
            .with_timeout(1635597186)
            .build();

    database.add(String::from("key_4"), added_item_4);

    let added_item_5 = ValueTimeItemBuilder::new(ValueType::StringType(String::from("Hello")))
        .with_timeout(1635597186)
        .build();
    database.add(String::from("mykey"), added_item_5);

    let added_item_6 = ValueTimeItemBuilder::new(ValueType::StringType(String::from("10")))
        .with_timeout(1635597186)
        .build();
    database.add(String::from("key_to_decr"), added_item_6);

    let added_item_7 = ValueTimeItemBuilder::new(ValueType::StringType(String::from("10")))
        .with_timeout(1635597186)
        .build();

    database.add(String::from("key_to_incr"), added_item_7);

    let added_item_8 = ValueTimeItemBuilder::new(ValueType::StringType(String::from("Hello")))
        .with_timeout(1635597186)
        .build();

    database.add(String::from("key_getdel"), added_item_8);

    let added_item_9 = ValueTimeItemBuilder::new(ValueType::StringType(String::from("OldValue")))
        .with_timeout(1635597186)
        .build();
    database.add(String::from("key_getset"), added_item_9);
    let added_item_10 =
        ValueTimeItemBuilder::new(ValueType::StringType("hola".to_string())).build();
    database.add(String::from("mget_1"), added_item_10);
    let added_item_11 =
        ValueTimeItemBuilder::new(ValueType::StringType("chau".to_string())).build();
    database.add(String::from("mget_2"), added_item_11);

    let added_item_12 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "15".to_string(),
        "18".to_string(),
        "12".to_string(),
        "54".to_string(),
        "22".to_string(),
        "45".to_string(),
    ]))
    .build();

    database.add(String::from("edades_amigos"), added_item_12);

    let added_item_13 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("10"))).build();
    database.add(String::from("edad_maria"), added_item_13);

    let added_item_14 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("11"))).build();
    database.add(String::from("edad_clara"), added_item_14);

    let added_item_15 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("12"))).build();
    database.add(String::from("edad_josefina"), added_item_15);

    let added_item_16 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("13"))).build();

    database.add(String::from("edad_luz"), added_item_16);

    let added_item_17 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "clara".to_string(),
        "maria".to_string(),
        "luz".to_string(),
        "josefina".to_string(),
    ]))
    .build();

    database.add(String::from("grupo_amigas"), added_item_17);

    let added_item_18 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("55"))).build();
    database.add(String::from("edad_mariana"), added_item_18);

    let added_item_list_19 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "pomelo".to_string(),
        "sandia".to_string(),
        "kiwi".to_string(),
        "mandarina".to_string(),
    ]))
    .build();
    database.add(String::from("frutas"), added_item_list_19);

    let added_item_list_20 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "tamarindo".to_string(),
        "grosella".to_string(),
        "pomelo_negro".to_string(),
        "coco".to_string(),
    ]))
    .build();
    database.add(String::from("frutas_raras"), added_item_list_20);

    let added_item_list_21 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "jinete_1".to_string(),
        "jinete_2".to_string(),
        "jinete_3".to_string(),
        "jinete_4".to_string(),
        "jinete_5".to_string(),
        "jinete_6".to_string(),
        "jinete_7".to_string(),
        "jinete_8".to_string(),
    ]))
    .build();
    database.add(String::from("jinetes_de_tucuman"), added_item_list_21);

    let added_item_22 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "argentina".to_string(),
        "brasil".to_string(),
        "uruguay".to_string(),
        "chile".to_string(),
    ]))
    .build();
    database.add(String::from("paises"), added_item_22);

    let added_item_23 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "jujuy".to_string(),
        "mendoza".to_string(),
        "corrientes".to_string(),
        "misiones".to_string(),
    ]))
    .build();
    database.add(String::from("provincias"), added_item_23);

    let added_item_24 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "italia".to_string(),
        "francia".to_string(),
        "españa".to_string(),
        "portugal".to_string(),
    ]))
    .build();
    database.add(String::from("paises2"), added_item_24);

    let added_item_25 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "chubut".to_string(),
        "formosa".to_string(),
        "chaco".to_string(),
        "catamarca".to_string(),
    ]))
    .build();
    database.add(String::from("provincias2"), added_item_25);

    let mut set = HashSet::new();
    set.insert("value_1".to_string());
    set.insert("value_2".to_string());
    let added_item_set_1 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("set_values_1"), added_item_set_1);

    let mut set = HashSet::new();
    set.insert("value_1".to_string());
    set.insert("value_2".to_string());
    let added_item_set_2 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("set_values_2"), added_item_set_2);

    let mut set = HashSet::new();
    set.insert("value_1".to_string());
    set.insert("value_2".to_string());
    set.insert("value_3".to_string());
    let added_item_list_26 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("set_remove_1"), added_item_list_26);

    let mut set = HashSet::new();
    set.insert("value_1".to_string());
    set.insert("value_2".to_string());
    set.insert("value_3".to_string());
    let added_item_list_27 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("set_remove_2"), added_item_list_27);

    let mut set = HashSet::new();
    set.insert("value_2".to_string());
    set.insert("value_3".to_string());
    let added_item_list_28 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("set_remove_3"), added_item_list_28);

    let added_item_list_29 =
        ValueTimeItemBuilder::new(ValueType::ListType(vec!["item_1".to_string()])).build();
    database.add(String::from("set_remove_4"), added_item_list_29);

    let added_item_list_30 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();
    database.add(String::from("love_the_dog"), added_item_list_30);

    let added_item_list_31 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "cat".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();
    database.add(String::from("love_the_cat"), added_item_list_31);

    let added_item_list_32 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "bunny".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();
    database.add(String::from("love_the_bunny"), added_item_list_32);

    let added_persistent =
        ValueTimeItemBuilder::new(ValueType::StringType("persistente".to_string())).build();
    database.add(String::from("persistente"), added_persistent);

    let added_item_33 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "chocolate".to_string(),
        "frutilla".to_string(),
    ]))
    .build();
    database.add(String::from("sabores"), added_item_33);

    let added_item_34 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "baldur".to_string(),
        "odin".to_string(),
        "freya".to_string(),
        "mimir".to_string(),
    ]))
    .build();
    database.add(String::from("norse_gods"), added_item_34);

    let added_item_35 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hera".to_string(),
        "afrodita".to_string(),
        "chaos".to_string(),
        "artemis".to_string(),
    ]))
    .build();
    database.add(String::from("greek_gods"), added_item_35);

    let added_item_36 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "isis".to_string(),
        "osiris".to_string(),
        "horus".to_string(),
        "set".to_string(),
    ]))
    .build();
    database.add(String::from("egyptian_gods"), added_item_36);

    let added_item_37 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "pineapple_1".to_string(),
        "pineapple_2".to_string(),
        "pineapple_3".to_string(),
        "pineapple_4".to_string(),
    ]))
    .build();
    database.add(String::from("pineapple_mascots"), added_item_37);

    let added_item_38 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "banana_1".to_string(),
        "banana_2".to_string(),
        "banana_3".to_string(),
        "banana_4".to_string(),
    ]))
    .build();
    database.add(String::from("banana_mascots"), added_item_38);

    let added_item_39 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "borussia".to_string(),
        "werder".to_string(),
        "bayer".to_string(),
    ]))
    .build();
    database.add(String::from("equipos_de_la_bundesliga"), added_item_39);

    let added_item_40 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "chocolate a la leche condensada".to_string(),
        "chocolate werder".to_string(),
        "chocolate aires del Huapi".to_string(),
    ]))
    .build();
    database.add(String::from("sabores_de_chocolate"), added_item_40);

    let added_item_41 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "banana_passion_1".to_string(),
        "banana_passion_2".to_string(),
        "banana_passion_3".to_string(),
        "banana_passion_4".to_string(),
    ]))
    .build();
    database.add(String::from("banana_passions"), added_item_41);

    let added_item_42 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "carpintero_1".to_string(),
        "carpintero_2".to_string(),
        "carpintero_3".to_string(),
        "carpintero_4".to_string(),
    ]))
    .build();
    database.add(String::from("pajaros_carpinteros"), added_item_42);

    let added_item_43 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("jugo de kiwi"))).build();
    database.add(String::from("botellon_de_jugo_1"), added_item_43);

    let added_item_44 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("jugo de grosellas"))).build();
    database.add(String::from("botellon_de_jugo_2"), added_item_44);

    let added_item_45 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("jugo de tamarindo"))).build();
    database.add(String::from("botellon_de_jugo_3"), added_item_45);

    let added_item_46 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("jugo de remolacha"))).build();
    database.add(String::from("botellon_de_jugo_10"), added_item_46);

    let added_item_47 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("jugo de damasco"))).build();
    database.add(String::from("botellon_de_jugo_23"), added_item_47);

    let mut set = HashSet::new();
    set.insert("granadero_espigado_1".to_string());
    set.insert("granadero_espigado_2".to_string());
    let added_item_set_48 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("granaderos_espigados"), added_item_set_48);

    let mut set = HashSet::new();
    set.insert("granadero_atolondrado_1".to_string());
    set.insert("granadero_atolondrado_2".to_string());
    let added_item_set_49 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("granaderos_atolondrados"), added_item_set_49);

    let mut set = HashSet::new();
    set.insert("granadero_taciturno_1".to_string());
    set.insert("granadero_taciturno_2".to_string());
    let added_item_set_50 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(String::from("granaderos_taciturnos"), added_item_set_50);

    let mut set = HashSet::new();
    set.insert("granadero_eliminado_1".to_string());
    set.insert("granadero_eliminado_2".to_string());
    let added_item_set_51 = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    database.add(
        String::from("granaderos_eliminados_tipo_set"),
        added_item_set_51,
    );

    let added_item_52 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "granadero_eliminado_1".to_string(),
        "granadero_eliminado_2".to_string(),
        "granadero_eliminado_3".to_string(),
        "granadero_eliminado_4".to_string(),
    ]))
    .build();
    database.add(
        String::from("granaderos_eliminados_tipo_list"),
        added_item_52,
    );

    let added_item_53 =
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("value_key_999")))
            .with_timeout(1635597186)
            .build();

    database.add(String::from("key_999"), added_item_53);

    let added_item_54 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "molleja_frita_1".to_string(),
        "molleja_frita_2".to_string(),
        "molleja_frita_3".to_string(),
        "molleja_frita_4".to_string(),
    ]))
    .build();
    database.add(String::from("mollejas_fritas"), added_item_54);

    let added_item_55 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "molleja_gratinada_1".to_string(),
        "molleja_gratinada_2".to_string(),
        "molleja_gratinada_3".to_string(),
        "molleja_gratinada_4".to_string(),
    ]))
    .build();
    database.add(String::from("mollejas_gratinadas"), added_item_55);

    let added_item_56 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "molleja_salteada_1".to_string(),
        "molleja_salteada_2".to_string(),
        "molleja_salteada_3".to_string(),
        "molleja_salteada_4".to_string(),
    ]))
    .build();
    database.add(String::from("mollejas_salteadas"), added_item_56);

    let added_item_57 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "molleja_estofada_1".to_string(),
        "molleja_estofada_2".to_string(),
        "molleja_estofada_3".to_string(),
        "molleja_estofada_4".to_string(),
    ]))
    .build();
    database.add(String::from("mollejas_estofadas"), added_item_57);

    //--------------------------------------------------------------------------------------------------------------------------------------------
    //--------------------------------------------------------------------------------------------------------------------------------------------
    //--------------------------------------------------------------------------------------------------------------------------------------------
    //--------------------------------------------------------------------------------------------------------------------------------------------
    //--------------------------------------------------------------------------------------------------------------------------------------------

    let (server_sender, server_receiver) = mpsc::channel();
    let server_receiver = Arc::new(Mutex::new(server_receiver));
    let port = String::from("8080");
    let verbose = String::from("0");
    let port_2 = port.clone();
    let dir = String::from("127.0.0.1");

    let _handle: thread::JoinHandle<()> = thread::spawn(|| {
        let h = thread::spawn(|| {
            let mut server =
                Server::new(port_2, log_file, verbose, server_receiver, config_path).unwrap();
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

    if let Ok(err) = receiver.recv_timeout(Duration::from_secs(20)) {
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
    Test {
        name: "server command: config get *",
        func: test_config_get_all,
    },
    Test {
        name: "server command: config get",
        func: test_config_get_returns_error_missing_parameter,
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
        name: "keys command: del string type",
        func: test_keys_del_string_type,
    },
    Test {
        name: "keys command: del set type",
        func: test_keys_del_set_type,
    },
    Test {
        name: "keys command: del list type",
        func: test_keys_del_list_type,
    },
    Test {
        name: "keys command: del ignore when key does not exist",
        func: test_keys_del_ignora_la_operacion_cuando_la_clave_no_existe,
    },
    Test {
        name: "keys command: exists",
        func: test_keys_exists,
    },
    Test {
        name: "keys command: exists returns 2 whe same key double-checked",
        func: test_keys_exists_devuelve_2_cuando_se_chequea_doble_por_la_misma_clave,
    },
    Test {
        name: "keys command: exists returns 2 as 2 out of 4 keys exist",
        func: test_keys_exists_arroja_2_porque_2_de_las_4_claves_existen,
    },
    Test {
        name: "keys command: exists returns 0 as key does not exist",
        func: test_keys_exists_arroja_cero_porque_la_clave_no_existe,
    },
    Test {
        name: "keys command: persist",
        func: test_keys_persist,
    },
    Test {
        name: "keys command: persist returns 0 as key has no timeout associated",
        func: test_keys_persist_arroja_cero_porque_la_clave_no_tiene_asociada_un_timeout,
    },
    Test {
        name: "keys command: persist returns 0 as key does not exist",
        func: test_keys_persist_arroja_cero_porque_la_clave_no_existe,
    },
    Test {
        name: "keys command: expire",
        func: test_keys_expire,
    },
    Test {
        name: "keys command: expire not performing as key does not exist",
        func: test_keys_expire_no_aplica_expiracion_porque_la_clave_no_existe,
    },
    Test {
        name: "keys command: expireat",
        func: test_keys_expireat,
    },
    Test {
        name: "keys command: expireat not performing as key does not exist",
        func: test_keys_expireat_no_aplica_expiracion_porque_la_clave_no_existe,
    },
    Test {
        name: "keys command: ttl",
        func: test_keys_ttl,
    },
    Test {
        name: "keys command: ttl returns -1 as key has no timeout associated",
        func: test_keys_ttl_clave_no_tiene_asociado_un_timeout,
    },
    Test {
        name: "keys command: ttl -2 as key does no exist",
        func: test_keys_ttl_clave_no_existe,
    },
    Test {
        name: "keys command: touch 2 keys exist and 1 does not exist and it is ignored",
        func: test_keys_touch,
    },
    Test {
        name: "keys command: rename",
        func: test_keys_rename,
    },
    Test {
        name: "keys command: rename - key does not exist throws error",
        func: test_keys_rename_clave_no_existe_arroja_error,
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
        name: "keys command: copy with no replace cannot replace destination key - error",
        func: test_keys_copy_sin_replace_arroja_error_porque_la_clave_destino_ya_existe,
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
        func: test_type_gets_value_type_list,
    },
    Test {
        name: "keys command: get value type list",
        func: test_type_gets_value_type_string,
    },
    Test {
        name: "keys command: get value type set",
        func: test_type_gets_value_type_set,
    },
    Test {
        name: "keys command: get value returns nothing as key does not exist",
        func: test_type_gets_value_type_set_no_devuelve_nadacuando_se_aplica_type_para_clave_inexistente,
    },
    Test {
        name: "keys command: keys return keys that match a pattern",
        func: test_keys_gets_keys_that_match_a_pattern,
    },
    Test {
        name: "keys command: keys return keys that match a pattern with ???",
        func: test_keys_gets_keys_that_match_a_pattern_con_signo_de_pregunta,
    },
     Test {
        name: "string command: get only string value else nil",
        func: test_se_obtienen_solo_las_claves_que_tienen_value_tipo_string,
    },
    Test {
        name: "string command: set multiple keys never fails",
        func: test_se_setean_multiples_claves_nunca_falla,
    },
    Test {
        name: "string command: set new value in existing key that is not string type, it never fails",
        func: test_se_setea_clave_a_una_clave_existente_que_no_aloja_un_valor_de_tipo_string_y_nunca_falla,
    },
    Test {
        name: "string command: append mykey newvalue",
        func: test_string_append,
    },
    Test {
        name: "string command: append mykey newvalue into non-existing key",
        func: test_string_append_clave_que_no_existe_por_lo_que_se_crea_y_se_almacena_el_valor,
    },
    Test {
        name: "string command: append mykey newvalue cannot as value type is not string, returns zero",
        func: test_string_append_into_not_string_type_returns_zero,
    },
    Test {
        name: "string command: decrby mykey 3",
        func: test_string_decrby,
    },
    Test {
        name: "string command: decrby mykey 3 to key that does not exists, -3 is result",
        func: test_string_decrby_en_clave_que_no_existe_crea_la_clave_y_la_decrementa_en_el_valor_pasado,
    },
    Test {
        name: "string command: decrby mykey returns error as type is not string",
        func: test_string_decrby_devuelve_error_si_el_tipo_de_dato_no_es_string,
    },
    Test {
        name: "string command: decrby mykey returns error as string type cannot be represented as integer",
        func: test_string_decrby_devuelve_error_porque_el_string_no_se_puede_representar_como_integer,
    },
    Test {
        name: "string command: incrby mykey 3",
        func: test_string_incrby,
    },
    Test {
        name: "string command: incrby mykey 3 to key that does not exists, 3 is result",
        func: test_string_incrby_en_clave_que_no_existe_crea_la_clave_y_la_incrementa_en_el_valor_pasado,
    },
    Test {
        name: "string command: incrby mykey returns error as type is not string",
        func: test_string_incrby_devuelve_error_si_el_tipo_de_dato_no_es_string,
    },
    Test {
        name: "string command: incrby mykey returns error as string type cannot be represented as integer",
        func: test_string_incrby_devuelve_error_porque_el_string_no_se_puede_representar_como_integer,
    },
    Test {
        name: "string command: get key_1",
        func: test_string_get,
    },
    Test {
        name: "string command: get cannot get non-existing key-value, nill is returned",
        func: test_string_get_devuelve_nulo_cuando_se_aplica_get_para_clave_inexistente,
    },
    Test {
        name: "string command: get cannot get non-string key-value, error is returned",
        func: test_string_get_devuelve_error_cuando_se_aplica_get_para_valor_que_no_es_string,
    },
    Test {
        name: "string command: getdel key_getdel",
        func: test_string_getdel,
    },
    Test {
        name: "string command: getdel returns nill as key does not exists",
        func: test_string_getdel_devuelve_nulo_cuando_se_aplica_getdel_para_clave_inexistente,
    },
    Test {
        name: "string command: getdel thorws error as key holds value which is not string",
        func: test_string_getdel_devuelve_error_cuando_se_aplica_getdel_para_valor_que_no_es_string,
    },
    Test {
        name: "string command: getset key_getset",
        func: test_string_getset,
    },
    Test {
        name: "string command: getset returns nill as key does not exists",
        func: test_string_getset_devuelve_nulo_cuando_se_aplica_getset_para_clave_inexistente,
    },
    Test {
        name: "string command: getset thorws error as key holds value which is not string",
        func: test_string_getset_devuelve_error_cuando_se_aplica_getset_para_valor_que_no_es_string,
    },
    Test {
        name: "string command: strlen key_1",
        func: test_string_strlen,
    },
    Test {
        name: "string command: strlen is zero when key does not exist",
        func: test_string_strlen_devuelve_nulo_cuando_se_aplica_get_para_clave_inexistente,
    },
    Test {
        name: "string command: get cannot get non-string key-value, error is returned",
        func: test_string_get_arroja_error_cuando_se_aplica_get_para_valor_que_no_es_string,
    },
    Test {
        name: "string command: mget key_1 mykey",
        func: test_string_mget,
    },
    Test {
        name: "string command: set mykeyset setvalue",
        func: test_string_set,
    },
    Test {
        name: "string command: set returns null when key does not exist",
        func: test_string_set_devuelve_nulo_cuando_se_aplica_set_para_clave_inexistente,
    },
    Test {
        name: "string command: set mykeyset setvalue with ex argument",
        func: test_string_set_with_ex_argument,
    },
    Test {
        name: "string command: set mykeyset setvalue with keepttl argument",
        func: test_string_set_with_keepttl_argument,
    },
    Test {
        name: "string command: set mykeyset setvalue with nx argument sets succesfully as key does not already exist",
        func: test_string_set_with_nx_argument_key_does_not_already_exist,
    },
    Test {
        name: "string command: set mykeyset setvalue with nx argument throws error as key already exists",
        func: test_string_set_with_nx_argument_key_already_exists_throws_error,
    },
    Test {
        name: "string command: set mykeyset setvalue with xx argument throws error as key does not already exist",
        func: test_string_set_with_xx_argument_key_does_not_already_exist_then_throws_error,
    },
    Test {
        name: "string command: set mykeyset setvalue with xx argument successful as key already exists",
        func: test_string_set_with_xx_argument_succesfull_as_key_already_exists,
    },
    Test {
        name: "list command: lpush values into key - list type",
        func: test_lpush_se_guardan_valores_en_una_lista_que_no_existe_previamente,
    },
    Test {
        name: "list command: lpush values into existing key - list type",
        func: test_lpush_se_guardan_valores_en_una_lista_ya_existente,
    },
    Test {
        name: "list command: cannot lpush values into existing non-list type key",
        func: test_lpush_no_se_guardan_valores_en_un_value_cuyo_tipo_no_es_una_lista,
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
    Test {
        name: "list command: lpushx values into key - list type",
        func: test_se_lpushx_valores_en_una_lista_ya_existente,
    },
    Test {
        name: "list command: cannot lpushx values into non_existing key",
        func: test_no_se_lpushx_valores_en_una_lista_no_existente,
    },
    Test {
        name: "list command: cannot lpushx values into existing non-list type key",
        func: test_lpushx_no_se_guardan_valores_en_un_value_cuyo_tipo_no_es_una_lista,
    },
    Test {
        name: "list command: lrange return value especified by lower and upper bounds",
        func: test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_en_rango,
    },
    Test {
        name: "list command: lrange return value especified by lower and upper bounds with ub>len of the list",
        func: test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_mayor_a_long_de_la_lista,
    },
    Test {
        name: "list command: lrange return value especified by lower and upper bounds with lb<first_element_position of the list",
        func: test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_menor_a_la_1ra_pos_de_la_lista,
    },
    Test {
        name: "list command: lrange return value especified by lower and upper bounds with lb<first_element_position of the list and ub>len",
        func: test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_menor_a_la_1ra_pos_de_la_lista_con_upper_bound_mayor_a_len,
    },
    Test {
        name: "list command: lrange return empty list as lb>ub",
        func: test_se_devuelve_lista_vacia_porque_limite_inferior_supera_al_limite_superior,
    },
    Test {
        name: "list command: lrem remove only 3 repeated values from left to right",
        func: test_se_eliminan_3_valores_repetidos_de_izquierda_a_derecha_de_un_value_de_tipo_list,
    },
    Test {
        name: "list command: lrem remove only 3 repeated values from left to right backwards",
        func: test_se_eliminan_3_valores_repetidos_de_izquierda_a_derecha_de_un_value_de_tipo_list_reverso,
    },
    Test {
        name: "list command: lrem remove all elements",
        func: test_se_eliminan_todos_los_valores_repetidos_un_value_de_tipo_list,
    },
    Test {
        name: "list command: lrem returns 0 as key does not exists",
        func: test_se_devuelve_cero_si_se_busca_remover_un_valor_cuya_clave_no_existe,
    },
    Test {
        name: "list command: lindex",
        func: test_list_index,
    },
    Test {
        name: "list command: lindex returns error when value not list type",
        func: test_list_index_no_list_type_error,
    },
    Test {
        name: "list command: lindex returns empty as index is outbounded",
        func: test_list_index_devuelve_vacio_porque_esta_outbounded,
    },
    Test {
        name: "list command: lindex returns value with negative index in bounds",
        func: test_list_index_devuelve_elemento_index_valido_pero_negativo,
    },
    Test {
        name: "list command: lpop mylist",
        func: test_list_lpop,
    },
    Test {
        name: "list command: lpop mylist 2",
        func: test_list_lpop_with_count,
    },
    Test {
        name: "list command: lpop (no count arg) return nill when key not found",
        func: test_list_lpop_sin_count_devuelve_nil_cuando_la_clave_no_existe,
    },
    Test {
        name: "list command: lpop (with count arg) return nill when key not found",
        func: test_list_lpop_con_count_devuelve_nil_cuando_la_clave_no_existe,
    },
    Test {
        name: "list command: lpop (no count arg) return nill when value type is not list",
        func: test_list_lpop_sin_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list,
    },
    Test {
        name: "list command: lpop (with count arg) return nill when value type is not list",
        func: test_list_lpop_con_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list,
    },
    Test {
        name: "list command: lpop (with count arg) return less elements than count as count is greater than list size",
        func: test_list_lpop_con_count_devuelve_menos_elementos_que_los_que_indica_count_porque_count_es_mayor_que_list_len,
    },
    Test {
        name: "list command: rpop mylist",
        func: test_list_rpop,
    },
    Test {
        name: "list command: rpop mylist 2",
        func: test_list_rpop_with_count,
    },
    Test {
        name: "list command: rpop asks 4 elements with count but returns only 3 as list_len is less that count",
        func: test_list_rpop_with_count_greater_than_list_lenght,
    },
    Test {
        name: "list command: rpop returns null as value is not list type (no count arg)",
        func: test_list_rpop_sin_count_devuelve_nil_cuando_la_clave_no_existe,
    },

    Test {
        name: "list command: rpop returns null as value is not list type (count arg)",
        func: test_list_rpop_con_count_devuelve_nil_cuando_la_clave_no_existe,
    },
    Test {
        name: "list command: rpop (no count arg) return nill when value type is not list",
        func: test_list_rpop_sin_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list,
    },
    Test {
        name: "list command: rpop (with count arg) return nill when value type is not list",
        func: test_list_rpop_con_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list,
    },
    Test {
        name: "list command: rpushx sabores vainilla coco",
        func: test_list_rpushx,
    },
    Test {
        name: "list command: rpushx paiseslimitrofes chile",
        func: test_list_rpushx_nonexisting_key_returns_zero,
    },
    Test {
        name: "list command: rpushx cannot store value in non value list type - error",
        func: test_list_rpushx_arrroja_error_cuando_se_intenta_almacenar_dato_en_una_clave_que_no_guarda_un_valor_de_tipo_list,
    },
    Test {
        name: "list command: rpush values into existing key with list type value",
        func: test_list_rpush,
    },
    Test {
        name: "list command: rpush values in non existing key - it is created",
        func: test_list_rpush_nonexisting_key_creates_key_value_pair_and_returns_list_size,
    },
    Test {
        name: "list command: rpush cannot store value in non value list type - error",
        func: test_list_rpush_arrroja_error_cuando_se_intenta_almacenar_dato_en_una_clave_que_no_guarda_un_valor_de_tipo_list,
    },
    Test {
        name: "list command: lset new element in list type value",
        func: test_list_reemplaza_un_elemento_de_value_list_type_exitosamente,
    },
    Test {
        name: "list command: lset new element in list type value with negative index inbound",
        func: test_list_reemplaza_un_elemento_de_value_list_type_exitosamente_empleando_indice_negativo_valido,
    },
    Test {
        name: "list command: lset cannot set new element in list type value out of bounds error",
        func: test_list_no_reemplaza_un_elemento_de_value_list_type_con_indice_fuera_de_rango_error,
    },
    Test {
        name: "set command: sadd",
        func: test_set_add,
    },
    Test {
        name: "set command: sadd to non-existing key , it is created and the value added",
        func: test_set_add_clave_no_existe_se_crea_la_clave_con_valor,
    },
    Test {
        name: "set command: sadd value is ignored as specified value is already a member of value",
        func: test_set_add_valor_ya_es_miembro,
    },
    Test {
        name: "set command: sadd cannot perform as key holds no set-value type , it is string type",
        func: test_set_add_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set_es_string_type,
    },
    Test {
        name: "set command: sadd cannot perform as key holds no set-value type , it is list type",
        func: test_set_add_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set_es_list_type,
    },
    Test {
        name: "set command: scard",
        func: test_set_scard,
    },
    Test {
        name: "set command: scard returns 0 as key does not exists",
        func: test_set_scard_devuelve_cero_para_clave_inexistente,
    },
    Test {
        name: "set command: scard cannot perform as key holds no-set value type - error", 
        func: test_set_add_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set,
    },
    Test {
        name: "set command: ismember",
        func: test_set_ismember,
    },
    Test {
        name: "set command: ismember cannot perform as key holds no-set value type - error",
        func: test_set_ismember_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set,
    },
    Test {
        name: "set command: ismember returns 0 as value is not member",
        func: test_set_ismember_devuelve_cero_porque_el_valor_no_es_miembro,
    },
    Test {
        name: "set command: ismember returns 0 as key-value does not exist",
        func: test_set_ismember_devuelve_cero_porque_la_clave_no_existe,
    },
    Test {
        name: "set command: smembers",
        func: test_set_smembers,
    },
    Test {
        name: "set command: smembers cannot perform as key holds no-set value type - error",
        func: test_set_members_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set,
    },
    Test {
        name: "set command: smembers return nill when key not found",
        func: test_set_smembers_devuelve_nil_cuando_la_clave_no_existe,
    },
    Test {
        name: "set command: srem",
        func: test_set_srem,
    },
    Test {
        name: "set command: srem multiple",
        func: test_set_srem_removes_multiple_values,
    },
    Test {
        name: "set command: srem is not set type",
        func: test_set_srem_removes_zero_values,
    },
    Test {
        name: "set command: srem error",
        func: test_set_srem_removes_returns_error,
    },
    Test {
        name: "pubsub commands: subscribe pubsub channels numsub",
        func: test_pubsub,
    },
    Test {
        name: "rpush command: new list",
        func: test_rpush_lista_inexistente
    },
    Test {
        name: "info",
        func: test_info
    }
];

fn connect() -> Result<redis::Connection, Box<dyn Error>> {
    let client = redis::Client::open(ADDR)?;
    let con = client.get_connection()?;
    Ok(con)
}

fn _shutdown() {
    let mut con = connect().unwrap();
    let _: redis::RedisResult<()> = redis::cmd("SHUTDOWN").query(&mut con);
}

//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-----------------------------------------------------SERVER COMMANDS-----------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------

pub fn test_info() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("INFO").query(&mut con);
    return if ret.is_ok() {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from(""),
            got: ret.err().unwrap().to_string(),
        }))
    };
}

fn test_config_get_verbose() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("CONFIG")
        .arg("get")
        .arg("verbose")
        .query(&mut con)?;

    if &ret[1] == &String::from("1") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: String::from(&ret[1]),
        }));
    }
}

fn test_config_get_all() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("CONFIG").arg("get").arg("*").query(&mut con)?;

    if ret[0].len() > 0 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Must contain verbose"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_config_get_returns_error_missing_parameter() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("CONFIG").arg("get").query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error missing parameter"),
            got: format!("{:?}", ret),
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

    if ret == String::from("Ok") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Ok"),
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

//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-----------------------------------------------------KEYS COMMANDS-------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------

fn test_keys_del_string_type() -> TestResult {
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

fn test_keys_del_set_type() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("DEL")
        .arg("granaderos_eliminados_tipo_set")
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

fn test_keys_del_list_type() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("DEL")
        .arg("granaderos_eliminados_tipo_list")
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

fn test_keys_del_ignora_la_operacion_cuando_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("DEL")
        .arg("esta_clave_es_demasiado_larga_para_ser_una_clave_real")
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

fn test_keys_exists_arroja_cero_porque_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXISTS")
        .arg("esta_clave_es_demasiado_larga_para_ser_una_clave_real")
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

fn test_keys_exists_arroja_2_porque_2_de_las_4_claves_existen() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXISTS")
        .arg("key_1")
        .arg("granaderos_enajenados")
        .arg("granaderos_espigados")
        .arg("granaderos_ausentes")
        .query(&mut con)?;

    if ret == 2 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("2"),
            got: ret.to_string(),
        }));
    }
}

fn test_keys_exists_devuelve_2_cuando_se_chequea_doble_por_la_misma_clave() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXISTS")
        .arg("key_1")
        .arg("key_1")
        .query(&mut con)?;

    if ret == 2 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("2"),
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

fn test_keys_persist_arroja_cero_porque_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("PERSIST")
        .arg("granaderos_enajenados")
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

fn test_keys_persist_arroja_cero_porque_la_clave_no_tiene_asociada_un_timeout() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("PERSIST")
        .arg("granaderos_espigados")
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

fn test_keys_expire_no_aplica_expiracion_porque_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXPIRE")
        .arg("esta_clave_es_demasiado_larga_para_ser_una_clave_real")
        .arg(15)
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("0"),
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

fn test_keys_expireat_no_aplica_expiracion_porque_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("EXPIREAT")
        .arg("esta_clave_es_demasiado_larga_para_ser_una_clave_real")
        .arg(1725487534)
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("0"),
            got: ret.to_string(),
        }))
    };
}

fn test_keys_ttl() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("TTL").arg("key_2").query(&mut con)?;

    return if ret > 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("Positive number"),
            got: ret.to_string(),
        }))
    };
}

fn test_keys_ttl_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: i64 = redis::cmd("TTL")
        .arg("esta_clave_es_demasiado_larga_para_ser_una_clave_real")
        .query(&mut con)?;

    return if ret == -2 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("-2"),
            got: ret.to_string(),
        }))
    };
}

fn test_keys_ttl_clave_no_tiene_asociado_un_timeout() -> TestResult {
    let mut con = connect()?;
    let ret: i64 = redis::cmd("TTL").arg("edad_maria").query(&mut con)?;

    return if ret == -1 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("-1"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_keys_touch() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("TOUCH")
        .arg("frutas")
        .arg("persistente")
        .arg("esta_clave_es_demasiado_larga_para_ser_una_clave_real")
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

fn test_keys_rename_clave_no_existe_arroja_error() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("RENAME")
        .arg("esta_clave_es_demasiado_larga_para_ser_una_clave_real")
        .arg("esta_clave_es_aun_muchisimo_demasiado_larga_para_ser_una_clave_real")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - key does not exist"),
            got: format!("{:?}", ret),
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

fn test_keys_copy_sin_replace_arroja_error_porque_la_clave_destino_ya_existe() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("COPY")
        .arg("key_1")
        .arg("key_999")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - key does not exist"),
            got: format!("{:?}", ret),
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
        .arg("edad_*")
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
        .arg("edad_*")
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

fn test_type_gets_value_type_list() -> TestResult {
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

fn test_type_gets_value_type_string() -> TestResult {
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

fn test_type_gets_value_type_set() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("TYPE")
        .arg("granaderos_espigados")
        .query(&mut con)?;
    if ret == "set" {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("set"),
            got: ret.to_string(),
        }));
    }
}

fn test_type_gets_value_type_set_no_devuelve_nadacuando_se_aplica_type_para_clave_inexistente(
) -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("TYPE")
        .arg("granaderos_amalgamados")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

fn test_keys_gets_keys_that_match_a_pattern() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("KEYS").arg("mollejas*").query(&mut con)?;

    return if ret.contains(&String::from("mollejas_estofadas"))
        && ret.contains(&String::from("mollejas_gratinadas"))
        && ret.contains(&String::from("mollejas_fritas"))
        && ret.contains(&String::from("mollejas_salteadas"))
    {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(
                "{:?}",
                vec![
                    String::from("mollejas_estofadas"),
                    String::from("mollejas_gratinadas"),
                    String::from("mollejas_fritas"),
                    String::from("mollejas_salteadas")
                ]
            ),
            got: format!("{:?}", ret),
        }))
    };
}
fn test_keys_gets_keys_that_match_a_pattern_con_signo_de_pregunta() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("KEYS").arg("love_the_???").query(&mut con)?;

    return if ret.contains(&String::from("love_the_dog"))
        && ret.contains(&String::from("love_the_cat"))
        && ret.contains(&String::from("love_the_bunny"))
    {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(
                "{:?}",
                vec![
                    String::from("love_the_dog"),
                    String::from("love_the_cat"),
                    String::from("love_the_bunny")
                ]
            ),
            got: format!("{:?}", ret),
        }))
    };
}

//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-----------------------------------------------------STRING COMMANDS-----------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------

fn test_se_obtienen_solo_las_claves_que_tienen_value_tipo_string() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("MGET")
        .arg("edad_luz")
        .arg("edad_maria")
        .arg("edades_amigos")
        .arg("grupo_amigas")
        .query(&mut con)?;

    if &ret[0] == &String::from("13") && &ret[1] == &String::from("10") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("13 10"),
            got: format!("{} {}", ret[0], ret[1]),
        }));
    }
}

fn test_string_mget() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("MGET")
        .arg("mget_1")
        .arg("mget_2")
        .query(&mut con)?;

    if &ret[0] == &String::from("hola") && &ret[1] == &String::from("chau") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("hola chau"),
            got: format!("{} {}", ret[0], ret[1]),
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

fn test_se_setea_clave_a_una_clave_existente_que_no_aloja_un_valor_de_tipo_string_y_nunca_falla(
) -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("MSET")
        .arg("banana_mascots")
        .arg("mister_banana")
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

fn test_string_append_clave_que_no_existe_por_lo_que_se_crea_y_se_almacena_el_valor() -> TestResult
{
    let mut con = connect()?;
    let ret: usize = redis::cmd("APPEND")
        .arg("nombre_del_nieto_del_sobrino_del_primo_del_hermano_del_abuelo")
        .arg("luciano")
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

fn test_string_append_into_not_string_type_returns_zero() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("APPEND")
        .arg("banana_passions")
        .arg(" World")
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

fn test_string_decrby_en_clave_que_no_existe_crea_la_clave_y_la_decrementa_en_el_valor_pasado(
) -> TestResult {
    let mut con = connect()?;
    let ret: i64 = redis::cmd("DECRBY")
        .arg("key_to_decr_that_doesnt_exists")
        .arg(3)
        .query(&mut con)?;

    if ret == -3 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("-3"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_decrby_devuelve_error_si_el_tipo_de_dato_no_es_string() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("DECRBY")
        .arg("jinetes_de_tucuman")
        .arg(3)
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not string type"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_decrby_devuelve_error_porque_el_string_no_se_puede_representar_como_integer(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("DECRBY").arg("key_1").arg(3).query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - string cannot be represented as integer"),
            got: format!("{:?}", ret),
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

fn test_string_incrby_en_clave_que_no_existe_crea_la_clave_y_la_incrementa_en_el_valor_pasado(
) -> TestResult {
    let mut con = connect()?;
    let ret: i64 = redis::cmd("INCRBY")
        .arg("key_to_incr_that_doesnt_exists")
        .arg(3)
        .query(&mut con)?;

    if ret == 3 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("3"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_incrby_devuelve_error_si_el_tipo_de_dato_no_es_string() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("INCRBY")
        .arg("jinetes_de_tucuman")
        .arg(3)
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not string type"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_incrby_devuelve_error_porque_el_string_no_se_puede_representar_como_integer(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("INCRBY").arg("key_1").arg(3).query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - string cannot be represented as integer"),
            got: format!("{:?}", ret),
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

fn test_string_get_devuelve_error_cuando_se_aplica_get_para_valor_que_no_es_string() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("GET")
        .arg("jinetes_de_tucuman")
        .arg(3)
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not string type"),
            got: format!("{:?}", ret),
        }));
    }
}
fn test_string_get_devuelve_nulo_cuando_se_aplica_get_para_clave_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("GET")
        .arg("ricardito_corazon_de_surubi")
        .arg(3)
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
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

fn test_string_strlen_devuelve_nulo_cuando_se_aplica_get_para_clave_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("STRLEN")
        .arg("ricardito_corazon_de_surubi")
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

fn test_string_get_arroja_error_cuando_se_aplica_get_para_valor_que_no_es_string() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("STRLEN")
        .arg("jinetes_de_tucuman")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not list type"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_getdel() -> TestResult {
    let mut con = connect()?;
    let ret_initial: usize = redis::cmd("DBSIZE").query(&mut con)?;
    let ret: String = redis::cmd("GETDEL").arg("key_getdel").query(&mut con)?;
    let ret_final: usize = redis::cmd("DBSIZE").query(&mut con)?;

    if ret == String::from("Hello") && ret_initial == (ret_final + 1) {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("'Hello' and db_size_after_getdel == db_size_before_getdel - 1"),
            got: format!(
                "value: {:?} , db_size_after_get_del {:?} , db_size_before_get_del {:?} ",
                ret, ret_final, ret_initial
            ),
        }));
    }
}

fn test_string_getdel_devuelve_nulo_cuando_se_aplica_getdel_para_clave_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("GETDEL")
        .arg("alfredito_corazon_de_cachalote")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

fn test_string_getdel_devuelve_error_cuando_se_aplica_getdel_para_valor_que_no_es_string(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("GETDEL")
        .arg("jinetes_de_tucuman")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not string type"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_getset() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("GETSET")
        .arg("key_getset")
        .arg("NewValue")
        .query(&mut con)?;

    let ret_stored_new_value: String = redis::cmd("GET").arg("key_getset").query(&mut con)?;
    if (ret == String::from("OldValue")) && (ret_stored_new_value == String::from("newvalue")) {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: format!("old value: OldValue , new value: NewValue"), //String::from("OldValue"),
            got: format!(
                "old value: {:?} , new value: {:?}",
                ret, ret_stored_new_value
            ),
        }));
    }
}

fn test_string_getset_devuelve_error_cuando_se_aplica_getset_para_valor_que_no_es_string(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("GETSET")
        .arg("jinetes_de_tucuman")
        .arg("NewValue")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not string type"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_getset_devuelve_nulo_cuando_se_aplica_getset_para_clave_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("GETSET")
        .arg("alfredito_corazon_de_cachalote")
        .arg("NewValue")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

fn test_string_set() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("SET")
        .arg("mykeyset")
        .arg("valueset")
        .query(&mut con)?;

    if ret == String::from("Ok") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Ok"),
            got: ret,
        }));
    }
}

fn test_string_set_with_ex_argument() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("SET")
        .arg("botellon_de_jugo_1")
        .arg("jugo de frambuesas")
        .arg("EX")
        .arg("60")
        .query(&mut con)?;

    if ret == String::from("Ok") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Ok"),
            got: ret,
        }));
    }
}

fn test_string_set_with_keepttl_argument() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("SET")
        .arg("botellon_de_jugo_2")
        .arg("jugo de frambuesas")
        .arg("KEEPTTL")
        .query(&mut con)?;

    if ret == String::from("Ok") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Ok"),
            got: ret,
        }));
    }
}

fn test_string_set_with_nx_argument_key_does_not_already_exist() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("SET")
        .arg("botellon_de_jugo_4")
        .arg("jugo de limon y lima")
        .arg("NX")
        .query(&mut con)?;

    if ret == String::from("Ok") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Ok"),
            got: ret,
        }));
    }
}

fn test_string_set_with_nx_argument_key_already_exists_throws_error() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("SET")
        .arg("botellon_de_jugo_23")
        .arg("jugo de zapallo")
        .arg("NX")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - key already exists"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_set_with_xx_argument_succesfull_as_key_already_exists() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("SET")
        .arg("botellon_de_jugo_10")
        .arg("jugo de palta")
        .arg("XX")
        .query(&mut con)?;

    if ret == String::from("Ok") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Ok"),
            got: ret,
        }));
    }
}

fn test_string_set_with_xx_argument_key_does_not_already_exist_then_throws_error() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("SET")
        .arg("botellon_de_jugo_87")
        .arg("jugo de mandioca")
        .arg("XX")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - key does not exist"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_string_set_devuelve_nulo_cuando_se_aplica_set_para_clave_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("SET")
        .arg("alfredito_corazon_de_cachalote")
        .arg("valueset")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-----------------------------------------------------LIST COMMANDS-------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------

fn test_lpush_se_guardan_valores_en_una_lista_que_no_existe_previamente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LPUSH")
        .arg("bandada_de_caranchos")
        .arg("carancho_1")
        .arg("carancho_2")
        .arg("carancho_3")
        .arg("carancho_4")
        .arg("carancho_5")
        .query(&mut con)?;

    if ret == 5 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "5".to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_lpush_no_se_guardan_valores_en_un_value_cuyo_tipo_no_es_una_lista() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("LPUSH")
        .arg("edad_luz")
        .arg("jacinta")
        .arg("leonela")
        .arg("margarita")
        .arg("leonilda")
        .arg("murcia")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not list type"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_lpush_se_guardan_valores_en_una_lista_ya_existente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LPUSH")
        .arg("grupo_amigas")
        .arg("jacinta")
        .arg("leonela")
        .arg("margarita")
        .arg("leonilda")
        .arg("murcia")
        .query(&mut con)?;

    if ret == 9 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "9".to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_se_obtiene_la_longitud_de_la_lista_en_value() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LLEN").arg("edades_amigos").query(&mut con)?;

    if ret == 6 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "6".to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_se_obtiene_cero_como_la_longitud_de_key_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LLEN")
        .arg("porotos_de_canasta")
        .query(&mut con)?;

    if ret == 0 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "0".to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_no_se_obtiene_len_de_value_cuyo_tipo_no_es_una_lista() -> TestResult {
    let mut con = connect()?;
    let ret: Result<usize, RedisError> = redis::cmd("LLEN").arg("edad_luz").query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "Not list type".to_string(),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_se_lpushx_valores_en_una_lista_ya_existente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LPUSHX")
        .arg("frutas_raras")
        .arg("granada")
        .arg("mango")
        .arg("morango")
        .arg("anana")
        .arg("kinoto")
        .query(&mut con)?;

    if ret == 9 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: 9.to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_no_se_lpushx_valores_en_una_lista_no_existente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LPUSHX")
        .arg("gorilas_y_mandriles")
        .arg("gorila_gutierrez")
        .arg("gorila_sosa")
        .arg("mandril_gonzalez")
        .arg("mandril_galvan")
        .query(&mut con)?;

    if ret == 0 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: 0.to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_lpushx_no_se_guardan_valores_en_un_value_cuyo_tipo_no_es_una_lista() -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("LPUSHX")
        .arg("edad_luz")
        .arg("jacinta")
        .arg("leonela")
        .arg("margarita")
        .arg("leonilda")
        .arg("murcia")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("error - not list type"),
            got: format!("{:?}", ret),
        }));
    }
}

fn test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_en_rango(
) -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("LRANGE")
        .arg("jinetes_de_tucuman")
        .arg("0")
        .arg("4")
        .query(&mut con)?;

    if &ret[0] == &String::from("jinete_1")
        && &ret[1] == &String::from("jinete_2")
        && &ret[2] == &String::from("jinete_3")
        && &ret[3] == &String::from("jinete_4")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("jinete_1 jinete_2 jinete_3 jinete_4"),
            got: format!("{} {} {} {}", ret[0], ret[1], ret[2], ret[3]),
        }));
    }
}

fn test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_mayor_a_long_de_la_lista(
) -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("LRANGE")
        .arg("jinetes_de_tucuman")
        .arg("0")
        .arg("20")
        .query(&mut con)?;

    if &ret[0] == &String::from("jinete_1")
        && &ret[1] == &String::from("jinete_2")
        && &ret[2] == &String::from("jinete_3")
        && &ret[3] == &String::from("jinete_4")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("jinete_1 jinete_2 jinete_3 jinete_4"),
            got: format!("{} {} {} {}", ret[0], ret[1], ret[2], ret[3]),
        }));
    }
}

fn test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_menor_a_la_1ra_pos_de_la_lista(
) -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("LRANGE")
        .arg("jinetes_de_tucuman")
        .arg("-3")
        .arg("7")
        .query(&mut con)?;

    if &ret[0] == &String::from("jinete_6")
        && &ret[1] == &String::from("jinete_7")
        && &ret[2] == &String::from("jinete_8")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("jinete_6 jinete_7 jinete_8"),
            got: format!("{} {} {}", ret[0], ret[1], ret[2]),
        }));
    }
}

fn test_se_devuelve_lista_de_elementos_especificado_por_limite_superior_e_inferior_menor_a_la_1ra_pos_de_la_lista_con_upper_bound_mayor_a_len(
) -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("LRANGE")
        .arg("jinetes_de_tucuman")
        .arg("-3")
        .arg("70")
        .query(&mut con)?;
    if &ret[0] == &String::from("jinete_6")
        && &ret[1] == &String::from("jinete_7")
        && &ret[2] == &String::from("jinete_8")
    {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("jinete_6 jinete_7 jinete_8"),
            got: format!("{} {} {}", ret[0], ret[1], ret[2]),
        }));
    }
}

fn test_se_devuelve_lista_vacia_porque_limite_inferior_supera_al_limite_superior() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("LRANGE")
        .arg("jinetes_de_tucuman")
        .arg("5")
        .arg("3")
        .query(&mut con)?;
    if &ret == &vec!["".to_string()] {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from(""),
            got: format!("{}", ret[0]),
        }));
    }
}

fn test_se_eliminan_3_valores_repetidos_de_izquierda_a_derecha_de_un_value_de_tipo_list(
) -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LREM")
        .arg("love_the_cat")
        .arg("3")
        .arg("my")
        .query(&mut con)?;
    if ret == 3 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: 3.to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_se_eliminan_todos_los_valores_repetidos_un_value_de_tipo_list() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LREM")
        .arg("love_the_bunny")
        .arg("0")
        .arg("my")
        .query(&mut con)?;
    if ret == 4 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: 4.to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_se_devuelve_cero_si_se_busca_remover_un_valor_cuya_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LREM")
        .arg("love_the_snake")
        .arg("2")
        .arg("my")
        .query(&mut con)?;
    if ret == 0 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: 0.to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_se_eliminan_3_valores_repetidos_de_izquierda_a_derecha_de_un_value_de_tipo_list_reverso(
) -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("LREM")
        .arg("love_the_dog")
        .arg("-3")
        .arg("my")
        .query(&mut con)?;

    if ret == 3 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: 3.to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_list_reemplaza_un_elemento_de_value_list_type_exitosamente() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LSET")
        .arg("norse_gods")
        .arg("2")
        .arg("bragi")
        .query(&mut con)?;

    if ret == "Ok".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "Ok".to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_list_reemplaza_un_elemento_de_value_list_type_exitosamente_empleando_indice_negativo_valido(
) -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LSET")
        .arg("greek_gods")
        .arg("-2")
        .arg("apollo")
        .query(&mut con)?;

    if ret == "Ok".to_string() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "Ok".to_string(),
            got: ret.to_string(),
        }));
    }
}

fn test_list_no_reemplaza_un_elemento_de_value_list_type_con_indice_fuera_de_rango_error(
) -> TestResult {
    let mut con = connect()?;
    let ret = redis::cmd("LSET")
        .arg("egyptian_gods")
        .arg("15")
        .arg("hathor")
        .query(&mut con);
    assert!(ret.is_err());

    return if ret.is_err() {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("Value stored at key set_remove_4 is not a Set"),
            got: ret.unwrap(),
        }))
    };
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

pub fn test_list_index_no_list_type_error() -> TestResult {
    let mut con = connect()?;
    let ret = redis::cmd("LINDEX")
        .arg("edad_maria")
        .arg("0")
        .query(&mut con);
    assert!(ret.is_err());

    return if ret.is_err() {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("Value is not a list"),
            got: ret.unwrap(),
        }))
    };
}

pub fn test_list_index_devuelve_vacio_porque_esta_outbounded() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LINDEX")
        .arg("marcas_de_vinos_en_damajuana")
        .arg("0")
        .query(&mut con)?;
    return if ret == String::from("") {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from(""),
            got: ret.to_string(),
        }))
    };
}

fn test_list_index_devuelve_elemento_index_valido_pero_negativo() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LINDEX")
        .arg("jinetes_de_tucuman")
        .arg("-1")
        .query(&mut con)?;
    if ret == String::from("jinete_8") {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("jinete_8"),
            got: ret.to_string(),
        }));
    }
}

pub fn test_list_lpop() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("LPOP").arg("paises").query(&mut con)?;

    return if ret == String::from("argentina") {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("argentina"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_list_lpop_sin_count_devuelve_nil_cuando_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("LPOP")
        .arg("listado_de_franceses_que_estudiaron_en_brest")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_lpop_con_count_devuelve_nil_cuando_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("LPOP")
        .arg("listado_de_franceses_que_estudiaron_en_lyon")
        .arg("2")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_lpop_con_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("LPOP")
        .arg("edad_maria")
        .arg("2")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_lpop_sin_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("LPOP").arg("edad_maria").query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_lpop_con_count_devuelve_menos_elementos_que_los_que_indica_count_porque_count_es_mayor_que_list_len(
) -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("LPOP")
        .arg("pineapple_mascots")
        .arg("8")
        .query(&mut con)?;

    return if ret.contains(&String::from("pineapple_1"))
        && ret.contains(&String::from("pineapple_2"))
        && ret.contains(&String::from("pineapple_3"))
        && ret.contains(&String::from("pineapple_4"))
    {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(
                "{:?}",
                vec![
                    String::from("pineapple_1"),
                    String::from("pineapple_2"),
                    String::from("pineapple_3"),
                    String::from("pineapple_4")
                ]
            ),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_lpop_with_count() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("LPOP")
        .arg("provincias")
        .arg("2")
        .query(&mut con)?;

    return if ret.contains(&String::from("jujuy")) && ret.contains(&String::from("mendoza")) {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!("{:?}", vec![String::from("jujuy"), String::from("mendoza")]),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_rpop() -> TestResult {
    let mut con = connect()?;
    let ret: String = redis::cmd("RPOP").arg("paises2").query(&mut con)?;

    return if ret == String::from("portugal") {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("portugal"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_list_rpop_with_count() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("RPOP")
        .arg("provincias2")
        .arg("2")
        .query(&mut con)?;

    return if ret.contains(&String::from("catamarca")) && ret.contains(&String::from("chaco")) {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(
                "{:?}",
                vec![String::from("catamarca"), String::from("chaco")]
            ),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_rpop_with_count_greater_than_list_lenght() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("RPOP")
        .arg("equipos_de_la_bundesliga")
        .arg("4")
        .query(&mut con)?;

    return if ret.contains(&String::from("borussia"))
        && ret.contains(&String::from("werder"))
        && ret.contains(&String::from("bayer"))
    {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(
                "{:?}",
                vec![
                    String::from("borussia"),
                    String::from("werder"),
                    String::from("bayer")
                ]
            ),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_rpop_sin_count_devuelve_nil_cuando_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("RPOP")
        .arg("listado_de_franceses_que_estudiaron_en_brest")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_rpop_con_count_devuelve_nil_cuando_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("RPOP")
        .arg("listado_de_franceses_que_estudiaron_en_brest")
        .arg("4")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_rpop_con_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("RPOP")
        .arg("edad_maria")
        .arg("2")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_rpop_sin_count_devuelve_nil_cuando_el_tipo_del_valor_no_es_list() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("RPOP").arg("edad_maria").query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_list_rpushx() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("RPUSHX")
        .arg("sabores")
        .arg("vainilla")
        .arg("coco")
        .query(&mut con)?;

    return if ret == 4 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: 4.to_string(),
            got: ret.to_string(),
        }))
    };
}

pub fn test_list_rpushx_nonexisting_key_returns_zero() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("RPUSHX")
        .arg("paiseslimitrofes")
        .arg("chile")
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: 0.to_string(),
            got: ret.to_string(),
        }))
    };
}

pub fn test_list_rpushx_arrroja_error_cuando_se_intenta_almacenar_dato_en_una_clave_que_no_guarda_un_valor_de_tipo_list(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("RPUSHX")
        .arg("edad_maria")
        .arg("25")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error missing parameter"),
            got: format!("{:?}", ret),
        }));
    };
}

pub fn test_list_rpush() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("RPUSHX")
        .arg("sabores_de_chocolate")
        .arg("chocolate amargo")
        .arg("chocolate marroc")
        .query(&mut con)?;

    return if ret == 5 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: 5.to_string(),
            got: ret.to_string(),
        }))
    };
}

pub fn test_list_rpush_nonexisting_key_creates_key_value_pair_and_returns_list_size() -> TestResult
{
    let mut con = connect()?;
    let ret: usize = redis::cmd("RPUSH")
        .arg("sabores_silvestres_de_helado")
        .arg("frutos rojos del este")
        .arg("menta de primera cosecha")
        .arg("grosellas aireadas")
        .query(&mut con)?;

    return if ret == 3 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: 3.to_string(),
            got: ret.to_string(),
        }))
    };
}
pub fn test_list_rpush_arrroja_error_cuando_se_intenta_almacenar_dato_en_una_clave_que_no_guarda_un_valor_de_tipo_list(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("RPUSHX")
        .arg("edad_maria")
        .arg("25")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error missing parameter"),
            got: format!("{:?}", ret),
        }));
    };
}

//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------SET COMMANDS------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------

pub fn test_set_add() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SADD")
        .arg("set_values_2")
        .arg("rust")
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

pub fn test_set_add_valor_ya_es_miembro() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SADD")
        .arg("granaderos_espigados")
        .arg("granadero_espigado_1")
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("0"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_add_clave_no_existe_se_crea_la_clave_con_valor() -> TestResult {
    let mut con = connect()?;
    let ret_initial: usize = redis::cmd("DBSIZE").query(&mut con)?;
    let ret: usize = redis::cmd("SADD")
        .arg("granaderos_agasajados")
        .arg("granadero_agasajado_1")
        .query(&mut con)?;

    let ret_final: usize = redis::cmd("DBSIZE").query(&mut con)?;
    return if ret == 1 && ret_initial == (ret_final - 1) {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("'0' and db_size_after_getdel == db_size_before_getdel - 1"),
            got: format!(
                "added_elements: {:?} , db_size_after_get_del {:?} , db_size_before_get_del {:?} ",
                ret, ret_final, ret_initial
            ),
        }))
    };
}

pub fn test_set_add_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set_es_string_type(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("SADD")
        .arg("edad_maria")
        .arg("maria parisina")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error - key does not hold set value type"),
            got: format!("{:?}", ret),
        }));
    };
}

pub fn test_set_add_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set_es_list_type(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("SADD")
        .arg("jinetes_de_tucuman")
        .arg("jinete sin caballo")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error - key does not hold set value type"),
            got: format!("{:?}", ret),
        }));
    };
}

pub fn test_set_scard() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SCARD").arg("set_values_1").query(&mut con)?;

    return if ret == 2 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("2"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_scard_devuelve_cero_para_clave_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SCARD")
        .arg("granaderos_empetrolados")
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("0"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_add_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("SCARD")
        .arg("jinetes_de_tucuman")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error - key does not hold set value type"),
            got: format!("{:?}", ret),
        }));
    };
}

pub fn test_set_ismember() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SISMEMBER")
        .arg("set_values_1")
        .arg("value_1")
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

pub fn test_set_ismember_devuelve_cero_porque_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SISMEMBER")
        .arg("granaderos_enajenados")
        .arg("granadero_enajenado_1")
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_ismember_devuelve_cero_porque_el_valor_no_es_miembro() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SISMEMBER")
        .arg("granaderos_espigados")
        .arg("granadero_espigado_5")
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("1"),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_ismember_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("SISMEMBER")
        .arg("jinetes_de_tucuman")
        .arg("jinete_1")
        .query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error - key does not hold set value type"),
            got: format!("{:?}", ret),
        }));
    };
}

pub fn test_set_smembers() -> TestResult {
    let mut con = connect()?;
    let ret: Vec<String> = redis::cmd("SMEMBERS").arg("set_values_1").query(&mut con)?;

    return if ret.contains(&&String::from("value_1")) && ret.contains(&&String::from("value_2")) {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(
                "{:?}",
                vec![String::from("value_1"), String::from("value_2")]
            ),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_set_smembers_devuelve_nil_cuando_la_clave_no_existe() -> TestResult {
    let mut con = connect()?;
    let ret: () = redis::cmd("SMEMBERS")
        .arg("granaderos_acobardados")
        .query(&mut con)?;

    return if ret == () {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: format!(""),
            got: format!("{:?}", ret),
        }))
    };
}

pub fn test_set_members_arroja_error_cuando_la_clave_contiene_un_valor_que_no_es_de_tipo_set(
) -> TestResult {
    let mut con = connect()?;
    let ret: Result<String, RedisError> = redis::cmd("SISMEMBER").arg("edad_maria").query(&mut con);

    if ret.is_err() {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: String::from("Error - key does not hold set value type"),
            got: format!("{:?}", ret),
        }));
    };
}

pub fn test_set_srem() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SREM")
        .arg("set_remove_1")
        .arg("value_1")
        .query(&mut con)?;

    return if ret == 1 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: 1.to_string(),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_srem_removes_multiple_values() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SREM")
        .arg("set_remove_2")
        .arg("value_1")
        .arg("value_2")
        .query(&mut con)?;

    return if ret == 2 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: 1.to_string(),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_srem_removes_zero_values() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("SREM")
        .arg("set_remove_3")
        .arg("value_1")
        .query(&mut con)?;

    return if ret == 0 {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: 1.to_string(),
            got: ret.to_string(),
        }))
    };
}

pub fn test_set_srem_removes_returns_error() -> TestResult {
    let mut con = connect()?;
    let ret = redis::cmd("SREM")
        .arg("set_remove_4")
        .arg("value_1")
        .query(&mut con);
    assert!(ret.is_err());

    return if ret.is_err() {
        Ok(())
    } else {
        Err(Box::new(ReturnError {
            expected: String::from("Value stored at key set_remove_4 is not a Set"),
            got: ret.unwrap(),
        }))
    };
}

fn test_rpush_lista_inexistente() -> TestResult {
    let mut con = connect()?;
    let ret: usize = redis::cmd("RPUSH")
        .arg("clubes")
        .arg("central")
        .arg("boca")
        .arg("river")
        .arg("racing")
        .arg("chacarita")
        .query(&mut con)?;

    if ret == 5 {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: "5".to_string(),
            got: ret.to_string(),
        }));
    }
}

//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-----------------------------------------------------PUBSUB COMMANDS-----------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------------------

fn test_pubsub() -> TestResult {
    // Connection for subscriber api
    let mut pubsub_con = connect().unwrap();

    // Barrier is used to make test thread wait to publish
    // until after the pubsub thread has subscribed.
    let barrier = Arc::new(Barrier::new(4));
    let close_barrier = Arc::new(Barrier::new(4));
    let pubsub_barrier = barrier.clone();
    let close_pubsub = close_barrier.clone();
    let _thread = thread::spawn(move || {
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
    let _thread_2 = thread::spawn(move || {
        let mut pubsub_2 = pubsub_con_2.as_pubsub();
        pubsub_2.subscribe("foo").unwrap();

        let _ = pubsub_barrier_2.wait();
        let _ = close_pubsub.wait();
    });

    let mut pubsub_con_3 = connect().unwrap();
    let pubsub_barrier_3 = barrier.clone();
    let close_pubsub = close_barrier.clone();
    let _thread_3 = thread::spawn(move || {
        let mut pubsub_3 = pubsub_con_3.as_pubsub();
        pubsub_3.subscribe("helloworld").unwrap();

        let _ = pubsub_barrier_3.wait();
        let _ = close_pubsub.wait();
    });

    let _ = barrier.wait();
    let mut con = connect().unwrap();
    let receivers: usize = redis::cmd("PUBLISH")
        .arg("foo")
        .arg(42)
        .query(&mut con)
        .unwrap();
    let subs: Vec<String> = redis::cmd("PUBSUB")
        .arg("NUMSUB")
        .arg("foo")
        .query(&mut con)
        .unwrap();
    let channels: Vec<String> = redis::cmd("PUBSUB")
        .arg("CHANNELS")
        .query(&mut con)
        .unwrap();
    let channels_pattern: Vec<String> = redis::cmd("PUBSUB")
        .arg("CHANNELS")
        .arg("*d")
        .query(&mut con)
        .unwrap();
    let _ = close_barrier.wait();

    let mut pass = true;
    if receivers != 2 {
        pass = false;
    }

    if subs != vec![String::from("foo"), String::from("2")] {
        pass = false;
    }

    if channels != vec![String::from("foo"), String::from("helloworld")]
        && channels != vec![String::from("helloworld"), String::from("foo")]
    {
        pass = false;
    }

    if channels_pattern != vec![String::from("helloworld")] {
        pass = false;
    }

    // thread.join().expect("Something went wrong");
    // thread_2.join().expect("Something went wrong");
    // thread_3.join().expect("Something went wrong");
    if pass {
        return Ok(());
    } else {
        return Err(Box::new(ReturnError {
            expected: format!(
                "publish: {}, numsub: {:?}, channels: {:?}, channels pattern: {:?}",
                2,
                vec![String::from("foo"), String::from("2")],
                vec![String::from("foo"), String::from("helloworld")],
                vec![String::from("helloworld")]
            ),
            got: format!(
                "publish: {}, numsub: {:?}, channels: {:?}, channels pattern: {:?}",
                receivers, subs, channels, channels_pattern
            ),
        }));
    }
}

//test unsubscribe -> falta funcionalidad para estado tal que no pueda mandar ningun otro comando que los de pubsub
