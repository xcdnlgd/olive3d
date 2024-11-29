use std::{
    fmt::Display,
    ops::{Add, Div, Index, IndexMut, Mul, Sub},
};

#[derive(Clone)]
pub struct Line2D {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}
impl Line2D {
    pub fn box_clip(&self, x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> Option<Self> {
        if x_max < x_min {
            return None;
        }
        if y_max < y_min {
            return None;
        }

        // Cohen–Sutherland
        // 	        left	central right
        // top	    1001	1000    1010
        // central	0001	0000	0010
        // bottom	0101	0100	0110
        const INSIDE: u8 = 0b0000;
        const LEFT: u8 = 0b0001;
        const RIGHT: u8 = 0b0010;
        const BOTTOM: u8 = 0b0100;
        const TOP: u8 = 0b1000;

        let outcode = |x, y| {
            let mut code = INSIDE;
            if x < x_min {
                code |= LEFT;
            } else if x > x_max {
                code |= RIGHT;
            }
            if y < y_min {
                code |= BOTTOM;
            } else if y > y_max {
                code |= TOP;
            }
            code
        };

        let mut line = self.clone();

        let mut outcode_start = outcode(line.x0, line.y0);
        let mut outcode_end = outcode(line.x1, line.y1);
        loop {
            if (outcode_start | outcode_end) == 0 {
                // bitwise OR is 0: both points inside window
                return Some(line);
            } else if (outcode_start & outcode_end) != 0 {
                // bitwise AND is not 0: see the top comment, the line is fully outside the window
                return None;
            }

            // At least one endpoint is outside the clip rectangle; pick it.
            // outcode_center is 0b0000
            let outcode_out = u8::max(outcode_start, outcode_end);

            let (x_s, y_s) = (line.x0, line.y0);
            let (x_e, y_e) = (line.x1, line.y1);
            // Now find the intersection point;
            // use formulas:
            // No need to worry about divide-by-zero because, in each case, the
            // outcode bit being tested guarantees the denominator is non-zero
            let dx = x_e - x_s;
            let dy = y_e - y_s;
            let x;
            let y;
            if (outcode_out & TOP) != 0 {
                // point above the window
                x = x_s + (y_max - y_s) / dy * dx;
                y = y_max;
            } else if (outcode_out & BOTTOM) != 0 {
                // point below the window
                x = x_s + (y_min - y_s) / dy * dx;
                y = y_min;
            } else if (outcode_out & RIGHT) != 0 {
                // point is to the right of the window
                y = y_s + (x_max - x_s) / dx * dy;
                x = x_max;
            } else if (outcode_out & LEFT) != 0 {
                // point is to the left of the window
                y = y_s + (x_min - x_s) / dx * dy;
                x = x_min;
            } else {
                panic!("what!!!?");
            }

            // Now we move outside point to intersection point to clip
            // and get ready for next pass.
            if outcode_start == outcode_out {
                outcode_start = outcode(x, y);
                line.x0 = x;
                line.y0 = y;
            } else {
                outcode_end = outcode(x, y);
                line.x1 = x;
                line.y1 = y;
            }
        }
    }
}

pub struct Ray {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub reached: bool,
    sx: i32,
    sy: i32,
    dx: i32,
    dy: i32,
    error: i32,
    x: i32,
    y: i32,
    next: fn(&mut Self) -> (i32, i32),
}
impl Ray {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        // bresenham
        let sx = if x1 < x0 { -1 } else { 1 };
        let sy = if y1 < y0 { -1 } else { 1 };

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();

        let x = x0;
        let y = y0;

        let error;
        let next: fn(&mut Self) -> (i32, i32);
        let reached;
        if dy < dx {
            reached = x == x1;
            error = -dx;
            next = Self::iter_x;
        } else {
            reached = y == y1;
            error = -dy;
            next = Self::iter_y;
        }

