#![feature(maybe_uninit_uninit_array, maybe_uninit_extra, maybe_uninit_slice)]

pub mod context;
pub mod error;
pub mod instance;
pub mod system;
pub mod types;
pub mod utils;
pub mod value;

// use std::ffi::{CStr, CString};
// use std::fmt;
// use std::marker::PhantomData;
// use std::mem;
// use std::path::Path;
// use std::ptr;
// use wasmedge_sys::ffi as we_ffi;

// pub type WasmEdgeValue = we_ffi::WasmEdge_Value;

// #[derive(Debug)]
// pub struct WasmEdgeError {
//     code: u32,
//     message: String,
// }
// impl fmt::Display for WasmEdgeError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
//         self.message.fmt(f)
//     }
// }
// impl std::error::Error for WasmEdgeError {}
// impl From<String> for WasmEdgeError {
//     fn from(message: String) -> WasmEdgeError {
//         WasmEdgeError { code: 49, message }
//     }
// }
// impl From<std::ffi::NulError> for WasmEdgeError {
//     fn from(e: std::ffi::NulError) -> WasmEdgeError {
//         WasmEdgeError {
//             code: 49,
//             message: e.to_string(),
//         }
//     }
// }

// pub type WasmEdgeResult<T> = Result<T, WasmEdgeError>;

// fn check(result: we_ffi::WasmEdge_Result) -> WasmEdgeResult<u32> {
//     unsafe {
//         let code = we_ffi::WasmEdge_ResultGetCode(result);
//         if we_ffi::WasmEdge_ResultOK(result) {
//             return Ok(code);
//         }
//         let message = CStr::from_ptr(we_ffi::WasmEdge_ResultGetMessage(result))
//             .to_string_lossy()
//             .into_owned();
//         Err(WasmEdgeError { code, message })
//     }
// }

// pub fn version() -> String {
//     let c_buf = unsafe { we_ffi::WasmEdge_VersionGet() };
//     let slice: &CStr = unsafe { CStr::from_ptr(c_buf) };
//     let str_slice: &str = slice.to_str().unwrap();
//     str_slice.to_owned()
// }

// pub fn major_version() -> usize {
//     let major = unsafe { we_ffi::WasmEdge_VersionGetMajor() };
//     major as usize
// }

// pub fn minor_version() -> usize {
//     let minor = unsafe { we_ffi::WasmEdge_VersionGetMinor() };
//     minor as usize
// }

// pub fn version_patch() -> usize {
//     let patch = unsafe { we_ffi::WasmEdge_VersionGetPatch() };
//     patch as usize
// }

// pub type WasmEdgeProposal = we_ffi::WasmEdge_Proposal;

// pub struct ConfigureContext {
//     raw: *mut we_ffi::WasmEdge_ConfigureContext,
// }
// impl ConfigureContext {
//     pub fn create() -> ConfigureContext {
//         ConfigureContext {
//             raw: unsafe { we_ffi::WasmEdge_ConfigureCreate() },
//         }
//     }

//     pub fn add_host_registration(&mut self, host: HostRegistration) {
//         unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(self.raw, host) }
//     }

//     pub fn add_proposal(&mut self, prop: WasmEdgeProposal) {
//         unsafe { we_ffi::WasmEdge_ConfigureAddProposal(self.raw, prop) }
//     }
// }
// impl Drop for ConfigureContext {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_ConfigureDelete(self.raw) }
//         }
//     }
// }

// pub struct StoreContext<'vm> {
//     raw: *mut we_ffi::WasmEdge_StoreContext,
//     _marker: PhantomData<&'vm VMContext>,
// }
// impl<'vm> StoreContext<'vm> {
//     pub fn create() -> StoreContext<'vm> {
//         StoreContext {
//             raw: unsafe { we_ffi::WasmEdge_StoreCreate() },
//             _marker: PhantomData,
//         }
//     }

//     pub fn find_memory(&self, mem_name: &str) -> MemoryInstanceContext {
//         let mem_name = WasmEdgeString::from_str(mem_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
//         let mem = unsafe { we_ffi::WasmEdge_StoreFindMemory(self.raw, mem_name.raw) };
//         MemoryInstanceContext {
//             raw: mem,
//             _marker: PhantomData,
//         }
//     }

