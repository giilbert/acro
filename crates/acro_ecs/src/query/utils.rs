use std::any::TypeId;

use crate::entity::EntityId;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum QueryBorrowType {
    Unknown,
    Borrow,
    BorrowMut,
    OptionBorrow,
    OptionBorrowMut,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum QueryFetchType {
    Unknown,
    EntityId,
    Component,
}

#[allow(unused)]
pub(crate) trait QueryInfoUtils {
    const BORROW: QueryBorrowType = QueryBorrowType::Unknown;
    const FETCH: QueryFetchType = QueryFetchType::Unknown;

    #[inline]
    fn type_id() -> TypeId;
}

impl<'a, T: 'static> QueryInfoUtils for &'a T {
    const BORROW: QueryBorrowType = QueryBorrowType::Borrow;
    const FETCH: QueryFetchType = QueryFetchType::Component;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

impl<'a, T: 'static> QueryInfoUtils for &'a mut T {
    const BORROW: QueryBorrowType = QueryBorrowType::BorrowMut;
    const FETCH: QueryFetchType = QueryFetchType::Component;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

impl<'a, T: 'static> QueryInfoUtils for Option<&'a T> {
    const BORROW: QueryBorrowType = QueryBorrowType::OptionBorrow;
    const FETCH: QueryFetchType = QueryFetchType::Component;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

impl<'a, T: 'static> QueryInfoUtils for Option<&'a mut T> {
    const BORROW: QueryBorrowType = QueryBorrowType::OptionBorrowMut;
    const FETCH: QueryFetchType = QueryFetchType::Component;

    fn type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

impl QueryInfoUtils for EntityId {
    const BORROW: QueryBorrowType = QueryBorrowType::Unknown;
    const FETCH: QueryFetchType = QueryFetchType::EntityId;

    fn type_id() -> TypeId {
        panic!("invalid type_id for EntityId")
    }
}
