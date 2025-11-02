use bridge::instance::InstanceID;
use gpui::{prelude::*, *};
use gpui_component::{
    h_flex, resizable::{h_resizable, resizable_panel, ResizableState}, sidebar::{Sidebar, SidebarFooter, SidebarGroup, SidebarMenu, SidebarMenuItem}, v_flex, ActiveTheme as _, Icon, IconName
};

use crate::{entity::DataEntities, pages::{debug_page::DebugPage, instance::instance_page::InstancePage, instances_page::InstancesPage, modrinth_page::ModrinthPage}};

pub struct LauncherUI {
    data: DataEntities,
    page: LauncherPage,
    sidebar_state: Entity<ResizableState>,
}

#[derive(Clone)]
pub enum LauncherPage {
    Instances(Entity<InstancesPage>),
    Debug(Entity<DebugPage>),
    Modrinth(Entity<ModrinthPage>),
    InstancePage(InstanceID, Entity<InstancePage>),
}

impl LauncherPage {
    pub fn into_any_element(self) -> AnyElement {
        match self {
            LauncherPage::Instances(entity) => entity.into_any_element(),
            LauncherPage::Debug(entity) => entity.into_any_element(),
            LauncherPage::Modrinth(entity) => entity.into_any_element(),
            LauncherPage::InstancePage(_, entity) => entity.into_any_element(),
        }
    }
    
    pub fn page_type(&self) -> PageType {
        match self {
            LauncherPage::Instances(_) => PageType::Instances,
            LauncherPage::Debug(_) => PageType::Debug,
            LauncherPage::Modrinth(_) => PageType::Modrinth,
            LauncherPage::InstancePage(id, _) => PageType::InstancePage(*id),
        }
    }
}

impl LauncherUI {
    pub fn new(data: &DataEntities, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let instance_page = cx.new(|cx| InstancesPage::new(data, window, cx));
        let sidebar_state = ResizableState::new(cx);
        
        Self {
            data: data.clone(),
            page: LauncherPage::Instances(instance_page),
            sidebar_state,
        }
    }

    pub fn switch_page(&mut self, page: PageType, window: &mut Window, cx: &mut Context<Self>) {
        let data = &self.data;
        match page {
            PageType::Instances => {
                if let LauncherPage::Instances(..) = self.page {
                    return;
                }
                self.page = LauncherPage::Instances(cx.new(|cx| InstancesPage::new(data, window, cx)));
                cx.notify();
            },
            PageType::Debug => {
                if let LauncherPage::Debug(..) = self.page {
                    return;
                }
                self.page = LauncherPage::Debug(cx.new(|cx| DebugPage::new(data, window, cx)));
                cx.notify();
            },
            PageType::Modrinth => {
                if let LauncherPage::Modrinth(..) = self.page {
                    return;
                }
                self.page = LauncherPage::Modrinth(cx.new(|cx| ModrinthPage::new(data, window, cx)));
                cx.notify();
            },
            PageType::InstancePage(id) => {
                if let LauncherPage::InstancePage(current_id, ..) = self.page && current_id == id {
                    return;
                }
                self.page = LauncherPage::InstancePage(id, cx.new(|cx| InstancePage::new(id, data, window, cx)));
                cx.notify();
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PageType {
    Instances,
    Debug,
    Modrinth,
    InstancePage(InstanceID),
}

impl Render for LauncherUI {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page_type = self.page.page_type();
        let sidebar = Sidebar::left()
            .width(relative(1.))
            .border_width(px(0.))
            .footer(
                v_flex()
                    .w_full()
                    .gap_4()
                    .child(
                        SidebarFooter::new()
                            .w_full()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(cx.theme().radius)
                                    .bg(cx.theme().primary)
                                    .text_color(cx.theme().primary_foreground)
                                    .size_8()
                                    .flex_shrink_0()
                                    .child(Icon::new(
                                        IconName::GalleryVerticalEnd,
                                    ))
                                    .rounded_lg(),
                            )
                            .child(
                                v_flex()
                                    .gap_0()
                                    .text_sm()
                                    .flex_1()
                                    .line_height(relative(1.25))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child("Moulberry"),
                            )
                    ))
            .children([SidebarGroup::new("Library").child(SidebarMenu::new().children([
                SidebarMenuItem::new("Instances").active(page_type == PageType::Instances).on_click(cx.listener(|launcher, _, window, cx| {
                    launcher.switch_page(PageType::Instances, window, cx);
                })),
                SidebarMenuItem::new("Mods"),
                SidebarMenuItem::new("Worlds"),
            ])),
            SidebarGroup::new("Launcher").child(SidebarMenu::new().children([
                SidebarMenuItem::new("Debug").active(page_type == PageType::Debug).on_click(cx.listener(|launcher, _, window, cx| {
                    launcher.switch_page(PageType::Debug, window, cx);
                })),
                SidebarMenuItem::new("Modrinth").active(page_type == PageType::Modrinth).on_click(cx.listener(|launcher, _, window, cx| {
                    launcher.switch_page(PageType::Modrinth, window, cx);
                })),
                SidebarMenuItem::new("Blah 3"),
            ]))/*,
            SidebarGroup::new("Recent Instances").child(SidebarMenu::new().children([
                SidebarMenuItem::new("Test Instance Page").active(page_type == PageType::InstancePage).on_click(cx.listener(|launcher, _, window, cx| {
                    launcher.switch_page(PageType::InstancePage, window, cx);
                })),
            ]))*/
            ]);

        h_resizable("container", self.sidebar_state.clone())
            .child(
                resizable_panel()
                    .size(px(150.))
                    .size_range(px(100.)..px(200.))
                    .child(sidebar),
            )
            .child(self.page.clone().into_any_element())
    }
}

pub fn page(cx: &App, title: impl IntoElement) -> gpui::Div {
    v_flex()
        .size_full()
        .child(
            h_flex()
                .p_4()
                .border_b_1()
                .border_color(cx.theme().border)
                .text_xl()
                .child(div().left_4().child(title)),
        )
}