//     pub fn find_memory_registered(&self, mod_name: &str, mem_name: &str) -> MemoryInstanceContext {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         let mem_name = WasmEdgeString::from_str(mem_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
//         let mem = unsafe {
//             we_ffi::WasmEdge_StoreFindMemoryRegistered(self.raw, mod_name.raw, mem_name.raw)
//         };
//         MemoryInstanceContext {
//             raw: mem,
//             _marker: PhantomData,
//         }
//     }

//     pub fn list_function_len(&self) -> usize {
//         unsafe { we_ffi::WasmEdge_StoreListFunctionLength(self.raw) as usize }
//     }

//     pub fn list_function_registered_len(&self, mod_name: &str) -> usize {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         unsafe {
//             we_ffi::WasmEdge_StoreListFunctionRegisteredLength(self.raw, mod_name.raw) as usize
//         }
//     }

//     pub fn list_table_len(&self) -> usize {
//         unsafe { we_ffi::WasmEdge_StoreListTableLength(self.raw) as usize }
//     }

//     pub fn list_table_registered_len(&self, mod_name: &str) -> usize {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         unsafe { we_ffi::WasmEdge_StoreListTableRegisteredLength(self.raw, mod_name.raw) as usize }
//     }

//     pub fn list_global_len(&self) -> usize {
//         unsafe { we_ffi::WasmEdge_StoreListGlobalLength(self.raw) as usize }
//     }

//     pub fn list_global_registered_len(&self, mod_name: &str) -> usize {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         unsafe { we_ffi::WasmEdge_StoreListGlobalRegisteredLength(self.raw, mod_name.raw) as usize }
//     }

//     pub fn list_memory_len(&self) -> usize {
//         let len = unsafe { we_ffi::WasmEdge_StoreListMemoryLength(self.raw) };
//         len as usize
//     }

//     pub fn list_memory(
//         &self,
//         buf: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
//     ) -> (usize, Vec<String>) {
//         let len = unsafe {
//             we_ffi::WasmEdge_StoreListMemory(
//                 self.raw,
//                 buf.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
//                 buf.len() as u32,
//             )
//         };
//         let s_vec = unsafe { mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]) };
//         let mut names = vec![];
//         for s in s_vec {
//             let str = WasmEdgeString { raw: *s };
//             let cow = str.to_string_lossy();
//             names.push(cow.into_owned())
//         }
//         (len as usize, names)
//     }

//     pub fn list_memory_registered_len(&self, mod_name: &str) -> usize {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         let len =
//             unsafe { we_ffi::WasmEdge_StoreListMemoryRegisteredLength(self.raw, mod_name.raw) };
//         len as usize
//     }

//     pub fn list_memory_registered(
//         &self,
//         mod_name: &str,
//         buf: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
//     ) -> (usize, Vec<String>) {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         let len = unsafe {
//             we_ffi::WasmEdge_StoreListMemoryRegistered(
//                 self.raw,
//                 mod_name.raw,
//                 buf.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
//                 buf.len() as u32,
//             )
//         };
//         let s_vec = unsafe { mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]) };
//         let mut names = vec![];
//         for s in s_vec {
//             let str = WasmEdgeString { raw: *s };
//             let cow = str.to_string_lossy();
//             names.push(cow.into_owned())
//         }
//         (len as usize, names)
//     }

//     pub fn list_module_len(&self) -> usize {
//         let len = unsafe { we_ffi::WasmEdge_StoreListModuleLength(self.raw) };
//         len as usize
//     }
// }
// impl<'vm> Drop for StoreContext<'vm> {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_StoreDelete(self.raw) }
//         }
//     }
// }

// pub type HostRegistration = we_ffi::WasmEdge_HostRegistration;

// pub fn add_host_registration(ctx: &mut ConfigureContext, host: HostRegistration) {
//     unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(ctx.raw, host) }
// }

// pub fn remove_host_registration(ctx: &mut ConfigureContext, host: HostRegistration) {
//     unsafe { we_ffi::WasmEdge_ConfigureRemoveHostRegistration(ctx.raw, host) }
// }

