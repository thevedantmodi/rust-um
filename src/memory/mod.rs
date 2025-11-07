use crate::um::UmWord;

pub struct Memory {
    pub segments: Vec<Option<Vec<UmWord>>>,
    pub free_list: Vec<usize>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            segments: vec![None],
            free_list: Vec::with_capacity(1 << 16),
        }
    }

    pub fn map_segment(&mut self, size: usize) -> usize {
        let segment = vec![0; size];
        /* if there is a free identifier, just use that */
        if let Some(idx) = self.free_list.pop() {
            self.segments[idx] = Some(segment);
            idx
        } else {
            /* add a new identifier */
            self.segments.push(Some(segment));
            self.segments.len() - 1
        }
    }

    pub fn unmap_segment(&mut self, idx: usize) {
        /* we can just set to None because of ownership 🙏 */
        self.segments[idx] = None;
        self.free_list.push(idx);
    }
}
