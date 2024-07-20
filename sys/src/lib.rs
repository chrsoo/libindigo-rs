#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::{ffi::{c_char, c_void, CStr, CString}, ptr};

    use super::*;

    unsafe extern "C" fn my_attach(client: *mut indigo_client) -> indigo_result {
        let c_msg = std::ffi::CString::new("attached to INDIGO bus...").unwrap();
        unsafe {
            indigo_log(c_msg.as_ptr());
            println!("attach client callback!");
            // Request property definitions
            indigo_enumerate_properties(client, &INDIGO_ALL_PROPERTIES as *const _ as *mut indigo_property);
        }
        return indigo_result_INDIGO_OK;
    }

    unsafe extern "C" fn my_define_property(
        _client: *mut indigo_client,
        _device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8) -> indigo_result {

        // let d = CStr::from_bytes_until_nul(&(*device).name.map(|i| i as u8)).unwrap().to_str().unwrap();
        if property != ptr::null_mut() {
            // let id = [0 as c_char; 256];
            // let rust_id = unsafe { CStr::from_ptr(id.as_ptr()) };
            // let rust_id = rust_id.to_owned();
            // println!("{:?}", rust_id);

            let ptr = (*property).name.as_ptr();
            let p = unsafe { CStr::from_ptr(ptr) };
            let p = p.to_owned();
            print!("property: {:?}", p);
        }

        if message != ptr::null() {
            let m = CStr::from_ptr(message).to_str().unwrap();
            print!("; message: {m}");
        } else {
            print!("null message");
        }

        // println!("d: {d}; p: {p}; m: {m}");

        return indigo_result_INDIGO_OK;
    }

    unsafe extern "C" fn my_update_property(
        client: *mut indigo_client,
        device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8) -> indigo_result {

        println!("update property callback!");
        // do something useful here ;)
        return indigo_result_INDIGO_OK;
    }

    unsafe extern "C" fn  my_detach(
        client: *mut indigo_client ) -> indigo_result {
        let c_msg = std::ffi::CString::new("detached from INDIGO bus").unwrap();
        unsafe { indigo_log(c_msg.as_ptr()) };
        return indigo_result_INDIGO_OK;
    }

    #[test]
    fn client_test() {
        // let name = std::ffi::CString::new("MyClient").unwrap();
        // let bytes: [i8; name.len() + 1] = name.as_bytes_with_nul().iter().map(|b| *b as i8).collect();
        // let buf = [0i8;128];
        // buf[..bytes.len()].copy_from_slice(bytes);

        let name = b"MyClient\0".map(|b| b as i8);
        let mut buf = [0i8;128];
        buf[..name.len()].copy_from_slice(&name);

        let mut indigo_client = indigo_client {
            name: buf,                                      // client name
            is_remote: false,                               // is this a remote client "no" - this is us
            client_context: ptr::null_mut(),                // we do not have client specific data
            last_result: indigo_result_INDIGO_OK,           // result of last bus operation
                                                            // - we just initialize it with ok
            version: indigo_version_INDIGO_VERSION_CURRENT, // the client speaks current indigo version
            enable_blob_mode_records: ptr::null_mut(),      // BLOB mode records -> Set this to NULL
            attach: Some(my_attach),
            define_property: Some(my_define_property),
            update_property: Some(my_update_property),
            delete_property: None,
            send_message: None,
            detach: Some(my_detach)
        };
        unsafe {
            indigo_start();

            /* We want to see debug messages on the screen */
            indigo_set_log_level(indigo_log_levels_INDIGO_LOG_DEBUG);
            indigo_attach_client(std::ptr::addr_of_mut!(indigo_client));

            /* We want to connect to a remote indigo host indigosky.local:7624 */
            let mut server = indigo_server_entry {
                name: [0i8; 128],
                host: [0i8; 128],
                port: 0,
                connection_id: 0,
                thread: ptr::null_mut(),
                thread_started: false,
                socket: 0,
                protocol_adapter: ptr::null_mut(),
                last_error: [0i8; 256],
                shutdown: false,
            };

            let server_name = CString::new("indigosky").unwrap();
            let host = CString::new("localhost").unwrap();

            let mut srv_ptr = ptr::addr_of_mut!(server);
            let srv_ptr_ptr = ptr::addr_of_mut!(srv_ptr);

            indigo_connect_server(server_name.as_ptr(), host.as_ptr(), 7624, srv_ptr_ptr);

            /* We can do whatever we want here while we are waiting for
            the client to complete. For example we can call some GUI
            framework main loop etc...
            Instead we will just sleep for 10 seconds.
            */
            indigo_usleep(10 * ONE_SECOND_DELAY);

            indigo_disconnect_server(srv_ptr);
            indigo_detach_client(ptr::addr_of_mut!(indigo_client));
            indigo_stop();
        }
    }
}