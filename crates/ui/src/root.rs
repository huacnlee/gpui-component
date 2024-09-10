use gpui::{
    div, px, AnyView, FocusHandle, InteractiveElement, IntoElement, ParentElement as _, Render,
    Styled, View, ViewContext, VisualContext as _, WindowContext,
};
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    drawer::Drawer,
    modal::{Modal, ModalId},
    notification::{Notification, NotificationList},
    theme::ActiveTheme,
};

/// Extension trait for [`WindowContext`] and [`ViewContext`] to add drawer functionality.
pub trait ContextModal: Sized {
    /// Opens a Drawer.
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static;

    /// Return true, if there is an active Drawer.
    fn has_active_drawer(&self) -> bool;

    /// Closes the active Drawer.
    fn close_drawer(&mut self);

    /// Opens a Modal.
    fn open_modal<F>(&mut self, id: ModalId, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static;

    /// Return true, if there is an active Modal.
    fn has_active_modal(&self) -> bool;

    /// Closes the active Modal.
    fn close_modal(&mut self, id: ModalId);

    /// Closes all active Modals.
    fn close_all_modals(&mut self);

    /// Pushes a notification to the notification list.
    fn push_notification(&mut self, note: impl Into<Notification>);
    fn clear_notifications(&mut self);
    /// Returns number of notifications.
    fn notifications(&self) -> Rc<Vec<View<Notification>>>;
}

impl<'a> ContextModal for WindowContext<'a> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        Root::update(self, move |root, cx| {
            root.previous_focus_handle = cx.focused();
            root.active_drawer = Some(Rc::new(build));
            cx.notify();
        })
    }

    fn has_active_drawer(&self) -> bool {
        Root::read(&self).active_drawer.is_some()
    }

    fn close_drawer(&mut self) {
        Root::update(self, |root, cx| {
            root.active_drawer = None;
            root.focus_back(cx);
            cx.notify();
        })
    }

    fn open_modal<F>(&mut self, id: ModalId, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static,
    {
        Root::update(self, move |root, cx| {
            root.previous_focus_handle = cx.focused();
            root.active_modals.remove(&id);
            root.active_modals.insert(id, Rc::new(build));
            cx.notify();
        })
    }

    fn has_active_modal(&self) -> bool {
        Root::read(&self).active_modals.len() > 0
    }

    fn close_modal(&mut self, id: ModalId) {
        Root::update(self, move |root, cx| {
            root.active_modals.remove(&id);
            root.focus_back(cx);
            cx.notify();
        })
    }

    fn close_all_modals(&mut self) {
        Root::update(self, |root, cx| {
            root.active_modals.clear();
            root.focus_back(cx);
            cx.notify();
        })
    }

    fn push_notification(&mut self, note: impl Into<Notification>) {
        let note = note.into();
        Root::update(self, move |root, cx| {
            root.notification.update(cx, |view, cx| view.push(note, cx));
            cx.notify();
        })
    }

    fn clear_notifications(&mut self) {
        Root::update(self, move |root, cx| {
            root.notification.update(cx, |view, cx| view.clear(cx));
            cx.notify();
        })
    }

    fn notifications(&self) -> Rc<Vec<View<Notification>>> {
        Rc::new(Root::read(&self).notification.read(&self).notifications())
    }
}
impl<'a, V> ContextModal for ViewContext<'a, V> {
    fn open_drawer<F>(&mut self, build: F)
    where
        F: Fn(Drawer, &mut WindowContext) -> Drawer + 'static,
    {
        self.deref_mut().open_drawer(build)
    }

    fn has_active_modal(&self) -> bool {
        self.deref().has_active_modal()
    }

    fn close_drawer(&mut self) {
        self.deref_mut().close_drawer()
    }

    fn open_modal<F>(&mut self, id: ModalId, build: F)
    where
        F: Fn(Modal, &mut WindowContext) -> Modal + 'static,
    {
        self.deref_mut().open_modal(id, build)
    }

    fn has_active_drawer(&self) -> bool {
        self.deref().has_active_drawer()
    }

    fn close_modal(&mut self, id: ModalId) {
        self.deref_mut().close_modal(id)
    }

    fn close_all_modals(&mut self) {
        self.deref_mut().close_all_modals()
    }

    fn push_notification(&mut self, note: impl Into<Notification>) {
        self.deref_mut().push_notification(note)
    }

    fn clear_notifications(&mut self) {
        self.deref_mut().clear_notifications()
    }

    fn notifications(&self) -> Rc<Vec<View<Notification>>> {
        self.deref().notifications()
    }
}

/// Root is a view for the App window for as the top level view (Must be the first view in the window).
///
/// It is used to manage the Drawer, Modal, and Notification.
pub struct Root {
    /// Used to store the focus handle of the previus revious view.
    /// When the Modal, Drawer closes, we will focus back to the previous view.
    previous_focus_handle: Option<FocusHandle>,
    active_drawer: Option<Rc<dyn Fn(Drawer, &mut WindowContext) -> Drawer + 'static>>,
    active_modals: BTreeMap<ModalId, Rc<dyn Fn(Modal, &mut WindowContext) -> Modal + 'static>>,
    pub notification: View<NotificationList>,
    child: AnyView,
}

impl Root {
    pub fn new(child: AnyView, cx: &mut ViewContext<Self>) -> Self {
        Self {
            previous_focus_handle: None,
            active_drawer: None,
            active_modals: BTreeMap::new(),
            notification: cx.new_view(NotificationList::new),
            child,
        }
    }

    pub fn update<F>(cx: &mut WindowContext, f: F)
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

    pub fn read<'a>(cx: &'a WindowContext) -> &'a Self {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        root.read(cx)
    }

    fn focus_back(&mut self, cx: &mut WindowContext) {
        if let Some(handle) = self.previous_focus_handle.take() {
            cx.focus(&handle);
        }
    }

    // Render Notification layer.
    pub fn render_notification_layer(cx: &mut WindowContext) -> Option<impl IntoElement> {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        Some(div().child(root.read(cx).notification.clone()))
    }

    /// Render the Drawer layer.
    pub fn render_drawer_layer(cx: &mut WindowContext) -> Option<impl IntoElement> {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        if let Some(builder) = root.read(cx).active_drawer.clone() {
            let drawer = Drawer::new(cx);
            return Some(builder(drawer, cx));
        }

        None
    }

    /// Render the Modal layer.
    pub fn render_modal_layer(cx: &mut WindowContext) -> Option<impl IntoElement> {
        let root = cx
            .window_handle()
            .downcast::<Root>()
            .and_then(|w| w.root_view(cx).ok())
            .expect("The window root view should be of type `ui::Root`.");

        let active_modals = root.read(cx).active_modals.clone();
        let mut has_overlay = false;

        if active_modals.is_empty() {
            return None;
        }

        Some(
            div().children(active_modals.iter().enumerate().map(|(i, (id, builder))| {
                let mut modal = Modal::new(*id, cx);
                modal = builder(modal, cx);
                modal.offset_top = px(i as f32 * 16.);

                // Keep only have one overlay, we only render the first modal with overlay.
                if has_overlay {
                    modal.overlay_visible = false;
                }
                if modal.has_overlay() {
                    has_overlay = true;
                }

                modal
            })),
        )
    }
}

impl Render for Root {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        div()
            .id("root")
            .size_full()
            .text_color(cx.theme().foreground)
            .child(self.child.clone())
    }
}
