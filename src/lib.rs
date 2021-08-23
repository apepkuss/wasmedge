#![feature(maybe_uninit_uninit_array, maybe_uninit_extra, maybe_uninit_slice)]

use std::ffi::{CStr, CString};
use std::fmt;
use std::mem;
use std::path::Path;
use std::ptr;
use wasmedge_sys::ffi as we_ffi;

pub type WasmEdgeValue = we_ffi::WasmEdge_Value;

#[derive(Debug)]
pub struct WasmEdgeError {
    code: u32,
    message: String,
}
impl fmt::Display for WasmEdgeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        self.message.fmt(f)
    }
}
impl std::error::Error for WasmEdgeError {}
impl From<String> for WasmEdgeError {
    fn from(message: String) -> WasmEdgeError {
        WasmEdgeError { code: 49, message }
    }
}
impl From<std::ffi::NulError> for WasmEdgeError {
    fn from(e: std::ffi::NulError) -> WasmEdgeError {
        WasmEdgeError {
            code: 49,
            message: e.to_string(),
        }
    }
}

pub type WasmEdgeResult<T> = Result<T, WasmEdgeError>;

fn check(result: we_ffi::WasmEdge_Result) -> WasmEdgeResult<u32> {
    unsafe {
        let code = we_ffi::WasmEdge_ResultGetCode(result);
        if we_ffi::WasmEdge_ResultOK(result) {
            return Ok(code);
        }
        let message = CStr::from_ptr(we_ffi::WasmEdge_ResultGetMessage(result))
            .to_string_lossy()
            .into_owned();
        Err(WasmEdgeError { code, message })
    }
}

pub fn version() -> String {
    let c_buf = unsafe { we_ffi::WasmEdge_VersionGet() };
    let slice: &CStr = unsafe { CStr::from_ptr(c_buf) };
    let str_slice: &str = slice.to_str().unwrap();
    str_slice.to_owned()
}

pub fn major_version() -> usize {
    let major = unsafe { we_ffi::WasmEdge_VersionGetMajor() };
    major as usize
}

pub fn minor_version() -> usize {
    let minor = unsafe { we_ffi::WasmEdge_VersionGetMinor() };
    minor as usize
}

pub fn version_patch() -> usize {
    let patch = unsafe { we_ffi::WasmEdge_VersionGetPatch() };
    patch as usize
}

pub struct WasmEdgeConfigureContext {
    raw: *mut we_ffi::WasmEdge_ConfigureContext,
}
impl WasmEdgeConfigureContext {
    pub fn create() -> WasmEdgeConfigureContext {
        WasmEdgeConfigureContext {
            raw: unsafe { we_ffi::WasmEdge_ConfigureCreate() },
        }
    }

    pub fn add_host_registration(&mut self, host: WasmEdgeHostRegistration) {
        unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(self.raw, host) }
    }
}
impl Drop for WasmEdgeConfigureContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ConfigureDelete(self.raw) }
        }
    }
}

pub struct WasmEdgeStoreContext {
    raw: *mut we_ffi::WasmEdge_StoreContext,
}
impl WasmEdgeStoreContext {
    pub fn new() -> WasmEdgeStoreContext {
        WasmEdgeStoreContext {
            raw: unsafe { we_ffi::WasmEdge_StoreCreate() },
        }
    }
}
impl Drop for WasmEdgeStoreContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_StoreDelete(self.raw) }
        }
    }
}

pub type WasmEdgeHostRegistration = we_ffi::WasmEdge_HostRegistration;

pub fn add_host_registration(ctx: &mut WasmEdgeConfigureContext, host: WasmEdgeHostRegistration) {
    unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(ctx.raw, host) }
}

pub fn remove_host_registration(
    ctx: &mut WasmEdgeConfigureContext,
    host: WasmEdgeHostRegistration,
) {
    unsafe { we_ffi::WasmEdge_ConfigureRemoveHostRegistration(ctx.raw, host) }
}

pub fn has_host_registration(
    ctx: &WasmEdgeConfigureContext,
    host: WasmEdgeHostRegistration,
) -> bool {
    {
        unsafe { we_ffi::WasmEdge_ConfigureHasHostRegistration(ctx.raw, host) }
    }
}

