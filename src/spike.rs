// #![feature(ptr_metadata,layout_for_ptr)]

// use std::ffi::{CStr, CString};

// // -- bindgen code, simplified for clarity -------------------------------------
// #[repr(C)]
// pub struct Property<FAM: ?Sized = [Item; 0]> {
//     ///< number of allocated property items
//     pub allocated_count: ::core::ffi::c_int,
//     ///< number of used property items
//     pub count: ::core::ffi::c_int,
//     ///< property items
//     pub items: FAM,
// }
// impl Property<[Item]> {
//     pub fn layout(len: usize) -> ::core::alloc::Layout {
//         unsafe {
//             let p: *const Self = ::core::ptr::from_raw_parts(
//                 ::core::ptr::null::<()>(),
//                 len,
//             );
//             ::core::alloc::Layout::for_value_raw(p)
//         }
//     }
//     #[inline]
//     pub fn fixed(&self) -> (&Property<[Item; 0]>, usize) {
//         unsafe {
//             let (ptr, len) = (self as *const Self).to_raw_parts();
//             (&*(ptr as *const Property<[Item; 0]>), len)
//         }
//     }
//     #[inline]
//     pub fn fixed_mut(&mut self) -> (&mut Property<[Item; 0]>, usize) {
//         unsafe {
//             let (ptr, len) = (self as *mut Self).to_raw_parts();
//             (&mut *(ptr as *mut Property<[Item; 0]>), len)
//         }
//     }
// }
// impl Property<[Item; 0]> {
//     /// Convert a sized prefix to an unsized structure with the given length.
//     ///
//     /// SAFETY: Underlying storage is initialized up to at least `len` elements.
//     pub unsafe fn flex_ref(&self, len: usize) -> &Property<[Item]> {
//         Self::flex_ptr(self, len)
//     }
//     /// Convert a mutable sized prefix to an unsized structure with the given length.
//     ///
//     /// SAFETY: Underlying storage is initialized up to at least `len` elements.
//     #[inline]
//     pub unsafe fn flex_ref_mut(
//         &mut self,
//         len: usize,
//     ) -> &mut Property<[Item]> {
//         Self::flex_ptr_mut(self, len).assume_init()
//     }
//     /// Construct DST variant from a pointer and a size.
//     ///
//     /// NOTE: lifetime of returned reference is not tied to any underlying storage.
//     /// SAFETY: `ptr` is valid. Underlying storage is fully initialized up to at least `len` elements.
//     #[inline]
//     pub unsafe fn flex_ptr<'unbounded>(
//         ptr: *const Self,
//         len: usize,
//     ) -> &'unbounded Property<[Item]> {
//         &*::core::ptr::from_raw_parts(ptr as *const (), len)
//     }
//     /// Construct mutable DST variant from a pointer and a
//     /// size. The returned `&mut` reference is initialized
//     /// pointing to memory referenced by `ptr`, but there's
//     /// no requirement that that memory be initialized.
//     ///
//     /// NOTE: lifetime of returned reference is not tied to any underlying storage.
//     /// SAFETY: `ptr` is valid. Underlying storage has space for at least `len` elements.
//     #[inline]
//     pub unsafe fn flex_ptr_mut<'unbounded>(
//         ptr: *mut Self,
//         len: usize,
//     ) -> ::core::mem::MaybeUninit<&'unbounded mut Property<[Item]>> {
//         let mut uninit = ::core::mem::MaybeUninit::<
//             &mut Property<[Item]>,
//         >::uninit();
//         (uninit.as_mut_ptr() as *mut *mut Property<[Item]>)
//             .write(::core::ptr::from_raw_parts_mut(ptr as *mut (), len));
//         uninit
//     }
// }
// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// pub struct Item {
//     buf: [u8;256],
// }


// // -- wrapper code for working with the bindgen code ---------------------------

// impl Property {
//     pub fn new(items: [Item]) -> Property<[Item]> {
//         todo!("this is what I can't figure out")
//     }
// }

// impl Item {
//     pub fn new(name: &str) -> Item {
//         let name = CString::new(name).expect("valid C-String");
//         let name = name.as_bytes_with_nul();
//         let mut buf = [0u8;256];
//         let len = name.len();
//         buf[..len].copy_from_slice(&name[..len]);
//         Item { buf }
//     }
//     pub fn name(&self) -> &str {
//         CStr::from_bytes_until_nul(&self.buf)
//             .expect("valid C-string")
//             .to_str()
//             .expect("valid R-string")

//     }
// }

// #[test]
// fn test() {
//     use std::alloc::{alloc, dealloc, Layout};
//     unsafe {
//         let items = &[Item::new("apple"), Item::new("banana")];
//         let property = Property::new(items);

//         let slice = property.items;
//         assert_eq!(slice[0].name(), "apple");
//         assert_eq!(slice[1].name(), "banana");
//     }
// }