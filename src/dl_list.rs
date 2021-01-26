#![deny(unsafe_code)]
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct DlList<T> {
    data: Vec<DlEntry<T>>,
    next_free: Option<usize>,
    head: Option<usize>,
    tail: Option<usize>,
    rm_cnt: usize,
}

impl<T> Default for DlList<T> {
    fn default() -> Self {
        DlList {
            data: Default::default(),
            next_free: Default::default(),
            head: Default::default(),
            tail: Default::default(),
            rm_cnt: Default::default(),
        }
    }
}

impl<T> DlList<T>
where
    T: PartialEq,
    T: std::fmt::Debug,
{
    #[allow(dead_code)]
    pub fn new() -> DlList<T> {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn head(&self) -> Option<&T> {
        let index = self.head?;

        self.data.get(index).and_then(|e| match e {
            DlEntry::Free { .. } => None,
            DlEntry::Occupied(e) => Some(&e.item),
        })
    }

    #[allow(dead_code)]
    pub fn head_index(&self) -> Option<DlIndex<T>> {
        let index = self.head?;

        self.data.get(index).and_then(|e| match e {
            DlEntry::Free { .. } => None,
            DlEntry::Occupied(e) => Some(DlIndex::new(index, e.rm_cnt)),
        })
    }

    #[allow(dead_code)]
    pub fn tail_index(&self) -> Option<DlIndex<T>> {
        let index = self.tail?;

        self.data.get(index).and_then(|e| match e {
            DlEntry::Free { .. } => None,
            DlEntry::Occupied(e) => Some(DlIndex::new(index, e.rm_cnt)),
        })
    }

    #[allow(dead_code)]
    pub fn push_back(&mut self, item: T) -> DlIndex<T> {
        if self.head.is_none() {
            let rm_cnt = self.rm_cnt;

            let index = if let Some(index) = self.next_free {
                match self.data[index] {
                    DlEntry::Occupied { .. } => panic!("list corrupted!"),
                    DlEntry::Free { next_free } => self.next_free = next_free,
                }

                self.data[index] = DlEntry::Occupied(OcEntry {
                    item,
                    next: None,
                    prev: None,
                    rm_cnt,
                });

                index
            } else {
                let index = self.data.len();

                self.data.push(DlEntry::Occupied(OcEntry {
                    item,
                    next: None,
                    prev: None,
                    rm_cnt,
                }));

                index
            };

            self.tail = Some(index);
            self.head = Some(index);

            return DlIndex::new(index, rm_cnt);
        }

        let tail_index = self.tail.unwrap();

        let position = if let Some(position) = self.next_free {
            match self.data[position] {
                DlEntry::Occupied { .. } => panic!("list corrupted!"),
                DlEntry::Free { next_free } => self.next_free = next_free,
            }

            self.data[position] = DlEntry::Occupied(OcEntry {
                item,
                next: None,
                prev: Some(tail_index),
                rm_cnt: self.rm_cnt,
            });

            position
        } else {
            let position = self.data.len();

            self.data.push(DlEntry::Occupied(OcEntry {
                item,
                next: None,
                prev: Some(tail_index),
                rm_cnt: self.rm_cnt,
            }));

            position
        };

        let new_index = DlIndex::new(position, self.rm_cnt);

        match &mut self.data[tail_index] {
            DlEntry::Free { .. } => panic!("list corrupted!"),
            DlEntry::Occupied(e) => e.next = Some(new_index.index),
        }

        self.tail = Some(position);

        new_index
    }

    #[allow(dead_code)]
    pub fn push_front(&mut self, item: T) -> DlIndex<T> {
        if self.head.is_none() {
            return self.push_back(item);
        }

        let head_index = self.head.unwrap();

        let position = if let Some(position) = self.next_free {
            match self.data[position] {
                DlEntry::Occupied { .. } => panic!("list corrupted!"),
                DlEntry::Free { next_free } => self.next_free = next_free,
            }

            self.data[position] = DlEntry::Occupied(OcEntry {
                item,
                next: Some(head_index),
                prev: None,
                rm_cnt: self.rm_cnt,
            });

            position
        } else {
            let position = self.data.len();

            self.data.push(DlEntry::Occupied(OcEntry {
                item,
                next: Some(head_index),
                prev: None,
                rm_cnt: self.rm_cnt,
            }));

            position
        };

        let new_index = DlIndex::new(position, self.rm_cnt);

        match &mut self.data[head_index] {
            DlEntry::Free { .. } => panic!("list corrupted!"),
            DlEntry::Occupied(e) => e.prev = Some(new_index.index),
        }

        self.head = Some(position);

        new_index
    }

    #[allow(dead_code)]
    pub fn contains(&self, value: &T) -> bool {
        self.iter().any(|e| e == value)
    }

    #[allow(dead_code)]
    pub fn get(&self, index: DlIndex<T>) -> Option<&T> {
        match self.data.get(index.index)? {
            DlEntry::Occupied(e) if e.rm_cnt == index.rm_cnt => Some(&e.item),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn next_index(&self, index: DlIndex<T>) -> Option<DlIndex<T>> {
        match self.data.get(index.index)? {
            DlEntry::Occupied(e) if e.rm_cnt == index.rm_cnt => {
                match e.next {
                    Some(index) => match self.data.get(index)? {
                        DlEntry::Occupied(e) => Some(DlIndex::new(index, e.rm_cnt)),
                        _ => panic!("list corrupted!"),
                    },
                    _ => None, 
                }
            }
            _ => None, 
        }
    }

    #[allow(dead_code)]
    pub fn prev_index(&self, index: DlIndex<T>) -> Option<DlIndex<T>> {
        match self.data.get(index.index)? {
            DlEntry::Occupied(e) if e.rm_cnt == index.rm_cnt => {
                match e.prev {
                    Some(index) => match self.data.get(index)? {
                        DlEntry::Occupied(e) => Some(DlIndex::new(index, e.rm_cnt)),
                        _ => panic!("list corrupted!"),
                    },
                    _ => None,
                }
            }
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, index: DlIndex<T>) -> Option<T> {
        let head_index = self.head?;
        let tail_index = self.tail?;

        let (prev_index, index, next_index) = match self.data.get(index.index)? {
            DlEntry::Free { .. } => return None,
            DlEntry::Occupied(e) => {
                if index.rm_cnt != e.rm_cnt {
                    return None;
                }

                (e.prev, index.index, e.next)
            }
        };

        let removed = std::mem::replace(
            &mut self.data[index],
            DlEntry::Free {
                next_free: self.next_free,
            },
        );

        self.next_free = Some(index);

        self.rm_cnt += 1;

        if (index == head_index) && (index == tail_index) {
            self.head = None;
            self.tail = None;
        } else if index == head_index {
            let next = match &mut self.data[next_index.unwrap()] {
                DlEntry::Free { .. } => panic!("list corrupted!"),
                DlEntry::Occupied(e) => e,
            };

            next.prev = None;
            self.head = next_index;

        } else if index == tail_index {
            let prev = match &mut self.data[prev_index.unwrap()] {
                DlEntry::Free { .. } => panic!("list corrupted!"),
                DlEntry::Occupied(e) => e,
            };

            prev.next = None;
            self.tail = prev_index;

        } else if index != head_index && index != tail_index {
            {
                let next = match &mut self.data[next_index.unwrap()] {
                    DlEntry::Free { .. } => panic!("list corrupted!"),
                    DlEntry::Occupied(e) => e,
                };

                next.prev = prev_index;
            }

            {
                let prev = match &mut self.data[prev_index.unwrap()] {
                    DlEntry::Free { .. } => panic!("list corrupted!"),
                    DlEntry::Occupied(e) => e,
                };
                prev.next = next_index;
            }
        }

        match removed {
            DlEntry::Free { .. } => panic!("list corrupted!"),
            DlEntry::Occupied(e) => Some(e.item),
        }
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        Iter {
            list: self,
            next_index: self.head,
        }
    }

    #[allow(dead_code)]
    pub fn index_of(&self, item: &T) -> Option<DlIndex<T>> {
        let mut next = self.head;

        while let Some(index) = next {
            let ref entry = match &self.data[index] {
                DlEntry::Free { .. } => panic!("list corrupted!"),
                DlEntry::Occupied(entry) => entry,
            };
            if &entry.item == item {
                return Some(DlIndex::new(index, entry.rm_cnt));
            } else {
                next = entry.next;
            }
        }

        None
    }

    #[allow(dead_code)]
    pub fn pop_front(&mut self) -> Option<T> {
        let head_index = self.head?;

        let (head_index, next_index) = match self.data.get(head_index)? {
            DlEntry::Free { .. } => return None,
            DlEntry::Occupied(e) => (head_index, e.next),
        };

        let removed = std::mem::replace(
            &mut self.data[head_index],
            DlEntry::Free {
                next_free: self.next_free,
            },
        );

        self.next_free = Some(head_index);

        self.rm_cnt += 1;

        if Some(head_index) == self.tail {
            self.head = None;
            self.tail = None;
        } else {
            let next = match &mut self.data[next_index.unwrap()] {
                DlEntry::Free { .. } => panic!("list corrupted!"),
                DlEntry::Occupied(e) => e,
            };

            next.prev = None;
            self.head = next_index;
        }

        match removed {
            DlEntry::Free { .. } => panic!("list corrupted!"),
            DlEntry::Occupied(e) => Some(e.item),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DlIndex<T> {
    index: usize,
    rm_cnt: usize,
    _marker: PhantomData<T>,
}

impl<T> DlIndex<T> {
    fn new(index: usize, rm_cnt: usize) -> DlIndex<T> {
        DlIndex {
            index,
            rm_cnt,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, PartialEq)]
enum DlEntry<T> {
    Free { next_free: Option<usize> },
    Occupied(OcEntry<T>),
}

#[derive(Debug, PartialEq)]
struct OcEntry<T> {
    item: T,
    next: Option<usize>,
    prev: Option<usize>,
    rm_cnt: usize,
}

struct Iter<'a, T>
where
    T: 'a,
{
    list: &'a DlList<T>,
    next_index: Option<usize>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.next_index?;

        match &self.list.data[next_index] {
            DlEntry::Free { .. } => panic!("list corrupted!"),
            DlEntry::Occupied(e) => {
                self.next_index = e.next;

                Some(&e.item)
            }
        }
    }
}

impl<T> std::ops::Index<DlIndex<T>> for DlList<T>
where
    T: PartialEq,
    T: std::fmt::Debug,
{
    type Output = T;

    fn index(&self, index: DlIndex<T>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

// Start Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list() {
        let _list: DlList<i32> = DlList::new();
    }

    #[test]
    fn head() {
        let mut list = DlList::new();

        assert!(list.head().is_none());

        let one = list.push_back(1);

        assert_eq!(list.head().unwrap(), &1);

        list.push_back(2);

        list.remove(one);

        assert_eq!(list.head().unwrap(), &2);
        assert_eq!(list.data[0], DlEntry::Free { next_free: None });
        assert_eq!(list.head, Some(1));
        assert_eq!(
            list.data[1],
            DlEntry::Occupied(OcEntry {
                item: 2,
                next: None,
                prev: None,
                rm_cnt: 0,
            })
        );
    }

    #[test]
    fn head_index() {
        let mut list = DlList::new();

        assert!(list.head_index().is_none());

        let one = list.push_back(1);

        assert_eq!(list.head_index().unwrap(), one);
    }

    #[test]
    fn tail_index() {
        let mut list = DlList::new();

        assert!(list.tail_index().is_none());

        list.push_back(1);
        let two = list.push_back(2);

        assert_eq!(list.tail_index().unwrap(), two);
    }

    #[test]
    fn push_back() {
        let mut list = DlList::new();

        list.push_back(1);

        assert_eq!(
            list.data[0],
            DlEntry::Occupied(OcEntry {
                item: 1,
                next: None,
                prev: None,
                rm_cnt: 0,
            })
        );
    }

    #[test]
    fn push_front() {
        let mut list = DlList::new();

        list.push_front(1);
        list.push_front(2);

        assert_eq!(
            list.data[0],
            DlEntry::Occupied(OcEntry {
                item: 1,
                next: None,
                prev: Some(1),
                rm_cnt: 0,
            })
        );
        assert_eq!(
            list.data[1],
            DlEntry::Occupied(OcEntry {
                item: 2,
                next: Some(0),
                prev: None,
                rm_cnt: 0,
            })
        );
    }

    #[test]
    fn contains() {
        let mut list = DlList::new();

        list.push_back(1);

        assert!(list.contains(&1));
    }

    #[test]
    fn get() {
        let mut list = DlList::new();

        let one = list.push_back(1);
        let entry = list.get(one);

        assert!(entry.is_some());
        assert_eq!(entry.unwrap(), &1);
    }

    #[test]
    fn next_index() {
        let mut list = DlList::new();

        let one = list.push_back(1);
        list.push_back(2);

        let two_index = list.next_index(one).unwrap();
        let two_value = list.get(two_index);

        assert_eq!(two_value.unwrap(), &2);
        assert_eq!(None, list.next_index(two_index));
    }

    #[test]
    fn prev_index() {
        let mut list = DlList::new();

        list.push_back(1);
        let two = list.push_back(2);

        let one_index = list.prev_index(two).unwrap();
        let one_value = list.get(one_index);

        assert_eq!(one_value.unwrap(), &1);
        assert_eq!(None, list.prev_index(one_index));
    }

    #[test]
    fn index() {
        let mut list = DlList::new();

        let one = list.push_back(1);
        let entry = list[one];

        assert_eq!(entry, 1);
    }

    #[test]
    fn remove() {
        let mut list = DlList::new();

        let one = list.push_back(1);
        let two = list.push_back(2);
        let three = list.push_back(3);
        let four = list.push_back(4);
        let five = list.push_back(5);

        let removed_middle = list.remove(three).unwrap();
        assert_eq!(removed_middle, 3);
        assert_eq!(
            list,
            DlList {
                data: vec![
                    DlEntry::Occupied(OcEntry {
                        item: 1,
                        next: Some(1),
                        prev: None,
                        rm_cnt: 0,
                    }),
                    DlEntry::Occupied(OcEntry {
                        item: 2,
                        next: Some(3),
                        prev: Some(0),
                        rm_cnt: 0,
                    }),
                    DlEntry::Free { next_free: None },
                    DlEntry::Occupied(OcEntry {
                        item: 4,
                        next: Some(4),
                        prev: Some(1),
                        rm_cnt: 0,
                    }),
                    DlEntry::Occupied(OcEntry {
                        item: 5,
                        next: None,
                        prev: Some(3),
                        rm_cnt: 0,
                    }),
                ],
                next_free: Some(2),
                head: Some(0),
                tail: Some(4),
                rm_cnt: 1,
            }
        );

        let removed_head = list.remove(one).unwrap();
        assert_eq!(removed_head, 1);
        assert_eq!(
            list,
            DlList {
                data: vec![
                    DlEntry::Free { next_free: Some(2) },
                    DlEntry::Occupied(OcEntry {
                        item: 2,
                        next: Some(3),
                        prev: None,
                        rm_cnt: 0,
                    }),
                    DlEntry::Free { next_free: None },
                    DlEntry::Occupied(OcEntry {
                        item: 4,
                        next: Some(4),
                        prev: Some(1),
                        rm_cnt: 0,
                    }),
                    DlEntry::Occupied(OcEntry {
                        item: 5,
                        next: None,
                        prev: Some(3),
                        rm_cnt: 0,
                    }),
                ],
                next_free: Some(0),
                head: Some(1),
                tail: Some(4),
                rm_cnt: 2,
            }
        );

        let removed_tail = list.remove(five).unwrap();
        assert_eq!(removed_tail, 5);
        assert_eq!(
            list,
            DlList {
                data: vec![
                    DlEntry::Free { next_free: Some(2) },
                    DlEntry::Occupied(OcEntry {
                        item: 2,
                        next: Some(3),
                        prev: None,
                        rm_cnt: 0,
                    }),
                    DlEntry::Free { next_free: None },
                    DlEntry::Occupied(OcEntry {
                        item: 4,
                        next: None,
                        prev: Some(1),
                        rm_cnt: 0,
                    }),
                    DlEntry::Free { next_free: Some(0) },
                ],
                next_free: Some(4),
                head: Some(1),
                tail: Some(3),
                rm_cnt: 3,
            }
        );

        let removed_two = list.remove(two).unwrap();
        let removed_four = list.remove(four).unwrap();
        assert_eq!(removed_two,2);
        assert_eq!(removed_four, 4);

        assert_eq!(
            list,
            DlList {
                data: vec![
                    DlEntry::Free { next_free: Some(2) },
                    DlEntry::Free { next_free: Some(4) },
                    DlEntry::Free { next_free: None },
                    DlEntry::Free { next_free: Some(1) },
                    DlEntry::Free { next_free: Some(0) },
                ],
                next_free: Some(3),
                head: None,
                tail: None,
                rm_cnt: 5,
            }
        );

        assert!(list.remove(five).is_none());
    }

    #[test]
    fn push_after_remove() {
        let mut list = DlList::new();

        list.push_back(1);
        let two = list.push_back(2);
        list.push_back(3);

        let two = list.remove(two).unwrap();

        assert_eq!(two, 2);

        list.push_back(4);

        assert_eq!(
            list.data[0],
            DlEntry::Occupied(OcEntry {
                item: 1,
                next: Some(2),
                prev: None,
                rm_cnt: 0,
            })
        );

        assert_eq!(
            list.data[1],
            DlEntry::Occupied(OcEntry {
                item: 4,
                next: None,
                prev: Some(2),
                rm_cnt: 1,
            })
        );

        assert_eq!(
            list.data[2],
            DlEntry::Occupied(OcEntry {
                item: 3,
                next: Some(1),
                prev: Some(0),
                rm_cnt: 0,
            })
        );
    }

    #[test]
    fn iter() {
        let mut list = DlList::new();

        list.push_back(1);
        let two = list.push_back(2);
        list.push_back(3);

        list.remove(two);

        let mut iter = list.iter();

        assert_eq!(iter.next().unwrap(), &1);
        assert_eq!(iter.next().unwrap(), &3);
        assert!(iter.next().is_none());
    }

    #[test]
    fn index_of() {
        let mut list = DlList::new();

        list.push_back(1);
        list.push_back(2);
        list.push_back(1);

        assert_eq!(list.index_of(&1).unwrap(), DlIndex::new(0, 0));
        assert!(list.index_of(&3).is_none());
    }

    #[test]
    fn pop_front() {
        let mut list = DlList::new();

        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(list.pop_front().unwrap(), 1);
        assert_eq!(list.pop_front().unwrap(), 2);
        assert_eq!(list.pop_front().unwrap(), 3);

        assert_eq!(
            list,
            DlList {
                data: vec![
                    DlEntry::Free { next_free: None },
                    DlEntry::Free { next_free: Some(0) },
                    DlEntry::Free { next_free: Some(1) },
                ],
                next_free: Some(2),
                head: None,
                tail: None,
                rm_cnt: 3,
            }
        );
    }
}
