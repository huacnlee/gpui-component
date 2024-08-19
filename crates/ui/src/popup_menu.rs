use std::ops::Deref;
use std::rc::Rc;

use gpui::{
    actions, div, prelude::FluentBuilder, px, Action, AppContext, DismissEvent, EventEmitter,
    FocusHandle, InteractiveElement, IntoElement, KeyBinding, ParentElement, Pixels, Render,
    SharedString, Styled as _, View, ViewContext, VisualContext as _, WindowContext,
};
use gpui::{anchored, rems, AnchorCorner, FocusableView};

use crate::StyledExt;
use crate::{
    button::Button, h_flex, list::ListItem, popover::Popover, theme::ActiveTheme, v_flex, Icon,
    IconName, Selectable, Sizable as _,
};

actions!(menu, [Confirm, Dismiss, SelectNext, SelectPrev]);

pub fn init(cx: &mut AppContext) {
    let context = Some("PopupMenu");
    cx.bind_keys([
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("escape", Dismiss, context),
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
    ]);
}

pub trait PopupMenuExt: Selectable + IntoElement + 'static {
    fn popup_menu(
        self,
        f: impl Fn(PopupMenu, &mut WindowContext) -> PopupMenu + 'static,
    ) -> Popover<PopupMenu> {
        Popover::new("popup-menu")
            .trigger(self)
            .content(move |cx| PopupMenu::build(cx, |menu, cx| f(menu, cx)))
    }
}
impl PopupMenuExt for Button {}

enum PopupMenuItem {
    Separator,
    Item {
        icon: Option<Icon>,
        label: SharedString,
        action: Option<Box<dyn Action>>,
        handler: Rc<dyn Fn(&mut WindowContext)>,
    },
    SubMenu {
        icon: Option<Icon>,
        label: SharedString,
        menu: View<PopupMenu>,
    },
}

impl PopupMenuItem {
    fn is_clickable(&self) -> bool {
        !matches!(self, PopupMenuItem::Separator)
    }

    fn has_icon(&self) -> bool {
        matches!(self, PopupMenuItem::Item { icon: Some(_), .. })
    }
}

pub struct PopupMenu {
    focus_handle: FocusHandle,
    menu_items: Vec<PopupMenuItem>,
    has_icon: bool,
    selected_index: Option<usize>,
    min_width: Pixels,
    max_width: Pixels,
    _subscriptions: [gpui::Subscription; 1],
}

impl PopupMenu {
    pub fn build(
        cx: &mut WindowContext,
        f: impl FnOnce(Self, &mut ViewContext<PopupMenu>) -> Self,
    ) -> View<Self> {
        cx.new_view(|cx| {
            let focus_handle = cx.focus_handle();
            let _on_blur_subscription = cx.on_blur(&focus_handle, |this: &mut PopupMenu, cx| {
                this.dismiss(&Dismiss, cx)
            });

            let menu = Self {
                focus_handle,
                menu_items: Vec::new(),
                selected_index: None,
                min_width: px(120.),
                max_width: px(500.),
                has_icon: false,
                _subscriptions: [_on_blur_subscription],
            };
            cx.refresh();
            f(menu, cx)
        })
    }

    /// Set min width of the popup menu, default is 120px
    pub fn min_w(mut self, width: impl Into<Pixels>) -> Self {
        self.min_width = width.into();
        self
    }

    /// Set max width of the popup menu, default is 500px
    pub fn max_w(mut self, height: impl Into<Pixels>) -> Self {
        self.max_width = height.into();
        self
    }

    /// Add Menu Item
    pub fn menu(mut self, label: impl Into<SharedString>, action: Box<dyn Action>) -> Self {
        self.add_menu_item(label, None, action);
        self
    }

    /// Add Menu to open link
    pub fn link(mut self, label: impl Into<SharedString>, href: impl Into<String>) -> Self {
        let href = href.into();
        self.menu_items.push(PopupMenuItem::Item {
            icon: None,
            label: label.into(),
            action: None,
            handler: Rc::new(move |cx| cx.open_url(&href)),
        });
        self
    }

