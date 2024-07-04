use gpui::{
    div, prelude::FluentBuilder as _, ClickEvent, Div, ElementId, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, Pixels, RenderOnce, Stateful,
    StatefulInteractiveElement as _, Styled, WindowContext,
};

use crate::{h_flex, theme::ActiveTheme, Disableable, Icon, IconName, Selectable};

#[derive(IntoElement)]
pub struct ListItem {
    base: Stateful<Div>,
    disabled: bool,
    selected: bool,
    check_icon: Option<Icon>,
    border_radius: Option<Pixels>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut WindowContext) + 'static>>,
    on_secondary_mouse_down: Option<Box<dyn Fn(&MouseDownEvent, &mut WindowContext) + 'static>>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id.into()),
            disabled: false,
            selected: false,
            on_click: None,
            on_secondary_mouse_down: None,
            check_icon: None,
            border_radius: None,
        }
    }

    pub fn check_icon(mut self, icon: IconName) -> Self {
        self.check_icon = Some(Icon::new(icon));
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn rounded(mut self, r: impl Into<Pixels>) -> Self {
        self.border_radius = Some(r.into());
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_secondary_mouse_down(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut WindowContext) + 'static,
    ) -> Self {
        self.on_secondary_mouse_down = Some(Box::new(handler));
        self
    }
}

impl Disableable for ListItem {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for ListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Styled for ListItem {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for ListItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for ListItem {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        h_flex()
            .id("list-item")
            .relative()
            .gap_x_2()
            .items_center()
            .justify_between()
            .text_base()
            .text_color(cx.theme().foreground)
            .when_some(self.border_radius, |this, r| this.rounded(r))
            .when_some(self.on_click, |this, on_click| {
                if !self.disabled {
                    this.cursor_pointer().on_click(on_click)
                } else {
                    this
                }
            })
            .when(self.selected, |this| this.bg(cx.theme().accent))
            .when(!self.selected && !self.disabled, |this| {
                this.hover(|this| this.bg(cx.theme().accent))
            })
            // Right click
            .when_some(self.on_secondary_mouse_down, |this, on_mouse_down| {
                if !self.disabled {
                    this.on_mouse_down(MouseButton::Right, move |ev, cx| (on_mouse_down)(ev, cx))
                } else {
                    this
                }
            })
            .child(self.base.w_full())
            .when(self.selected, |this| {
                if let Some(icon) = self.check_icon {
                    this.child(icon.text_color(cx.theme().muted_foreground).mr_2())
                } else {
                    this
                }
            })
    }
}