pub struct WasmEdgeVMContext {
    raw: *mut we_ffi::WasmEdge_VMContext,
}
impl WasmEdgeVMContext {
    pub fn create(
        conf_ctx: Option<&WasmEdgeConfigureContext>,
        store_ctx: Option<&mut WasmEdgeStoreContext>,
    ) -> WasmEdgeVMContext {
        match (conf_ctx, store_ctx) {
            (Some(conf_ctx), Some(store_ctx)) => WasmEdgeVMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, store_ctx.raw) },
            },
            (Some(conf_ctx), None) => WasmEdgeVMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, ptr::null_mut()) },
            },
            (None, Some(store_ctx)) => WasmEdgeVMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), store_ctx.raw) },
            },
            (None, None) => WasmEdgeVMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), ptr::null_mut()) },
            },
        }
    }

    pub fn register_module_from_import(
        &mut self,
        import_ctx: &WasmEdgeImportObjectContext,
    ) -> WasmEdgeResult<u32> {
        unsafe {
            check(we_ffi::WasmEdge_VMRegisterModuleFromImport(
                self.raw,
                import_ctx.raw,
            ))
        }
    }

    pub fn register_module_from_file<P: AsRef<Path>>(
        &mut self,
        mod_name: &str,
        path: P,
    ) -> WasmEdgeResult<u32> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let path = path_to_cstring(path.as_ref())?;
        unsafe {
            check(we_ffi::WasmEdge_VMRegisterModuleFromFile(
                self.raw,
                mod_name.raw,
                path.as_ptr(),
            ))
        }
    }

    pub fn run_wasm_from_file<'vm, P: AsRef<Path>>(
        &mut self,
        path: P,
        func_name: &str,
        params: &[WasmEdgeValue],
        buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe {
            let func_name = WasmEdgeString::from_str(func_name)
                .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

            let result = check(we_ffi::WasmEdge_VMRunWasmFromFile(
                self.raw,
                path.as_ptr(),
                func_name.raw,
                params.as_ptr() as *const WasmEdgeValue,
                params.len() as u32,
                buf.as_mut_ptr() as *mut WasmEdgeValue,
                buf.len() as u32,
            ));

            match result {
                Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])),
                Err(err) => Err(err),
            }
        }
    }

    pub fn run_wasm_from_buffer<'vm>(
        &mut self,
        buf: &[u8],
        func_name: &str,
        params: &[WasmEdgeValue],
        returns: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

        unsafe {
            let result = check(we_ffi::WasmEdge_VMRunWasmFromBuffer(
                self.raw,
                buf.as_ptr(),
                buf.len() as u32,
                func_name.raw,
                params.as_ptr() as *const WasmEdgeValue,
                params.len() as u32,
                returns.as_mut_ptr() as *mut WasmEdgeValue,
                returns.len() as u32,
            ));

            match result {
                Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(
                    &returns[..returns.len()],
                )),
                Err(err) => Err(err),
            }
        }
    }

    pub fn function_type(&self, func_name: &str) -> Option<WasmEdgeFunctionTypeContext> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let result = unsafe { we_ffi::WasmEdge_VMGetFunctionType(self.raw, func_name.raw) };
        if result.is_null() {
            return None;
        }

        Some(WasmEdgeFunctionTypeContext { raw: result })
    }

    pub fn function_type_registered(
        &self,
        mod_name: &str,
        func_name: &str,
    ) -> Option<WasmEdgeFunctionTypeContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let result = unsafe {
            we_ffi::WasmEdge_VMGetFunctionTypeRegistered(self.raw, mod_name.raw, func_name.raw)
        };
        if result.is_null() {
            return None;
        }

        Some(WasmEdgeFunctionTypeContext { raw: result })
    }

    pub fn function_list_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_VMGetFunctionListLength(self.raw) as usize }
    }
}
impl Drop for WasmEdgeVMContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_VMDelete(self.raw) }
        }
    }
}

#[cfg(unix)]
fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    use std::os::unix::ffi::OsStrExt;

    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path '{}' to UTF-8", path.display());
            Err(message.into())
        }
    }
}

