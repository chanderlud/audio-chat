// MIT License
// Copyright (c) oligamiq 2024

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software to deal in the Software without restriction, subject to the
// following conditions:

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.

use std::sync::OnceLock;
use kanal::{bounded, Receiver};
use wasm_bindgen::{prelude::Closure, JsCast as _};
use wasm_bindgen_futures::JsFuture;
use web_sys::BlobPropertyBag;

// https://zenn.dev/tetter/articles/web-realtime-audio-processing
// https://qiita.com/okaxaki/items/c807bdfe3e96d6ef7960

static WEB_INPUT: OnceLock<Receiver<Vec<f32>>> = OnceLock::new();
static SAMPLE_RATE: OnceLock<u32> = OnceLock::new();

pub(crate) struct WebAudioStream(pub(crate) Receiver<Vec<f32>>);

impl WebAudioStream {
    pub(crate) fn new() -> Self {
        Self(WEB_INPUT.get().unwrap().clone())
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        SAMPLE_RATE.get().unwrap().clone()
    }
}

struct OnWebStruct {
    receiver: Receiver<Vec<f32>>,
    sample_rate: Option<f32>,
    _audio_ctx: web_sys::AudioContext,
    _source: web_sys::MediaStreamAudioSourceNode,
    _media_devices: web_sys::MediaDevices,
    _stream: web_sys::MediaStream,
    _js_closure: Closure<dyn FnMut(wasm_bindgen::JsValue)>,
    _worklet_node: web_sys::AudioWorkletNode,
}

impl std::fmt::Debug for OnWebStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OnWebStruct")
    }
}

impl OnWebStruct {
    async fn new() -> Self {
        let audio_ctx = web_sys::AudioContext::new().unwrap();

        let sample_rate = audio_ctx.sample_rate();

        let media_devices = web_sys::window()
            .unwrap()
            .navigator()
            .media_devices()
            .unwrap();

        let constraints = web_sys::MediaStreamConstraints::new();

        let js_true = wasm_bindgen::JsValue::from(true);

        constraints.set_audio(&js_true);

        let stream = media_devices
            .get_user_media_with_constraints(&constraints)
            .unwrap();

        let stream = JsFuture::from(stream).await.unwrap();

        let stream = stream.dyn_into::<web_sys::MediaStream>().unwrap();

        let source = audio_ctx.create_media_stream_source(&stream).unwrap();

        // 明示的にresumeを呼ぶ
        JsFuture::from(audio_ctx.resume().unwrap()).await.unwrap();

        // Return about Float32Array
        // return first input's first channel's samples
        // https://developer.mozilla.org/ja/docs/Web/API/AudioWorkletProcessor/process
        let processor_js_code = r#"
            class MyProcessor extends AudioWorkletProcessor {
                process(inputs, outputs, parameters) {
                    this.port.postMessage(Float32Array.from(inputs[0][0]));

                    return true;
                }
            }

            registerProcessor('my-processor', MyProcessor);

            console.log('MyProcessor is registered');
        "#;

        let blob_parts = js_sys::Array::new();
        blob_parts.push(&wasm_bindgen::JsValue::from_str(processor_js_code));

        let type_: BlobPropertyBag = BlobPropertyBag::new();
        type_.set_type("application/javascript");

        let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &type_).unwrap();

        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

        let processor = audio_ctx
            .audio_worklet()
            .expect("Failed to get audio worklet")
            .add_module(&url)
            .unwrap();

        JsFuture::from(processor).await.unwrap();

        web_sys::Url::revoke_object_url(&url).unwrap();

        let worklet_node = web_sys::AudioWorkletNode::new(&audio_ctx, "my-processor")
            .expect("Failed to create audio worklet node");

        source.connect_with_audio_node(&worklet_node).unwrap();

        let (sender, receiver) = bounded(10);

        // Float32Array
        let js_closure = Closure::wrap(Box::new(move |msg: wasm_bindgen::JsValue| {
            let msg_event = msg.dyn_into::<web_sys::MessageEvent>().unwrap();

            let data = msg_event.data();

            let data: Vec<f32> = serde_wasm_bindgen::from_value(data).unwrap();
            sender.send(data).unwrap();
        }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);

        let js_func = js_closure.as_ref().unchecked_ref();

        worklet_node
            .port()
            .expect("Failed to get port")
            .set_onmessage(Some(js_func));

        OnWebStruct {
            receiver,
            sample_rate: Some(sample_rate),
            _audio_ctx: audio_ctx,
            _source: source,
            _media_devices: media_devices,
            _stream: stream,
            _js_closure: js_closure,
            _worklet_node: worklet_node,
        }
    }
}

impl Drop for OnWebStruct {
    fn drop(&mut self) {
        let _ = self._audio_ctx.close();
    }
}

pub(crate) async fn init_on_web_struct() {
    let on_web = OnWebStruct::new().await;

    let receiver = on_web.receiver.clone();

    WEB_INPUT.get_or_init(|| receiver);

    let sample_rate = on_web.sample_rate.unwrap();

    SAMPLE_RATE.get_or_init(|| sample_rate as u32);

    core::mem::forget(on_web);
}
