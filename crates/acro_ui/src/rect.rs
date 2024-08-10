use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    rc::{Rc, Weak},
};

use acro_ecs::{entity, query, EntityId, Query, SystemRunContext};
use acro_math::{Children, Parent, Vec2};
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Clone, Copy, Default)]
pub enum Dim {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Default)]
pub struct Rect {
    inner: Rc<RefCell<RectInner>>,
}

#[derive(Debug, Default)]
pub struct RectInner {
    pub(crate) child_index: usize,

    pub(crate) size: Vec2,

    // Position of this Rect from the top-left corner of the screen
    pub(crate) offset: Vec2,

    pub(crate) options: PositioningOptions,
    children_top_left_offsets: Vec<Vec2>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Dir<T: Debug + Default + Copy> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T: Debug + Default + Copy> Dir<T> {
    pub fn all(val: T) -> Dir<T> {
        Dir {
            left: val.clone(),
            right: val.clone(),
            top: val.clone(),
            bottom: val.clone(),
        }
    }

    pub fn xy(x: T, y: T) -> Dir<T> {
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
    pub flex: FlexOptions,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FlexOptions {
    pub direction: FlexDirection,
    pub gap: Dim,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    // RowReverse,
    #[default]
    Column,
    // ColumnReverse,
}

#[derive(Debug, Default)]
pub struct RootOptions {
    pub size: Vec2,
    pub padding: Dir<Dim>,
    pub flex: FlexOptions,
}

pub struct RectQueries<'a> {
    pub ctx: &'a SystemRunContext<'a>,
    pub entity_id: EntityId,
    pub children_query: &'a Query<&'a Children>,
    pub parent_query: &'a Query<&'a Parent>,
    pub rect_query: &'a Query<&'a Rect>,
}

impl RectQueries<'_> {
    pub fn new<'a>(
        ctx: &'a SystemRunContext,
        entity_id: EntityId,
        children_query: &'a Query<&'a Children>,
        parent_query: &'a Query<&'a Parent>,
        rect_query: &'a Query<&'a Rect>,
    ) -> RectQueries<'a> {
        RectQueries {
            ctx,
            entity_id,
            children_query,
            parent_query,
            rect_query,
        }
    }
    pub fn with_entity_id(&self, entity_id: EntityId) -> RectQueries {
        RectQueries {
            ctx: self.ctx,
            entity_id,
            children_query: self.children_query,
            parent_query: self.parent_query,
            rect_query: self.rect_query,
        }
    }

    pub fn get_parent(&self) -> Option<EntityId> {
        self.parent_query
            .get(self.ctx, self.entity_id)
            .map(|parent| parent.0)
    }

    pub fn get_children(&self) -> Vec<EntityId> {
        self.children_query
            .get(self.ctx, self.entity_id)
            .map(|children| children.0.clone())
            .unwrap_or_default()
    }

    pub fn get_rect(&self, entity_id: EntityId) -> Ref<RectInner> {
        self.rect_query
            .get(self.ctx, entity_id)
            .expect("rect should exist")
            .inner()
    }

    pub fn get_rect_mut(&self, entity_id: EntityId) -> RefMut<RectInner> {
        self.rect_query
            .get(self.ctx, entity_id)
            .expect("rect should exist")
            .inner_mut()
    }
}

impl Rect {
    pub fn new_root(options: RootOptions) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RectInner {
                child_index: 0,
                size: options.size,
                offset: Vec2::zeros(),
                options: PositioningOptions {
                    width: Dim::Px(options.size.x),
                    height: Dim::Px(options.size.y),
                    padding: options.padding,
                    margin: Dir::default(),
                    flex: options.flex,
                },
                children_top_left_offsets: Vec::new(),
            })),
        }
    }

    pub fn inner(&self) -> Ref<RectInner> {
        self.inner.borrow()
    }

    pub fn inner_mut(&self) -> RefMut<RectInner> {
        self.inner.borrow_mut()
    }

    pub fn recalculate(&self, queries: &RectQueries) {
        let children_offsets = {
            let inner = self.inner.borrow();
            inner.recalculate_children_top_left_offset(queries)
        };

        {
            let mut inner = self.inner.borrow_mut();
            inner.children_top_left_offsets = children_offsets;
            inner.recalculate_self();
        }

        let inner = self.inner();

        for child in &inner.children {
            let child = child.upgrade().expect("child dropped without being freed");
            Rect {
                inner: Rc::clone(&child),
            }
            .recalculate()
        }
    }
}

