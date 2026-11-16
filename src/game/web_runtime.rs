//! Delay shared-world IO until chapter preparation has completed.
//! Handles remain real AssetServer handles; the gate delays IO, not identity.
#[cfg(any(target_arch = "wasm32", test))]
use std::{
    future::poll_fn,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Poll, Waker},
};

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ReadGate {
    open: AtomicBool,
    waiters: Mutex<Vec<Waker>>,
}
#[cfg(any(target_arch = "wasm32", test))]
impl ReadGate {
    async fn wait(&self) {
        poll_fn(|context| {
            let mut waiters = self.waiters.lock().unwrap();
            if self.open.load(Ordering::Acquire) {
                Poll::Ready(())
            } else {
                if !waiters.iter().any(|w| w.will_wake(context.waker())) {
                    waiters.push(context.waker().clone());
                }
                Poll::Pending
            }
        })
        .await
    }
    fn release(&self) {
        self.open.store(true, Ordering::Release);
        for waker in self.waiters.lock().unwrap().drain(..) {
            waker.wake();
        }
    }
}
#[cfg(any(target_arch = "wasm32", test))]
fn menu_asset(path: &std::path::Path) -> bool {
    ["fonts", "ui/title_bg.png"]
        .iter()
        .any(|root| path.starts_with(root))
}

