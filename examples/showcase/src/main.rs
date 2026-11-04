use akari::{SimpleComponent, run};
use akari_showcase::Showcase;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    run(Showcase::init()).map_err(|error| eyre::eyre!(error.to_string()))?;
    Ok(())
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: akari::winit::platform::android::activity::AndroidApp) {
    use akari::winit::{event_loop::EventLoop, platform::android::EventLoopBuilderExtAndroid};

    let event_loop = EventLoop::with_user_event()
        .with_android_app(app)
        .build()
        .expect("Android event loop");
    if let Err(error) = akari::run_with_event_loop(Showcase::init(), event_loop) {
        eprintln!("{error}");
    }
}