    /// Add Menu to open link
    pub fn link_with_icon(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<Icon>,
        href: impl Into<String>,
    ) -> Self {
        let href = href.into();
        self.menu_items.push(PopupMenuItem::Item {
            icon: Some(icon.into()),
            label: label.into(),
            action: None,
            handler: Rc::new(move |cx| cx.open_url(&href)),
        });
        self
    }
    /// Add Menu Item with Icon
    pub fn menu_with_icon(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<Icon>,
        action: Box<dyn Action>,
    ) -> Self {
        self.add_menu_item(label, Some(icon.into()), action);
        self
    }

    /// Add Menu Item with check icon
    pub fn menu_with_check(
        mut self,
        label: impl Into<SharedString>,
        checked: bool,
        action: Box<dyn Action>,
    ) -> Self {
        if checked {
            self.add_menu_item(label, Some(IconName::Check.into()), action);
        } else {
            self.add_menu_item(label, None, action);
        }

        self
    }

    fn add_menu_item(
        &mut self,
        label: impl Into<SharedString>,
        icon: Option<Icon>,
        action: Box<dyn Action>,
    ) -> &mut Self {
        if icon.is_some() {
            self.has_icon = true;
        }

        self.menu_items.push(PopupMenuItem::Item {
            icon,
            label: label.into(),
            action: Some(action.boxed_clone()),
            handler: Rc::new(move |cx| {
                cx.activate_window();
                cx.dispatch_action(action.boxed_clone());
            }),
        });
        self
    }

    /// Add a separator Menu Item
    pub fn separator(mut self) -> Self {
        self.menu_items.push(PopupMenuItem::Separator);
        self
    }

    pub fn sub_menu(
        self,
        label: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
        f: impl Fn(PopupMenu, &mut ViewContext<PopupMenu>) -> PopupMenu + 'static,
    ) -> Self {
        self.sub_menu_with_icon(None, label, cx, f)
    }

    /// Add a SubMenu item with icon
    pub fn sub_menu_with_icon(
        mut self,
        icon: Option<Icon>,
        label: impl Into<SharedString>,
        cx: &mut ViewContext<Self>,
        f: impl Fn(PopupMenu, &mut ViewContext<PopupMenu>) -> PopupMenu + 'static,
    ) -> Self {
        let sub_menu = PopupMenu::build(cx, f);
        // Subscribe the SubMenu DismissEvent to dissmiss the parent menu
        cx.subscribe(&sub_menu, |parent, _, _: &DismissEvent, cx| {
            parent.dismiss(&Dismiss, cx);
        })
        .detach();

        self.menu_items.push(PopupMenuItem::SubMenu {
            icon,
            label: label.into(),
            menu: sub_menu,
        });
        self
    }

