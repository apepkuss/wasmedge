#![feature(maybe_uninit_uninit_array, maybe_uninit_extra, maybe_uninit_slice)]

use std::ffi::{CStr, CString};
use std::fmt;
use std::marker::PhantomData;
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

pub type WasmEdgeProposal = we_ffi::WasmEdge_Proposal;

pub struct ConfigureContext {
    raw: *mut we_ffi::WasmEdge_ConfigureContext,
}
impl ConfigureContext {
    pub fn create() -> ConfigureContext {
        ConfigureContext {
            raw: unsafe { we_ffi::WasmEdge_ConfigureCreate() },
        }
    }

    pub fn add_host_registration(&mut self, host: HostRegistration) {
        unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(self.raw, host) }
    }

    pub fn add_proposal(&mut self, prop: WasmEdgeProposal) {
        unsafe { we_ffi::WasmEdge_ConfigureAddProposal(self.raw, prop) }
    }
}
impl Drop for ConfigureContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ConfigureDelete(self.raw) }
        }
    }
}

pub struct StoreContext<'vm> {
    raw: *mut we_ffi::WasmEdge_StoreContext,
    _marker: PhantomData<&'vm VMContext>,
}
impl<'vm> StoreContext<'vm> {
    pub fn create() -> StoreContext<'vm> {
        StoreContext {
            raw: unsafe { we_ffi::WasmEdge_StoreCreate() },
            _marker: PhantomData,
        }
    }

    pub fn find_memory(&self, mem_name: &str) -> MemoryInstanceContext {
        let mem_name = WasmEdgeString::from_str(mem_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
        let mem = unsafe { we_ffi::WasmEdge_StoreFindMemory(self.raw, mem_name.raw) };
        MemoryInstanceContext {
            raw: mem,
            _marker: PhantomData,
        }
    }

    pub fn find_memory_registered(&self, mod_name: &str, mem_name: &str) -> MemoryInstanceContext {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let mem_name = WasmEdgeString::from_str(mem_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
        let mem = unsafe {
            we_ffi::WasmEdge_StoreFindMemoryRegistered(self.raw, mod_name.raw, mem_name.raw)
        };
        MemoryInstanceContext {
            raw: mem,
            _marker: PhantomData,
        }
    }

    pub fn list_function_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListFunctionLength(self.raw) as usize }
    }

    pub fn list_function_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe {
            we_ffi::WasmEdge_StoreListFunctionRegisteredLength(self.raw, mod_name.raw) as usize
        }
    }

    pub fn list_table_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListTableLength(self.raw) as usize }
    }

    pub fn list_table_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe { we_ffi::WasmEdge_StoreListTableRegisteredLength(self.raw, mod_name.raw) as usize }
    }

    pub fn list_global_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListGlobalLength(self.raw) as usize }
    }

    pub fn list_global_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe { we_ffi::WasmEdge_StoreListGlobalRegisteredLength(self.raw, mod_name.raw) as usize }
    }

    pub fn list_memory_len(&self) -> usize {
        let len = unsafe { we_ffi::WasmEdge_StoreListMemoryLength(self.raw) };
        len as usize
    }

    pub fn list_memory(
        &self,
        buf: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> (usize, Vec<String>) {
        let len = unsafe {
            we_ffi::WasmEdge_StoreListMemory(
                self.raw,
                buf.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                buf.len() as u32,
            )
        };
        let s_vec = unsafe { mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]) };
        let mut names = vec![];
        for s in s_vec {
            let str = WasmEdgeString { raw: *s };
            let cow = str.to_string_lossy();
            names.push(cow.into_owned())
        }
        (len as usize, names)
    }

    pub fn list_memory_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let len =
            unsafe { we_ffi::WasmEdge_StoreListMemoryRegisteredLength(self.raw, mod_name.raw) };
        len as usize
    }

    pub fn list_memory_registered(
        &self,
        mod_name: &str,
        buf: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> (usize, Vec<String>) {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let len = unsafe {
            we_ffi::WasmEdge_StoreListMemoryRegistered(
                self.raw,
                mod_name.raw,
                buf.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                buf.len() as u32,
            )
        };
        let s_vec = unsafe { mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()]) };
        let mut names = vec![];
        for s in s_vec {
            let str = WasmEdgeString { raw: *s };
            let cow = str.to_string_lossy();
            names.push(cow.into_owned())
        }
        (len as usize, names)
    }

    pub fn list_module_len(&self) -> usize {
        let len = unsafe { we_ffi::WasmEdge_StoreListModuleLength(self.raw) };
        len as usize
    }
}
impl<'vm> Drop for StoreContext<'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_StoreDelete(self.raw) }
        }
    }
}

pub type HostRegistration = we_ffi::WasmEdge_HostRegistration;

pub fn add_host_registration(ctx: &mut ConfigureContext, host: HostRegistration) {
    unsafe { we_ffi::WasmEdge_ConfigureAddHostRegistration(ctx.raw, host) }
}

