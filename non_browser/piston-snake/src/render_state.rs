use crate::render::TilePatch;
use vec_map::VecMap;

pub struct Tiles {
    storage: VecMap<TilePatch>,
}

impl Tiles {
    pub fn new() -> Self {
        Tiles {
            storage: VecMap::new(),
        }
    }
    #[inline]
    pub fn apply_patch(&mut self, patch: TilePatch) {
        self.storage.insert(patch.hash, patch);
    }
    pub fn clear(&mut self) {
        self.storage.clear();
    }
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = TilePatch> + 'a {
        self.storage.iter().map(|(_, v)| *v)
    }
}
