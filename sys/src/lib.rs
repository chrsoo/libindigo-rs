// #![feature(ptr_metadata,layout_for_ptr)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use core::slice;
use std::{
    alloc::{alloc, dealloc, Layout, LayoutErr, LayoutError},
    cmp,
    ffi::{c_char, CStr, CString, NulError},
    hash::Hash,
    marker::PhantomData,
    mem,
    ops::Index,
    ptr,
    str::FromStr,
};

use enum_primitive::*;
use fambox::FamHeader;
use log::{error, warn};

pub const DEFAULT_PORT: u16 = 7624;
pub const DEFAULT_HOST: &str = "indigo.local";
pub const BUF_SIZE: usize = 256;

pub const DEVICE_NAME_SIZE: usize = 128usize;
pub const PROPERTY_NAME_SIZE: usize = 128usize;
pub const ITEM_NAME_SIZE: usize = 128usize;
pub const TEXT_ITEM_SIZE: usize = 512usize;

pub type indigo_item_array = __IncompleteArrayField<indigo_item>;

impl indigo_property_type {
    /// Undefined property type.
    pub const UNDEFINED: indigo_property_type = indigo_property_type(0);
}

impl indigo_rule {
    pub const UNDEFINED: indigo_rule = indigo_rule(0);
}

// impl From<&[indigo_item]> for &mut indigo_property<[indigo_item]> {
//     fn from(items: &[indigo_item]) -> Self {
//         let len = items.len();
//         let ptr = unsafe { indigo_property::flex_ptr_mut(items.as_mut_ptr(), len) }
//         unsafe { indigo_property::flex_ptr_mut(ptr, len).assume_init() }
//     }
// }
impl indigo_property {
    /// Create a new indigo_property with default values for most fields.
    pub fn new(name: &str, device: &str, type_: indigo_property_type) -> indigo_property {
        // let len = items.len();
        // let item_layout = indigo_property::layout(len);
        // let buffer = unsafe { alloc(item_layout) } as *mut indigo_property<[indigo_item;0]>;
        // let property = unsafe { indigo_property::flex_ptr_mut(buffer, len).assume_init() };

        // Write data
        // property.items.clone_from_slice(items);

        // let data_slice = unsafe { property.fixed_mut() };
        // data_slice.clone_from_slice(items);

        indigo_property {
            device: str_to_buf(device),
            name: str_to_buf(name),
            type_,
            group: [0i8; 128],
            label: [0i8; 512],
            hints: [0i8; 512],
            state: indigo_property_state::INDIGO_OK_STATE,
            perm: indigo_property_perm::INDIGO_RW_PERM,
            rule: indigo_rule::INDIGO_ANY_OF_MANY_RULE,
            access_token: 0,
            // FIXME ensure that indigo_version enum is an i16 or change version from i16 to type
            version: indigo_version::INDIGO_VERSION_NONE.0 as i16,
            hidden: false,
            defined: false,
            allocated_count: 0,
            count: 0,
            items: __IncompleteArrayField::new(),
        }
    }

    /*
    /// Allocate memory and set name, decvice and type of property.
    ///
    /// # Safety
    /// Memory allocated by the method muste be [freed](free) after usage.
    pub fn alloc(name: &str, device: &str, type_: indigo_property_type) -> *mut indigo_property {
        let p = indigo_property::new(name, device, type_);
        let b = Box::new(p);
        Box::into_raw(b)
    }

    /// Drops the indigo_property and frees the memory used.
    ///
    /// # Safety
    /// This method cleans up memory referenced by the `*mut indigo_property` pointer.
    pub unsafe fn free(ptr: *mut indigo_property) {
        drop(Box::from_raw(ptr));
    }
     */
}

unsafe impl FamHeader for indigo_property {
    type Element = indigo_item;

    fn fam_len(&self) -> usize {
        self.allocated_count as usize
    }
}

// impl Drop for indigo_property {
//     fn drop(&mut self) {
//         if let Ok(item_layout) = Layout::array::<indigo_item>(self.allocated_count as usize) {
//             let ptr = self.items.as_ptr() as *mut u8;
//             unsafe { dealloc(ptr, item_layout) };
//         } else {
//             let p = buf_to_str(&self.name);
//             let d = buf_to_str(&self.device);
//             error!("could not deallocate items for property {p} on device {d}");
//             panic!("deallocation failure")
//         }
//     }
// }

impl indigo_item {
    pub fn new(
        name: &str,
        label: &str,
        hints: &str,
        value: indigo_value,
    ) -> Result<indigo_item, String> {
        let name = str_to_buf(name);
        let label = str_to_buf(label);
        let hints = str_to_buf(hints);

        Ok(indigo_item {
            name,
            label,
            hints,
            __bindgen_anon_1: value,
        })
    }

