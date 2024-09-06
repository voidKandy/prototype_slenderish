use bevy::prelude::Triangle2d;
use bevy_tnua::math::Vector2;
//
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryNode(u32);

impl From<u32> for BinaryNode {
    ///         1
    ///        / \
    ///       2   3
    fn from(value: u32) -> Self {
        if value <= 1 {
            panic!("THIS BINARY TREE STARTS AT 2");
        }
        Self(value)
    }
}
impl AsRef<u32> for BinaryNode {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

/// Most significant bit. Finds first with a value of 1 in binary
/// following implementation of this video: https://www.youtube.com/watch?v=2-zmWlM5XSE
/// Returns bit POSITION, so 1000 should return 4, not 8
pub fn msb(mut val: u32) -> u32 {
    // let mut intervals = vec![1, 2, 4, 8, 16];
    // intervals.reverse();
    // while let Some(interval) = intervals.pop() {
    //     val |= val >> interval;
    // }
    // val += 1;
    // val.trailing_zeros()
    32 - val.leading_zeros()
}

impl BinaryNode {
    // Assuming root node is level 0
    fn level(&self) -> u32 {
        let msb = msb(self.0);
        // YOU CHANGED THIS 2 FROM A 1
        msb - 1
    }

    fn index_in_level(&self) -> u32 {
        self.as_ref() - (1 << (msb(*self.as_ref()) - 1))
    }

    pub fn children_ids(&self) -> (BinaryNode, BinaryNode) {
        // let level = self.level();
        // let left_bin_id = self.as_ref() + (1 << (level + 2)) - (1 << (level + 1));
        // let right_bin_id = left_bin_id + 1;
        let left = self.as_ref() * 2;
        let right = left + 1;
        (left.into(), right.into())
    }

    pub fn level_start_index(level: u32) -> u32 {
        ((2 << level) - 1) & (!1u32)
    }

    pub fn from_triangle_index(idx: u32) -> Self {
        let mut level = 0;
        let mut index_level_start = 0;

        for i in 0..32 {
            let new_index_level_start = Self::level_start_index(i);
            if idx >= new_index_level_start {
                level = i;
                index_level_start = new_index_level_start;
            } else {
                break;
            }
        }

        ((1 << (level + 1)) + (idx - index_level_start)).into()
    }

    pub fn triangle_index(&self) -> u32 {
        let level = self.level() - 1;
        let index_level_start = Self::level_start_index(level);
        let index_in_level = self.index_in_level();

        index_level_start + index_in_level
    }

    /// We only need the x and y values because we will be mapping these over 2d noise
    /// So, given a grid size we can return the coordinates on a triangle given our constraints
    pub fn triangle_coords(&self, grid_size: f32) -> Triangle2d {
        let left_steps = self.steps_to_node();
        let last_pos = grid_size - 1. as f32;

        let (mut a, mut b, mut c) = if left_steps[0] {
            (
                Vector2::new(last_pos, last_pos),
                Vector2::new(0., 0.),
                Vector2::new(0., last_pos),
            )
        } else {
            (
                Vector2::new(0., 0.),
                Vector2::new(last_pos, last_pos),
                Vector2::new(last_pos, 0.),
            )
        };

        for left_step in left_steps.iter().skip(1) {
            if *left_step {
                let (new_a, new_b, new_c) = (c, a, (a + b) / 2.);
                a = new_a;
                b = new_b;
                c = new_c;
            } else {
                let (new_a, new_b, new_c) = (b, c, (a + b) / 2.);
                a = new_a;
                b = new_b;
                c = new_c;
            }
        }

        Triangle2d {
            vertices: [a, b, c],
        }
    }

    /// When we find the coordinates of a triangle we need to go through our triangles, iteratively
    /// splitting them in half and keep track of which triangle our binID is correlated with
    /// each step marked as true if it is a left step when going down from the root
    fn steps_to_node(&self) -> Vec<bool> {
        let mut steps = vec![false; self.level() as usize];
        let mut val = self.as_ref().clone();

        let mut i = 0;
        while val > 1 {
            if val % 2 == 0 {
                steps[(self.level() as usize - 1) - i] = true;
            }
            val /= 2;
            i += 1;
        }
        steps
    }

    pub fn errors_vec_index(&self, grid_size: f32) -> usize {
        let triangle_midpoint = self.midpoint_pixel_coords(grid_size);
        let midpoint_error_vec_index = triangle_midpoint[1] * grid_size + triangle_midpoint[0];
        midpoint_error_vec_index as usize
    }