// pub fn has_host_registration(ctx: &ConfigureContext, host: HostRegistration) -> bool {
//     {
//         unsafe { we_ffi::WasmEdge_ConfigureHasHostRegistration(ctx.raw, host) }
//     }
// }

// pub struct VMContext {
//     raw: *mut we_ffi::WasmEdge_VMContext,
// }
// impl VMContext {
//     pub fn create(
//         conf_ctx: Option<&ConfigureContext>,
//         store_ctx: Option<&mut StoreContext>,
//     ) -> VMContext {
//         match (conf_ctx, store_ctx) {
//             (Some(conf_ctx), Some(store_ctx)) => VMContext {
//                 raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, store_ctx.raw) },
//             },
//             (Some(conf_ctx), None) => VMContext {
//                 raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, ptr::null_mut()) },
//             },
//             (None, Some(store_ctx)) => VMContext {
//                 raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), store_ctx.raw) },
//             },
//             (None, None) => VMContext {
//                 raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), ptr::null_mut()) },
//             },
//         }
//     }

//     pub fn register_module_from_import_object(
//         &mut self,
//         import_ctx: &ImportObjectContext,
//     ) -> WasmEdgeResult<u32> {
//         unsafe {
//             check(we_ffi::WasmEdge_VMRegisterModuleFromImport(
//                 self.raw,
//                 import_ctx.raw,
//             ))
//         }
//     }

//     pub fn register_module_from_file<P: AsRef<Path>>(
//         &mut self,
//         mod_name: &str,
//         path: P,
//     ) -> WasmEdgeResult<u32> {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         let path = path_to_cstring(path.as_ref())?;
//         unsafe {
//             check(we_ffi::WasmEdge_VMRegisterModuleFromFile(
//                 self.raw,
//                 mod_name.raw,
//                 path.as_ptr(),
//             ))
//         }
//     }

//     pub fn run_wasm_from_file<'vm, P: AsRef<Path>>(
//         &mut self,
//         path: P,
//         func_name: &str,
//         params: &[WasmEdgeValue],
//         buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
//     ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
//         let path = path_to_cstring(path.as_ref())?;
//         unsafe {
//             let func_name = WasmEdgeString::from_str(func_name)
//                 .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

//             let result = check(we_ffi::WasmEdge_VMRunWasmFromFile(
//                 self.raw,
//                 path.as_ptr(),
//                 func_name.raw,
//                 params.as_ptr() as *const WasmEdgeValue,
//                 params.len() as u32,
//                 buf.as_mut_ptr() as *mut WasmEdgeValue,
//                 buf.len() as u32,
//             ));

//             match result {
//                 Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])),
//                 Err(err) => Err(err),
//             }
//         }
//     }

//     pub fn run_wasm_from_buffer<'vm>(
//         &mut self,
//         buf: &[u8],
//         func_name: &str,
//         params: &[WasmEdgeValue],
//         returns: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
//     ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
//         let func_name = WasmEdgeString::from_str(func_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

//         unsafe {
//             let result = check(we_ffi::WasmEdge_VMRunWasmFromBuffer(
//                 self.raw,
//                 buf.as_ptr(),
//                 buf.len() as u32,
//                 func_name.raw,
//                 params.as_ptr() as *const WasmEdgeValue,
//                 params.len() as u32,
//                 returns.as_mut_ptr() as *mut WasmEdgeValue,
//                 returns.len() as u32,
//             ));

//             match result {
//                 Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(
//                     &returns[..returns.len()],
//                 )),
//                 Err(err) => Err(err),
//             }
//         }
//     }

//     pub fn function_type(&self, func_name: &str) -> Option<FunctionTypeContext> {
//         let func_name = WasmEdgeString::from_str(func_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
//         let result = unsafe { we_ffi::WasmEdge_VMGetFunctionType(self.raw, func_name.raw) };
//         if result.is_null() {
//             return None;
//         }

//         Some(FunctionTypeContext { raw: result })
//     }

//     pub fn function_type_registered(
//         &self,
//         mod_name: &str,
//         func_name: &str,
//     ) -> Option<FunctionTypeContext> {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         let func_name = WasmEdgeString::from_str(func_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
//         let result = unsafe {
//             we_ffi::WasmEdge_VMGetFunctionTypeRegistered(self.raw, mod_name.raw, func_name.raw)
//         };
//         if result.is_null() {
//             return None;
//         }

