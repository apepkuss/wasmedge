use crate::{
    context::vm::VMContext,
    error::{WasmEdgeError, WasmEdgeResult},
    instance::{
        function::FunctionInstanceContext, global::GlobalInstanceContext,
        memory::MemoryInstanceContext, table::TableInstanceContext,
    },
    types::WasmEdgeString,
};
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem;
use wasmedge_sys::ffi as we_ffi;

pub struct StoreContext<'vm> {
    pub(crate) raw: *mut we_ffi::WasmEdge_StoreContext,
    pub(crate) _marker: PhantomData<&'vm VMContext>,
    pub(crate) _drop: bool,
}
impl<'vm> StoreContext<'vm> {
    pub fn create() -> Self {
        StoreContext {
            raw: unsafe { we_ffi::WasmEdge_StoreCreate() },
            _marker: PhantomData,
            _drop: true,
        }
    }

    pub fn find_function(&self, func_name: &str) -> Option<FunctionInstanceContext> {
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_StoreFindFunction(self.raw, func_name.raw) };
        match raw.is_null() {
            true => None,
            false => Some(FunctionInstanceContext { raw }),
        }
    }

    pub fn find_function_registered(
        &self,
        mod_name: &str,
        func_name: &str,
    ) -> Option<FunctionInstanceContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let func_name = WasmEdgeString::from_str(func_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", func_name).as_str());
        let raw = unsafe {
            we_ffi::WasmEdge_StoreFindFunctionRegistered(self.raw, mod_name.raw, func_name.raw)
        };
        match raw.is_null() {
            true => None,
            false => Some(FunctionInstanceContext { raw }),
        }
    }

    pub fn find_table(&self, table_name: &str) -> Option<TableInstanceContext> {
        let table_name = WasmEdgeString::from_str(table_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", table_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_StoreFindTable(self.raw, table_name.raw) };
        match raw.is_null() {
            true => None,
            false => Some(TableInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn find_table_registered(
        &self,
        mod_name: &str,
        table_name: &str,
    ) -> Option<TableInstanceContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let table_name = WasmEdgeString::from_str(table_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", table_name).as_str());
        let raw = unsafe {
            we_ffi::WasmEdge_StoreFindTableRegistered(self.raw, mod_name.raw, table_name.raw)
        };
        match raw.is_null() {
            true => None,
            false => Some(TableInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn find_memory(&self, mem_name: &str) -> Option<MemoryInstanceContext> {
        let mem_name = WasmEdgeString::from_str(mem_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_StoreFindMemory(self.raw, mem_name.raw) };
        match raw.is_null() {
            true => None,
            false => Some(MemoryInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn find_memory_registered(
        &'vm self,
        mod_name: &str,
        mem_name: &str,
    ) -> Option<MemoryInstanceContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let mem_name = WasmEdgeString::from_str(mem_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mem_name).as_str());
        let raw = unsafe {
            we_ffi::WasmEdge_StoreFindMemoryRegistered(self.raw, mod_name.raw, mem_name.raw)
        };
        match raw.is_null() {
            true => None,
            false => Some(MemoryInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn find_global(&self, global_name: &str) -> Option<GlobalInstanceContext> {
        let global_name = WasmEdgeString::from_str(global_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", global_name).as_str());
        let raw = unsafe { we_ffi::WasmEdge_StoreFindGlobal(self.raw, global_name.raw) };
        match raw.is_null() {
            true => None,
            false => Some(GlobalInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn find_global_registered(
        &self,
        mod_name: &str,
        global_name: &str,
    ) -> Option<GlobalInstanceContext> {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        let global_name = WasmEdgeString::from_str(global_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", global_name).as_str());
        let raw = unsafe {
            we_ffi::WasmEdge_StoreFindGlobalRegistered(self.raw, mod_name.raw, global_name.raw)
        };
        match raw.is_null() {
            true => None,
            false => Some(GlobalInstanceContext {
                raw,
                _marker: PhantomData,
                _drop: false,
            }),
        }
    }

    pub fn list_function_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListFunctionLength(self.raw) as usize }
    }

    pub fn list_function(
        &self,
        func_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_function_len();
        match 0 < func_names.len() && func_names.len() <= max_len {
            true => {
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListFunction(
                        self.raw,
                        func_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        func_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&func_names[..func_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'func_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn list_function_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe {
            we_ffi::WasmEdge_StoreListFunctionRegisteredLength(self.raw, mod_name.raw) as usize
        }
    }

    pub fn list_function_registered(
        &self,
        mod_name: &str,
        func_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_function_registered_len(mod_name);
        match 0 < func_names.len() && func_names.len() <= max_len {
            true => {
                let mod_name = WasmEdgeString::from_str(mod_name).expect(
                    format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str(),
                );
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListFunctionRegistered(
                        self.raw,
                        mod_name.raw,
                        func_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        func_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&func_names[..func_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'func_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn list_table_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListTableLength(self.raw) as usize }
    }

    pub fn list_table(
        &self,
        table_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_global_len();
        match 0 < table_names.len() && table_names.len() <= max_len {
            true => {
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListTable(
                        self.raw,
                        table_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        table_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&table_names[..table_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'table_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn list_table_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe { we_ffi::WasmEdge_StoreListTableRegisteredLength(self.raw, mod_name.raw) as usize }
    }

    pub fn list_table_registered(
        &self,
        mod_name: &str,
        table_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_global_registered_len(mod_name);
        match 0 < table_names.len() && table_names.len() <= max_len {
            true => {
                let mod_name = WasmEdgeString::from_str(mod_name).expect(
                    format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str(),
                );
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListTableRegistered(
                        self.raw,
                        mod_name.raw,
                        table_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        table_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&table_names[..table_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'table_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn list_global_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_StoreListGlobalLength(self.raw) as usize }
    }

    pub fn list_global(
        &self,
        global_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_global_len();
        match 0 < global_names.len() && global_names.len() <= max_len {
            true => {
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListGlobal(
                        self.raw,
                        global_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        global_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&global_names[..global_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    // let x = unsafe { std::ffi::CString::from_raw(s.Buf as *mut _) };
                    // names.push(x.to_string_lossy().into_owned());
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'global_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn list_global_registered_len(&self, mod_name: &str) -> usize {
        let mod_name = WasmEdgeString::from_str(mod_name)
            .expect(format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str());
        unsafe { we_ffi::WasmEdge_StoreListGlobalRegisteredLength(self.raw, mod_name.raw) as usize }
    }

    pub fn list_global_registered(
        &self,
        mod_name: &str,
        global_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_global_registered_len(mod_name);
        match 0 < global_names.len() && global_names.len() <= max_len {
            true => {
                let mod_name = WasmEdgeString::from_str(mod_name).expect(
                    format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str(),
                );
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListGlobalRegistered(
                        self.raw,
                        mod_name.raw,
                        global_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        global_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&global_names[..global_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    // let x = unsafe { std::ffi::CString::from_raw(s.Buf as *mut _) };
                    // names.push(x.to_string_lossy().into_owned());
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'global_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn list_memory_len(&self) -> usize {
        let len = unsafe { we_ffi::WasmEdge_StoreListMemoryLength(self.raw) };
        len as usize
    }

    pub fn list_memory(
        &self,
        memory_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_memory_len();
        match 0 < memory_names.len() && memory_names.len() <= max_len {
            true => {
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListMemory(
                        self.raw,
                        memory_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        memory_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&memory_names[..memory_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'memory_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
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
        memory_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_memory_registered_len(mod_name);
        match 0 < memory_names.len() && memory_names.len() <= max_len {
            true => {
                let mod_name = WasmEdgeString::from_str(mod_name).expect(
                    format!("Failed to create WasmEdgeString from '{}'", mod_name).as_str(),
                );
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListMemoryRegistered(
                        self.raw,
                        mod_name.raw,
                        memory_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        memory_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&memory_names[..memory_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    // let x = unsafe { std::ffi::CString::from_raw(s.Buf as *mut _) };
                    // names.push(x.to_string_lossy().into_owned());
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'memory_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn list_module_len(&self) -> usize {
        let len = unsafe { we_ffi::WasmEdge_StoreListModuleLength(self.raw) };
        len as usize
    }

    pub fn list_module(
        &self,
        mod_names: &mut [mem::MaybeUninit<we_ffi::WasmEdge_String>],
    ) -> WasmEdgeResult<(usize, Vec<String>)> {
        let max_len = self.list_module_len();
        match 0 < mod_names.len() && mod_names.len() <= max_len {
            true => {
                let len = unsafe {
                    we_ffi::WasmEdge_StoreListModule(
                        self.raw,
                        mod_names.as_mut_ptr() as *mut we_ffi::WasmEdge_String,
                        mod_names.len() as u32,
                    )
                };
                let s_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&mod_names[..mod_names.len()])
                };
                let mut names = vec![];
                for s in s_vec {
                    // let x = unsafe { std::ffi::CString::from_raw(s.Buf as *mut _) };
                    // names.push(x.to_string_lossy().into_owned());
                    let slice = unsafe { CStr::from_ptr(s.Buf as *const _) };
                    names.push(slice.to_string_lossy().into_owned());
                }

                Ok((len as usize, names))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'mod_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }
}
impl<'vm> Drop for StoreContext<'vm> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if self._drop {
                unsafe { we_ffi::WasmEdge_StoreDelete(self.raw) }
            } else {
                self.raw = std::ptr::null_mut();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::function::HostFunctionContext;
    use crate::{
        context::{
            ast::ASTModuleContext, configure::ConfigureContext, import_object::ImportObjectContext,
            interpreter::InterpreterContext, loader::LoaderContext, validator::Validator,
        },
        instance::function::FunctionTypeContext,
        types::*,
        value::*,
    };
    use std::ptr;
    use wasmedge_sys::ffi as we_ffi;

    const TPATH: &str = "/root/workspace/wasmedge-ml/wasmedge/tests/data/test.wasm";

    #[test]
    fn test_context_store() {
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
        let res = create_extern_module("extern");
        assert!(res.is_some());
        let imp_obj = res.unwrap();
        assert!(!imp_obj.raw.is_null());
        let res = load_module(&conf);
        assert!(res.is_some());
        let ast_mod = res.unwrap();
        assert!(!ast_mod.raw.is_null());
        assert!(validate_module(&conf, &ast_mod));
        assert!(instantiate_module(&conf, &mut store, &ast_mod, &imp_obj));

        // Store list function exports
        assert_eq!(store.list_function_len(), 11);
        let mut func_names = mem::MaybeUninit::uninit_array::<4>();
        let result = store.list_function(&mut func_names);
        assert!(result.is_ok());
        let (len, func_names) = result.unwrap();
        assert_eq!(len, 11);
        for name in func_names.into_iter() {
            drop(name);
        }
        let mut func_names = mem::MaybeUninit::uninit_array::<11>();
        let result = store.list_function(&mut func_names);
        assert!(result.is_ok());
        let (len, func_names) = result.unwrap();
        assert_eq!(len, 11);

        // store find function
        let res = store.find_function(&func_names[0]);
        assert!(res.is_some());

        // Store list function exports registered
        assert_eq!(store.list_function_registered_len(mod_name[0]), 11);
        assert_eq!(store.list_function_registered_len(mod_name[1]), 6);
        assert_eq!(store.list_function_registered_len(mod_name[2]), 0);
        let mut func_names = mem::MaybeUninit::uninit_array::<11>();
        let result = store.list_function_registered(mod_name[0], &mut func_names);
        assert!(result.is_ok());
        let (len, func_names) = result.unwrap();
        assert_eq!(len, 11);
        println!("func_names: {:?}", func_names);

        // Store find function registered
        assert!(store
            .find_function_registered(mod_name[0], err_name)
            .is_none());

        // Store list table exports
        assert_eq!(store.list_table_len(), 2);
        let mut table_names = mem::MaybeUninit::uninit_array::<2>();
        let result = store.list_table(&mut table_names);
        assert!(result.is_ok());
        let (len, table_names) = result.unwrap();
        assert_eq!(len, 2);
        println!("table_names: {:?}", table_names);

        // Store find table
        assert!(store.find_table(&table_names[1]).is_some());
        assert!(store.find_table(err_name).is_none());

        // Store list table exports registered
        assert_eq!(store.list_table_registered_len(mod_name[0]), 2);
        assert_eq!(store.list_table_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_table_registered_len(mod_name[2]), 0);
        let mut table_names = mem::MaybeUninit::uninit_array::<2>();
        let result = store.list_table_registered(mod_name[0], &mut table_names);
        assert!(result.is_ok());
        let (len, table_names) = result.unwrap();
        assert_eq!(len, 2);
        println!("table_name: {:?}", table_names);

        // Store find table registered
        assert!(store
            .find_table_registered(mod_name[0], &table_names[0])
            .is_some());
        assert!(store
            .find_table_registered(mod_name[0], &table_names[1][..8])
            .is_some());
        assert!(store.find_table_registered(mod_name[0], err_name).is_none());

        // Store list memory exports
        assert_eq!(store.list_memory_len(), 1);
        let mut memory_names = mem::MaybeUninit::uninit_array::<1>();
        let result = store.list_memory(&mut memory_names);
        assert!(result.is_ok());
        let (len, memory_names) = result.unwrap();
        assert_eq!(len, 1);
        println!("memory_names: {:?}", memory_names);

        // Store find memory
        // assert!(store.find_memory(&memory_names[0]).is_some()); // ! error: caused by invalid utf-8 characters
        assert!(store.find_memory(&err_name).is_none());

        // Store list memory exports registered
        assert_eq!(store.list_memory_registered_len(mod_name[0]), 1);
        assert_eq!(store.list_memory_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_memory_registered_len(mod_name[2]), 0);
        let mut memory_names = mem::MaybeUninit::uninit_array::<1>();
        let result = store.list_memory_registered(mod_name[0], &mut memory_names);
        assert!(result.is_ok());
        let (len, memory_names) = result.unwrap();
        assert_eq!(len, 1);
        println!("memory_reg_names: {:?}", memory_names);

        // Store find memory registered
        // ! error: caused by invalid utf-8 characters
        // assert!(store
        //     .find_memory_registered(&mod_name[0], &memory_names[0])
        //     .is_some());
        assert!(store
            .find_memory_registered(&mod_name[0], &err_name)
            .is_none());
        assert!(store
            .find_memory_registered(&mod_name[2], &memory_names[0])
            .is_none());

        // Store list global exports
        assert_eq!(store.list_global_len(), 2);
        let mut global_names = mem::MaybeUninit::uninit_array::<15>();
        let result = store.list_global(&mut global_names);
        assert!(result.is_err());
        let mut global_names = mem::MaybeUninit::uninit_array::<2>();
        let result = store.list_global(&mut global_names);
        assert!(result.is_ok());
        let (len, global_names) = result.unwrap();
        assert_eq!(len, 2);
        println!("global_names: {:?}", global_names);

        // Store find global
        assert!(store.find_global(&global_names[0]).is_some());
        assert!(store.find_global(err_name).is_none());

        // Store list global exports registered
        assert_eq!(store.list_global_registered_len(mod_name[0]), 2);
        assert_eq!(store.list_global_registered_len(mod_name[1]), 0);
        assert_eq!(store.list_global_registered_len(mod_name[2]), 0);
        let mut global_names = mem::MaybeUninit::uninit_array::<2>();
        let result = store.list_global_registered(mod_name[0], &mut global_names);
        assert!(result.is_ok());
        let (len, global_names) = result.unwrap();
        assert_eq!(len, 2);

        // Store find global registered
        assert!(store
            .find_global_registered(mod_name[0], &global_names[1])
            .is_none());
        assert!(store
            .find_global_registered(mod_name[0], err_name)
            .is_none());
        assert!(store
            .find_global_registered(mod_name[2], &global_names[1])
            .is_none());

        // Store list module
        assert_eq!(store.list_module_len(), 2);
        let mut mod_names = mem::MaybeUninit::uninit_array::<1>();
        let result = store.list_module(&mut mod_names);
        assert!(result.is_ok());
        let (len, _mod_names) = result.unwrap();
        assert_eq!(len, 2);
        let mut mod_names = mem::MaybeUninit::uninit_array::<15>();
        let result = store.list_module(&mut mod_names);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.message,
            format!(
                "The length of the argument 'mod_names' should be between 1 and the max length ({}).",
                store.list_module_len()
            )
        );
    }

    fn create_extern_module(name: &str) -> Option<ImportObjectContext<'_>> {
        // create import object
        let result = ImportObjectContext::create(name, ptr::null_mut());
        assert!(result.is_some());
        let mut imp_obj = result.unwrap();

        let params = [
            WasmEdgeValType::WasmEdge_ValType_ExternRef,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let returns = [WasmEdgeValType::WasmEdge_ValType_I32];
        let result = FunctionTypeContext::create(Some(&params), Some(&returns));
        assert!(result.is_some());
        let host_ftype = result.unwrap();
        assert!(!host_ftype.raw.is_null());

        // add host function "func-add"
        let host_name = "func-add";
        let result = HostFunctionContext::create(&host_ftype, Some(extern_add), 0);
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-sub"
        let host_name = "func-sub";
        let result = HostFunctionContext::create(&host_ftype, Some(extern_sub), 0);
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-mul"
        let host_name = "func-mul";
        let result = HostFunctionContext::create(&host_ftype, Some(extern_mul), 0);
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-div"
        let host_name = "func-div";
        let result = HostFunctionContext::create(&host_ftype, Some(extern_div), 0);
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        let returns = [WasmEdgeValType::WasmEdge_ValType_I32];
        let result = FunctionTypeContext::create(None, Some(&returns));
        assert!(result.is_some());
        let host_ftype = result.unwrap();
        assert!(!host_ftype.raw.is_null());

        // add host function "func-term"
        let host_name = "func-term";
        let result = HostFunctionContext::create(&host_ftype, Some(extern_term), 0);
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        // add host function "func-fail"
        let host_name = "func-fail";
        let result = HostFunctionContext::create(&host_ftype, Some(extern_fail), 0);
        assert!(result.is_some());
        let mut host_func = result.unwrap();
        imp_obj.add_host_function(host_name, &mut host_func);

        Some(imp_obj)
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
            Some(validator) => {
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
        imp_obj: &ImportObjectContext,
    ) -> bool {
        let res = InterpreterContext::create(Some(conf), None);
        if res.is_none() {
            return false;
        }
        let mut interp = res.unwrap();
        if !interp.register_import_object_module(store, imp_obj).is_ok() {
            return false;
        }
        if !interp.register_ast_module(store, ast_mod, "module").is_ok() {
            return false;
        }
        if !interp.instantiate(store, ast_mod).is_ok() {
            return false;
        }
        true
    }

    #[no_mangle]
    unsafe extern "C" fn extern_add(
        _data: *mut std::os::raw::c_void,
        _mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
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
        _data: *mut std::os::raw::c_void,
        _mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
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
        _data: *mut std::os::raw::c_void,
        _mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
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
        _data: *mut std::os::raw::c_void,
        _mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
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
        _data: *mut std::os::raw::c_void,
        _mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        _params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let res = WasmEdgeValueGenI32(1234);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 1 }
    }

    #[no_mangle]
    unsafe extern "C" fn extern_fail(
        _data: *mut std::os::raw::c_void,
        _mem_ctx: *mut we_ffi::WasmEdge_MemoryInstanceContext,
        _params: *const WasmEdgeValue,
        returns: *mut WasmEdgeValue,
    ) -> we_ffi::WasmEdge_Result {
        let res = WasmEdgeValueGenI32(5678);
        returns.write(res);

        we_ffi::WasmEdge_Result { Code: 2 }
    }
}