    fn clickable_menu_items(&self) -> impl Iterator<Item = (usize, &PopupMenuItem)> {
        self.menu_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.is_clickable())
    }

    fn on_click(&mut self, ix: usize, cx: &mut ViewContext<Self>) {
        cx.stop_propagation();
        cx.prevent_default();
        self.selected_index = Some(ix);
        self.confirm(&Confirm, cx);
    }

    fn confirm(&mut self, _: &Confirm, cx: &mut ViewContext<Self>) {
        match self.selected_index {
            Some(index) => {
                let item = self.menu_items.get(index);
                match item {
                    Some(PopupMenuItem::Item { handler, .. }) => {
                        handler(cx);
                        self.dismiss(&Dismiss, cx)
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn select_next(&mut self, _: &SelectNext, cx: &mut ViewContext<Self>) {
        let count = self.clickable_menu_items().count();
        if count > 0 {
            let ix = self
                .selected_index
                .map(|index| if index == count - 1 { 0 } else { index + 1 })
                .unwrap_or(0);

            self.selected_index = Some(ix);
            cx.notify();
        }
    }

    fn select_prev(&mut self, _: &SelectPrev, cx: &mut ViewContext<Self>) {
        let count = self.clickable_menu_items().count();
        if count > 0 {
            let ix = self
                .selected_index
                .map(|index| if index == count - 1 { 0 } else { index - 1 })
                .unwrap_or(count - 1);
            self.selected_index = Some(ix);
            cx.notify();
        }
    }

    fn dismiss(&mut self, _: &Dismiss, cx: &mut ViewContext<Self>) {
        cx.emit(DismissEvent);
    }

    fn render_keybinding(
        action: Option<Box<dyn Action>>,
        cx: &ViewContext<Self>,
    ) -> Option<impl IntoElement> {
        if let Some(action) = action {
            if let Some(keybinding) = cx.bindings_for_action(action.deref()).first() {
                let el = div().text_color(cx.theme().muted_foreground).children(
                    keybinding
                        .keystrokes()
                        .into_iter()
                        .map(|keystroke| format!("{}", keystroke)),
                );

                return Some(el);
            }
        }

        return None;
    }

    fn render_icon(
        has_icon: bool,
        icon: Option<Icon>,
        _: &ViewContext<Self>,
    ) -> Option<impl IntoElement> {
        let icon_placeholder = if has_icon { Some(Icon::empty()) } else { None };

        if !has_icon {
            return None;
        }

        let icon = h_flex()
            .w_3p5()
            .h_3p5()
            .items_center()
            .justify_center()
            .text_sm()
            .map(|this| {
                if let Some(icon) = icon {
                    this.child(icon.clone().small().clone())
                } else {
                    this.children(icon_placeholder.clone())
                }
            });

        Some(icon)
    }
}

impl FluentBuilder for PopupMenu {}
impl EventEmitter<DismissEvent> for PopupMenu {}
impl FocusableView for PopupMenu {
    fn focus_handle(&self, _cx: &gpui::AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopupMenu {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let has_icon = self.menu_items.iter().any(|item| item.has_icon());

        v_flex()
            .key_context("PopupMenu")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .on_mouse_down_out(cx.listener(|this, _, cx| this.dismiss(&Dismiss, cx)))
            .max_h(self.max_width)
            .min_w(self.min_width)
            .p_1()
            .gap_y_0p5()
            .min_w(rems(8.))
            .text_color(cx.theme().popover_foreground)
            .children(self.menu_items.iter_mut().enumerate().map(|(ix, item)| {
                let group_id = format!("item:{}", ix);
                let this = ListItem::new(("menu-item", ix))
                    .group(group_id.clone())
                    .p_0()
                    .relative()
                    .py_1p5()
                    .px_2()
                    .rounded_md()
                    .text_sm()
                    .line_height(rems(1.25))
                    .items_center()
                    .on_click(cx.listener(move |this, _, cx| this.on_click(ix, cx)));
                match item {
                    PopupMenuItem::Separator => this.disabled(true).child(
                        div()
                            .p_0()
                            .rounded_none()
                            .h(px(1.))
                            .mx_neg_1()
                            .my_px()
                            .border_0()
                            .bg(cx.theme().muted),
                    ),
                    PopupMenuItem::Item {
                        icon,
                        label,
                        action,
                        ..
                    } => {
                        let action = action.as_ref().map(|action| action.boxed_clone());
                        let key = Self::render_keybinding(action, cx);

                        this.child(
                            h_flex()
                                .items_center()
                                .gap_x_1p5()
                                .children(Self::render_icon(has_icon, icon.clone(), cx))
                                .child(
                                    h_flex()
                                        .flex_1()
                                        .gap_2()
                                        .items_center()
                                        .justify_between()
                                        .child(label.clone())
                                        .children(key),
                                ),
                        )
                    }
                    PopupMenuItem::SubMenu { icon, label, menu } => this
                        .child(
                            h_flex()
                                .items_center()
                                .gap_x_1p5()
                                .children(Self::render_icon(has_icon, icon.clone(), cx))
                                .child(
                                    h_flex()
                                        .flex_1()
                                        .gap_2()
                                        .items_center()
                                        .justify_between()
                                        .child(label.clone())
                                        .child(IconName::ChevronRight),
                                ),
                        )
                        .child(
                            div()
                                .invisible()
                                .group_hover(group_id, |this| this.visible())
                                .child(
                                    anchored()
                                        .snap_to_window()
                                        .anchor(AnchorCorner::TopLeft)
                                        .child(
                                            div()
                                                .top_neg_7()
                                                .left_24()
                                                .popover_style(cx)
                                                .child(menu.clone()),
                                        ),
                                ),
                        ),
                }
            }))
    }
}
