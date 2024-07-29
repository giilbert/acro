use crate::archetype::ArchetypeId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityId {
    generation: u32,
    index: u32,
}

impl EntityId {
    pub fn new(generation: u32, index: u32) -> Self {
        Self { generation, index }
    }
}

#[derive(Debug)]
pub struct Entities {
    entities: Vec<EntityMeta>,
    free_list: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityMeta {
    pub generation: u32,
    pub archetype_id: ArchetypeId,
    pub table_index: usize,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn spawn(&mut self, empty_archetype_table_index: usize) -> EntityId {
        // If an entity has been freed, reuse that location instead of creating a new one
        if let Some(index) = self.free_list.pop() {
            let meta = &mut self.entities[index as usize];
            meta.archetype_id = ArchetypeId::EMPTY;
            meta.table_index = empty_archetype_table_index;
            EntityId {
                generation: meta.generation,
                index,
            }
        } else {
            // No entities are unused
            let generation = 0;
            let index = self.entities.len() as u32;
            self.entities.push(EntityMeta {
                generation,
                archetype_id: ArchetypeId::EMPTY,
                table_index: empty_archetype_table_index,
            });
            EntityId { generation, index }
        }
    }

    pub fn free(&mut self, id: EntityId) {
        let meta = &mut self.entities[id.index as usize];
        meta.generation = meta.generation.wrapping_add(1);
        self.free_list.push(id.index);
    }

    pub fn get(&self, id: EntityId) -> Option<&EntityMeta> {
        let meta = self.entities.get(id.index as usize)?;
        if meta.generation == id.generation {
            Some(meta)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut EntityMeta> {
        let meta = self.entities.get_mut(id.index as usize)?;
        if meta.generation == id.generation {
            Some(meta)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_free() {
        let mut entities = Entities::new();
        let id = entities.spawn(0);
        assert_eq!(entities.get(id).unwrap().generation, 0);
        entities.free(id);
        assert_eq!(entities.get(id), None);
        let id = entities.spawn(0);
        assert_eq!(entities.get(id).unwrap().generation, 1);
    }

    #[test]
    fn spawn_reuse() {
        let mut entities = Entities::new();

        let id1 = entities.spawn(0);
        assert_eq!(id1.index, 0);
        assert_eq!(id1.generation, 0);
        entities.free(id1);

        let id2 = entities.spawn(0);
        assert_eq!(id2.index, 0);
        assert_eq!(id2.generation, 1);

        assert_eq!(entities.get(id1), None);
        assert_eq!(
            entities.get(id2),
            Some(&EntityMeta {
                generation: 1,
                archetype_id: ArchetypeId::EMPTY,
                table_index: 0,
            })
        );
    }
}
