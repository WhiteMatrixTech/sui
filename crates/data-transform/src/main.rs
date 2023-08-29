use data_transform::*;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::QueryableByName;
use diesel::pg::sql_types::Bytea;
use anyhow::anyhow;
use std::sync::Arc;
use std::process::exit;

use sui_types::object::MoveObject;
use sui_types::object::ObjectFormatOptions;
use move_bytecode_utils::module_cache::SyncModuleCache;
use move_core_types::value::MoveStruct;

use sui_indexer::new_pg_connection_pool;
use self::models::*;
use std::env;
use sui_indexer::store::module_resolver::IndexerModuleResolver;
use sui_indexer::errors::IndexerError;

use sui_types::parse_sui_struct_tag;
use sui_json_rpc_types::SuiMoveStruct;
use move_core_types::language_storage::ModuleId;
use move_bytecode_utils::module_cache::GetModule;
use std::collections::HashMap;
use move_core_types::resolver::ModuleResolver;

use tracing::debug;
extern crate base64;
use base64::{decode, DecodeError};

struct GrootModuleResolver {
    module_map: HashMap<String, Vec<u8>>,
    original: IndexerModuleResolver,
}

impl GrootModuleResolver {
    fn new(blocking_cp: PgConnectionPool) -> Self {
        let original = IndexerModuleResolver::new(blocking_cp)
        let  module_bytes = "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AQEDAwEDAwEFAwMDBwsLAQgJCxABCQALAwIJAAkBCgsDAgkACQEDAwMLEAEJAAMGCAgHCAgDAwMGCw4BAwMDCBQLEAEJAQsQAQkBAQMDAwEDBwgJAwEGCwsBCQABCwMCCQAJAQIHCwsBCQADAQcJAAEGCxMCCQAJAQEGCw4BCQABBgkAAgYLEwIJAAkBCQABBgkBAwcLDQEJAAUDAgEDAgcLEAEJAAMCBwsQAQkACxABCQACBwsXAgkACQEJAAEHCQECBwsTAgkACQEJAAIGCwsBCQADAQYKCQABCwQCCQAJASQBAwEDAwEDAwEFAwMDBwsLAQgJCxABCQALAwIJAAkBCgsDAgkACQEDAwsQAQkAAwYICAcICAMGCw4BAwMDCBQLEAEJAQEDAwsQAQkBAwcICQMlAQMBAwMBAwMBBQMDAwcLCwEICQsQAQkACwMCCQAJAQoLAwIJAAkBAwMLEAEJAQMGCAgHCAgDAwYLDgEDAwMIFAsQAQkBAQMDCxABCQEDBwgJAwULEAEJAAsQAQkACxIBCQALEAEJAQsQAQkBAgcLEgEJAAsSAQkAAwcLEgEJAAMHCBgHAwcLCwEICQgIAwUDAwMHCw0BCQAGCAwDAwcLCwEJAAMJAAMHCxMCCQAJAQkACQEBCwECCQAJAQIGCxcCCQAJAQkAAwcLFwIJAAkBCQAJAQwLEAEJAAsQAQkACxABCQADAwMFCxABCQELEAEJAQsQAQkBAwMCBgsNAQkABQgIFAMDAQUDAwMBCwICCQAJAQ0IFAMDAwMDAwUDAQUDAwELBQIJAAkBCwMDBgsLAQgJBwsLAQgJAwEICAUDAwcLEwIDAwMDBwgJCAgWAwMDAwcLCwEICQMDAwEFAwsDAgkACQEKCwMCCQAJAQEHCwsBCAkICAMDBQgUAwcLEwIDAxoFAwMDAwMGCwsBCAkHCwsBCAkDAwEDCwMCCQAJAQoLAwIJAAkBAwEDAwMICAMFCBQDAwcLEwIDAxwBBQMDAwMDBwsLAQgJAwMDAwsDAgkACQEKCwMCCQAJAQMBAwMDAwcLCwEICQgIAwUIFAMDBwsTAgMDBwYICQoICAYICAYLDgEDAwUGCxMCAwMFAwMFAwMECw4BAwsOAQMLDgEDCw4BAwELDgEJAAYDCgMDAwMKAwQDBggIBgsOAQMGCxMCAwgIBAYLCwEICQMFBgsTAgMDCkFjY291bnRDYXARQWxsT3JkZXJzQ2FuY2VsZWQaQWxsT3JkZXJzQ2FuY2VsZWRDb21wb25lbnQHQmFsYW5jZQVDbG9jawRDb2luC0NyaXRiaXRUcmVlCUN1c3RvZGlhbgxEZXBvc2l0QXNzZXQCSUQLTGlua2VkVGFibGUGT3B0aW9uBU9yZGVyDU9yZGVyQ2FuY2VsZWQLT3JkZXJGaWxsZWQLT3JkZXJQbGFjZWQEUG9vbAtQb29sQ3JlYXRlZANTVUkFVGFibGUJVGlja0xldmVsCVR4Q29udGV4dAhUeXBlTmFtZQNVSUQNV2l0aGRyYXdBc3NldBlhY2NvdW50X2F2YWlsYWJsZV9iYWxhbmNlD2FjY291bnRfYmFsYW5jZQ1hY2NvdW50X293bmVyA2FkZARhc2tzBGJhY2sHYmFsYW5jZQpiYXNlX2Fzc2V0HGJhc2VfYXNzZXRfcXVhbnRpdHlfY2FuY2VsZWQaYmFzZV9hc3NldF9xdWFudGl0eV9maWxsZWQaYmFzZV9hc3NldF9xdWFudGl0eV9wbGFjZWQdYmFzZV9hc3NldF9xdWFudGl0eV9yZW1haW5pbmcXYmFzZV9hc3NldF90cmFkaW5nX2ZlZXMOYmFzZV9jdXN0b2RpYW4SYmF0Y2hfY2FuY2VsX29yZGVyBGJpZHMGYm9ycm93FGJvcnJvd19sZWFmX2J5X2luZGV4EmJvcnJvd19sZWFmX2J5X2tleQpib3Jyb3dfbXV0GGJvcnJvd19tdXRfbGVhZl9ieV9pbmRleBFjYW5jZWxfYWxsX29yZGVycwxjYW5jZWxfb3JkZXIXY2xlYW5fdXBfZXhwaXJlZF9vcmRlcnMPY2xpZW50X29yZGVyX2lkB2Nsb2JfdjIFY2xvY2sEY29pbghjb250YWlucw5jcmVhdGVfYWNjb3VudBZjcmVhdGVfY3VzdG9taXplZF9wb29sC2NyZWF0ZV9wb29sDGNyZWF0ZV9wb29sXwxjcmVhdGlvbl9mZWUHY3JpdGJpdAxjdXN0b2RpYW5fdjIfZGVjcmVhc2VfdXNlcl9hdmFpbGFibGVfYmFsYW5jZRxkZWNyZWFzZV91c2VyX2xvY2tlZF9iYWxhbmNlDGRlcG9zaXRfYmFzZQ1kZXBvc2l0X3F1b3RlDWRlc3Ryb3lfZW1wdHkTZGVzdHJveV9lbXB0eV9sZXZlbARlbWl0E2VtaXRfb3JkZXJfY2FuY2VsZWQRZW1pdF9vcmRlcl9maWxsZWQFZXZlbnQQZXhwaXJlX3RpbWVzdGFtcBBmaW5kX2Nsb3Nlc3Rfa2V5CWZpbmRfbGVhZgxmcm9tX2JhbGFuY2UFZnJvbnQDZ2V0FmdldF9sZXZlbDJfYm9va19zdGF0dXMfZ2V0X2xldmVsMl9ib29rX3N0YXR1c19hc2tfc2lkZR9nZXRfbGV2ZWwyX2Jvb2tfc3RhdHVzX2JpZF9zaWRlEGdldF9tYXJrZXRfcHJpY2UQZ2V0X29yZGVyX3N0YXR1cwJpZB9pbmNyZWFzZV91c2VyX2F2YWlsYWJsZV9iYWxhbmNlEmluamVjdF9saW1pdF9vcmRlcgtpbnNlcnRfbGVhZgxpbnRvX2JhbGFuY2UGaXNfYmlkCGlzX2VtcHR5B2lzX25vbmUEam9pbgxsaW5rZWRfdGFibGUQbGlzdF9vcGVuX29yZGVycwxsb2NrX2JhbGFuY2UIbG90X3NpemUNbWFrZXJfYWRkcmVzcxVtYWtlcl9jbGllbnRfb3JkZXJfaWQRbWFrZXJfcmViYXRlX3JhdGUNbWFrZXJfcmViYXRlcwltYXRjaF9hc2sJbWF0Y2hfYmlkHW1hdGNoX2JpZF93aXRoX3F1b3RlX3F1YW50aXR5BG1hdGgIbWF4X2xlYWYIbWluX2xlYWYQbWludF9hY2NvdW50X2NhcANtdWwDbmV3BG5leHQRbmV4dF9hc2tfb3JkZXJfaWQRbmV4dF9iaWRfb3JkZXJfaWQJbmV4dF9sZWFmBG5vbmUGb2JqZWN0C29wZW5fb3JkZXJzBm9wdGlvbghvcmRlcl9pZAxvcmRlcl9pc19iaWQPb3JkZXJzX2NhbmNlbGVkEW9yaWdpbmFsX3F1YW50aXR5BW93bmVyEXBsYWNlX2xpbWl0X29yZGVyEnBsYWNlX21hcmtldF9vcmRlcgdwb29sX2lkDXByZXZpb3VzX2xlYWYFcHJpY2UJcHVzaF9iYWNrCHF1YW50aXR5C3F1b3RlX2Fzc2V0GHF1b3RlX2Fzc2V0X3RyYWRpbmdfZmVlcw9xdW90ZV9jdXN0b2RpYW4GcmVtb3ZlFHJlbW92ZV9sZWFmX2J5X2luZGV4DHJlbW92ZV9v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".as_bytes();
        let mut module_map: HashMap<String, Vec<u8>> = HashMap::new();
        match decode(module_bytes) {
            Ok(decoded_bytes) => {
                println!("we have decoded bytes!");
                //let decoded_str = String::from_utf8(decoded_bytes).unwrap();
                //println!("we are groot {}", decoded_str);
                module_map.insert("clob_v2".to_string(), decoded_bytes);
            }
            Err(_) => {
                println!("Invalid base64 encoding");
            }
        }
        //module_map.insert("clob_v2".to_string(), vec![0, 0, 1, 1]);
        //module_map.insert("clob_v2".to_string(), module_bytes.to_vec());
        Self{ module_map, original }
    }
}

