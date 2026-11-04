use akari_core::{Element, Key};

/// Simple non-virtualized rows and columns for small data sets.
pub struct Table<M> {
    key: Key,
    rows: Vec<Vec<Element<M>>>,
}

impl<M: Clone> Table<M> {
    pub fn new(key: Key, rows: Vec<Vec<Element<M>>>) -> Self {
        Self { key, rows }
    }

    pub fn build(self) -> Element<M> {
        let rows = self
            .rows
            .into_iter()
            .enumerate()
            .map(|(index, cells)| {
                Element::row(
                    self.key
                        .wrapping_add(0x8000_0000_0000_0000)
                        .wrapping_add(index as u64),
                    cells,
                )
                .height(36.)
            })
            .collect();
        Element::column(self.key, rows)
    }
}