pub fn remove_host_registration(ctx: &mut ConfigureContext, host: HostRegistration) {
    unsafe { we_ffi::WasmEdge_ConfigureRemoveHostRegistration(ctx.raw, host) }
}

pub fn has_host_registration(ctx: &ConfigureContext, host: HostRegistration) -> bool {
    {
        unsafe { we_ffi::WasmEdge_ConfigureHasHostRegistration(ctx.raw, host) }
    }
}

pub struct VMContext {
    raw: *mut we_ffi::WasmEdge_VMContext,
}
impl VMContext {
    pub fn create(
        conf_ctx: Option<&ConfigureContext>,
        store_ctx: Option<&mut StoreContext>,
    ) -> VMContext {
        match (conf_ctx, store_ctx) {
            (Some(conf_ctx), Some(store_ctx)) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, store_ctx.raw) },
            },
            (Some(conf_ctx), None) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(conf_ctx.raw, ptr::null_mut()) },
            },
            (None, Some(store_ctx)) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), store_ctx.raw) },
            },
            (None, None) => VMContext {
                raw: unsafe { we_ffi::WasmEdge_VMCreate(ptr::null(), ptr::null_mut()) },
            },
        }
    }

    pub fn register_module_from_import_object(
        &mut self,
        import_ctx: &ImportObjectContext,
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

    pub fn function_type(&self, func_name: &str) -> Option<FunctionTypeContext> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let result = unsafe { we_ffi::WasmEdge_VMGetFunctionType(self.raw, func_name.raw) };
        if result.is_null() {
            return None;
        }

        Some(FunctionTypeContext { raw: result })
    }

    pub fn function_type_registered(
        &self,
        mod_name: &str,
        func_name: &str,
    ) -> Option<FunctionTypeContext> {
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

        Some(FunctionTypeContext { raw: result })
    }

    pub fn function_list_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_VMGetFunctionListLength(self.raw) as usize }
    }

    pub fn importobject_module(&self, reg: HostRegistration) -> Option<ImportObjectContext> {
        let raw = unsafe { we_ffi::WasmEdge_VMGetImportModuleContext(self.raw, reg) };
        if raw.is_null() {
            return None;
        } else {
        }
        match raw.is_null() {
            true => None,
            false => Some(ImportObjectContext {
                raw,
                _marker: PhantomData,
            }),
        }
    }

    pub fn execute_registered<'vm>(
        &self,
        mod_name: &str,
        func_name: &str,
        params: &[WasmEdgeValue],
        buf: &'vm mut [mem::MaybeUninit<WasmEdgeValue>],
    ) -> WasmEdgeResult<&'vm [WasmEdgeValue]> {
        unsafe {
            let mod_name = WasmEdgeString::from_str(mod_name)
                .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
            let func_name = WasmEdgeString::from_str(func_name)
                .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());

            let result = check(we_ffi::WasmEdge_VMExecuteRegistered(
                self.raw,
                mod_name.raw,
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

    pub fn store_context(&self) -> StoreContext {
        let store_ctx = unsafe { we_ffi::WasmEdge_VMGetStoreContext(self.raw) };
        StoreContext {
            raw: store_ctx,
            _marker: PhantomData,
        }
    }
}
impl Drop for VMContext {
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

pub fn WasmEdgeValueGenExternRef(ptr: *mut ::std::os::raw::c_void) -> WasmEdgeValue {
    unsafe { we_ffi::WasmEdge_ValueGenExternRef(ptr) }
}

pub fn WasmEdgeValueGetI32(val: WasmEdgeValue) -> i32 {
    unsafe { we_ffi::WasmEdge_ValueGetI32(val) }
}

pub fn WasmEdgeValueGetI64(val: WasmEdgeValue) -> i64 {
    unsafe { we_ffi::WasmEdge_ValueGetI64(val) }
}

pub fn WasmEdgeValueGetExternRef(val: WasmEdgeValue) -> *mut ::std::os::raw::c_void {
    unsafe { we_ffi::WasmEdge_ValueGetExternRef(val) }
}

pub fn vm_run_wasm_from_file<'vm, P: AsRef<Path>>(
    ctx: &'vm mut VMContext,
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

pub struct FunctionTypeContext {
    raw: *mut we_ffi::WasmEdge_FunctionTypeContext,
}
impl FunctionTypeContext {
    pub fn create(
        params: Option<&[WasmEdgeValType]>,
        returns: &[WasmEdgeValType],
    ) -> FunctionTypeContext {
        let (param_list, param_len) = match params {
            Some(params) => (params.as_ptr(), params.len()),
            None => (ptr::null(), 0),
        };
        FunctionTypeContext {
            raw: unsafe {
                we_ffi::WasmEdge_FunctionTypeCreate(
                    param_list,
                    param_len as u32,
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
}
impl Drop for FunctionTypeContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_FunctionTypeDelete(self.raw) }
    }
}

pub struct HostFunctionContext {
    raw: *mut we_ffi::WasmEdge_HostFunctionContext,
}
impl HostFunctionContext {
    pub fn create(
        func_type: &FunctionTypeContext,
        host_func: we_ffi::WasmEdge_HostFunc_t,
        cost: u64,
    ) -> Option<HostFunctionContext> {
        let raw = unsafe { we_ffi::WasmEdge_HostFunctionCreate(func_type.raw, host_func, cost) };
        println!("raw: {:?}", raw);
        match raw.is_null() {
            true => None,
            false => Some(HostFunctionContext { raw }),
        }
    }

    pub fn create_binding(
        func_type: &FunctionTypeContext,
        wrap_func: we_ffi::WasmEdge_WrapFunc_t,
        binding: *mut std::os::raw::c_void,
        cost: u64,
    ) -> HostFunctionContext {
        HostFunctionContext {
            raw: unsafe {
                we_ffi::WasmEdge_HostFunctionCreateBinding(func_type.raw, wrap_func, binding, cost)
            },
        }
    }
}
impl Drop for HostFunctionContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_HostFunctionDelete(self.raw) }
    }
}

