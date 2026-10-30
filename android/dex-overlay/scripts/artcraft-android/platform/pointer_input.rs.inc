                // ArtCraft pointer source routing: keep mouse motion out of Touch.
                use android_activity::input::{Source, ToolType, Button};
                if motion_event.pointer_count() == 0 { return InputStatus::Unhandled; }
                let pointer = motion_event.pointer_at_index(0);
                let source: u32 = motion_event.source().into();
                let is_mouse = source & 0x2002 == 0x2002;
                if is_mouse && matches!(pointer.tool_type(), ToolType::Finger)
                    && motion_event.button_state().0 == 0
                    && !matches!(motion_event.action(), MotionAction::ButtonPress | MotionAction::ButtonRelease)
                    && !matches!(motion_event.action(), MotionAction::HoverEnter | MotionAction::HoverMove | MotionAction::HoverExit)
                {
                    // Android may clear buttonState on the final Up of a physical
                    // drag. Release it before routing the buttonless event to the SDK.
                    if matches!(motion_event.action(), MotionAction::Up | MotionAction::Cancel) {
                        callback(event::Event::WindowEvent { window_id, event: event::WindowEvent::MouseInput {
                            device_id, button: event::MouseButton::Left, state: event::ElementState::Released,
                        } }, self.window_target());
                    }
                    match touchpad_sdk::process(android_app, motion_event) {
                        Ok(events) => {
                            for e in events.chunks_exact(3) {
                                if !e.iter().all(|v| v.is_finite()) { continue; }
                                let event = match e[0] as i32 {
                                    0 | 3 => event::WindowEvent::CursorMoved { device_id, position: PhysicalPosition::new(e[1] as f64, e[2] as f64) },
                                    1 => event::WindowEvent::MouseWheel { device_id, delta: event::MouseScrollDelta::PixelDelta(PhysicalPosition::new(e[1] as f64, e[2] as f64)), phase: event::TouchPhase::Moved },
                                    2 => event::WindowEvent::PinchGesture { device_id, delta: e[1] as f64, phase: event::TouchPhase::Moved },
                                    _ => continue,
                                };
                                callback(event::Event::WindowEvent { window_id, event }, self.window_target());
                                if e[0] == 0.0 {
                                    for state in [event::ElementState::Pressed, event::ElementState::Released] {
                                        callback(event::Event::WindowEvent { window_id, event: event::WindowEvent::MouseInput { device_id, button: event::MouseButton::Left, state } }, self.window_target());
                                    }
                                }
                            }
                            return InputStatus::Handled;
                        }
                        Err(error) => { warn!("Android gesture SDK unavailable: {error}"); }
                    }
                }
                if is_mouse || matches!(motion_event.source(), Source::Touchpad)
                    || matches!(pointer.tool_type(), ToolType::Mouse)
                {
                    callback(event::Event::WindowEvent {
                        window_id,
                        event: event::WindowEvent::CursorMoved {
                            device_id,
                            position: PhysicalPosition::new(pointer.x() as f64, pointer.y() as f64),
                        },
                    }, self.window_target());
                    let action = motion_event.action();
                    let button = match action {
                        MotionAction::Down if motion_event.button_state().primary() => Some(event::MouseButton::Left),
                        MotionAction::Up | MotionAction::Cancel => Some(event::MouseButton::Left),
                        MotionAction::ButtonPress | MotionAction::ButtonRelease => match motion_event.action_button() {
                            // Primary is already represented by Down/Up on Android.
                            Button::Secondary => Some(event::MouseButton::Right),
                            Button::Tertiary => Some(event::MouseButton::Middle),
                            Button::Back => Some(event::MouseButton::Back),
                            Button::Forward => Some(event::MouseButton::Forward),
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(button) = button {
                        callback(event::Event::WindowEvent {
                            window_id,
                            event: event::WindowEvent::MouseInput {
                                device_id, button,
                                state: if matches!(action, MotionAction::Down | MotionAction::ButtonPress) {
                                    event::ElementState::Pressed
                                } else { event::ElementState::Released },
                            },
                        }, self.window_target());
                    }
                    return InputStatus::Handled;
                }