pub fn WasmEdgeValueGenI32(val: i32) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenI32(val) }
}

pub fn WasmEdgeValueGenI64(val: i64) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenI64(val) }
}

pub fn WasmEdgeValueGetI32(val: WasmEdgeValue) -> i32 {
    unsafe { we_ffi::WasmEdge_ValueGetI32(val) }
}

pub fn WasmEdgeValueGetI64(val: WasmEdgeValue) -> i64 {
    unsafe { we_ffi::WasmEdge_ValueGetI64(val) }
}

pub fn vm_run_wasm_from_file<'vm, P: AsRef<Path>>(
    ctx: &'vm mut WasmEdgeVMContext,
    path: P,
    func_name: &str,
    params: &[WasmEdgeValue],
    buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
    let path = path_to_cstring(path.as_ref())?;
    unsafe {
        let funcname = we_ffi::WasmEdge_StringCreateByCString(CString::new(func_name)?.as_ptr());

        let result = check(we_ffi::WasmEdge_VMRunWasmFromFile(
            ctx.raw,
            path.as_ptr(),
            funcname,
            params.as_ptr() as *const WasmEdgeValue,
            params.len() as u32,
            buf.as_mut_ptr() as *mut WasmEdgeValue,
            buf.len() as u32,
        ));

        we_ffi::WasmEdge_StringDelete(funcname);
        // mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]);

        match result {
            Ok(_) => Ok(mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])),
            Err(err) => Err(err),
        }
    }
}

pub type WasmEdgeValType = we_ffi::WasmEdge_ValType;

pub struct WasmEdgeFunctionTypeContext {
    raw: *mut we_ffi::WasmEdge_FunctionTypeContext,
}
impl WasmEdgeFunctionTypeContext {
    pub fn new(
        params: &[WasmEdgeValType],
        returns: &[WasmEdgeValType],
    ) -> WasmEdgeFunctionTypeContext {
        WasmEdgeFunctionTypeContext {
            raw: unsafe {
                we_ffi::WasmEdge_FunctionTypeCreate(
                    params.as_ptr(),
                    params.len() as u32,
                    returns.as_ptr(),
                    returns.len() as u32,
                )
            },
        }
    }

    pub fn parameters_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_FunctionTypeGetParametersLength(self.raw) as usize }
    }

    pub fn parameters(&self, list: &mut [WasmEdgeValType]) -> usize {
        unsafe {
            we_ffi::WasmEdge_FunctionTypeGetParameters(
                self.raw,
                list.as_mut_ptr(),
                list.len() as u32,
            ) as usize
        }
    }

    pub fn returns_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_FunctionTypeGetReturnsLength(self.raw) as usize }
    }

    pub fn returns<'val>(
        &self,
        list: &'val mut [mem::MaybeUninit<WasmEdgeValType>],
    ) -> (usize, &'val [WasmEdgeValType]) {
        let length = unsafe {
            we_ffi::WasmEdge_FunctionTypeGetReturns(
                self.raw,
                list.as_mut_ptr() as *mut WasmEdgeValType,
                list.len() as u32,
            ) as usize
        };

        (length, unsafe {
            mem::MaybeUninit::slice_assume_init_ref(&list[..list.len()])
        })
    }

    pub fn into_raw(&self) -> *mut we_ffi::WasmEdge_FunctionTypeContext {
        self.raw
    }
}
impl Drop for WasmEdgeFunctionTypeContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_FunctionTypeDelete(self.raw) }
    }
}

pub struct WasmEdgeHostFunctionContext {
    raw: *mut we_ffi::WasmEdge_HostFunctionContext,
}
impl WasmEdgeHostFunctionContext {
    pub fn new(
        func_type: &WasmEdgeFunctionTypeContext,
        host_func: we_ffi::WasmEdge_HostFunc_t,
        cost: u64,
    ) -> WasmEdgeHostFunctionContext {
        WasmEdgeHostFunctionContext {
            raw: unsafe {
                we_ffi::WasmEdge_HostFunctionCreate(func_type.into_raw(), host_func, cost)
            },
        }
    }
}
impl Drop for WasmEdgeHostFunctionContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_HostFunctionDelete(self.raw) }
    }
}

