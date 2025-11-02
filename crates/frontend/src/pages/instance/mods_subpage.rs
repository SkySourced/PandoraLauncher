use std::{ffi::OsString, sync::{atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering}, Arc, RwLock}};

use bridge::{handle::BackendHandle, instance::{InstanceID, InstanceServerSummary, InstanceWorldSummary}, message::{AtomicBridgeDataLoadState, MessageToBackend, QuickPlayLaunch}};
use gpui::{prelude::*, *};
use gpui_component::{
    alert::Alert, button::{Button, ButtonGroup, ButtonVariants}, checkbox::Checkbox, dropdown::{Dropdown, DropdownDelegate, DropdownItem, DropdownState, SearchableVec}, form::form_field, group_box::GroupBox, h_flex, input::{InputEvent, InputState, TextInput}, resizable::{h_resizable, resizable_panel, ResizableState}, sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem}, skeleton::Skeleton, tab::{Tab, TabBar}, table::{Column, ColumnFixed, ColumnSort, Table, TableDelegate}, v_flex, ActiveTheme as _, ContextModal, Icon, IconName, IndexPath, List, ListDelegate, ListItem, Root, Selectable, Sizable, StyledExt
};

use crate::{entity::instance::InstanceEntry, png_render_cache, root};

pub struct InstanceModsSubpage {
    instance: InstanceID,
    backend_handle: BackendHandle,
}

impl InstanceModsSubpage {
    pub fn new(instance: &Entity<InstanceEntry>, backend_handle: BackendHandle, mut window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self {
        let instance = instance.read(cx);
        let instance_id = instance.id;
        
        Self {
            instance: instance_id,
            backend_handle,
        }
    }
}

impl Render for InstanceModsSubpage {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        "Hello World"
        // let theme = cx.theme();
        
        // let state = self.worlds_state.load(Ordering::SeqCst);
        // if state.should_send_load_request() {
        //     self.backend_handle.blocking_send(MessageToBackend::RequestLoadWorlds { id: self.instance });
        // }
        
        // v_flex()
        //     .p_4()
        //     .gap_4()
        //     .size_full()
        //     .child(h_flex()
        //         .size_full()
        //         .gap_4()
        //         .child(v_flex().size_full().text_lg().child("Worlds")
        //             .child(v_flex().text_base().size_full().border_1().rounded(theme.radius).border_color(theme.border)
        //                 .child(self.world_list.clone())))
        //         .child(v_flex().size_full().text_lg().child("Servers")
        //             .child(v_flex().text_base().size_full().border_1().rounded(theme.radius).border_color(theme.border)
        //                 .child(self.server_list.clone())))
        //     )
    }
}