//         Some(FunctionTypeContext { raw: result })
//     }

//     pub fn function_list_len(&self) -> usize {
//         unsafe { we_ffi::WasmEdge_VMGetFunctionListLength(self.raw) as usize }
//     }

//     pub fn importobject_module(&self, reg: HostRegistration) -> Option<ImportObjectContext> {
//         let raw = unsafe { we_ffi::WasmEdge_VMGetImportModuleContext(self.raw, reg) };
//         if raw.is_null() {
//             return None;
//         } else {
//         }
//         match raw.is_null() {
//             true => None,
//             false => Some(ImportObjectContext {
//                 raw,
//                 _marker: PhantomData,
//             }),
//         }
//     }

//     pub fn execute_registered<'vm>(
//         &self,
//         mod_name: &str,
//         func_name: &str,
//         params: &[WasmEdgeValue],
//         buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
//     ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
//         unsafe {
//             let mod_name = WasmEdgeString::from_str(mod_name)
//                 .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//             let func_name = WasmEdgeString::from_str(func_name)
//                 .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

//             let result = check(we_ffi::WasmEdge_VMExecuteRegistered(
//                 self.raw,
//                 mod_name.raw,
//                 func_name.raw,
//                 params.as_ptr() as *const WasmEdgeValue,
//                 params.len() as u32,
//                 buf.as_mut_ptr() as *mut WasmEdgeValue,
//                 buf.len() as u32,
//             ));

//             match result {
//                 Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])),
//                 Err(err) => Err(err),
//             }
//         }
//     }

//     pub fn store_context(&self) -> StoreContext {
//         let store_ctx = unsafe { we_ffi::WasmEdge_VMGetStoreContext(self.raw) };
//         StoreContext {
//             raw: store_ctx,
//             _marker: PhantomData,
//         }
//     }
// }
// impl Drop for VMContext {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_VMDelete(self.raw) }
//         }
//     }
// }

// #[cfg(unix)]
// fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
//     use std::os::unix::ffi::OsStrExt;

//     Ok(CString::new(path.as_os_str().as_bytes())?)
// }

// #[cfg(windows)]
// fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
//     match path.to_str() {
//         Some(s) => Ok(CString::new(s)?),
//         None => {
//             let message = format!("Couldn't convert path '{}' to UTF-8", path.display());
//             Err(message.into())
//         }
//     }
// }

// pub fn WasmEdgeValueGenI32(val: i32) -> WasmEdgeValue {
//     unsafe { we_ffi::WasmEdge_ValueGenI32(val) }
// }

// pub fn WasmEdgeValueGenI64(val: i64) -> WasmEdgeValue {
//     unsafe { we_ffi::WasmEdge_ValueGenI64(val) }
// }

// pub fn WasmEdgeValueGenExternRef(ptr: *mut ::std::os::raw::c_void) -> WasmEdgeValue {
//     unsafe { we_ffi::WasmEdge_ValueGenExternRef(ptr) }
// }

// pub fn WasmEdgeValueGetI32(val: WasmEdgeValue) -> i32 {
//     unsafe { we_ffi::WasmEdge_ValueGetI32(val) }
// }

// pub fn WasmEdgeValueGetI64(val: WasmEdgeValue) -> i64 {
//     unsafe { we_ffi::WasmEdge_ValueGetI64(val) }
// }

// pub fn WasmEdgeValueGetExternRef(val: WasmEdgeValue) -> *mut ::std::os::raw::c_void {
//     unsafe { we_ffi::WasmEdge_ValueGetExternRef(val) }
// }

// pub fn vm_run_wasm_from_file<'vm, P: AsRef<Path>>(
//     ctx: &'vm mut VMContext,
//     path: P,
//     func_name: &str,
//     params: &[WasmEdgeValue],
//     buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
// ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
//     let path = path_to_cstring(path.as_ref())?;
//     unsafe {
//         let funcname = we_ffi::WasmEdge_StringCreateByCString(CString::new(func_name)?.as_ptr());

