use std::{cell::UnsafeCell, marker::PhantomData};

use bevy_ecs::{
    archetype::{Archetype, ArchetypeComponentId},
    component::{Component, ComponentId, ComponentStorage, StorageType},
    entity::Entity,
    query::{
        Access, Fetch, FetchState, FilteredAccess, QueryItem, ReadOnlyWorldQuery, WorldQuery,
        WorldQueryGats,
    },
    storage::{ComponentSparseSet, Table, Tables},
    world::World,
};
use bevy_ptr::{ThinSlicePtr, UnsafeCellDeref};

use crate::{Check, Predicate};

// SAFETY: `ROQueryFetch<Self>` is the same as `QueryFetch<Self>`
unsafe impl<T: Component, Pred: Predicate<T>> WorldQuery for Check<T, Pred> {
    type ReadOnly = Self;

    type State = CheckState<T>;

    fn shrink<'wlong: 'wshort, 'wshort>(item: QueryItem<'wlong, Self>) -> QueryItem<'wshort, Self> {
        item
    }
}

impl<'w, T: Component, Pred: Predicate<T>> WorldQueryGats<'w> for Check<T, Pred> {
    type Fetch = CheckFetch<'w, T, Pred>;
    type _State = CheckState<T>;
}

// SAFETY: read-only access
unsafe impl<T: Component, Pred: Predicate<T>> ReadOnlyWorldQuery for Check<T, Pred> {}

/// The [`FetchState`] of [`Equals`].
pub struct CheckState<T> {
    component_id: ComponentId,
    marker: PhantomData<T>,
}

impl<T: Component> FetchState for CheckState<T> {
    fn init(world: &mut World) -> Self {
        Self {
            component_id: world.init_component::<T>(),
            marker: PhantomData,
        }
    }

    fn matches_component_set(&self, set_contains_id: &impl Fn(ComponentId) -> bool) -> bool {
        set_contains_id(self.component_id)
    }
}

/// The [`Fetch`] of [`Equals`].
pub struct CheckFetch<'w, T, Pred> {
    pred_marker: PhantomData<Pred>,
    // T::Storage = TableStorage
    table_components: Option<ThinSlicePtr<'w, UnsafeCell<T>>>,
    entity_table_rows: Option<ThinSlicePtr<'w, usize>>,
    // T::Storage = SparseStorage
    entities: Option<ThinSlicePtr<'w, Entity>>,
    sparse_set: Option<&'w ComponentSparseSet>,
}

impl<T, Pred> Clone for CheckFetch<'_, T, Pred> {
    fn clone(&self) -> Self {
        Self {
            pred_marker: PhantomData,
            table_components: self.table_components,
            entity_table_rows: self.entity_table_rows,
            entities: self.entities,
            sparse_set: self.sparse_set,
        }
    }
}

// SAFETY: this reads the T component. archetype component access and component access are updated to reflect that
unsafe impl<'w, T: Component, Pred: Predicate<T>> Fetch<'w> for CheckFetch<'w, T, Pred> {
    type Item = bool;
    type State = CheckState<T>;

    unsafe fn init(
        world: &'w World,
        state: &Self::State,
        _last_change_tick: u32,
        _change_tick: u32,
    ) -> Self {
        Self {
            pred_marker: PhantomData,
            table_components: None,
            entity_table_rows: None,
            entities: None,
            sparse_set: (T::Storage::STORAGE_TYPE == StorageType::SparseSet).then(|| {
                world
                    .storages()
                    .sparse_sets
                    .get(state.component_id)
                    .unwrap()
            }),
        }
    }

    const IS_DENSE: bool = {
        match T::Storage::STORAGE_TYPE {
            StorageType::Table => true,
            StorageType::SparseSet => false,
        }
    };

    const IS_ARCHETYPAL: bool = true;

    unsafe fn set_archetype(
        &mut self,
        state: &Self::State,
        archetype: &'w Archetype,
        tables: &'w Tables,
    ) {
        match T::Storage::STORAGE_TYPE {
            StorageType::Table => {
                self.entity_table_rows = Some(archetype.entity_table_rows().into());
                let column = tables[archetype.table_id()]
                    .get_column(state.component_id)
                    .unwrap();
                self.table_components = Some(column.get_data_slice().into());
            }
            StorageType::SparseSet => self.entities = Some(archetype.entities().into()),
        }
    }

    unsafe fn set_table(&mut self, state: &Self::State, table: &'w Table) {
        self.table_components = Some(
            table
                .get_column(state.component_id)
                .unwrap()
                .get_data_slice()
                .into(),
        );
    }

    unsafe fn archetype_fetch(&mut self, archetype_index: usize) -> Self::Item {
        let item = match T::Storage::STORAGE_TYPE {
            StorageType::Table => {
                let (entity_table_rows, table_components) = self
                    .entity_table_rows
                    .zip(self.table_components)
                    .unwrap_or_else(|| debug_checked_unreachable());
                let table_row = *entity_table_rows.get(archetype_index);
                table_components.get(table_row).deref()
            }
            StorageType::SparseSet => {
                let (entities, sparse_set) = self
                    .entities
                    .zip(self.sparse_set)
                    .unwrap_or_else(|| debug_checked_unreachable());
                let entity = *entities.get(archetype_index);
                sparse_set
                    .get(entity)
                    .unwrap_or_else(|| debug_checked_unreachable())
                    .deref::<T>()
            }
        };
        Pred::test(item)
    }

    unsafe fn table_fetch(&mut self, table_row: usize) -> Self::Item {
        let components = self
            .table_components
            .unwrap_or_else(|| debug_checked_unreachable());
        let item = components.get(table_row).deref();
        Pred::test(item)
    }

    unsafe fn archetype_filter_fetch(&mut self, archetype_index: usize) -> bool {
        self.archetype_fetch(archetype_index)
    }

    unsafe fn table_filter_fetch(&mut self, table_row: usize) -> bool {
        self.table_fetch(table_row)
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        assert!(
            !access.access().has_write(state.component_id),
            "Equals<{}, _> conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<T>(),
        );
        access.add_read(state.component_id);
    }

    fn update_archetype_component_access(
        state: &Self::State,
        archetype: &Archetype,
        access: &mut Access<ArchetypeComponentId>,
    ) {
        if let Some(archetype_component_id) =
            archetype.get_archetype_component_id(state.component_id)
        {
            access.add_read(archetype_component_id);
        }
    }
}

unsafe fn debug_checked_unreachable() -> ! {
    #[cfg(debug_assertions)]
    unreachable!();
    #[cfg(not(debug_assertions))]
    std::hint::unreachable_unchecked();
}
