// The following is a modified version of the code found at
// https://github.com/RustAudio/cpal/issues/813#issuecomment-2413007276

use std::sync::Arc;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_sync::{Condvar, Mutex};
use web_sys::BlobPropertyBag;

pub(crate) struct WebAudioWrapper {
    pub(crate) pair: Arc<(Mutex<Vec<f32>>, Condvar)>,
    pub(crate) sample_rate: f32,
    audio_ctx: web_sys::AudioContext,
    _source: web_sys::MediaStreamAudioSourceNode,
    _media_devices: web_sys::MediaDevices,
    _stream: web_sys::MediaStream,
    _js_closure: Closure<dyn FnMut(JsValue)>,
    _worklet_node: web_sys::AudioWorkletNode,
}

// TODO latency controls for WEB_INPUT and web output
impl WebAudioWrapper {
    pub(crate) async fn new() -> Result<Self, JsValue> {
        let audio_ctx = web_sys::AudioContext::new()?;
        let sample_rate = audio_ctx.sample_rate();

        let media_devices = web_sys::window().unwrap().navigator().media_devices()?;

        let constraints = web_sys::MediaStreamConstraints::new();

        let js_true = wasm_bindgen::JsValue::from(true);

        constraints.set_audio(&js_true);

        let stream = media_devices.get_user_media_with_constraints(&constraints)?;

        let stream = JsFuture::from(stream).await?;

        let stream = stream.dyn_into::<web_sys::MediaStream>()?;

        let source = audio_ctx.create_media_stream_source(&stream)?;

        JsFuture::from(audio_ctx.resume()?).await?;

        // Return about Float32Array
        // return first input's first channel's samples
        // https://developer.mozilla.org/ja/docs/Web/API/AudioWorkletProcessor/process
        let processor_js_code = r#"
            class AudioChatProcessor extends AudioWorkletProcessor {
                process(inputs, outputs, parameters) {
                    this.port.postMessage(Float32Array.from(inputs[0][0]));

                    return true;
                }
            }

            registerProcessor('audio-chat-processor', AudioChatProcessor);
        "#;

        let blob_parts = js_sys::Array::new();
        blob_parts.push(&JsValue::from_str(processor_js_code));

        let type_: BlobPropertyBag = BlobPropertyBag::new();
        type_.set_type("application/javascript");

        let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &type_)?;

        let url = web_sys::Url::create_object_url_with_blob(&blob)?;

        let processor = audio_ctx.audio_worklet()?.add_module(&url)?;

        JsFuture::from(processor).await?;

        web_sys::Url::revoke_object_url(&url)?;

        let worklet_node = web_sys::AudioWorkletNode::new(&audio_ctx, "audio-chat-processor")?;

        source.connect_with_audio_node(&worklet_node)?;

        let pair: Arc<(Mutex<Vec<f32>>, _)> = Arc::new((Mutex::default(), Condvar::new()));
        let pair_clone = Arc::clone(&pair);

        // Float32Array
        let js_closure = Closure::wrap(Box::new(move |msg: JsValue| {
            let msg_event = msg.dyn_into::<web_sys::MessageEvent>().unwrap();

            let data = msg_event.data();

            let data: Vec<f32> = serde_wasm_bindgen::from_value(data).unwrap();

            if let Ok(mut data_clone) = pair_clone.0.lock() {
                data_clone.extend(data);
                pair_clone.1.notify_one();
            }
        }) as Box<dyn FnMut(JsValue)>);

        let js_func = js_closure.as_ref().unchecked_ref();

        worklet_node.port()?.set_onmessage(Some(js_func));

        Ok(WebAudioWrapper {
            pair,
            sample_rate,
            audio_ctx,
            _source: source,
            _media_devices: media_devices,
            _stream: stream,
            _js_closure: js_closure,
            _worklet_node: worklet_node,
        })
    }
}

impl Drop for WebAudioWrapper {
    fn drop(&mut self) {
        let _ = self.audio_ctx.close();
    }
}

unsafe impl Send for WebAudioWrapper {}

unsafe impl Sync for WebAudioWrapper {}