//         let result = check(we_ffi::WasmEdge_VMRunWasmFromFile(
//             ctx.raw,
//             path.as_ptr(),
//             funcname,
//             params.as_ptr() as *const WasmEdgeValue,
//             params.len() as u32,
//             buf.as_mut_ptr() as *mut WasmEdgeValue,
//             buf.len() as u32,
//         ));

//         we_ffi::WasmEdge_StringDelete(funcname);
//         // mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]);

//         match result {
//             Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])),
//             Err(err) => Err(err),
//         }
//     }
// }

// pub type WasmEdgeValType = we_ffi::WasmEdge_ValType;

// pub struct FunctionTypeContext {
//     raw: *mut we_ffi::WasmEdge_FunctionTypeContext,
// }
// impl FunctionTypeContext {
//     pub fn create(
//         params: Option<&[WasmEdgeValType]>,
//         returns: &[WasmEdgeValType],
//     ) -> FunctionTypeContext {
//         let (param_list, param_len) = match params {
//             Some(params) => (params.as_ptr(), params.len()),
//             None => (ptr::null(), 0),
//         };
//         FunctionTypeContext {
//             raw: unsafe {
//                 we_ffi::WasmEdge_FunctionTypeCreate(
//                     param_list,
//                     param_len as u32,
//                     returns.as_ptr(),
//                     returns.len() as u32,
//                 )
//             },
//         }
//     }

//     pub fn parameters_len(&self) -> usize {
//         unsafe { we_ffi::WasmEdge_FunctionTypeGetParametersLength(self.raw) as usize }
//     }

//     pub fn parameters(&self, list: &mut [WasmEdgeValType]) -> usize {
//         unsafe {
//             we_ffi::WasmEdge_FunctionTypeGetParameters(
//                 self.raw,
//                 list.as_mut_ptr(),
//                 list.len() as u32,
//             ) as usize
//         }
//     }

//     pub fn returns_len(&self) -> usize {
//         unsafe { we_ffi::WasmEdge_FunctionTypeGetReturnsLength(self.raw) as usize }
//     }

//     pub fn returns<'val>(
//         &self,
//         list: &'val mut [mem::MaybeUninit<WasmEdgeValType>],
//     ) -> (usize, &'val [WasmEdgeValType]) {
//         let length = unsafe {
//             we_ffi::WasmEdge_FunctionTypeGetReturns(
//                 self.raw,
//                 list.as_mut_ptr() as *mut WasmEdgeValType,
//                 list.len() as u32,
//             ) as usize
//         };

//         (length, unsafe {
//             mem::MaybeUninit::slice_assume_init_ref(&list[..list.len()])
//         })
//     }
// }
// impl Drop for FunctionTypeContext {
//     fn drop(&mut self) {
//         unsafe { we_ffi::WasmEdge_FunctionTypeDelete(self.raw) }
//     }
// }

// pub struct HostFunctionContext {
//     raw: *mut we_ffi::WasmEdge_HostFunctionContext,
// }
// impl HostFunctionContext {
//     pub fn create(
//         func_type: &FunctionTypeContext,
//         host_func: we_ffi::WasmEdge_HostFunc_t,
//         cost: u64,
//     ) -> Option<HostFunctionContext> {
//         let raw = unsafe { we_ffi::WasmEdge_HostFunctionCreate(func_type.raw, host_func, cost) };
//         println!("raw: {:?}", raw);
//         match raw.is_null() {
//             true => None,
//             false => Some(HostFunctionContext { raw }),
//         }
//     }

//     pub fn create_binding(
//         func_type: &FunctionTypeContext,
//         wrap_func: we_ffi::WasmEdge_WrapFunc_t,
//         binding: *mut std::os::raw::c_void,
//         cost: u64,
//     ) -> HostFunctionContext {
//         HostFunctionContext {
//             raw: unsafe {
//                 we_ffi::WasmEdge_HostFunctionCreateBinding(func_type.raw, wrap_func, binding, cost)
//             },
//         }
//     }
// }
// impl Drop for HostFunctionContext {
//     fn drop(&mut self) {
//         unsafe { we_ffi::WasmEdge_HostFunctionDelete(self.raw) }
//     }
// }

