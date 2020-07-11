// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
};

pub struct Resize {
    pub new_texture: wgpu::Texture,
    pub size: PhysicalSize<u32>,
}

#[derive(Debug)]
pub enum SceneEvent {
    MouseInput {
        button: MouseButton,
        state: ElementState,
    },
    CursorMoved {
        new_pos: PhysicalPosition<u32>,
    },
    CursorLeft,
    Zoom {
        delta: MouseScrollDelta,
        phase: TouchPhase,
    },
}

pub struct NotApplicable;

impl TryFrom<&'_ WindowEvent<'_>> for SceneEvent {
    type Error = NotApplicable;

    fn try_from(window_event: &WindowEvent) -> Result<Self, Self::Error> {
        Ok(match *window_event {
            WindowEvent::MouseInput { state, button, .. } => Self::MouseInput { state, button },
            WindowEvent::CursorMoved { position, .. } => Self::CursorMoved {
                new_pos: position.cast(),
            },
            WindowEvent::CursorLeft { .. } => Self::CursorLeft,
            WindowEvent::MouseWheel { delta, phase, .. } => Self::Zoom { delta, phase },
            WindowEvent::Resized(_) => {
                // This window event is special and should be handled by the
                // the hub.
                unreachable!(
                    "the WindowEvent::Resized event is special and should be handled by the hub"
                )
            }
            _ => return Err(NotApplicable),
        })
    }
}