impl RectInner {
    fn calculate_offset_dim(&self, dim: Dim, parent_dim: f32) -> f32 {
        match dim {
            Dim::Px(px) => px,
            Dim::Percent(percent) => parent_dim * percent,
            Dim::Auto => 0.0,
        }
    }

    fn calculate_size_dim(
        &self,
        dim: Dim,
        margin_dir: f32,
        get_parent_available_size: impl Fn(Vec2) -> f32,
        get_child_size: impl Fn(&RectInner) -> f32,
        queries: &RectQueries,
    ) -> f32 {
        let size_with_margin = match dim {
            // TODO: auto size calculations will be different when using flex layouts
            Dim::Auto => {
                let min_children_size =
                    queries
                        .get_children()
                        .iter()
                        .fold(std::f32::MAX, |current_min_size, child| {
                            let child = queries
                                .rect_query
                                .get(ctx, *child)
                                .expect("child rect should exist")
                                .inner();
                            let child_size = get_child_size(&*child);
                            current_min_size.min(child_size)
                        });

                min_children_size
            }
            Dim::Percent(percent) => {
                let parent = queries
                    .parent_query
                    .get(ctx, queries.entity_id)
                    .expect("parent should exist")
                    .0;
                let parent_rect = queries
                    .rect_query
                    .get(ctx, parent)
                    .expect("parent rect should exist")
                    .inner();
                get_parent_available_size(
                    parent_rect.calculate_available_space(ctx, queries) * percent,
                )
            }
            Dim::Px(f) => f,
        };

        size_with_margin - margin_dir
    }

    // TODO: handle reversed flex directions
    // TODO: cache
    /// Calculates the offset of the child from the top-left corner of self
    pub fn recalculate_children_top_left_offset(&self, queries: &RectQueries) -> Vec<Vec2> {
        let direction = self.options.flex.direction;

        let mut offsets = Vec::new();
        let mut running_offset = 0.0;

        let children = queries
            .children_query
            .get(ctx, entity_id)
            .expect("children should exist");

        for child in children.0.iter().cloned() {
            let child = queries
                .rect_query
                .get(ctx, child)
                .expect("child rect should exist")
                .inner_mut();

            offsets.push(if matches!(direction, FlexDirection::Column) {
                Vec2::new(0.0, running_offset)
            } else {
                Vec2::new(running_offset, 0.0)
            });
            running_offset += if matches!(direction, FlexDirection::Column) {
                child.size.y + self.calculate_offset_dim(self.options.flex.gap, self.size.y)
            } else {
                child.size.x + self.calculate_offset_dim(self.options.flex.gap, self.size.x)
            };
        }

        offsets
    }

    /// Calculates the offset of the top-left corner of
    /// this element from the top-left corner of the screen
    pub fn calculate_total_top_left_offset(&self, queries: &RectQueries) -> Vec2 {
        let parent = match queries.get_parent() {
            Some(parent) => queries.get_rect(parent),
            None => return Vec2::zeros(),
        };

        // TODO: this can be cached
        let offset_from_children = parent.children_top_left_offsets[self.child_index];

        self.calculate_margin_top_left()
            + parent.calculate_total_top_left_offset(queries)
            + parent.calculate_padding_top_left()
            + offset_from_children
    }

    pub fn calculate_margin_top_left(&self) -> Vec2 {
        Vec2::new(
            self.calculate_offset_dim(self.options.margin.left, self.size.x),
            self.calculate_offset_dim(self.options.margin.top, self.size.y),
        )
    }

    pub fn calculate_margin_bottom_right(&self) -> Vec2 {
        Vec2::new(
            self.calculate_offset_dim(self.options.margin.right, self.size.x),
            self.calculate_offset_dim(self.options.margin.bottom, self.size.y),
        )
    }

    pub fn calculate_margin_absolute(&self) -> Vec2 {
        self.calculate_margin_top_left() + self.calculate_margin_bottom_right()
    }

    pub fn calculate_padding_top_left(&self) -> Vec2 {
        Vec2::new(
            self.calculate_offset_dim(self.options.padding.left, self.size.x),
            self.calculate_offset_dim(self.options.padding.top, self.size.y),
        )
    }