// #[derive(Clone)]
// pub struct ImportObjectContext<'vm> {
//     raw: *mut we_ffi::WasmEdge_ImportObjectContext,
//     _marker: PhantomData<&'vm VMContext>,
// }
// impl<'vm> ImportObjectContext<'vm> {
//     pub fn create(mod_name: &str, data: *mut std::os::raw::c_void) -> Option<ImportObjectContext> {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         let raw = unsafe { we_ffi::WasmEdge_ImportObjectCreate(mod_name.raw, data) };
//         match raw.is_null() {
//             true => None,
//             false => Some(ImportObjectContext {
//                 raw,
//                 _marker: PhantomData,
//             }),
//         }
//     }

//     pub fn create_tensorflow_import_object() -> ImportObjectContext<'vm> {
//         ImportObjectContext {
//             raw: unsafe { we_ffi::WasmEdge_Tensorflow_ImportObjectCreate() },
//             _marker: PhantomData,
//         }
//     }

//     pub fn create_tensorflowlite_import_object() -> ImportObjectContext<'vm> {
//         ImportObjectContext {
//             raw: unsafe { we_ffi::WasmEdge_TensorflowLite_ImportObjectCreate() },
//             _marker: PhantomData,
//         }
//     }

//     pub fn init_wasi(&self, args: &[&str], envs: &[&str], dirs: &[&str], preopens: &[&str]) {
//         let mut cargs = vec![];
//         for &arg in args.iter() {
//             cargs.push(arg.as_ptr() as *const _);
//         }
//         let mut cenvs = vec![];
//         for &env in envs.iter() {
//             cenvs.push(env.as_ptr() as *const _);
//         }
//         let mut cdirs = vec![];
//         for &dir in dirs.iter() {
//             cdirs.push(dir.as_ptr() as *const _);
//         }
//         let mut cpreopens = vec![];
//         for &pre in preopens.iter() {
//             cpreopens.push(pre.as_ptr() as *const _);
//         }
//         unsafe {
//             we_ffi::WasmEdge_ImportObjectInitWASI(
//                 self.raw,
//                 cargs.as_ptr(),
//                 cargs.len() as u32,
//                 cenvs.as_ptr(),
//                 cenvs.len() as u32,
//                 cdirs.as_ptr(),
//                 cdirs.len() as u32,
//                 cpreopens.as_ptr(),
//                 cpreopens.len() as u32,
//             )
//         };
//     }

//     pub fn add_host_function(&mut self, name: &str, host_func_ctx: &mut HostFunctionContext) {
//         let name = WasmEdgeString::from_str(name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
//         unsafe {
//             we_ffi::WasmEdge_ImportObjectAddHostFunction(self.raw, name.raw, host_func_ctx.raw)
//         }
//     }

//     pub fn add_table(&mut self, name: &str, table_ctx: &mut TableInstanceContext) {
//         let name = WasmEdgeString::from_str(name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
//         unsafe { we_ffi::WasmEdge_ImportObjectAddTable(self.raw, name.raw, table_ctx.raw) }
//     }

//     pub fn add_memory(&mut self, name: &str, mem_ctx: &mut MemoryInstanceContext) {
//         let name = WasmEdgeString::from_str(name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
//         unsafe {
//             we_ffi::WasmEdge_ImportObjectAddMemory(self.raw, name.raw, mem_ctx.raw);
//         }
//     }

//     pub fn add_global(&mut self, name: &str, global_ctx: &mut GlobalInstanceContext) {
//         let name = WasmEdgeString::from_str(name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
//         unsafe {
//             we_ffi::WasmEdge_ImportObjectAddGlobal(self.raw, name.raw, global_ctx.raw);
//         }
//     }
// }
// impl<'vm> Drop for ImportObjectContext<'vm> {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_ImportObjectDelete(self.raw) }
//         }
//     }
// }

// pub struct WasmEdgeString {
//     raw: we_ffi::WasmEdge_String,
// }
// impl WasmEdgeString {
//     pub fn from_str(s: &str) -> WasmEdgeResult<WasmEdgeString> {
//         let cstring = CString::new(s)?;
//         Ok(WasmEdgeString {
//             raw: unsafe { we_ffi::WasmEdge_StringCreateByCString(cstring.as_ptr()) },
//         })
//     }

