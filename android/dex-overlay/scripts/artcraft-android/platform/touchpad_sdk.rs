//! Delegate Android 13 synthesized pointer gestures to the Android SDK detectors.
use android_activity::{AndroidApp, input::MotionEvent};
use jni::objects::{JFloatArray, JObject, JValue};

pub(super) fn process(app: &AndroidApp, motion: &MotionEvent<'_>) -> Result<Vec<f32>, jni::errors::Error> {
    // SAFETY: AndroidApp owns both handles throughout this synchronous callback.
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let result = env.with_local_frame(16, |env| {
        // Borrowed global Activity; JObject does not delete its JNI reference.
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
        let mut data = Vec::with_capacity(motion.pointer_count() * 4);
        for p in motion.pointers() {
            data.extend_from_slice(&[p.pointer_id() as f32, p.x(), p.y(), p.pressure()]);
        }
        let input = env.new_float_array(data.len() as i32)?;
        env.set_float_array_region(&input, 0, &data)?;
        let action: u32 = motion.action().into();
        let action = action | ((motion.pointer_index() as u32) << 8);
        let output = env.call_method(&activity, "processTouchpad", "(IIJJ[F)[F", &[
            JValue::Int(motion.device_id()), JValue::Int(action as i32),
            JValue::Long(motion.down_time() / 1_000_000), JValue::Long(motion.event_time() / 1_000_000),
            JValue::Object(input.as_ref()),
        ])?.l()?;
        let output = JFloatArray::from(output);
        let len = env.get_array_length(&output)?;
        let mut data = vec![0.0; len as usize];
        env.get_float_array_region(&output, 0, &mut data)?;
        Ok(data)
    });
    if result.is_err() { let _ = env.exception_clear(); }
    result
}