    pub fn text(name: &str, label: &str, hints: &str, text: &str) -> Result<indigo_item, String> {
        let name = str_to_buf(name);
        let label = str_to_buf(label);
        let hints = str_to_buf(hints);

        let cstring = CString::from_str(text).expect("valid C-string");
        let bytes = cstring.as_bytes_with_nul();

        let length = bytes.len();

        let text = if length > TEXT_ITEM_SIZE {
            // empty value and pointer
            let value = [0i8; TEXT_ITEM_SIZE];
            let bytes = Box::new(bytes);
            let long_value = Box::into_raw(bytes) as *mut i8;
            indigo_text {
                value,
                long_value,
                length: length as i64,
            }
        } else {
            // text value and null pointer
            let mut value = [0i8; TEXT_ITEM_SIZE];
            for i in 0..length {
                value[i] = bytes[i] as i8;
            }
            indigo_text {
                value,
                long_value: ptr::null_mut(),
                length: length as i64,
            }
        };

        Ok(indigo_item {
            name,
            label,
            hints,
            __bindgen_anon_1: indigo_value { text },
        })
    }

    pub fn number(
        name: &str,
        label: &str,
        hints: &str,
        number: indigo_number,
    ) -> Result<indigo_item, String> {
        let name = str_to_buf(name);
        let label = str_to_buf(label);
        let hints = str_to_buf(hints);

        Ok(indigo_item {
            name,
            label,
            hints,
            __bindgen_anon_1: indigo_value { number },
        })
    }
}

// /// ```
// /// use libindigo_sys::*;
// /// let items = [indigo_item::default(), indigo_item::default()];
// /// let arr = indigo_item_array::try_from(&items[..]).expect("indigo_item_array");
// /// let slice = unsafe { arr.as_slice(2) };
// /// assert_eq!(slice.len(), 2);
// /// ```
// impl TryFrom<&'_ [indigo_item]> for indigo_item_array {
//     type Error = LayoutError;

//     fn try_from(values: &'_ [indigo_item]) -> Result<Self, Self::Error> {
//         // https://users.rust-lang.org/t/memory-allocation-and-writing-to-incomplete-array-field-generated-by-bindgen
//         unsafe {
//             let len = values.len();
//             let item_layout = Layout::array::<indigo_item>(len)?;
//             let buffer = alloc(item_layout) as *const [indigo_item; 0];

//             let arr = buffer as *mut indigo_item_array;

//             // Write data
//             let slice = indigo_item_array::as_mut_slice(&mut *arr, len);
//             slice.copy_from_slice(values);

//             // Ok(*arr)
//             todo!()
//         }
//     }
// }

impl Default for indigo_item {
    fn default() -> Self {
        let text = indigo_text {
            value: [0i8; 512],
            long_value: ptr::null_mut(),
            length: 0,
        };
        Self {
            name: [0i8; 128],
            label: [0i8; 512],
            hints: [0i8; 512],
            __bindgen_anon_1: indigo_value { text },
        }
    }
}

/// convert an `i8` C string buffer of arbitrary length to a `&str` slice.
pub fn buf_to_str<const N: usize>(buf: &[c_char; N]) -> &str {
    let bytes = unsafe { slice::from_raw_parts(buf.as_ptr() as *const u8, buf.len()) };
    CStr::from_bytes_until_nul(&bytes[0..N])
        .expect("could not read CStr")
        .to_str()
        .expect("could not convert to UTF8 str")
}

pub fn str_to_buf<const N: usize>(s: &str) -> [c_char; N] {
    let s = CString::new(s).expect("a string without \\0 bytes");
    let mut buf = [0; N];
    let bytes = s.as_bytes_with_nul();
    for (i, b) in bytes.iter().enumerate() {
        buf[i] = *b as c_char;
    }
    buf
}

/// Returns `Some(&str)` if the `message` pointer is not `null``, else `None``
pub fn ptr_to_str<'a>(message: *const c_char) -> Option<&'a str> {
    if message.is_null() {
        None
    } else {
        let m = message;
        match unsafe { CStr::from_ptr(m) }.to_str() {
            Ok(s) => Some(s),
            Err(e) => {
                warn!("unsafe c-string to string conversion error: {e}");
                None
            }
        }
    }
}

pub fn copy_from_str<const N: usize>(mut target: [i8; N], source: &str) {
    let buf: [i8; N] = str_to_buf(source);
    target.copy_from_slice(&buf);
}

impl PartialEq for indigo_client {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for indigo_client {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Eq for indigo_client {}

unsafe impl Sync for indigo_client {}
unsafe impl Send for indigo_client {}

#[cfg(test)]
mod tests {
    use core::{ffi::CStr, ptr};

    use super::*;

    // #[test]
    // fn indigo_item_array_try_from_slice() {
    //     let items = [
    //         indigo_item::text("f1", "Fruit 1", "usually round and green", "Apple").expect("indigo text item"),
    //         indigo_item::text("f2", "Fruit 2", "usually oblong and yellow", "Banana").expect("indigo text item"),
    //     ];

    //     let arr = &indigo_item_array::try_from(&items[..]).expect("indigo_item_array");