//     pub fn from_buffer(buf: &[i8]) -> WasmEdgeString {
//         WasmEdgeString {
//             raw: unsafe { we_ffi::WasmEdge_StringCreateByBuffer(buf.as_ptr(), buf.len() as u32) },
//         }
//     }

//     pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
//         let cstr = unsafe { CStr::from_ptr(self.raw.Buf) };
//         cstr.to_string_lossy()
//     }

//     pub fn into_raw(&self) -> we_ffi::WasmEdge_String {
//         self.raw
//     }
// }
// impl Drop for WasmEdgeString {
//     fn drop(&mut self) {
//         unsafe { we_ffi::WasmEdge_StringDelete(self.raw) }
//     }
// }

// pub type WasmEdgeLimit = we_ffi::WasmEdge_Limit;
// pub type WasmEdgeRefType = we_ffi::WasmEdge_RefType;
// pub type WasmEdgeMutability = we_ffi::WasmEdge_Mutability;

// pub struct TableInstanceContext {
//     raw: *mut we_ffi::WasmEdge_TableInstanceContext,
// }
// impl TableInstanceContext {
//     pub fn create(ref_type: WasmEdgeRefType, limit: WasmEdgeLimit) -> TableInstanceContext {
//         TableInstanceContext {
//             raw: unsafe { we_ffi::WasmEdge_TableInstanceCreate(ref_type, limit) },
//         }
//     }
// }
// impl Drop for TableInstanceContext {
//     fn drop(&mut self) {
//         unsafe { we_ffi::WasmEdge_TableInstanceDelete(self.raw) }
//     }
// }

// pub struct MemoryInstanceContext<'store, 'vm> {
//     raw: *mut we_ffi::WasmEdge_MemoryInstanceContext,
//     _marker: PhantomData<&'store StoreContext<'vm>>,
// }
// impl<'store, 'vm> MemoryInstanceContext<'store, 'vm> {
//     pub fn create(limit: WasmEdgeLimit) -> Self {
//         MemoryInstanceContext {
//             raw: unsafe { we_ffi::WasmEdge_MemoryInstanceCreate(limit) },
//             _marker: PhantomData,
//         }
//     }
// }
// impl<'store, 'vm> Drop for MemoryInstanceContext<'store, 'vm> {
//     fn drop(&mut self) {
//         unsafe { we_ffi::WasmEdge_MemoryInstanceDelete(self.raw) }
//     }
// }

// pub struct GlobalInstanceContext {
//     raw: *mut we_ffi::WasmEdge_GlobalInstanceContext,
// }
// impl GlobalInstanceContext {
//     pub fn create(value: WasmEdgeValue, mutable: WasmEdgeMutability) -> Self {
//         GlobalInstanceContext {
//             raw: unsafe { we_ffi::WasmEdge_GlobalInstanceCreate(value, mutable) },
//         }
//     }
// }
// impl Drop for GlobalInstanceContext {
//     fn drop(&mut self) {
//         unsafe { we_ffi::WasmEdge_GlobalInstanceDelete(self.raw) }
//     }
// }

// pub struct ASTModuleContext {
//     raw: *mut we_ffi::WasmEdge_ASTModuleContext,
// }
// impl ASTModuleContext {
//     pub fn into_raw(&mut self) -> *mut we_ffi::WasmEdge_ASTModuleContext {
//         self.raw
//     }
// }
// impl Drop for ASTModuleContext {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_ASTModuleDelete(self.raw) }
//         }
//     }
// }
// impl Default for ASTModuleContext {
//     fn default() -> Self {
//         ASTModuleContext {
//             raw: ptr::null_mut(),
//         }
//     }
// }

// pub struct LoaderContext {
//     raw: *mut we_ffi::WasmEdge_LoaderContext,
// }
// impl LoaderContext {
//     pub fn create(conf: &ConfigureContext) -> Option<Self> {
//         let raw = unsafe { we_ffi::WasmEdge_LoaderCreate(conf.raw) };
//         match raw.is_null() {
//             true => None,
//             false => Some(LoaderContext { raw }),
//         }
//     }