    pub fn midpoint_pixel_coords(&self, grid_size: f32) -> Vector2 {
        let triangle_coords = self.triangle_coords(grid_size);
        let mid_point = (triangle_coords.vertices[0] + triangle_coords.vertices[1]) / 2.;
        Vector2::new(mid_point[0], mid_point[1])
    }
}
mod tests {
    #![allow(unused)]
    use bevy::prelude::Triangle2d;
    use bevy_tnua::math::Vector2;

    use super::{msb, BinaryNode};

    #[test]
    fn errors_vec_idx_correct() {
        let size = 16.;
        let node = BinaryNode::from(2);
        let expected = 127;
        assert_eq!(node.errors_vec_index(size), expected);
    }

    #[test]
    fn midpoint_correct() {
        let size = 16.;

        let node = BinaryNode::from(2);
        let expected = Vector2::new(7.5, 7.5);
        assert_eq!(node.midpoint_pixel_coords(size), expected);
        let node = BinaryNode::from(3);
        assert_eq!(node.midpoint_pixel_coords(size), expected);

        let node = BinaryNode::from(4);
        let expected = Vector2::new(7.5, 15.);
        assert_eq!(node.midpoint_pixel_coords(size), expected);
        let node = BinaryNode::from(5);
        let expected = Vector2::new(0., 7.5);
        assert_eq!(node.midpoint_pixel_coords(size), expected);
    }

    #[test]
    fn msb_works() {
        let val = 0b1010u32;
        assert_eq!(4, msb(val));

        let val = 0b0110u32;
        assert_eq!(3, msb(val));

        let val = 0b10110u32;
        assert_eq!(5, msb(val));

        let val = 0b100110u32;
        assert_eq!(6, msb(val));

        let val = 2u32;
        assert_eq!(2, msb(val));
    }

    #[test]
    fn triangle_coords_works() {
        let size = 50.0 - 1.;
        let node = BinaryNode::from(2);
        let expected = Triangle2d {
            vertices: [
                Vector2::new(size, size),
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, size),
            ],
        };
        assert_eq!(node.triangle_coords(size + 1.), expected);

        let node = BinaryNode::from(3);
        let expected = Triangle2d {
            vertices: [
                Vector2::new(0.0, 0.0),
                Vector2::new(size, size),
                Vector2::new(size, 0.0),
            ],
        };
        assert_eq!(node.triangle_coords(size + 1.), expected);
    }

    #[test]
    fn level_correct() {
        let node = BinaryNode::from(2);
        assert_eq!(node.level(), 1);
        let node = BinaryNode::from(6);
        assert_eq!(node.level(), 2);
        let node = BinaryNode::from(30);
        assert_eq!(node.level(), 4);
    }

    #[test]
    fn triangle_indices_correct() {
        assert_eq!(BinaryNode::level_start_index(0), 0);
        assert_eq!(BinaryNode::level_start_index(1), 2);
        assert_eq!(BinaryNode::level_start_index(2), 6);
        assert_eq!(BinaryNode::level_start_index(3), 14);

        assert_eq!(BinaryNode::from_triangle_index(0), 2.into());
        assert_eq!(BinaryNode::from_triangle_index(1), 3.into());
        assert_eq!(BinaryNode::from_triangle_index(2), 4.into());
        assert_eq!(BinaryNode::from_triangle_index(5), 7.into());
        assert_eq!(BinaryNode::from_triangle_index(9), 11.into());

        assert_eq!(BinaryNode::from(2).index_in_level(), 0);
        assert_eq!(BinaryNode::from(3).index_in_level(), 1);
        assert_eq!(BinaryNode::from(4).index_in_level(), 0);
        assert_eq!(BinaryNode::from(5).index_in_level(), 1);
        assert_eq!(BinaryNode::from(6).index_in_level(), 2);
        assert_eq!(BinaryNode::from(7).index_in_level(), 3);

        assert_eq!(BinaryNode::from(2).triangle_index(), 0);
        assert_eq!(BinaryNode::from(3).triangle_index(), 1);
        assert_eq!(BinaryNode::from(7).triangle_index(), 5);
        assert_eq!(BinaryNode::from(13).triangle_index(), 11);
    }

    #[test]
    fn steps_work() {
        let node = BinaryNode::from(6);
        let expected = vec![false, true];
        assert_eq!(node.steps_to_node(), expected);
    }

    #[test]
    fn children_correct() {
        let node = BinaryNode::from(2);
        let (left, right) = node.children_ids();
        assert_eq!(left, 4.into());
        assert_eq!(right, 5.into());

        let node = BinaryNode::from(8);
        let (left, right) = node.children_ids();
        assert_eq!(left, 16.into());
        assert_eq!(right, 17.into());
    }
}