#[derive(Clone)]
pub struct ImportObjectContext<'vm> {
    raw: *mut we_ffi::WasmEdge_ImportObjectContext,
    _marker: PhantomData<&'vm VMContext>,
}
impl<'vm> ImportObjectContext<'vm> {
    pub fn create(mod_name: &str, data: *mut std::os::raw::c_void) -> Option<ImportObjectContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_ImportObjectCreate(mod_name.raw, data) };
        match raw.is_null() {
            true => None,
            false => Some(ImportObjectContext {
                raw,
                _marker: PhantomData,
            }),
        }
    }

    pub fn create_tensorflow_import_object() -> ImportObjectContext<'vm> {
        ImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_Tensorflow_ImportObjectCreate() },
            _marker: PhantomData,
        }
    }

    pub fn create_tensorflowlite_import_object() -> ImportObjectContext<'vm> {
        ImportObjectContext {
            raw: unsafe { we_ffi::WasmEdge_TensorflowLite_ImportObjectCreate() },
            _marker: PhantomData,
        }
    }

    pub fn init_wasi(&self, args: &[&str], envs: &[&str], dirs: &[&str], preopens: &[&str]) {
        let mut cargs = vec![];
        for &arg in args.iter() {
            cargs.push(arg.as_ptr() as *const _);
        }
        let mut cenvs = vec![];
        for &env in envs.iter() {
            cenvs.push(env.as_ptr() as *const _);
        }
        let mut cdirs = vec![];
        for &dir in dirs.iter() {
            cdirs.push(dir.as_ptr() as *const _);
        }
        let mut cpreopens = vec![];
        for &pre in preopens.iter() {
            cpreopens.push(pre.as_ptr() as *const _);
        }
        unsafe {
            we_ffi::WasmEdge_ImportObjectInitWASI(
                self.raw,
                cargs.as_ptr(),
                cargs.len() as u32,
                cenvs.as_ptr(),
                cenvs.len() as u32,
                cdirs.as_ptr(),
                cdirs.len() as u32,
                cpreopens.as_ptr(),
                cpreopens.len() as u32,
            )
        };
    }

    pub fn add_host_function(&mut self, name: &str, host_func_ctx: &mut HostFunctionContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddHostFunction(self.raw, name.raw, host_func_ctx.raw)
        }
    }

    pub fn add_table(&mut self, name: &str, table_ctx: &mut TableInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe { we_ffi::WasmEdge_ImportObjectAddTable(self.raw, name.raw, table_ctx.raw) }
    }

    pub fn add_memory(&mut self, name: &str, mem_ctx: &mut MemoryInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddMemory(self.raw, name.raw, mem_ctx.raw);
        }
    }

    pub fn add_global(&mut self, name: &str, global_ctx: &mut GlobalInstanceContext) {
        let name = WasmEdgeString::from_str(name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", name).as_str());
        unsafe {
            we_ffi::WasmEdge_ImportObjectAddGlobal(self.raw, name.raw, global_ctx.raw);
        }
    }
}
impl<'vm> Drop for ImportObjectContext<'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ImportObjectDelete(self.raw) }
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

    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        let cstr = unsafe { CStr::from_ptr(self.raw.Buf) };
        cstr.to_string_lossy()
    }

    pub fn into_raw(&self) -> we_ffi::WasmEdge_String {
        self.raw
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

pub struct TableInstanceContext {
    raw: *mut we_ffi::WasmEdge_TableInstanceContext,
}
impl TableInstanceContext {
    pub fn create(ref_type: WasmEdgeRefType, limit: WasmEdgeLimit) -> TableInstanceContext {
        TableInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_TableInstanceCreate(ref_type, limit) },
        }
    }
}
impl Drop for TableInstanceContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_TableInstanceDelete(self.raw) }
    }
}

