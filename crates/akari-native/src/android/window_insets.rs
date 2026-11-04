use crate::WindowInsets;
use jni::{
    JavaVM, jni_sig, jni_str,
    objects::{Global, JObject, JString},
};
use winit::platform::android::activity::AndroidApp;

pub(super) fn from_activity(app: &AndroidApp, scale: f32) -> Option<WindowInsets> {
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    vm.attach_current_thread(|env| -> jni::errors::Result<Option<WindowInsets>> {
        let activity_raw: jni::sys::jobject = app.activity_as_ptr().cast();
        let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&activity_raw)? };
        let activity_class = env.get_object_class(activity.as_ref())?;
        if env
            .get_method_id(
                &activity_class,
                jni_str!("akariWindowInsets"),
                jni_sig!("()Ljava/lang/String;"),
            )
            .is_err()
        {
            env.exception_clear();
            return Ok(None);
        }
        let raw = env
            .call_method(
                activity.as_ref(),
                jni_str!("akariWindowInsets"),
                jni_sig!("()Ljava/lang/String;"),
                &[],
            )?
            .l()?;
        if raw.is_null() {
            return Ok(None);
        }
        let value = unsafe { JString::from_raw(env, raw.as_raw()) }.try_to_string(env)?;
        let Ok(pixels) = value
            .split(',')
            .map(str::parse::<i32>)
            .collect::<Result<Vec<_>, _>>()
        else {
            return Ok(None);
        };
        if pixels.len() != 4 {
            return Ok(None);
        }
        Ok(Some(WindowInsets {
            left: pixels[0].max(0) as f32 / scale,
            top: pixels[1].max(0) as f32 / scale,
            right: pixels[2].max(0) as f32 / scale,
            bottom: pixels[3].max(0) as f32 / scale,
        }))
    })
    .ok()
    .flatten()
}