impl ModuleResolver for GrootModuleResolver {
    type Error = IndexerError;

    fn get_module(&self, id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        let module_name = id.name().to_string();
        println!("i am groot, module_name: {}", module_name);
        match self.module_map.get(&module_name) {
            None => self.original.get_module(id),
            Some(bytes) => Ok(Some(bytes.clone())),
        }
    }
}


fn main() {
    #[derive(QueryableByName)]
    #[derive(Debug)]
    struct ModuleBytes {
        #[diesel(sql_type = Bytea)]
        data: Vec<u8>,
    }


    use self::schema::events::dsl::*;
    use self::schema::events_json::dsl::*;

    // get the starting id from the arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: data-transform <id>");
        exit(0);
    }

    let start_id: i64 = match args[1].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Invalid integer: {}", args[1]);
            exit(0);
        }
    };

    println!("start id = {}", start_id);

    let mut end_id: i64 = start_id +1;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = &mut establish_connection();

    let blocking_cp = new_pg_connection_pool(&database_url).map_err(|e| anyhow!("Unable to connect to Postgres, is it running? {e}"));
    //let module_cache = Arc::new(SyncModuleCache::new(IndexerModuleResolver::new(blocking_cp.expect("REASON").clone())));
    //
    let module_cache = Arc::new(SyncModuleCache::new(GrootModuleResolver::new(blocking_cp.expect("REASON").clone())));

    for target_id in start_id..end_id {

        let event = events
            .find(target_id)
            .select(Event::as_select())
            .first(connection)
            .optional();

        match event {
            Ok(Some(event)) => {
                println!("-----------\n");
                println!("event id = {}", event.id);
                debug!("event sequence = {:#?}", event.event_sequence);
                debug!("sender = {:#?}", event.sender);
                println!("package = {:#?}", event.package);
                debug!("module = {:#?}", event.module);
                debug!("type = {:#?}", event.event_type);
                let text = String::from_utf8_lossy(&event.event_bcs);
                debug!("bcs in text = {:#?}", text);

                if event.package != "0x000000000000000000000000000000000000000000000000000000000000dee9" {
                    println!("not deepbook skipping...");
                    continue;
                }

                // check for the previous record in events_json
                let eventj = events_json
                    .find(target_id)
                    .select(EventsJson::as_select())
                    .first(connection)
                    .optional();

                match eventj {
                    Ok(Some(_eventj)) => {
                        println!("Already processed {}, skipping...", target_id);
                        continue;
                    }
                    Ok(None) => {
                        println!("Unable to find event_json {}", target_id);
                    }
                    Err(_) => {
                        println!("An error occured while fetching event_json {}", target_id);
                    }
                }


                // JSON parsing starts here
                let type_ = parse_sui_struct_tag(&event.event_type).expect("cannot load StructTag");
                let module_id = ModuleId::new(type_.address, type_.module.clone());
                println!("module id = {}", module_id);

                let newmodule = module_cache.get_module_by_id(&module_id).expect("Module {module_id} must load").unwrap();
                println!("new module = {newmodule:#?}");

                /*
                println!("iterating...");
                for type_def in &newmodule.struct_defs {
                    println!("- {:#?}", newmodule.struct_handles[type_def.struct_handle.0 as usize]);
                    let handle = &newmodule.struct_handles[type_def.struct_handle.0 as usize];
                    let name_idx = handle.name;
                    println!("struct {:?}", newmodule.identifiers[name_idx.0 as usize]);
                }
                */

                let layout = MoveObject::get_layout_from_struct_tag(
                    type_,
                    ObjectFormatOptions::default(),
                    &module_cache,
                    );

                match layout {
                    Ok(l) => {
                        let move_object = MoveStruct::simple_deserialize(&event.event_bcs, &l)
                            .map_err(|e| IndexerError::SerdeError(e.to_string()));

                        match move_object {
                            Ok(m) => {
                                let parsed_json = SuiMoveStruct::from(m).to_json_value();
                                let final_result = serde_json::to_string_pretty(&parsed_json).unwrap();
                                println!("event json = {}", final_result);

                                let new_event_json = EventsJson { id: event.id, event_json: final_result };

                                let _inserted_event_json = diesel::insert_into(events_json)
                                    .values(&new_event_json)
                                    .execute(connection)
                                    .expect("Error saving new events json");

                                println!("Inserted new event_json id: {}", event.id);

                            }|
                            Err(e) => {
                                println!("error in deserialize:{}", e);
                                exit(0);
                            }
                        }
                    }
                    Err(err) => {
                        println!("error in get_layout: {}", err);
                        exit(0);
                    }
                }
            }
            Ok(None) => {
                println!("Unable to find event {}", target_id);
                exit(0);
            }
            Err(_) => {
                println!("An error occured while fetching event {}", target_id);
                exit(0);
            }
        }
    }
}