        Self {
            x0,
            y0,
            x1,
            y1,
            reached,
            sx,
            sy,
            dx,
            dy,
            error,
            x,
            y,
            next,
        }
    }
    pub fn next_xy(&mut self) -> (i32, i32) {
        (self.next)(self)
    }
    fn iter_x(&mut self) -> (i32, i32) {
        let result = (self.x, self.y);
        self.error += self.dy + self.dy;
        if self.error >= 0 {
            self.y += self.sy;
            self.error -= self.dx + self.dx;
        }
        self.x += self.sx;
        if self.x == self.x1 {
            self.reached = true;
        }
        result
    }
    fn iter_y(&mut self) -> (i32, i32) {
        let result = (self.x, self.y);
        self.error += self.dx + self.dx;
        if self.error >= 0 {
            self.x += self.sx;
            self.error -= self.dy + self.dy;
        }
        self.y += self.sy;
        if self.y == self.y1 {
            self.reached = true;
        }
        result
    }
}

pub trait Dot<Rhs = Self> {
    type Output;
    fn dot(self, rhs: Rhs) -> Self::Output;
}

pub trait Cross<Rhs = Self> {
    type Output;
    fn cross(self, rhs: Rhs) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector<const D: usize> {
    data: [f32; D],
}

impl<const D: usize> Vector<D> {
    pub fn zero() -> Self {
        Self { data: [0f32; D] }
    }
    #[inline]
    fn get(&self, name: &str) -> Option<f32> {
        match name {
            "x" => self.data.first(),
            "y" => self.data.get(1),
            "z" => self.data.get(2),
            "w" => self.data.get(3),
            _ => None,
        }
        .copied()
    }
    pub fn length_square(&self) -> f32 {
        self.dot(self)
    }
    pub fn length(&self) -> f32 {
        self.length_square().sqrt()
    }
    pub fn normalize(&self) -> Self {
        self / self.length()
    }
}

impl<const D: usize> Index<usize> for Vector<D> {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const D: usize> IndexMut<usize> for Vector<D> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

macro_rules! impl_bin_op {
    (impl$(<$(const $T:ident: $t:ty),+>)? $Op:ident<$rhs:ty> for $lhs:ty, $op:ident, $output:ty) => {
        impl$(<$(const $T: $t),+>)? $Op<$rhs> for $lhs {
            type Output = $output;
            fn $op(self, rhs: $rhs) -> Self::Output {
                (&self).$op(&rhs)
            }
        }
        impl$(<$(const $T: $t),+>)? $Op<&$rhs> for $lhs {
            type Output = $output;
            fn $op(self, rhs: &$rhs) -> Self::Output {
                (&self).$op(rhs)
            }
        }
        impl$(<$(const $T: $t),+>)? $Op<$rhs> for &$lhs {
            type Output = $output;
            fn $op(self, rhs: $rhs) -> Self::Output {
                (self).$op(&rhs)
            }
        }
    };
}

impl<const D: usize> Dot<&Vector<D>> for &Vector<D> {
    type Output = f32;
    fn dot(self, rhs: &Vector<D>) -> Self::Output {
        let mut result = 0.0;
        for i in 0..D {
            result += self.data[i] * rhs.data[i];
        }
        result
    }
}
impl_bin_op!(impl<const D: usize> Dot<Vector<D>> for Vector<D>, dot, f32);

impl<const D: usize> Add<&Vector<D>> for &Vector<D> {
    type Output = Vector<D>;
    fn add(self, rhs: &Vector<D>) -> Self::Output {
        let mut vector = self.clone();
        for i in 0..D {
            vector.data[i] += rhs.data[i];
        }
        vector
    }
}
impl_bin_op!(impl<const D: usize> Add<Vector<D>> for Vector<D>, add, Vector<D>);

impl<const D: usize> Sub<&Vector<D>> for &Vector<D> {
    type Output = Vector<D>;
    fn sub(self, rhs: &Vector<D>) -> Self::Output {
        let mut vector = self.clone();
        for i in 0..D {
            vector.data[i] -= rhs.data[i];
        }
        vector
    }
}
impl_bin_op!(impl<const D: usize> Sub<Vector<D>> for Vector<D>, sub, Vector<D>);

impl<const D: usize> Div<&f32> for &Vector<D> {
    type Output = Vector<D>;
    fn div(self, rhs: &f32) -> Self::Output {
        let mut vector = self.clone();
        vector.data.iter_mut().for_each(|n| *n /= *rhs);
        vector
    }
}
impl_bin_op!(impl<const D: usize> Div<f32> for Vector<D>, div, Vector<D>);

impl<const D: usize> Mul<&f32> for &Vector<D> {
    type Output = Vector<D>;
    fn mul(self, rhs: &f32) -> Self::Output {
        let mut vector = self.clone();
        vector.data.iter_mut().for_each(|n| *n *= *rhs);
        vector
    }
}
impl_bin_op!(impl<const D: usize> Mul<f32> for Vector<D>, mul, Vector<D>);

impl<const D: usize> Mul<&Vector<D>> for &f32 {
    type Output = Vector<D>;
    fn mul(self, rhs: &Vector<D>) -> Self::Output {
        let mut vector = rhs.clone();
        vector.data.iter_mut().for_each(|n| *n *= *self);
        vector
    }
}
impl_bin_op!(impl<const D: usize> Mul<Vector<D>> for f32, mul, Vector<D>);

pub type Vector2 = Vector<2>;
pub type Vector3 = Vector<3>;
pub type Vector4 = Vector<4>;

macro_rules! impl_vector_methods {
    ($ty:ident, $($name:ident),+) => {
        impl $ty {
            $(pub fn $name(&self) -> f32 {
                self.get(stringify!($name)).unwrap_or(0.0)
            })+
            #[allow(unused_assignments)]
            pub fn new($($name: f32),+) -> Self {
                let mut v = Self::zero();
                let mut i = 0;
                $(
                    v.data[i] = $name;
                    i += 1;
                )+
                v
            }
        }
    };
}

impl_vector_methods!(Vector2, x, y);
impl_vector_methods!(Vector3, x, y, z);
impl_vector_methods!(Vector4, x, y, z, w);

impl Cross<&Vector3> for &Vector3 {
    type Output = Vector3;
    fn cross(self, rhs: &Vector3) -> Self::Output {
        let x = self.y() * rhs.z() - self.z() * rhs.y();
        let y = self.z() * rhs.x() - self.x() * rhs.z();
        let z = self.x() * rhs.y() - self.y() * rhs.x();
        Self::Output::new(x, y, z)
    }
}
impl_bin_op!(impl Cross<Vector3> for Vector3, cross, Vector3);

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<const R: usize, const C: usize> {
    pub rows: [[f32; C]; R],
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    #[inline]
    pub fn zero() -> Self {
        Self {
            rows: [[0f32; C]; R],
        }
    }
    #[inline]
    pub fn from_rows(rows: [[f32; C]; R]) -> Self {
        Self::from(rows)
    }
}
impl<const N: usize> Matrix<N, N> {
    #[inline]
    pub fn identity() -> Self {
        let mut mat = Self::zero();
        for i in 0..N {
            mat[i][i] = 1f32;
        }
        mat
    }
}

impl<const R: usize, const C: usize> From<[[f32; C]; R]> for Matrix<R, C> {
    fn from(rows: [[f32; C]; R]) -> Self {
        Self { rows }
    }
}
impl<const R: usize, const C: usize> Display for Matrix<R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output += "[\n";
        for row in self.rows.iter() {
            output += &format!("    {:.5?},\n", row);
        }
        output += "]";
        write!(f, "{output}")
    }
}
impl<const R: usize, const C: usize> Index<usize> for Matrix<R, C> {
    type Output = [f32; C];
    fn index(&self, index: usize) -> &Self::Output {
        &self.rows[index]
    }
}
impl<const R: usize, const C: usize> IndexMut<usize> for Matrix<R, C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.rows[index]
    }
}
impl<const A: usize, const B: usize, const C: usize> Mul<&Matrix<B, C>> for &Matrix<A, B> {
    type Output = Matrix<A, C>;
    fn mul(self, rhs: &Matrix<B, C>) -> Self::Output {
        let mut result = Self::Output::zero();
        for a in 0..A {
            for c in 0..C {
                for b in 0..B {
                    result[a][c] += self[a][b] * rhs[b][c];
                }
            }
        }
        result
    }
}
impl_bin_op!(impl<const A: usize, const B: usize, const C: usize> Mul<Matrix<B, C>> for Matrix<A, B>, mul, Matrix<A, C>);

pub type Matrix2 = Matrix<2, 2>;
pub type Matrix3 = Matrix<3, 3>;
pub type Matrix4 = Matrix<4, 4>;