    pub fn calculate_padding_bottom_right(&self) -> Vec2 {
        Vec2::new(
            self.calculate_offset_dim(self.options.padding.right, self.size.x),
            self.calculate_offset_dim(self.options.padding.bottom, self.size.y),
        )
    }

    pub fn calculate_padding_absolute(&self) -> Vec2 {
        self.calculate_padding_top_left() + self.calculate_padding_bottom_right()
    }

    /// "Available space" of an element = its width - padding_left - padding_right
    pub fn calculate_available_space(&self, ctx: &SystemRunContext, queries: &RectQueries) -> Vec2 {
        self.calculate_size(ctx, queries)
            - self.calculate_padding_top_left()
            - self.calculate_padding_bottom_right()
    }

    // Auto:    makes the element big enough to fit all its children with padding
    // Percent: makes the element take up a % of the available space given by its parent
    // Px:      makes the element have a fixed width
    pub fn calculate_width(&self, queries: &RectQueries) -> f32 {
        self.calculate_size_dim(
            self.options.width,
            self.calculate_margin_absolute().x,
            |size| size.x,
            |child| child.calculate_width(ctx, queries),
            queries,
        )
    }

    pub fn calculate_height(&self, queries: &RectQueries) -> f32 {
        self.calculate_size_dim(
            ctx,
            self.options.height,
            self.calculate_margin_absolute().y,
            |size| size.y,
            |child| child.calculate_height(ctx, queries),
            queries,
        )
    }

    pub fn calculate_size(&self, queries: &RectQueries) -> Vec2 {
        Vec2::new(
            self.calculate_width(queries),
            self.calculate_height(queries),
        )
    }

    pub fn recalculate_self(&mut self, queries: &RectQueries) {
        self.offset = self.calculate_total_top_left_offset(queries);
        self.size = self.calculate_size(queries);
    }

    // fn get_tree_string_recurse(&self, level: u32) -> String {
    //     let mut output = format!(
    //         "size: {:?} | offset: {:?} | margin offset: {:?} x {:?} | padding offset: {:?} x {:?}.\n",
    //         self.size,
    //         self.offset,
    //         self.options.margin.left,
    //         self.options.margin.top,
    //         self.options.padding.left,
    //         self.options.padding.top,
    //     );

    //     for child in self.children.iter() {
    //         let child = child.upgrade().expect("child should exist");
    //         let child = child.borrow();

    //         output += &"  ".repeat(level as usize);
    //         output += &child.get_tree_string_recurse(level + 1);
    //     }

    //     output
    // }

    // pub fn get_tree_string(&self) -> String {
    //     self.get_tree_string_recurse(1)
    // }
}

#[cfg(test)]
mod tests {
    // use acro_math::Vec2;

    // use crate::rect::{FlexDirection, FlexOptions, PositioningOptions, RootOptions};

    // use super::{Dim, Dir, Rect};

    // #[test]
    // fn test_left_top_calculation_1() {
    //     let mut root = Rect::new_root(RootOptions {
    //         size: Vec2::new(1200.0, 800.0),
    //         ..Default::default()
    //     });
    //     let mut child = root.new_child(PositioningOptions {
    //         width: Dim::Px(160.0),
    //         height: Dim::Px(40.0),
    //         padding: Dir::xy(Dim::Px(10.0), Dim::Px(0.0)),
    //         margin: Dir::xy(Dim::Px(0.0), Dim::Px(10.0)),
    //         ..Default::default()
    //     });
    //     let child_of_child = child.new_child(PositioningOptions {
    //         width: Dim::Percent(1.0),
    //         height: Dim::Percent(1.0),
    //         padding: Dir::default(),
    //         margin: Dir::xy(Dim::Px(20.0), Dim::Px(20.0)),
    //         ..Default::default()
    //     });

    //     println!("{}", root.get_tree_string());

    //     assert_eq!(child.inner().offset, Vec2::new(0.0, 10.0));
    //     assert_eq!(child_of_child.inner().offset, Vec2::new(30.0, 30.0));
    // }

    // #[test]
    // fn test_size_calculation_1() {
    //     let mut root = Rect::new_root(RootOptions {
    //         size: Vec2::new(1200.0, 800.0),
    //         padding: Dir::all(Dim::Px(10.0)),
    //         ..Default::default()
    //     });
    //     let mut child = root.new_child(PositioningOptions {
    //         width: Dim::Percent(1.0),
    //         height: Dim::Percent(1.0),
    //         margin: Dir::all(Dim::Px(10.0)),
    //         ..Default::default()
    //     });
    //     let child_of_child = child.new_child(PositioningOptions {
    //         width: Dim::Percent(0.5),
    //         height: Dim::Percent(1.0),
    //         margin: Dir {
    //             right: Dim::Px(10.0),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     });

