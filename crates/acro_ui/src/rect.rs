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
}

#[derive(Debug)]
pub struct RectInner {
    pub(crate) size: Vec2,

    // Position of this rect from the top-left corner of the screen
    pub(crate) offset: Vec2,

    options: PositioningOptions,

    parent: Option<Rc<RefCell<RectInner>>>,
    children: Vec<Weak<RefCell<RectInner>>>,
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
    pub padding: Dir<Dim>,
    pub margin: Dir<Dim>,
}

impl Rect {
    pub fn new_root(width: f32, height: f32) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RectInner {
                size: Vec2::zeros(),
                offset: Vec2::zeros(),
                options: PositioningOptions {
                    width: Dim::Px(width),
                    height: Dim::Px(height),
                    padding: Dir::default(),
                    margin: Dir::default(),
                },
                parent: None,
                children: Vec::new(),
            })),
        }
    }

    pub fn new_child(&mut self, options: PositioningOptions) -> Self {
        let child = Self::new_root(0.0, 0.0);
        child.inner.borrow_mut().options = options;

        child.inner.borrow_mut().parent = Some(Rc::clone(&self.inner));
        self.inner
            .borrow_mut()
            .children
            .push(Rc::downgrade(&child.inner));

        self.recalculate();
        child.recalculate();

        child
    }

    pub fn inner(&self) -> Ref<RectInner> {
        self.inner.borrow()
    }

    pub fn recalculate(&self) {
        self.inner.borrow_mut().recalculate();
    }

    pub fn get_tree_string(&self) -> String {
        self.inner.borrow().get_tree_string()
    }
}

impl RectInner {
    fn calculate_offset(&self, dim: Dim, parent_dim: f32) -> f32 {
        match dim {
            Dim::Px(px) => px,
            Dim::Percent(percent) => parent_dim * percent,
            Dim::Auto => 0.0,
        }
    }

    pub fn parent(&self) -> Ref<RectInner> {
        self.parent
            .as_ref()
            .expect("RectInner has no parent")
            .borrow()
    }

    pub fn calculate_top_left_offset(&self) -> Vec2 {
        if self.parent.is_none() {
            return Vec2::zeros();
        }

        self.calculate_margin()
            + self.parent().calculate_top_left_offset()
            + self.parent().calculate_padding()
    }

    pub fn calculate_margin(&self) -> Vec2 {
        Vec2::new(
            self.calculate_offset(self.options.margin.left, self.size.x),
            self.calculate_offset(self.options.margin.top, self.size.y),
        )
    }

    pub fn calculate_padding(&self) -> Vec2 {
        Vec2::new(
            self.calculate_offset(self.options.padding.left, self.size.x),
            self.calculate_offset(self.options.padding.top, self.size.y),
        )
    }

    pub fn calculate_size(&self) -> Vec2 {
        // TODO:
        Vec2::zeros()
    }

    pub fn recalculate(&mut self) {
        if self.parent.is_none() {
            // TODO: recalculate root?
            return;
        }

        self.offset = self.calculate_top_left_offset();
        self.size = self.calculate_size();
    }

    fn get_tree_string_recurse(&self, level: u32) -> String {
        let mut output = format!(
            "size: {:?} | offset: {:?} | margin: {:?} x {:?} | padding: {:?} x {:?}.\n",
            self.size,
            self.offset,
            self.options.margin.left,
            self.options.margin.top,
            self.options.padding.left,
            self.options.padding.top,
        );

        for child in self.children.iter() {
            let child = child.upgrade().expect("child should exist");
            let child = child.borrow();

            output += &"  ".repeat(level as usize);
            output += &child.get_tree_string_recurse(level + 1);

            output += "\n";
        }

        output
    }

    pub fn get_tree_string(&self) -> String {
        self.get_tree_string_recurse(1)
    }
}

#[cfg(test)]
mod tests {
    use acro_math::Vec2;

    use super::{Dim, Dir, Rect};

    #[test]
    fn test_left_top_calculation_1() {
        let mut root = Rect::new_root(1200.0, 800.0);
        let mut child = root.new_child(super::PositioningOptions {
            width: Dim::Px(160.0),
            height: Dim::Px(40.0),
            padding: Dir::new_xy(Dim::Px(10.0), Dim::Px(0.0)),
            margin: Dir::new_xy(Dim::Px(0.0), Dim::Px(10.0)),
        });
        let child_of_child = child.new_child(super::PositioningOptions {
            width: Dim::Percent(1.0),
            height: Dim::Percent(1.0),
            padding: Dir::default(),
            margin: Dir::new_xy(Dim::Px(20.0), Dim::Px(20.0)),
        });

        println!("{}", root.get_tree_string());

        assert_eq!(child.inner().offset, Vec2::new(0.0, 10.0));
        assert_eq!(child_of_child.inner().offset, Vec2::new(30.0, 30.0));
    }
}