pub struct MemoryInstanceContext<'store, 'vm> {
    raw: *mut we_ffi::WasmEdge_MemoryInstanceContext,
    _marker: PhantomData<&'store StoreContext<'vm>>,
}
impl<'store, 'vm> MemoryInstanceContext<'store, 'vm> {
    pub fn create(limit: WasmEdgeLimit) -> Self {
        MemoryInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_MemoryInstanceCreate(limit) },
            _marker: PhantomData,
        }
    }
}
impl<'store, 'vm> Drop for MemoryInstanceContext<'store, 'vm> {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_MemoryInstanceDelete(self.raw) }
    }
}

pub struct GlobalInstanceContext {
    raw: *mut we_ffi::WasmEdge_GlobalInstanceContext,
}
impl GlobalInstanceContext {
    pub fn create(value: WasmEdgeValue, mutable: WasmEdgeMutability) -> Self {
        GlobalInstanceContext {
            raw: unsafe { we_ffi::WasmEdge_GlobalInstanceCreate(value, mutable) },
        }
    }
}
impl Drop for GlobalInstanceContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_GlobalInstanceDelete(self.raw) }
    }
}

pub struct ASTModuleContext {
    raw: *mut we_ffi::WasmEdge_ASTModuleContext,
}
impl ASTModuleContext {
    pub fn into_raw(&mut self) -> *mut we_ffi::WasmEdge_ASTModuleContext {
        self.raw
    }
}
impl Drop for ASTModuleContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ASTModuleDelete(self.raw) }
        }
    }
}
impl Default for ASTModuleContext {
    fn default() -> Self {
        ASTModuleContext {
            raw: ptr::null_mut(),
        }
    }
}

pub struct LoaderContext {
    raw: *mut we_ffi::WasmEdge_LoaderContext,
}
impl LoaderContext {
    pub fn create(conf: &ConfigureContext) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_LoaderCreate(conf.raw) };
        match raw.is_null() {
            true => None,
            false => Some(LoaderContext { raw }),
        }
    }

    pub fn parse_from_file<P: AsRef<Path>>(&mut self, module: &mut ASTModuleContext, path: P) {
        let path = path_to_cstring(path.as_ref()).unwrap();
        unsafe { we_ffi::WasmEdge_LoaderParseFromFile(self.raw, &mut module.raw, path.as_ptr()) };
    }
}
impl Drop for LoaderContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_LoaderDelete(self.raw) }
        }
    }
}

pub struct InterpreterContext {
    raw: *mut we_ffi::WasmEdge_InterpreterContext,
}
impl InterpreterContext {
    pub fn create(
        conf: Option<&ConfigureContext>,
        stat: Option<&mut StatisticsContext>,
    ) -> Option<Self> {
        let conf = match conf {
            Some(conf) => conf.raw,
            None => ptr::null(),
        };
        let stat = match stat {
            Some(stat) => stat.raw,
            None => ptr::null_mut(),
        };
        let raw = unsafe { we_ffi::WasmEdge_InterpreterCreate(conf, stat) };
        match raw.is_null() {
            true => None,
            false => Some(InterpreterContext { raw }),
        }
    }

    pub fn register_import_object_module(
        &mut self,
        store: &mut StoreContext,
        imp_obj: *const we_ffi::WasmEdge_ImportObjectContext,
    ) -> bool {
        let res = unsafe {
            check(we_ffi::WasmEdge_InterpreterRegisterImport(
                self.raw, store.raw, imp_obj,
            ))
        };
        match res {
            Err(_) => false,
            Ok(_) => true,
        }
    }

    pub fn register_ast_module(
        &mut self,
        store: &mut StoreContext,
        ast_mod: &ASTModuleContext,
        mod_name: &str,
    ) -> bool {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let res = unsafe {
            check(we_ffi::WasmEdge_InterpreterRegisterModule(
                self.raw,
                store.raw,
                ast_mod.raw,
                mod_name.raw,
            ))
        };
        match res {
            Err(_) => false,
            Ok(_) => true,
        }
    }

    pub fn instantiate(&mut self, store: &mut StoreContext, ast_mod: &ASTModuleContext) -> bool {
        let res = unsafe {
            check(we_ffi::WasmEdge_InterpreterInstantiate(
                self.raw,
                store.raw,
                ast_mod.raw,
            ))
        };
        match res {
            Err(_) => false,
            Ok(_) => true,
        }
    }
}
impl Drop for InterpreterContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_InterpreterDelete(self.raw) }
    }
}

pub struct StatisticsContext {
    raw: *mut we_ffi::WasmEdge_StatisticsContext,
}
impl StatisticsContext {
    pub fn create() -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_StatisticsCreate() };
        match raw.is_null() {
            true => None,
            false => Some(StatisticsContext { raw }),
        }
    }
}
impl Drop for StatisticsContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_StatisticsDelete(self.raw) }
        }
    }
}