//     pub fn parse_from_file<P: AsRef<Path>>(&mut self, module: &mut ASTModuleContext, path: P) {
//         let path = path_to_cstring(path.as_ref()).unwrap();
//         unsafe { we_ffi::WasmEdge_LoaderParseFromFile(self.raw, &mut module.raw, path.as_ptr()) };
//     }
// }
// impl Drop for LoaderContext {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_LoaderDelete(self.raw) }
//         }
//     }
// }

// pub struct InterpreterContext {
//     raw: *mut we_ffi::WasmEdge_InterpreterContext,
// }
// impl InterpreterContext {
//     pub fn create(
//         conf: Option<&ConfigureContext>,
//         stat: Option<&mut StatisticsContext>,
//     ) -> Option<Self> {
//         let conf = match conf {
//             Some(conf) => conf.raw,
//             None => ptr::null(),
//         };
//         let stat = match stat {
//             Some(stat) => stat.raw,
//             None => ptr::null_mut(),
//         };
//         let raw = unsafe { we_ffi::WasmEdge_InterpreterCreate(conf, stat) };
//         match raw.is_null() {
//             true => None,
//             false => Some(InterpreterContext { raw }),
//         }
//     }

//     pub fn register_import_object_module(
//         &mut self,
//         store: &mut StoreContext,
//         imp_obj: *const we_ffi::WasmEdge_ImportObjectContext,
//     ) -> bool {
//         let res = unsafe {
//             check(we_ffi::WasmEdge_InterpreterRegisterImport(
//                 self.raw, store.raw, imp_obj,
//             ))
//         };
//         match res {
//             Err(_) => false,
//             Ok(_) => true,
//         }
//     }

//     pub fn register_ast_module(
//         &mut self,
//         store: &mut StoreContext,
//         ast_mod: &ASTModuleContext,
//         mod_name: &str,
//     ) -> bool {
//         let mod_name = WasmEdgeString::from_str(mod_name)
//             .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
//         let res = unsafe {
//             check(we_ffi::WasmEdge_InterpreterRegisterModule(
//                 self.raw,
//                 store.raw,
//                 ast_mod.raw,
//                 mod_name.raw,
//             ))
//         };
//         match res {
//             Err(_) => false,
//             Ok(_) => true,
//         }
//     }

//     pub fn instantiate(&mut self, store: &mut StoreContext, ast_mod: &ASTModuleContext) -> bool {
//         let res = unsafe {
//             check(we_ffi::WasmEdge_InterpreterInstantiate(
//                 self.raw,
//                 store.raw,
//                 ast_mod.raw,
//             ))
//         };
//         match res {
//             Err(_) => false,
//             Ok(_) => true,
//         }
//     }
// }
// impl Drop for InterpreterContext {
//     fn drop(&mut self) {
//         unsafe { we_ffi::WasmEdge_InterpreterDelete(self.raw) }
//     }
// }

// pub struct StatisticsContext {
//     raw: *mut we_ffi::WasmEdge_StatisticsContext,
// }
// impl StatisticsContext {
//     pub fn create() -> Option<Self> {
//         let raw = unsafe { we_ffi::WasmEdge_StatisticsCreate() };
//         match raw.is_null() {
//             true => None,
//             false => Some(StatisticsContext { raw }),
//         }
//     }
// }
// impl Drop for StatisticsContext {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_StatisticsDelete(self.raw) }
//         }
//     }
// }

// pub struct Validator {
//     raw: *mut we_ffi::WasmEdge_ValidatorContext,
// }
// impl Validator {
//     pub fn create(conf: &ConfigureContext) -> Option<Self> {
//         let raw = unsafe { we_ffi::WasmEdge_ValidatorCreate(conf.raw) };
//         match raw.is_null() {
//             true => None,
//             false => Some(Validator { raw }),
//         }
//     }

//     pub fn validate(&mut self, ast_mod: &ASTModuleContext) -> WasmEdgeResult<u32> {
//         unsafe { check(we_ffi::WasmEdge_ValidatorValidate(self.raw, ast_mod.raw)) }
//     }
// }
// impl Drop for Validator {
//     fn drop(&mut self) {
//         if !self.raw.is_null() {
//             unsafe { we_ffi::WasmEdge_ValidatorDelete(self.raw) }
//         }
//     }
// }
// #[cfg(test)]
// mod tests {
//     use super::*;

//     // use std::path::Path;
// }
