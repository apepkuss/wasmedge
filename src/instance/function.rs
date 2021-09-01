use crate::error::{WasmEdgeError, WasmEdgeResult};
use crate::{context::store::StoreContext, types::WasmEdgeValType};
use std::marker::PhantomData;
use std::{mem, ptr};
use wasmedge_sys::ffi as we_ffi;

pub struct FunctionTypeContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_FunctionTypeContext,
}
impl FunctionTypeContext {
    pub fn create(
        params: Option<&[WasmEdgeValType]>,
        returns: Option<&[WasmEdgeValType]>,
    ) -> Option<FunctionTypeContext> {
        let (param_len, params) = match params {
            Some(params) => (params.len(), params.as_ptr()),
            None => (0, ptr::null()),
        };
        let (ret_len, returns) = match returns {
            Some(returns) => (returns.len(), returns.as_ptr()),
            None => (0, ptr::null()),
        };

        let raw = unsafe {
            we_ffi::WasmEdge_FunctionTypeCreate(
                params as *const _,
                param_len as u32,
                returns as *const _,
                ret_len as u32,
            )
        };

        match raw.is_null() {
            true => None,
            false => Some(FunctionTypeContext { raw }),
        }
    }

    pub fn parameters_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_FunctionTypeGetParametersLength(self.raw) as usize }
    }

    pub fn parameters(
        &self,
        buf: &mut [mem::MaybeUninit<WasmEdgeValType>],
    ) -> WasmEdgeResult<(usize, Vec<WasmEdgeValType>)> {
        let max_len = self.parameters_len();
        match 0 < buf.len() && buf.len() <= max_len {
            true => {
                let len = unsafe {
                    we_ffi::WasmEdge_FunctionTypeGetParameters(
                        self.raw,
                        buf.as_mut_ptr() as *mut _,
                        buf.len() as u32,
                    )
                };
                let ty_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])
                };
                let mut val_types = vec![];
                for ty in ty_vec {
                    val_types.push(ty.clone());
                }

                Ok((len as usize, val_types))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'memory_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }

    pub fn returns_len(&self) -> usize {
        unsafe { we_ffi::WasmEdge_FunctionTypeGetReturnsLength(self.raw) as usize }
    }

    pub fn returns(
        &self,
        buf: &mut [mem::MaybeUninit<WasmEdgeValType>],
    ) -> WasmEdgeResult<(usize, Vec<WasmEdgeValType>)> {
        let max_len = self.returns_len();
        match 0 < buf.len() && buf.len() <= max_len {
            true => {
                let len = unsafe {
                    we_ffi::WasmEdge_FunctionTypeGetReturns(
                        self.raw,
                        buf.as_mut_ptr() as *mut _,
                        buf.len() as u32,
                    )
                };
                let ty_vec = unsafe {
                    mem::MaybeUninit::slice_assume_init_ref(&buf[..buf.len()])
                };
                let mut val_types = vec![];
                for ty in ty_vec {
                    val_types.push(ty.clone());
                }

                Ok((len as usize, val_types))
            }
            false => Err(WasmEdgeError::from(format!(
                "The length of the argument 'memory_names' should be between 1 and the max length ({}).",
                max_len
            ))),
        }
    }
}
impl Drop for FunctionTypeContext {
    fn drop(&mut self) {
        unsafe { we_ffi::WasmEdge_FunctionTypeDelete(self.raw) }
    }
}

pub struct HostFunctionContext {
    pub(crate) raw: *mut we_ffi::WasmEdge_HostFunctionContext,
}
impl HostFunctionContext {
    pub fn create(
        func_type: &FunctionTypeContext,
        host_func: we_ffi::WasmEdge_HostFunc_t,
        cost: u64,
    ) -> Option<HostFunctionContext> {
        let raw = unsafe { we_ffi::WasmEdge_HostFunctionCreate(func_type.raw, host_func, cost) };
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

pub struct FunctionInstanceContext<'a> {
    pub(crate) raw: *mut we_ffi::WasmEdge_FunctionInstanceContext,
    pub(crate) _marker: PhantomData<&'a StoreContext<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::import_object::ImportObjectContext, context::vm::VMContext, types::*, value::*,
    };
    use std::mem;

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
        let params = [
            WasmEdgeValType::WasmEdge_ValType_I32,
            WasmEdgeValType::WasmEdge_ValType_I32,
        ];
        let returns = [WasmEdgeValType::WasmEdge_ValType_I32];
        let result = FunctionTypeContext::create(Some(&params), Some(&returns));
        assert!(result.is_some());
        let func_type = result.unwrap();
        assert!(!func_type.raw.is_null());
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
        let result = func_type.returns(&mut out);
        let (size, list) = result.unwrap();
        println!("size of returns: {}", size);
        println!("len of list: {}", list.len());
        println!("result value: {:?}", list[0]);
    }

    #[test]
    fn test_wasmedge_function_type() {
        let params = vec![
            WasmEdgeValType::WasmEdge_ValType_I32,
            WasmEdgeValType::WasmEdge_ValType_I64,
            WasmEdgeValType::WasmEdge_ValType_ExternRef,
            WasmEdgeValType::WasmEdge_ValType_V128,
            WasmEdgeValType::WasmEdge_ValType_F64,
            WasmEdgeValType::WasmEdge_ValType_F32,
        ];
        let returns = vec![
            WasmEdgeValType::WasmEdge_ValType_FuncRef,
            WasmEdgeValType::WasmEdge_ValType_ExternRef,
            WasmEdgeValType::WasmEdge_ValType_V128,
        ];
        let result = FunctionTypeContext::create(Some(&params), Some(&returns));
        assert!(result.is_some());
        let func_type = result.unwrap();
        assert!(!func_type.raw.is_null());
        assert_eq!(func_type.parameters_len(), 6 as usize);
        assert_eq!(func_type.returns_len(), 3 as usize);

        let mut buf1 = mem::MaybeUninit::<WasmEdgeValType>::uninit_array::<6>();
        let result = func_type.parameters(&mut buf1);
        assert!(result.is_ok());
        let (len, val_types) = result.unwrap();
        assert_eq!(len, 6);
        assert_eq!(params, val_types);

        let mut buf2 = mem::MaybeUninit::<WasmEdgeValType>::uninit_array::<2>();
        let result = func_type.parameters(&mut buf2);
        assert!(result.is_ok());
        let (len, val_types) = result.unwrap();
        assert_eq!(len, 6);
        assert_eq!(&params[..2], &val_types);

        let mut buf1 = mem::MaybeUninit::<WasmEdgeValType>::uninit_array::<3>();
        let result = func_type.returns(&mut buf1);
        assert!(result.is_ok());
        let (len, val_types) = result.unwrap();
        assert_eq!(len, 3);
        assert_eq!(returns, val_types);
        let mut buf1 = mem::MaybeUninit::<WasmEdgeValType>::uninit_array::<2>();
        let result = func_type.returns(&mut buf1);
        assert!(result.is_ok());
        let (len, val_types) = result.unwrap();
        assert_eq!(len, 3);
        assert_eq!(&returns[..2], &val_types);

        let result = FunctionTypeContext::create(None, None);
        assert!(result.is_some());
        let func_type = result.unwrap();
        assert!(!func_type.raw.is_null());
    }
}