pub struct Validator {
    raw: *mut we_ffi::WasmEdge_ValidatorContext,
}
impl Validator {
    pub fn create(conf: &ConfigureContext) -> Option<Self> {
        let raw = unsafe { we_ffi::WasmEdge_ValidatorCreate(conf.raw) };
        match raw.is_null() {
            true => None,
            false => Some(Validator { raw }),
        }
    }

    pub fn validate(&mut self, ast_mod: &ASTModuleContext) -> WasmEdgeResult<u32> {
        unsafe { check(we_ffi::WasmEdge_ValidatorValidate(self.raw, ast_mod.raw)) }
    }
}
impl Drop for Validator {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { we_ffi::WasmEdge_ValidatorDelete(self.raw) }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    // use std::path::Path;

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
        assert!(major_version() >= 0);
        assert!(minor_version() >= 0);
        assert!(version_patch() >= 0);
    }

    #[test]
    fn test_wasmedge_vm_run_wasm_from_file() {
        let mut conf_ctx = ConfigureContext::create();
        add_host_registration(
            &mut conf_ctx,
            HostRegistration::WasmEdge_HostRegistration_Wasi,
        );

        let mut vm_ctx = VMContext::create(Some(&conf_ctx), None);
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

        // Create the VM context.
        let mut vm_ctx = VMContext::create(None, None);

        // create import object
        let mut imp_obj = ImportObjectContext::create("extern", ptr::null_mut()).unwrap();

        // Create and add a host function instance into the import object.
        let param_list = [
            WasmEdgeValType::WasmEdge_ValType_I32,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let return_list = [WasmEdgeValType::WasmEdge_ValType_I32];
        let func_type = FunctionTypeContext::create(Some(&param_list), &return_list);
        let res = HostFunctionContext::create(&func_type, Some(Add), 0);
        let mut host_func = res.unwrap();
        imp_obj.add_host_function("func-add", &mut host_func);

        // register import-object
        vm_ctx.register_module_from_import_object(&imp_obj).unwrap();

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
    fn test_wasmedge_memory_ctx() {
        #[no_mangle]
        unsafe extern "C" fn FuncAdd(
            data: *mut std::os::raw::c_void,
            mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
            params: *const WasmEdgeValue,
            returns: *mut WasmEdgeValue,
        ) -> we_ffi::WasmEdge_Result {
            let params = std::slice::from_raw_parts(params, 2);
            let val1 = WasmEdgeValueGetI32(params[0]);
            let val2 = WasmEdgeValueGetI32(params[1]);
            let res = WasmEdgeValueGenI32(val1 + val2);
            returns.write(res);

            we_ffi::WasmEdge_Result { Code: 0 }
        }

        // Create the VM context.
        let mut vm_ctx = VMContext::create(None, None);

        // create import object
        let mod_name = "calculator";
        let mut imp_obj = ImportObjectContext::create(mod_name, ptr::null_mut()).unwrap();

        // Create and add a host function instance into the import object.
        let param_list = [
            WasmEdgeValType::WasmEdge_ValType_I32,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let return_list = [WasmEdgeValType::WasmEdge_ValType_I32];
        let func_type = FunctionTypeContext::create(Some(&param_list), &return_list);
        let res = HostFunctionContext::create(&func_type, Some(FuncAdd), 0);
        let mut host_func = res.unwrap();
        imp_obj.add_host_function("func-add", &mut host_func);

        // register import-object
        vm_ctx.register_module_from_import_object(&imp_obj).unwrap();

        unsafe {
            // let tf_imp_obj = we_ffi::WasmEdge_Tensorflow_ImportObjectCreate();
            // vm_ctx
            //     .register_module_from_import_object(&tf_imp_obj)
            //     .unwrap();
            // we_ffi::WasmEdge_VMRegisterModuleFromImport(vm_ctx.raw, tf_imp_obj);

            // let ipt_ctx = we_ffi::WasmEdge_InterpreterCreate(ptr::null(), ptr::null_mut());
            // let store_ctx = we_ffi::WasmEdge_StoreCreate();
            // let result =
            //     we_ffi::WasmEdge_InterpreterRegisterImport(ipt_ctx, store_ctx, imp_obj.raw);
            // if !we_ffi::WasmEdge_ResultOK(result) {
            //     println!("Import object registration failed");
            // }

            let x = we_ffi::WasmEdge_VMGetFunctionListLength(vm_ctx.raw);
            println!("x: {}", x);

            let store = we_ffi::WasmEdge_VMGetStoreContext(vm_ctx.raw);
            // let mod_name = WasmEdgeString::from_str(mod_name).unwrap();
            // let len = we_ffi::WasmEdge_StoreListMemoryRegisteredLength(store_ctx, mod_name.raw);

            // let mod_name = WasmEdgeString::from_str("wasi_ephemeral_nn").unwrap();
            let len = we_ffi::WasmEdge_StoreListFunctionLength(store);
            println!("len: {}", len);
        }
    }

    #[test]
    fn test_wasmedge_tensorflow() {
        let mut conf_ctx = ConfigureContext::create();
        conf_ctx.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        conf_ctx
            .add_host_registration(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
        let mut vm_ctx = VMContext::create(Some(&conf_ctx), None);

        // create tensorflow and tensorflowlite modules: mod name: "wasmedge_tensorflow", "wasmedge_tensorflowlite"
        let mut result: Result<u32, WasmEdgeError>;
        let tensorflow_mod = ImportObjectContext::create_tensorflow_import_object();
        result = vm_ctx.register_module_from_import_object(&tensorflow_mod);
        assert!(result.is_ok());
        let tensorflowlite_mod = ImportObjectContext::create_tensorflowlite_import_object();
        result = vm_ctx.register_module_from_import_object(&tensorflowlite_mod);
        assert!(result.is_ok());

        // check the registered function: wasmedge_tensorflow_create_session
        let result = vm_ctx
            .function_type_registered("wasmedge_tensorflow", "wasmedge_tensorflow_create_session");
        assert!(result.is_some());
        let func_type = result.unwrap();
        let param_len = func_type.parameters_len();
        println!(
            "param len of wasmedge_tensorflow_create_session func: {}",
            param_len
        );
        let result = vm_ctx.function_type_registered(
            "wasmedge_tensorflowlite",
            "wasmedge_tensorflowlite_create_session",
        );
        assert!(result.is_some());
        let func_type = result.unwrap();
        let param_len = func_type.parameters_len();
        println!(
            "param len of lite wasmedge_tensorflowlite_create_session: {}",
            param_len
        );

        // check the registered function: wasmedge_tensorflow_get_output_tensor
        let result = vm_ctx.function_type_registered(
            "wasmedge_tensorflow",
            "wasmedge_tensorflow_get_output_tensor",
        );
        assert!(result.is_some());
        let func_type = result.unwrap();
        let param_len = func_type.parameters_len();
        println!(
            "param len of wasmedge_tensorflow_get_output_tensor func: {}",
            param_len
        );

        {
            let wasi_module =
                vm_ctx.importobject_module(HostRegistration::WasmEdge_HostRegistration_Wasi);
            assert!(wasi_module.is_some());
        }

        // get import-objects
        // let wasi_mod =
        //     vm_ctx.import_module_mut(WasmEdgeHostRegistration::WasmEdge_HostRegistration_Wasi);
        // let proc_mod = vm_ctx.import_module_mut(
        //     WasmEdgeHostRegistration::WasmEdge_HostRegistration_WasmEdge_Process,
        // );

        // // init wasi_mod
        // let args = vec!["using_add.wasm"];
        // let envs = vec![];
        // let dirs = vec![".:."];
        // let preopens = vec![];
        // wasi_mod.init_wasi(
        //     args.as_slice(),
        //     envs.as_slice(),
        //     dirs.as_slice(),
        //     preopens.as_slice(),
        // );

        // register wasmedge-wasi-nn module
        let result = vm_ctx.register_module_from_file(
            "calculator",
            "/root/workspace/wasmedge-ml/wasmedge-wasi-nn/target/wasm32-wasi/debug/wasmedge_wasi_nn.wasm",
        );
        assert!(result.is_ok());

        // // load tensorflow model
        // let buf = std::fs::read("/root/workspace/wasmedge-ml/docs/add.pb").unwrap();
        // let vec = [&buf];
        // let ptr = vec.as_ptr();
        // let ptr = ptr as i64;

        // // ! debug
        // println!("ptr: {:#X}", ptr);

        // // run target wasm module
        // let params = vec![WasmEdgeValueGenI64(ptr), WasmEdgeValueGenI32(1)];
        // let mut out: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();
        // let result = vm_ctx.run_wasm_from_file(
        //     "/root/workspace/examples/using_add/target/wasm32-wasi/debug/using_add.wasm",
        //     "consume_load",
        //     params.as_slice(),
        //     &mut out,
        // );
        // assert!(result.is_ok());

        // let values = result.unwrap();
        // assert_eq!(values.len(), 1);
        // assert_eq!(WasmEdgeValueGetI32(values[0]), 1);
    }

    #[test]
    fn test_wasmedge_run_wasm() {
        let mut conf_ctx = ConfigureContext::create();
        conf_ctx.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        conf_ctx
            .add_host_registration(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
        let mut vm_ctx = VMContext::create(Some(&conf_ctx), None);

        // create tensorflow and tensorflowlite modules: mod name: "wasmedge_tensorflow", "wasmedge_tensorflowlite"
        let mut result: Result<u32, WasmEdgeError>;
        let tensorflow_mod = ImportObjectContext::create_tensorflow_import_object();
        result = vm_ctx.register_module_from_import_object(&tensorflow_mod);
        assert!(result.is_ok());
        let tensorflowlite_mod = ImportObjectContext::create_tensorflowlite_import_object();
        result = vm_ctx.register_module_from_import_object(&tensorflowlite_mod);
        assert!(result.is_ok());
        // check the registered function
        let result = vm_ctx
            .function_type_registered("wasmedge_tensorflow", "wasmedge_tensorflow_create_session");
        assert!(result.is_some());
        let result = vm_ctx.function_type_registered(
            "wasmedge_tensorflowlite",
            "wasmedge_tensorflowlite_create_session",
        );
        assert!(result.is_some());

        // register wasmedge_wasi_nn.wasm module
        let res = vm_ctx.register_module_from_file(
            "calculator",
            "/root/workspace/wasmedge-ml/wasmedge-wasi-nn/target/wasm32-wasi/debug/wasmedge_wasi_nn.wasm",
        );
        assert!(res.is_ok());

        // register using_add.wasm module
        let mod_name = "using_add";
        let result = vm_ctx.register_module_from_file(
            mod_name,
            "/root/workspace/examples/using_add/target/wasm32-wasi/debug/using_add.wasm",
        );
        assert!(result.is_ok());

        // call consume_add function in registered using_add.wasm module
        let func_name = "consume_add";
        let params = vec![WasmEdgeValueGenI32(2), WasmEdgeValueGenI32(8)];
        let mut out: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();
        let result = vm_ctx.execute_registered(mod_name, func_name, params.as_slice(), &mut out);
        assert!(result.is_ok());

        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(WasmEdgeValueGetI32(values[0]), 10);

        {
            // run consume_add function in using_add.wasm
            let buf = std::fs::read("/root/workspace/wasmedge-ml/docs/add.pb").unwrap();
            let buf2 = std::fs::read("/root/workspace/mtcnn/mtcnn.pb").unwrap();
            let vec = [&buf];
            let ptr = vec.as_ptr();
            println!("ptr: {:?}", ptr);

            // let xml = std::fs::read_to_string("fixture/model.xml")
            //     .unwrap()
            //     .into_bytes();
            // let weights = std::fs::read("fixture/model.bin").unwrap();
            // let x = &[&xml, &weights];

            let func_name = "consume_load";
            let params = vec![WasmEdgeValueGenI64(ptr as i64), WasmEdgeValueGenI32(1)];
            let mut out: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();
            let result =
                vm_ctx.execute_registered(mod_name, func_name, params.as_slice(), &mut out);
            assert!(result.is_ok());

            // let values = result.unwrap();
            // assert_eq!(values.len(), 1);
            // assert_eq!(WasmEdgeValueGetI32(values[0]), 10);
        }
    }

    #[test]
    fn test_wasmedge_store() {
        // create contexts
        let mut conf = ConfigureContext::create();
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
        conf.add_host_registration(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
        conf.add_proposal(WasmEdgeProposal::WasmEdge_Proposal_ReferenceTypes);
        let mut store = StoreContext::create();
        let mod_name = vec!["module", "extern", "no-such-module"];
        let err_name = "invalid-instance-name";

        // Store list exports before instantiation
        assert_eq!(store.list_function_len(), 0);
        assert_eq!(store.list_table_len(), 0);
        assert_eq!(store.list_memory_len(), 0);
        assert_eq!(store.list_global_len(), 0);
        assert_eq!(store.list_function_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_table_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_memory_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_global_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_module_len(), 0);

        // Register host module and instantiate wasm module
        let res = create_extern_module("extern", false);
        assert!(res.is_some());
        let imp_obj = res.unwrap();
        assert!(!imp_obj.is_null());
        let res = load_module(&conf);
        assert!(res.is_some());
        let mut ast_mod = res.unwrap();
        assert!(!ast_mod.into_raw().is_null());
        assert!(validate_module(&conf, &ast_mod));
        assert!(instantiate_module(&conf, &mut store, &ast_mod, imp_obj));

        // unsafe { we_ffi::WasmEdge_ImportObjectDelete(imp_obj) };
    }

    fn create_extern_module(
        name: &str,
        is_wrap: bool,
    ) -> Option<*mut we_ffi::WasmEdge_ImportObjectContext> {
        unsafe {
            // let host_name = WasmEdgeString::from_str(name).unwrap().into_raw();
            let x = CString::new(name).unwrap();
            let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            let imp_obj = we_ffi::WasmEdge_ImportObjectCreate(host_name, ptr::null_mut());

            let param = [
                WasmEdgeValType::WasmEdge_ValType_ExternRef,
                WasmEdgeValType::WasmEdge_ValType_I32,
            ];
            let result = [WasmEdgeValType::WasmEdge_ValType_I32];
            let host_ftype = FunctionTypeContext::create(Some(&param), &result);

            // Add host function "func-add"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_add), 0);
            println!("host_func: {:?}", host_func);
            // let res = HostFunctionContext::create(host_ftype, Some(extern_add), 0);
            // assert!(res.is_some());
            // let host_func = res.unwrap();
            // println!("host_func.raw: {:?}", host_func.raw);
            let host_name = WasmEdgeString::from_str("func-add").unwrap();
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name.raw, host_func);

            // let host_func =
            //     we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_add), 0);
            // let x = CString::new("func-add").unwrap();
            // let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            // we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name, host_func);

            // add host function "func-sub"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_sub), 0);
            let x = CString::new("func-sub").unwrap();
            let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name, host_func);

            // add host function "func-mul"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_mul), 0);
            let x = CString::new("func-mul").unwrap();
            let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name, host_func);

            // add host function "func-div"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_div), 0);
            let x = CString::new("func-div").unwrap();
            let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name, host_func);

            let param = [
                WasmEdgeValType::WasmEdge_ValType_ExternRef,
                WasmEdgeValType::WasmEdge_ValType_I32,
            ];
            let result = [WasmEdgeValType::WasmEdge_ValType_I32];
            let host_ftype = FunctionTypeContext::create(None, &result);

            // add host function "func-term"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_term), 0);
            let x = CString::new("func-term").unwrap();
            let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name, host_func);

            // add host function "func-fail"
            let host_func =
                we_ffi::WasmEdge_HostFunctionCreate(host_ftype.raw, Some(extern_fail), 0);
            let x = CString::new("func-fail").unwrap();
            let host_name = we_ffi::WasmEdge_StringCreateByCString(x.as_ptr());
            we_ffi::WasmEdge_ImportObjectAddHostFunction(imp_obj, host_name, host_func);

            Some(imp_obj)
        }
        // // create import object
        // let mut imp_obj = ImportObjectContext::create(name, ptr::null_mut())?;

        // let param = [
        //     WasmEdgeValType::WasmEdge_ValType_ExternRef,
        //     WasmEdgeValType::WasmEdge_ValType_I32,
        // ];
        // let result = [WasmEdgeValType::WasmEdge_ValType_I32];
        // let host_ftype = WasmEdgeFunctionTypeContext::create(&param, &result);

        // // add host function "func-add"
        // let host_name = "func-add";
        // let mut host_func: WasmEdgeHostFunctionContext = if is_wrap {
        //     // WasmEdgeHostFunctionContext::create_binding(
        //     //     &host_ftype,
        //     //     extern_wrap,
        //     //     &extern_add as *mut std::os::raw::c_int as *mut std::os::raw::c_void,
        //     //     0,
        //     // )
        //     todo!()
        // } else {
        //     WasmEdgeHostFunctionContext::create(&host_ftype, Some(extern_add), 0)
        // };
        // imp_obj.add_host_function(host_name, &mut host_func);
        // Some(imp_obj)
    }

    fn load_module(conf: &ConfigureContext) -> Option<ASTModuleContext> {
        let mut module = ASTModuleContext::default();
        let mut loader = LoaderContext::create(conf)?;
        loader.parse_from_file(&mut module, TPATH);
        Some(module)
    }

    fn validate_module(conf: &ConfigureContext, ast_mod: &ASTModuleContext) -> bool {
        let res = Validator::create(conf);
        match res {
            None => false,
            Some(mut validator) => {
                let res = validator.validate(ast_mod);
                match res {
                    Err(_) => false,
                    Ok(_) => true,
                }
            }
        }
    }

    fn instantiate_module(
        conf: &ConfigureContext,
        store: &mut StoreContext,
        ast_mod: &ASTModuleContext,
        imp_obj: *const we_ffi::WasmEdge_ImportObjectContext,
    ) -> bool {
        let res = InterpreterContext::create(Some(conf), None);
        if res.is_none() {
            return false;
        }
        let mut interp = res.unwrap();
        if !interp.register_import_object_module(store, imp_obj) {
            return false;
        }
        // if !interp.register_ast_module(store, ast_mod, "module") {
        //     return false;
        // }
        // if !interp.instantiate(store, ast_mod) {
        //     return false;
        // }
        true
    }

    #[no_mangle]
    unsafe extern "C" fn extern_add(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 + val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_sub(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 - val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_mul(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 * val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_div(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let params = std::slice::from_raw_parts(params, 2);
        let val1 = *(WasmEdgeValueGetExternRef(params[0]) as *const ::std::os::raw::c_int);
        let val2 = WasmEdgeValueGetI32(params[1]);
        let res = WasmEdgeValueGenI32(val1 / val2);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 0 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_term(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let res = WasmEdgeValueGenI32(1234);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 1 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_fail(
        data: *mut std::os::raw::c_void,
        mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let res = WasmEdgeValueGenI32(5678);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 2 }
    }
}
