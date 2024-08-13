use gpui::{
    div, prelude::FluentBuilder as _, AnyView, ParentElement as _, Render, Styled, ViewContext,
    WindowContext,
};
use std::{ops::DerefMut, rc::Rc};

use crate::{drawer::Drawer, theme::ActiveTheme};

/// Extension trait for [`WindowContext`] and [`ViewContext`] to add drawer functionality.
pub trait ContextModal: Sized {
    /// Opens a drawer.
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static;

    /// Closes the active drawer.
    fn close_drawer(&mut self);
}

impl<'a> ContextModal for WindowContext<'a> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        Root::update_root(self, move |root, cx| {
            root.active_drawer = Some(Rc::new(build));
            cx.notify();
        })
    }

    fn close_drawer(&mut self) {
        Root::update_root(self, |root, cx| {
            root.active_drawer = None;
            cx.notify();
        })
    }
}
impl<'a, V> ContextModal for ViewContext<'a, V> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        self.deref_mut().open_drawer(build)
    }

    fn close_drawer(&mut self) {
        self.deref_mut().close_drawer()
    }
}

pub struct Root {
    active_drawer: Option<Rc<dyn Fn(Drawer, &mut WindowContext) -> Drawer + 'static>>,
    root_view: AnyView,
}

impl Root {
    pub fn new(root_view: AnyView, _cx: &mut ViewContext<Self>) -> Self {
        Self {
            active_drawer: None,
            root_view,
        }
    }

    fn update_root<F>(cx: &mut WindowContext, f: F)
    where
        F: FnOnce(&mut Self, &mut ViewContext<Self>) + 'static,
    {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        root.update(cx, |root, cx| f(root, cx))
    }
}

impl Render for Root {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div()
            .size_full()
            .text_color(cx.theme().foreground)
            .child(self.root_view.clone())
            .when_some(self.active_drawer.clone(), |this, build| {
                let drawer = Drawer::new(cx);
                this.child(build(drawer, cx))
            })
    }
}