    //     println!("{}", root.get_tree_string());

    //     assert_eq!(
    //         root.inner().calculate_available_space(),
    //         Vec2::new(1180.0, 780.0)
    //     );
    //     assert_eq!(root.inner().size, Vec2::new(1200.0, 800.0));

    //     assert_eq!(child.inner().size, Vec2::new(1160.0, 760.0));

    //     assert_eq!(child_of_child.inner().size, Vec2::new(570.0, 760.0));
    // }

    // #[test]
    // fn multiple_elements_1() {
    //     let mut root = Rect::new_root(RootOptions {
    //         size: Vec2::new(400.0, 1000.0),
    //         flex: FlexOptions {
    //             gap: Dim::Px(10.0),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     });

    //     let child_1 = root.new_child(PositioningOptions {
    //         width: Dim::Percent(1.0),
    //         height: Dim::Px(200.0),
    //         ..Default::default()
    //     });

    //     let mut child_2 = root.new_child(PositioningOptions {
    //         width: Dim::Percent(1.0),
    //         height: Dim::Px(400.0),
    //         padding: Dir::all(Dim::Px(10.0)),
    //         ..Default::default()
    //     });

    //     let child_3 = root.new_child(PositioningOptions {
    //         width: Dim::Percent(1.0),
    //         height: Dim::Px(200.0),
    //         ..Default::default()
    //     });

    //     let child_of_child_2 = child_2.new_child(PositioningOptions {
    //         width: Dim::Percent(1.0),
    //         height: Dim::Percent(1.0),
    //         ..Default::default()
    //     });

    //     println!("{}", root.get_tree_string());

    //     let root = root.inner();
    //     let child_1 = child_1.inner();
    //     let child_2 = child_2.inner();
    //     let child_3 = child_3.inner();
    //     let child_of_child_2 = child_of_child_2.inner();

    //     assert_eq!(root.size, Vec2::new(400.0, 1000.0));
    //     assert_eq!(child_1.size, Vec2::new(400.0, 200.0));
    //     assert_eq!(child_2.size, Vec2::new(400.0, 400.0));
    //     assert_eq!(child_3.size, Vec2::new(400.0, 200.0));

    //     assert_eq!(child_1.offset, Vec2::new(0.0, 0.0));
    //     assert_eq!(child_2.offset, Vec2::new(0.0, 210.0));
    //     assert_eq!(child_3.offset, Vec2::new(0.0, 620.0));

    //     assert_eq!(child_2.children_top_left_offsets, &[Vec2::new(0.0, 0.0)]);
    //     assert_eq!(child_of_child_2.offset, Vec2::new(10.0, 220.0));
    // }

    // #[test]
    // fn flex_row_1() {
    //     let mut root = Rect::new_root(RootOptions {
    //         size: Vec2::new(800.0, 400.0),
    //         flex: FlexOptions {
    //             gap: Dim::Px(10.0),
    //             direction: FlexDirection::Row,
    //         },
    //         ..Default::default()
    //     });

    //     let child_1 = root.new_child(PositioningOptions {
    //         width: Dim::Px(100.0),
    //         height: Dim::Percent(1.0),
    //         ..Default::default()
    //     });

    //     let child_2 = root.new_child(PositioningOptions {
    //         width: Dim::Px(100.0),
    //         height: Dim::Percent(1.0),
    //         ..Default::default()
    //     });

    //     let child_3 = root.new_child(PositioningOptions {
    //         width: Dim::Px(100.0),
    //         height: Dim::Percent(1.0),
    //         ..Default::default()
    //     });

    //     println!("{}", root.get_tree_string());

    //     let root = root.inner();
    //     let child_1 = child_1.inner();
    //     let child_2 = child_2.inner();
    //     let child_3 = child_3.inner();

    //     assert_eq!(root.offset, Vec2::new(0.0, 0.0));
    //     assert_eq!(child_1.offset, Vec2::new(0.0, 0.0));
    //     assert_eq!(child_2.offset, Vec2::new(110.0, 0.0));
    //     assert_eq!(child_3.offset, Vec2::new(220.0, 0.0));
    // }
}
