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
    pub(crate) raw: *mut we_ffi::WasmEdge_HostFunctionContext,
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
}
