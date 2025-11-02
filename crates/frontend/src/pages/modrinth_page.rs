
use bridge::handle::BackendHandle;
use gpui::{prelude::*, *};

use crate::entity::DataEntities;

pub struct ModrinthPage {
    backend_handle: BackendHandle,
}

impl ModrinthPage {
    pub fn new(data: &DataEntities, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            backend_handle: data.backend_handle.clone(),
        }
    }
}

impl Render for ModrinthPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(gpui::green())
    }
}