    //     let slice = unsafe { indigo_item_array::as_slice(arr, 2) };
    //     assert_eq!(buf_to_str(&slice[0].name), "f1");
    //     assert_eq!(buf_to_str(&slice[1].name), "f2");

    //     let item_layout = Layout::array::<indigo_item>(items.len()).expect("layout for array of indigo_item");
    //     let ptr = arr.as_ptr() as *mut u8;
    //     unsafe { dealloc(ptr, item_layout) };
    // }

    unsafe extern "C" fn my_attach(client: *mut indigo_client) -> indigo_result {
        let c_msg = std::ffi::CString::new("attached to INDIGO bus...").unwrap();
        unsafe {
            indigo_log(c_msg.as_ptr());
            println!("attach client callback!");
            // Request property definitions
            indigo_enumerate_properties(client, &raw mut INDIGO_ALL_PROPERTIES);
        }
        indigo_result::INDIGO_OK
    }

    unsafe extern "C" fn my_define_property(
        _client: *mut indigo_client,
        _device: *mut indigo_device,
        property: *mut indigo_property,
        message: *const i8,
    ) -> indigo_result {
        // let d = CStr::from_bytes_until_nul(&(*device).name.map(|i| i as u8)).unwrap().to_str().unwrap();
        if !property.is_null() {
            // let id = [0 as c_char; 256];
            // let rust_id = unsafe { CStr::from_ptr(id.as_ptr()) };
            // let rust_id = rust_id.to_owned();
            // println!("{:?}", rust_id);

            let ptr = (*property).name.as_ptr();
            let p = unsafe { CStr::from_ptr(ptr) };
            let p = p.to_owned();
            print!("property: {:?}", p);
        }

        if message.is_null() {
            println!("; null message");
        } else {
            let m = CStr::from_ptr(message).to_str().unwrap();
            println!("; message: {m}");
        }

        // println!("d: {d}; p: {p}; m: {m}");

        indigo_result::INDIGO_OK
    }

    unsafe extern "C" fn my_update_property(
        _client: *mut indigo_client,
        _device: *mut indigo_device,
        _property: *mut indigo_property,
        _message: *const i8,
    ) -> indigo_result {
        println!("update property callback!");
        // do something useful here ;)
        indigo_result::INDIGO_OK
    }

    unsafe extern "C" fn my_detach(_client: *mut indigo_client) -> indigo_result {
        let c_msg = std::ffi::CString::new("detached from INDIGO bus").unwrap();
        unsafe { indigo_log(c_msg.as_ptr()) };
        indigo_result::INDIGO_OK
    }

    fn map_indigo_result(code: indigo_result) -> Result<(), indigo_result> {
        if code == indigo_result::INDIGO_OK {
            Ok(())
        } else {
            Err(code)
        }
    }

    #[test]
    fn client_test() -> Result<(), indigo_result> {
        // let name = std::ffi::CString::new("MyClient").unwrap();
        // let bytes: [i8; name.len() + 1] = name.as_bytes_with_nul().iter().map(|b| *b as i8).collect();
        // let buf = [0i8;128];
        // buf[..bytes.len()].copy_from_slice(bytes);

        let name = b"MyClient\0".map(|b| b as i8);
        let mut buf = [0i8; 128];
        buf[..name.len()].copy_from_slice(&name);

        let mut indigo_client = indigo_client {
            name: buf,                             // client name
            is_remote: false,                      // is this a remote client "no" - this is us
            client_context: ptr::null_mut(),       // we do not have client specific data
            last_result: indigo_result::INDIGO_OK, // result of last bus operation
            // - we just initialize it with ok
            version: indigo_version::INDIGO_VERSION_CURRENT, // the client speaks current indigo version
            enable_blob_mode_records: ptr::null_mut(), // BLOB mode records -> Set this to NULL
            attach: Some(my_attach),
            define_property: Some(my_define_property),
            update_property: Some(my_update_property),
            delete_property: None,
            send_message: None,
            detach: Some(my_detach),
        };
        unsafe {
            map_indigo_result(indigo_start())?;

            /* We want to see debug messages on the screen */
            indigo_set_log_level(indigo_log_levels::INDIGO_LOG_DEBUG);
            map_indigo_result(indigo_attach_client(std::ptr::addr_of_mut!(indigo_client)))?;

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

            map_indigo_result(indigo_connect_server(
                server_name.as_ptr(),
                host.as_ptr(),
                7624,
                srv_ptr_ptr,
            ))?;

            /* We can do whatever we want here while we are waiting for
            the client to complete. For example we can call some GUI
            framework main loop etc...
            Instead we will just sleep for 10 seconds.
            */
            indigo_usleep(10 * ONE_SECOND_DELAY);

            map_indigo_result(indigo_disconnect_server(srv_ptr))?;
            map_indigo_result(indigo_detach_client(ptr::addr_of_mut!(indigo_client)))?;
            map_indigo_result(indigo_stop())
        }
    }
}
