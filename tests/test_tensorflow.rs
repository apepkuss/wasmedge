#![feature(maybe_uninit_uninit_array, maybe_uninit_extra, maybe_uninit_slice)]

use std::mem;
use wasmedge::{
    context::{configure::ConfigureContext, import_object::ImportObjectContext, vm::VMContext},
    error::WasmEdgeError,
    types::*,
    value::*,
};

#[test]
fn test_wasmedge_tensorflow() {
    let mut conf_ctx = ConfigureContext::create();
    conf_ctx.add_host_registration(HostRegistration::WasmEdge_HostRegistration_Wasi);
    conf_ctx.add_host_registration(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
    let result = VMContext::create(Some(&conf_ctx), None);
    assert!(result.is_some());
    let mut vm = result.unwrap();

    // create tensorflow and tensorflowlite modules: mod name: "wasmedge_tensorflow", "wasmedge_tensorflowlite"
    let mut result: Result<(), WasmEdgeError>;
    let tensorflow_mod = ImportObjectContext::create_tensorflow_import_object();
    result = vm.register_module_from_import_object(&tensorflow_mod);
    assert!(result.is_ok());
    let tensorflowlite_mod = ImportObjectContext::create_tensorflowlite_import_object();
    result = vm.register_module_from_import_object(&tensorflowlite_mod);
    assert!(result.is_ok());

    // check the registered function: wasmedge_tensorflow_create_session
    let result =
        vm.function_type_registered("wasmedge_tensorflow", "wasmedge_tensorflow_create_session");
    assert!(result.is_some());
    let func_type = result.unwrap();
    let param_len = func_type.parameters_len();
    println!(
        "param len of wasmedge_tensorflow_create_session func: {}",
        param_len
    );
    let result = vm.function_type_registered(
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
    let result = vm.function_type_registered(
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
        let wasi_module = vm.import_object(HostRegistration::WasmEdge_HostRegistration_Wasi);
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
    let result = vm.register_module_from_file(
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
    conf_ctx.add_host_registration(HostRegistration::WasmEdge_HostRegistration_WasmEdge_Process);
    let result = VMContext::create(Some(&conf_ctx), None);
    assert!(result.is_some());
    let mut vm = result.unwrap();

    // create tensorflow and tensorflowlite modules: mod name: "wasmedge_tensorflow", "wasmedge_tensorflowlite"
    let mut result: Result<(), WasmEdgeError>;
    let tensorflow_mod = ImportObjectContext::create_tensorflow_import_object();
    result = vm.register_module_from_import_object(&tensorflow_mod);
    assert!(result.is_ok());
    let tensorflowlite_mod = ImportObjectContext::create_tensorflowlite_import_object();
    result = vm.register_module_from_import_object(&tensorflowlite_mod);
    assert!(result.is_ok());
    // check the registered function
    let result =
        vm.function_type_registered("wasmedge_tensorflow", "wasmedge_tensorflow_create_session");
    assert!(result.is_some());
    let result = vm.function_type_registered(
        "wasmedge_tensorflowlite",
        "wasmedge_tensorflowlite_create_session",
    );
    assert!(result.is_some());

    // register wasmedge_wasi_nn.wasm module
    let res = vm.register_module_from_file(
        "calculator",
        "/root/workspace/wasmedge-ml/wasmedge-wasi-nn/target/wasm32-wasi/debug/wasmedge_wasi_nn.wasm",
    );
    assert!(res.is_ok());

    // register using_add.wasm module
    let mod_name = "using_add";
    let result = vm.register_module_from_file(
        mod_name,
        "/root/workspace/examples/using_add/target/wasm32-wasi/debug/using_add.wasm",
    );
    assert!(result.is_ok());

    // call consume_add function in registered using_add.wasm module
    let func_name = "consume_add";
    let params = vec![WasmEdgeValueGenI32(2), WasmEdgeValueGenI32(8)];
    let mut out: [mem::MaybeUninit<WasmEdgeValue>; 1] = mem::MaybeUninit::uninit_array();
    let result = vm.execute_registered(mod_name, func_name, params.as_slice(), &mut out);
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
        let result = vm.execute_registered(mod_name, func_name, params.as_slice(), &mut out);
        assert!(result.is_ok());

        // let values = result.unwrap();
        // assert_eq!(values.len(), 1);
        // assert_eq!(WasmEdgeValueGetI32(values[0]), 10);
    }
}