pub struct WasmEdgeImportObjectContext {
    raw: *mut we_ffi::WasmEdge_ImportObjectContext,
}
impl WasmEdgeImportObjectContext {
    pub fn create(mod_name: &str, data: *mut std::os::raw::c_void) -> WasmEdgeImportObjectContext {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        WasmEdgeImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_ImportObjectCreate(mod_name.raw, data) },
        }
    }

    pub fn create_tensorflow_import_object() -> WasmEdgeImportObjectContext {
        WasmEdgeImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_Tensorflow_ImportObjectCreate() },
        }
    }

    pub fn add_host_function(
        &mut self,
        name: &str,
        host_func_ctx: &mut WasmEdgeHostFunctionContext,
    ) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddHostFunction(self.raw, name.raw, host_func_ctx.raw)
        }
    }

    pub fn add_table(&mut self, name: &str, table_ctx: &mut WasmEdgeTableInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe { we_ffi::WasmEdge_ImportObjectAddTable(self.raw, name.raw, table_ctx.raw) }
    }

    pub fn add_memory(&mut self, name: &str, mem_ctx: &mut WasmEdgeMemoryInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddMemory(self.raw, name.raw, mem_ctx.raw);
        }
    }

    pub fn add_global(&mut self, name: &str, global_ctx: &mut WasmEdgeGlobalInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddGlobal(self.raw, name.raw, global_ctx.raw);
        }
    }
}

pub struct WasmEdgeString {
    raw: we_ffi::WasmEdge_String,
}
impl WasmEdgeString {
    pub fn from_str(s: &str) -> WasmEdgeResult<WasmEdgeString> {
        let cstring = CString::new(s)?;
        Ok(WasmEdgeString {
            raw: unsafe { we_ffi::WasmEdge_StringCreateByCString(cstring.as_ptr()) },
        })
    }

    pub fn from_buffer(buf: &[i8]) -> WasmEdgeString {
        WasmEdgeString {
            raw: unsafe { we_ffi::WasmEdge_StringCreateByBuffer(buf.as_ptr(), buf.len() as u32) },
        }
    }
}
impl Drop for WasmEdgeString {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_StringDelete(self.raw) }
    }
}

pub type WasmEdgeLimit = we_ffi::WasmEdge_Limit;
pub type WasmEdgeRefType = we_ffi::WasmEdge_RefType;
pub type WasmEdgeMutability = we_ffi::WasmEdge_Mutability;

pub struct WasmEdgeTableInstanceContext {
    raw: *mut we_ffi::WasmEdge_TableInstanceContext,
}
impl WasmEdgeTableInstanceContext {
    pub fn create(ref_type: WasmEdgeRefType, limit: WasmEdgeLimit) -> WasmEdgeTableInstanceContext {
        WasmEdgeTableInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_TableInstanceCreate(ref_type, limit) },
        }
    }
}
impl Drop for WasmEdgeTableInstanceContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_TableInstanceDelete(self.raw) }
    }
}

pub struct WasmEdgeMemoryInstanceContext {
    raw: *mut we_ffi::WasmEdge_MemoryInstanceContext,
}
impl WasmEdgeMemoryInstanceContext {
    pub fn create(limit: WasmEdgeLimit) -> Self {
        WasmEdgeMemoryInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_MemoryInstanceCreate(limit) },
        }
    }
}
impl Drop for WasmEdgeMemoryInstanceContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_MemoryInstanceDelete(self.raw) }
    }
}

