pub use bit_vec::BitVec;

use crate::{BinaryImage, Shape};

impl BinaryImage {
    pub fn operation(
        &self,
        other: &BinaryImage,
        operator: impl FnMut((&mut u8, &u8)),
    ) -> BinaryImage {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);
        let mut i = self.pixels.to_bytes();
        let u = other.pixels.to_bytes();
        i.iter_mut().zip(u.iter()).for_each(operator);
        BinaryImage {
            pixels: BitVec::from_bytes(&i),
            width: self.width,
            height: self.height,
        }
    }

    pub fn negative(&self) -> BinaryImage {
        let i = self.pixels.to_bytes();
        use std::ops::Not;
        let ii = i.iter().map(|x| x.not()).collect::<Vec<u8>>();
        BinaryImage {
            pixels: BitVec::from_bytes(&ii.as_slice()),
            width: self.width,
            height: self.height,
        }
    }

    pub fn diff(&self, other: &BinaryImage) -> BinaryImage {
        self.operation(other, |(x1, x2)| *x1 ^= *x2)
    }

    pub fn union(&self, other: &BinaryImage) -> BinaryImage {
        self.operation(other, |(x1, x2)| *x1 |= *x2)
    }

    pub fn intersect(&self, other: &BinaryImage) -> BinaryImage {
        self.operation(other, |(x1, x2)| *x1 &= *x2)
    }

    pub fn clustered_diff(&self, other: &BinaryImage) -> u32 {
        self.diff(other).significance(self.area(), std::u32::MAX)
    }

    /// early return if diff >= threshold, so maximum return value is equal to threshold
    pub fn significance(&self, area: u64, threshold: u32) -> u32 {
        let clusters = self.to_clusters(false);
        let mut diff: u64 = 0;
        let scale = 4 * 128 * 128;
        let divisor = area * self.width as u64;
        let threshold_u64 = threshold as u64 * divisor;
        for cluster in clusters.iter() {
            let size = cluster.size() as u64;
            let cluster_image = cluster.to_binary_image();
            let boundary = Shape::image_boundary_list(&cluster_image);
            let skeleton = cluster_image.to_skeleton();
            diff += scale * size *
                skeleton.stat.mean as u64 *
                skeleton.stat.count as u64 /
                boundary.len() as u64;
            if diff >= threshold_u64 {
                break;
            }
        }
        (diff / divisor) as u32
    }

    pub fn diff_and_count(&self, other: &BinaryImage) -> usize {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);
        let mut i = self.pixels.to_bytes();
        let u = other.pixels.to_bytes();
        i.iter_mut().zip(u.iter()).for_each(|(x1, x2)| *x1 ^= *x2);
        while i.len() % 4 != 0 {
            i.push(0);
        }
        let mut count = 0;
        for ii in (0..i.len()).step_by(4) {
            count += Self::popcount(u32::from_be_bytes([i[ii], i[ii + 1], i[ii + 2], i[ii + 3]]))
                as usize;
        }
        count
    }

    #[inline(always)]
    pub fn popcount(i: u32) -> u32 {
        i.count_ones()
    }

    /// expand a binary image using a circular brush
    pub fn stroke(&self, s: u32) -> BinaryImage {
        let mut new_image = BinaryImage::new_w_h(self.width + s as usize, self.height + s as usize);
        let ox = (s as usize) >> 1;
        let oy = (s as usize) >> 1;
        let ss = (s >> 1) as i32;
        for y in 0..self.height {
            for x in 0..self.width {
                let v = self.get_pixel(x, y);
                if v {
                    for yy in -ss..ss {
                        for xx in -ss..ss {
                            if (((xx * xx + yy * yy) as f64).sqrt() as i32) < ss {
                                new_image.set_pixel(
                                    (x as i32 + xx + ox as i32) as usize,
                                    (y as i32 + yy + oy as i32) as usize,
                                    true,
                                );
                            }
                        }
                    }
                }
            }
        }
        new_image
    }

    pub fn dilate(&self) -> BinaryImage {
        let mut result = BinaryImage::new_w_h(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_pixel(x, y) {
                    // Set the current pixel
                    result.set_pixel(x, y, true);

                    // Set neighboring pixels (8-connected)
                    for i in -2..=2 {
                        for j in -2..=2 {
                            let nx = x as isize + i;
                            let ny = y as isize + j;
                            if nx >= 0 && ny >= 0 && nx < self.width as isize && ny < self.height as isize {
                                result.set_pixel(nx as usize, ny as usize, true);
                            }
                        }
                    }
                }
            }
        }

        result
    }

    pub fn remove_disjoint_components(&self) -> BinaryImage {
        // Find connected components using 8-connectivity
        let mut visited = vec![false; self.width * self.height];
        let mut components = Vec::new();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_pixel(x, y) && !visited[y * self.width + x] {
                    let mut component = Vec::new();
                    let mut stack = vec![(x, y)];
                    
                    while let Some((cx, cy)) = stack.pop() {
                        let idx = cy * self.width + cx;
                        if visited[idx] {
                            continue;
                        }
                        visited[idx] = true;
                        component.push((cx, cy));
                        
                        // Check 8-connected neighbors (including diagonals)
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let nx = cx as i32 + dx;
                                let ny = cy as i32 + dy;
                                if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                                    let nx = nx as usize;
                                    let ny = ny as usize;
                                    if self.get_pixel(nx, ny) && !visited[ny * self.width + nx] {
                                        stack.push((nx, ny));
                                    }
                                }
                            }
                        }
                    }
                    components.push(component);
                }
            }
        }

        // Find largest component
        let largest = components.iter()
            .max_by_key(|c| c.len())
            .unwrap();
            
        // Create new image with only the largest component
        let mut image = BinaryImage::new_w_h(self.width, self.height);
        for &(x, y) in largest {
            image.set_pixel(x, y, true);
        }

        // Fix any remaining diagonal connections
        let mut fixed_image = image.fix_diagonal_cc();
        
        // Recursively remove any new disjoint components that may have been created
        if components.len() > 1 {
            fixed_image = fixed_image.remove_disjoint_components();
        }

        fixed_image
    }

    pub fn fix_diagonal_cc(&self) -> BinaryImage {
        let mut to_add = Vec::new();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                if self.get_pixel(x, y) {
                    // Check diagonal slots
                    if self.get_pixel(x - 1, y - 1) && !self.get_pixel(x - 1, y) && !self.get_pixel(x, y - 1) {
                        to_add.push((y - 1) * self.width + x);
                    }

                    if self.get_pixel(x + 1, y - 1) && !self.get_pixel(x + 1, y) && !self.get_pixel(x, y - 1) {
                        to_add.push((y - 1) * self.width + x);
                    }

                    if self.get_pixel(x - 1, y + 1) && !self.get_pixel(x - 1, y) && !self.get_pixel(x, y + 1) {
                        to_add.push((y + 1) * self.width + x);
                    }

                    if self.get_pixel(x + 1, y + 1) && !self.get_pixel(x + 1, y) && !self.get_pixel(x, y + 1) {
                        to_add.push((y + 1) * self.width + x);
                    }
                }
            }
        }

        let mut result = self.clone();

        // if to_add.len() > 0 {
        //     println!("Filling in {} diagonal cc slots in segment", to_add.len());
        // }

        // Add pixels in the diagonal slots
        for index in to_add {
            let x = index % self.width;
            let y = index / self.width;
            result.set_pixel(x, y, true);
        }


        return result
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_diff() {
        let mut a = BinaryImage::new_w_h(2, 2);
        a.set_pixel(0, 0, true);
        let mut b = BinaryImage::new_w_h(2, 2);
        b.set_pixel(1, 1, true);
        assert_eq!(a.diff_and_count(&b), 2);

        let mut a = BinaryImage::new_w_h(3, 3);
        a.set_pixel(1, 1, true);
        let mut b = BinaryImage::new_w_h(3, 3);
        b.set_pixel(1, 1, true);
        assert_eq!(a.diff_and_count(&b), 0);

        let mut a = BinaryImage::new_w_h(3, 3);
        a.set_pixel(0, 0, true);
        a.set_pixel(1, 1, true);
        let mut b = BinaryImage::new_w_h(3, 3);
        b.set_pixel(1, 1, true);
        b.set_pixel(2, 2, true);
        assert_eq!(a.diff_and_count(&b), 2);
    }

    #[test]
    fn negative_image() {
        assert_eq!(
            BinaryImage::from_string(&(
                "*-*\n".to_owned() +
                "-*-\n" +
                "*-*\n"
            ))
            .negative()
            .to_string(),
            BinaryImage::from_string(&(
                "-*-\n".to_owned() +
                "*-*\n" +
                "-*-\n"
            )).to_string()
        );
    }
}