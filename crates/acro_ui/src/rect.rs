use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    rc::{Rc, Weak},
};

use acro_math::Vec2;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default)]
pub enum Dim {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}

#[derive(Debug, Clone)]
pub struct Rect {
    inner: Rc<RefCell<RectInner>>,
    parent: Option<Rc<RefCell<RectInner>>>,
    children: Vec<Weak<RefCell<RectInner>>>,
}

#[derive(Debug)]
pub struct RectInner {
    pub(crate) width: f32,
    pub(crate) height: f32,

    // How many pixels from the left edge of the parent container
    pub(crate) left: f32,
    // How many pixels from the top edge of the parent container
    pub(crate) top: f32,

    options: PositioningOptions,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Dir<T: Debug + Default + Copy> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T: Debug + Default + Copy> Dir<T> {
    pub fn new_xy(x: T, y: T) -> Dir<T> {
        Self {
            left: x.clone(),
            right: x,
            top: y.clone(),
            bottom: y,
        }
    }
}

#[derive(Debug, Default)]
pub struct PositioningOptions {
    pub width: Dim,
    pub height: Dim,
    pub margin: Dir<Dim>,
}

impl Rect {
    pub fn new_root(width: f32, height: f32) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RectInner {
                width,
                height,
                left: 0.0,
                top: 0.0,
                options: PositioningOptions {
                    width: Dim::Px(width),
                    height: Dim::Px(height),
                    margin: Dir::default(),
                },
            })),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn new_child(&mut self, options: PositioningOptions) -> Self {
        let mut child = Self::new_root(0.0, 0.0);
        child.inner.borrow_mut().options = options;

        child.parent = Some(Rc::clone(&self.inner));
        self.children.push(Rc::downgrade(&child.inner));

        self.recalculate();
        child.recalculate();

        child
    }

    pub fn inner(&self) -> Ref<RectInner> {
        self.inner.borrow()
    }

    pub fn parent(&self) -> Ref<RectInner> {
        self.parent.as_ref().expect("no parent").borrow()
    }

    fn calculate_offset(dim: Dim, parent_dim: f32) -> f32 {
        match dim {
            Dim::Px(px) => px,
            Dim::Percent(percent) => parent_dim * percent,
            Dim::Auto => 0.0,
        }
    }

    pub fn recalculate(&self) {
        if self.parent.is_none() {
            return;
        }

        let mut inner = self.inner.borrow_mut();

        let left_offset = Self::calculate_offset(inner.options.margin.left, self.parent().width);
        let right_offset = Self::calculate_offset(inner.options.margin.right, self.parent().width);
        inner.left = left_offset;

        let top_offset = Self::calculate_offset(inner.options.margin.top, self.parent().height);
        let bottom_offset =
            Self::calculate_offset(inner.options.margin.bottom, self.parent().height);

        inner.left = left_offset;
        inner.top = top_offset;

        // inner.left = match inner.options.margin.left {
        //     Dim::Px(px) => px,
        //     Dim::Percent(percent) => self.parent().width * percent,
        //     Dim::Auto => 0.0,
        // };

        inner.width = match inner.options.width {
            // The width of the element is determined by the comfortable width of the content
            Dim::Auto => todo!(),
            Dim::Percent(percent) => self.parent().width * percent,
            Dim::Px(px) => px,
        };

        inner.height = match inner.options.height {
            // The height of the element is determined by the comfortable width of the content
            Dim::Auto => todo!(),
            Dim::Percent(percent) => self.parent().height * percent,
            Dim::Px(px) => px,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{Dim, Dir, Rect};

    #[test]
    fn test_tree() {
        let mut root = Rect::new_root(1200.0, 800.0);
        let mut child = root.new_child(super::PositioningOptions {
            width: Dim::Px(160.0),
            height: Dim::Px(40.0),
            margin: Dir::new_xy(Dim::Px(0.0), Dim::Px(10.0)),
        });
        let mut child_of_child = child.new_child(super::PositioningOptions {
            width: Dim::Percent(0.5),
            height: Dim::Percent(0.5),
            margin: Dir::new_xy(Dim::Px(10.0), Dim::Px(10.0)),
        });

        assert_eq!(child.inner().width, 160.0);
        assert_eq!(child.inner().height, 40.0);
        assert_eq!(child.inner().left, 0.0);
        assert_eq!(child.inner().top, 10.0);
    }
}