pub struct WasmEdgeGlobalInstanceContext {
    raw: *mut we_ffi::WasmEdge_GlobalInstanceContext,
}
impl WasmEdgeGlobalInstanceContext {
    pub fn create(value: WasmEdgeValue, mutable: WasmEdgeMutability) -> Self {
        WasmEdgeGlobalInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_GlobalInstanceCreate(value, mutable) },
        }
    }
}
impl Drop for WasmEdgeGlobalInstanceContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_GlobalInstanceDelete(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::path::Path;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
        assert!(major_version() >= 0);
        assert!(minor_version() >= 0);
        assert!(version_patch() >= 0);
    }

    #[test]
    fn test_wasmedge_vm_run_wasm_from_file() {
        let mut conf_ctx = WasmEdgeConfigureContext::create();
        add_host_registration(
            &mut conf_ctx,
            WasmEdgeHostRegistration::WasmEdge_HostRegistration_Wasi,
        );

        let mut vm_ctx = WasmEdgeVMContext::create(Some(&conf_ctx), None);
        let wasm_name = "../../add.wasm";
        let func_name = "add";
        let params = vec![WasmEdgeValueGenI32(2), WasmEdgeValueGenI32(8)];
        let mut buf: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();

        let result = vm_run_wasm_from_file(
            &mut vm_ctx,
            wasm_name,
            func_name,
            params.as_slice(),
            &mut buf,
        );

        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(WasmEdgeValueGetI32(values[0]), 10);
    }

    #[test]
    fn test_wasmedge_host_function() {
        #[no_mangle]
        unsafe extern "C" fn Add(
            data: *mut std::os::raw::c_void,
            mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
            params: *const WasmEdgeValue,
            returns: *mut WasmEdgeValue,
        ) -> we_ffi::WasmEdge_Result {
            let params = std::slice::from_raw_parts(params, 2);
            let val1 = WasmEdgeValueGetI32(params[0]);
            let val2 = WasmEdgeValueGetI32(params[1]);
            println!("Host function \"Add\": {} + {}\n", val1, val2);
            let res = WasmEdgeValueGenI32(val1 + val2);
            returns.write(res);

            we_ffi::WasmEdge_Result { Code: 0 }
        }

        // The WASM module buffer.
        let wasm_buf: Vec<u8> = vec![
            /* WASM header */
            0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, /* Type section */
            0x01, 0x07, 0x01, /* function type {i32, i32} -> {i32} */
            0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F, /* Import section */
            0x02, 0x13, 0x01, /* module name: "extern" */
            0x06, 0x65, 0x78, 0x74, 0x65, 0x72, 0x6E, /* extern name: "func-add" */
            0x08, 0x66, 0x75, 0x6E, 0x63, 0x2D, 0x61, 0x64, 0x64,
            /* import desc: func 0 */
            0x00, 0x00, /* Function section */
            0x03, 0x02, 0x01, 0x00, /* Export section */
            0x07, 0x0A, 0x01, /* export name: "addTwo" */
            0x06, 0x61, 0x64, 0x64, 0x54, 0x77, 0x6F, /* export desc: func 0 */
            0x00, 0x01, /* Code section */
            0x0A, 0x0A, 0x01, /* code body */
            0x08, 0x00, 0x20, 0x00, 0x20, 0x01, 0x10, 0x00, 0x0B,
        ];

        // create import object
        let mut imp_obj = WasmEdgeImportObjectContext::create("extern", ptr::null_mut());

        // Create and add a host function instance into the import object.
        let param_list = [
            WasmEdgeValType::WasmEdge_ValType_I32,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let return_list = [WasmEdgeValType::WasmEdge_ValType_I32];
        let func_type = WasmEdgeFunctionTypeContext::new(&param_list, &return_list);
        let mut host_func_ctx = WasmEdgeHostFunctionContext::new(&func_type, Some(Add), 0);
        imp_obj.add_host_function("func-add", &mut host_func_ctx);

        // Create the VM context.
        let mut vm_ctx = WasmEdgeVMContext::create(None, None);
        vm_ctx.register_module_from_import(&imp_obj).unwrap();

        // The parameters and returns arrays.
        let params = vec![WasmEdgeValueGenI32(1234), WasmEdgeValueGenI32(5678)];
        let mut out: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();

        // Run the WASM function from file.
        let result =
            vm_ctx.run_wasm_from_buffer(wasm_buf.as_slice(), "addTwo", params.as_slice(), &mut out);

        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(WasmEdgeValueGetI32(values[0]), 6912);

        let result = vm_ctx.function_type("addTwo");
        assert!(result.is_some());
        let func_type = result.unwrap();

        println!("len of returns: {}", func_type.returns_len());
        println!("len of params: {}", func_type.parameters_len());

        // let mut list: Vec<WasmEdgeValType> = vec![];
        let mut out: [mem::MaybeUninit<WasmEdgeValType>; 1] = mem::MaybeUninit::uninit_array();
        let (size, list) = func_type.returns(&mut out);
        println!("size of returns: {}", size);
        println!("len of list: {}", list.len());
        println!("result value: {:?}", list[0]);
    }

    #[test]
    fn test_wasmedge_tensorflow() {
        let mut conf_ctx = WasmEdgeConfigureContext::create();
        conf_ctx.add_host_registration(WasmEdgeHostRegistration::WasmEdge_HostRegistration_Wasi);
        conf_ctx.add_host_registration(
            WasmEdgeHostRegistration::WasmEdge_HostRegistration_WasmEdge_Process,
        );
        let mut vm_ctx = WasmEdgeVMContext::create(Some(&conf_ctx), None);

        // create tensorflow module: mod name: "wasmedge_tensorflow"
        let tensorflow_mod = WasmEdgeImportObjectContext::create_tensorflow_import_object();
        let result = vm_ctx.register_module_from_import(&tensorflow_mod);
        assert!(result.is_ok());
        // check the registered function
        let result = vm_ctx
            .function_type_registered("wasmedge_tensorflow", "wasmedge_tensorflow_create_session");
        assert!(result.is_some());

        // register wasmedge-wasi-nn module
        let result = vm_ctx.register_module_from_file(
            "calculator",
            "/root/workspace/wasmedge-ml/target/wasm32-wasi/debug/wasmedge_wasi_nn.wasm",
        );
        assert!(result.is_ok());

        // load tensorflow model
        let buf = std::fs::read("/root/workspace/wasmedge-ml/docs/add.pb").unwrap();
        let vec = [&buf];
        let ptr = vec.as_ptr();
        let ptr = ptr as i64;

        // run target wasm module
        let params = vec![WasmEdgeValueGenI64(ptr), WasmEdgeValueGenI32(1)];
        let mut out: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();
        let result = vm_ctx.run_wasm_from_file(
            "/root/workspace/wasmedge-ml/docs/using_add.wasm",
            "consume_load",
            params.as_slice(),
            &mut out,
        );
        assert!(result.is_ok());

        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(WasmEdgeValueGetI32(values[0]), 101);
    }

    // #[test]
    // fn test_wasmedge_run_wasm() {
    //     // create VM    turning on the Wasi support
    //     let mut config_ctx = WasmEdgeConfigureContext::create();
    //     config_ctx.add_host_registration(
    //         we_ffi::WasmEdge_HostRegistration::WasmEdge_HostRegistration_Wasi,
    //     );
    //     let mut vm_ctx = WasmEdgeVMContext::create(Some(&config_ctx), None);

    //     // let mut vm_ctx = WasmEdgeVMContext::create(None, None);

    //     // * register wasm_calc.wasm
    //     // register the wasm module into vm
    //     let res = vm_ctx.register_module_from_file(
    //         "calculator",
    //         "/root/workspace/wasmedge-ml/target/wasm32-wasi/debug/wasmedge_wasi_nn.wasm",
    //     );
    //     assert!(res.is_ok());

    //     // * run consume_add function in using_add.wasm
    //     let buf = std::fs::read("/root/workspace/wasmedge-ml/docs/add.pb").unwrap();
    //     let vec = [&buf];
    //     let ptr = vec.as_ptr();
    //     let ptr = ptr as i64;

    //     let params = vec![WasmEdgeValueGenI64(ptr), WasmEdgeValueGenI32(1)];
    //     let mut out: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();

    //     // Run the WASM function from file.
    //     let result = vm_ctx.run_wasm_from_file(
    //         Path::new("/root/workspace/wasmedge-ml/docs/using_add.wasm")
    //             .canonicalize()
    //             .unwrap(),
    //         "consume_load",
    //         params.as_slice(),
    //         &mut out,
    //     );
    //     assert!(result.is_ok());

    //     let values = result.unwrap();
    //     assert_eq!(values.len(), 1);
    //     assert_eq!(WasmEdgeValueGetI32(values[0]), 1);
    // }
}