#[cfg(any(target_arch = "wasm32", test))]
use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader};
#[cfg(any(target_arch = "wasm32", test))]
use std::path::Path;
#[cfg(any(target_arch = "wasm32", test))]
struct DemandReader<R> {
    inner: R,
    gate: std::sync::Arc<ReadGate>,
}
#[cfg(any(target_arch = "wasm32", test))]
impl<R: AssetReader> AssetReader for DemandReader<R> {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        if !menu_asset(path) {
            self.gate.wait().await;
        }
        self.inner.read(path).await
    }
    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        if !menu_asset(path) {
            self.gate.wait().await;
        }
        self.inner.read_meta(path).await
    }
    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        self.inner.read_directory(path).await
    }
    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        self.inner.is_directory(path).await
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use bevy::{
        asset::io::{AssetSourceBuilder, AssetSourceId, wasm::HttpWasmAssetReader},
        prelude::*,
    };
    use js_sys::wasm_bindgen::JsCast;
    use std::sync::Arc;
    #[derive(Resource)]
    struct DemandGate(Arc<ReadGate>);
    #[derive(Resource)]
    struct TitleImage(Handle<Image>);
    fn value(key: &str) -> js_sys::wasm_bindgen::JsValue {
        web_sys::window()
            .and_then(|w| js_sys::Reflect::get(w.as_ref(), &key.into()).ok())
            .unwrap_or_default()
    }
    fn set(key: &str, value: impl Into<js_sys::wasm_bindgen::JsValue>) {
        if let Some(w) = web_sys::window() {
            let _ = js_sys::Reflect::set(w.as_ref(), &key.into(), &value.into());
        }
    }
    fn call(key: &str, args: &[js_sys::wasm_bindgen::JsValue]) {
        if let Some(w) = web_sys::window() {
            if let Some(f) = value(key).dyn_ref::<js_sys::Function>() {
                let a = js_sys::Array::new();
                for v in args {
                    a.push(v);
                }
                let _ = f.apply(w.as_ref(), &a);
            }
        }
    }
    pub fn install(app: &mut App) {
        let gate = Arc::new(ReadGate::default());
        let reader_gate = gate.clone();
        app.register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                Box::new(DemandReader {
                    inner: HttpWasmAssetReader::new("assets"),
                    gate: reader_gate.clone(),
                })
            }),
        )
        .insert_resource(DemandGate(gate))
        .add_systems(
            Startup,
            |mut commands: Commands, assets: Res<AssetServer>| {
                commands.insert_resource(TitleImage(assets.load("ui/title_bg.png")))
            },
        )
        .add_systems(
            PreUpdate,
            bridge.after(super::super::gather_keyboard_intent),
        );
    }
    pub fn chapter_ready(stage: usize, transition: bool) -> bool {
        let state = value("loveChapterDownload");
        let field = |k: &str| js_sys::Reflect::get(&state, &k.into()).ok();
        let matches = field("stage").and_then(|v| v.as_f64()) == Some(stage as f64);
        let status = field("status").and_then(|v| v.as_string());
        if matches && status.as_deref() == Some("ready") {
            return true;
        }
        if !matches || status.as_deref() != Some("downloading") {
            call(
                "lovePrepareChapter",
                &[(stage as u32).into(), transition.into()],
            );
        }
        false
    }
    fn bridge(
        state: Option<Res<State<super::super::AppState>>>,
        font: Option<Res<super::super::core::GameFont>>,
        image: Option<Res<TitleImage>>,
        assets: Res<AssetServer>,
        gate: Res<DemandGate>,
        dialogue: Option<Res<super::super::roguelike::event::RunDialogue>>,
        inventory: Option<Res<super::super::roguelike::scene::InventoryOpen>>,
        mut intent: Option<ResMut<super::super::Intent>>,
        mut clock: ResMut<Time<Virtual>>,
        mut previous: Local<Option<super::super::AppState>>,
    ) {
        let Some(state) = state else {
            return;
        };
        if previous.as_ref() != Some(state.get()) {
            set("loveState", format!("{:?}", state.get()));
            if *state.get() == super::super::AppState::Title {
                call("loveReleaseChapter", &[]);
                set("loveChapterDownload", js_sys::wasm_bindgen::JsValue::NULL);
            }
            *previous = Some(*state.get());
        }
        set("loveDialogue", dialogue.is_some_and(|d| d.active));
        set("loveInventory", inventory.is_some_and(|i| i.0));
        if let (Some(font), Some(image)) = (font, image) {
            for id in [font.0.id().untyped(), image.0.id().untyped()] {
                if let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(id) {
                    set("loveUiError", error.to_string());
                }
            }
            if assets.is_loaded_with_dependencies(font.0.id())
                && assets.is_loaded_with_dependencies(image.0.id())
            {
                set("loveUiReady", true);
            }
        }
        let download = value("loveChapterDownload");
        let ready = js_sys::Reflect::get(&download, &"status".into())
            .ok()
            .and_then(|v| v.as_string())
            .as_deref()
            == Some("ready");
        if ready {
            gate.0.release();
        }
        let Some(mut intent) = intent.take() else {
            return;
        };
        if value("loveMenuOpen").as_bool() == Some(true)
            || value("loveDownloadsOpen").as_bool() == Some(true)
        {
            intent.clear();
            clock.pause();
            return;
        }
        clock.unpause();
        if ready && *state.get() == super::super::AppState::Title {
            intent.confirm = true;
        }
        let input = value("loveInput");
        let field = |k: &str| js_sys::Reflect::get(&input, &k.into()).ok();
        let x = field("dx").and_then(|v| v.as_f64()).unwrap_or(0.) as i32;
        let y = field("dy").and_then(|v| v.as_f64()).unwrap_or(0.) as i32;
        if x != 0 || y != 0 {
            intent.move_dir = Some(IVec2::new(x, y));
        }
        intent.confirm |= field("confirm").and_then(|v| v.as_bool()).unwrap_or(false);
        intent.cancel |= field("cancel").and_then(|v| v.as_bool()).unwrap_or(false);
        for key in ["confirm", "cancel"] {
            let _ = js_sys::Reflect::set(&input, &key.into(), &false.into());
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::{chapter_ready, install};

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        pin::pin,
        sync::Arc,
        task::{Context, Wake},
    };
    #[derive(Default)]
    struct Signal(AtomicBool);
    impl Wake for Signal {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::Release);
        }
    }
    #[test]
    fn pending_asset_reads_wake_only_after_preparation_and_future_reads_pass() {
        let gate = ReadGate::default();
        let signal = Arc::new(Signal::default());
        let waker = Waker::from(signal.clone());
        let mut context = Context::from_waker(&waker);
        let mut pending = pin!(gate.wait());
        assert!(pending.as_mut().poll(&mut context).is_pending());
        assert!(!signal.0.load(Ordering::Acquire));
        gate.release();
        assert!(signal.0.load(Ordering::Acquire));
        assert!(pending.as_mut().poll(&mut context).is_ready());
        assert!(pin!(gate.wait()).as_mut().poll(&mut context).is_ready());
    }
    #[test]
    fn only_title_and_fonts_bypass_world_download() {
        assert!(menu_asset(Path::new("fonts/story.otf")));
        assert!(menu_asset(Path::new("ui/title_bg.png")));
        assert!(!menu_asset(Path::new("actors/hero-motion.png")));
        assert!(!menu_asset(Path::new("ui/anim/chapter2_sheet.png")));
    }
}
