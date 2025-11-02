use bridge::{handle::BackendHandle, message::MessageToBackend};
use gpui::{prelude::*, *};
use gpui_component::{
    button::Button, IconName
};

use crate::entity::DataEntities;

pub struct DebugPage {
    backend_handle: BackendHandle,
}

impl DebugPage {
    pub fn new(data: &DataEntities, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            backend_handle: data.backend_handle.clone()
        }
    }
}

impl Render for DebugPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(Button::new("download_all_metadata")
                .label("Download All Metadata")
                .on_click(cx.listener(|this, _, _, _| {
                    this.backend_handle.blocking_send(MessageToBackend::DownloadAllMetadata);
                }))
            )
            .child(div().size_full().grid().grid_cols(16).grid_rows(16)
                .child(IconName::ALargeSmall)
                .child(IconName::ArrowDown)
                .child(IconName::ArrowLeft)
                .child(IconName::ArrowRight)
                .child(IconName::ArrowUp)
                .child(IconName::Asterisk)
                .child(IconName::Bell)
                .child(IconName::BookOpen)
                .child(IconName::Bot)
                .child(IconName::Building2)
                .child(IconName::Calendar)
                .child(IconName::CaseSensitive)
                .child(IconName::ChartPie)
                .child(IconName::Check)
                .child(IconName::ChevronDown)
                .child(IconName::ChevronLeft)
                .child(IconName::ChevronRight)
                .child(IconName::ChevronsUpDown)
                .child(IconName::ChevronUp)
                .child(IconName::CircleCheck)
                .child(IconName::CircleUser)
                .child(IconName::CircleX)
                .child(IconName::Close)
                .child(IconName::Copy)
                .child(IconName::Dash)
                .child(IconName::Delete)
                .child(IconName::Ellipsis)
                .child(IconName::EllipsisVertical)
                .child(IconName::ExternalLink)
                .child(IconName::Eye)
                .child(IconName::EyeOff)
                .child(IconName::File)
                .child(IconName::Folder)
                .child(IconName::FolderClosed)
                .child(IconName::FolderOpen)
                .child(IconName::Frame)
                .child(IconName::GalleryVerticalEnd)
                .child(IconName::GitHub)
                .child(IconName::Globe)
                .child(IconName::Heart)
                .child(IconName::HeartOff)
                .child(IconName::Inbox)
                .child(IconName::Info)
                .child(IconName::Inspector)
                .child(IconName::LayoutDashboard)
                .child(IconName::Loader)
                .child(IconName::LoaderCircle)
                .child(IconName::Map)
                .child(IconName::Maximize)
                .child(IconName::Menu)
                .child(IconName::Minimize)
                .child(IconName::Minus)
                .child(IconName::Moon)
                .child(IconName::Palette)
                .child(IconName::PanelBottom)
                .child(IconName::PanelBottomOpen)
                .child(IconName::PanelLeft)
                .child(IconName::PanelLeftClose)
                .child(IconName::PanelLeftOpen)
                .child(IconName::PanelRight)
                .child(IconName::PanelRightClose)
                .child(IconName::PanelRightOpen)
                .child(IconName::Plus)
                .child(IconName::Replace)
                .child(IconName::ResizeCorner)
                .child(IconName::Search)
                .child(IconName::Settings)
                .child(IconName::Settings2)
                .child(IconName::SortAscending)
                .child(IconName::SortDescending)
                .child(IconName::SquareTerminal)
                .child(IconName::Star)
                .child(IconName::StarOff)
                .child(IconName::Sun)
                .child(IconName::ThumbsDown)
                .child(IconName::ThumbsUp)
                .child(IconName::TriangleAlert)
                .child(IconName::User)
                .child(IconName::WindowClose)
                .child(IconName::WindowMaximize)
                .child(IconName::WindowMinimize)
                .child(IconName::WindowRestore)
            )
    }
}
