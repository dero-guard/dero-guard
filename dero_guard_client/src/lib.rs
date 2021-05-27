use std::sync::Mutex;

use neon::prelude::*;
use lazy_static::lazy_static;
// use tokio::runtime::Runtime;

mod service;
mod vpn;

use service::Service;
use vpn::VPN;

lazy_static! {
    static ref SERVICE: Mutex<Service> = Mutex::new(futures::executor::block_on(
        service::Service::new("http://127.0.0.1:40404/json_rpc", VPN::new().unwrap())
    ).unwrap());

    // static ref RUNTIME: Runtime = Runtime::new().unwrap();
}

fn providers(mut cx: FunctionContext) -> JsResult<JsArray> {
    let service = SERVICE.lock().unwrap();

    let js_array = JsArray::new(&mut cx, 3);
    let providers = service.get_providers();

    for (i, provider) in providers.into_iter().enumerate() {
        let object = JsObject::new(&mut cx);

        let location = cx.string(&provider.location);
        let name = cx.string(&provider.name);
        let rate = cx.number(provider.rate);
        let public_key = cx.string(&provider.public_key);

        object.set(&mut cx, "location", location)?;
        object.set(&mut cx, "name", name)?;
        object.set(&mut cx, "rate", rate)?;
        object.set(&mut cx, "public_key", public_key)?;

        js_array.set(&mut cx, i as u32, object)?;
    }

    Ok(js_array)
}

fn refill(mut cx: FunctionContext) -> JsResult<JsString> {
    let mut service = SERVICE.lock().unwrap();

    let public_key = cx.argument::<JsString>(0)?.value(&mut cx);
    let amount = cx.argument::<JsNumber>(1)?.value(&mut cx);

    // let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);

    /*RUNTIME.spawn(async move {
        service.connect(public_key, (amount * 100000f64) as u64).await.unwrap();
        let address = cx.string(result);
        address.
    });*/
    let result = futures::executor::block_on(service.connect(public_key, (amount * 100000f64) as u64)).unwrap();
    let address = cx.string(result);

    Ok(address)
    //Ok(cx.undefined())
}

fn disconnect(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut service = SERVICE.lock().unwrap();
    service.disconnect().unwrap();

    Ok(cx.undefined())
}

fn bandwidth(mut cx: FunctionContext) -> JsResult<JsObject> {
    let service = SERVICE.lock().unwrap();

    let public_key = cx.argument::<JsString>(0)?.value(&mut cx);
    let bandwidth = service.get_bandwidth(public_key).unwrap();

    let object = JsObject::new(&mut cx);

    let download = cx.number(bandwidth.download as f64);
    let upload = cx.number(bandwidth.upload as f64);

    object.set(&mut cx, "download", download)?;
    object.set(&mut cx, "upload", upload)?;

    Ok(object)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("providers", providers)?;
    cx.export_function("refill", refill)?;
    cx.export_function("disconnect", disconnect)?;
    cx.export_function("bandwidth", bandwidth)?;

    Ok(())
}