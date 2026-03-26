/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

use core::fmt::Display;
use core::ops::{Add, Sub};


mod sealed {
    pub trait Sealed {}
    impl Sealed for i8 {}
    impl Sealed for u8 {}
    impl Sealed for i16 {}
    impl Sealed for u16 {}
    impl Sealed for i32 {}
    impl Sealed for u32 {}
    impl Sealed for i64 {}
    impl Sealed for u64 {}
    impl Sealed for i128 {}
    impl Sealed for u128 {}
    impl Sealed for isize {}
    impl Sealed for usize {}
}

pub trait Integer: sealed::Sealed + Copy + PartialOrd + Add<Output = Self> + Sub<Output = Self> + Display {
    fn one() -> Self;
}
impl Integer for i8    { fn one() -> Self { 1 } }
impl Integer for u8    { fn one() -> Self { 1 } }
impl Integer for i16   { fn one() -> Self { 1 } }
impl Integer for u16   { fn one() -> Self { 1 } }
impl Integer for i32   { fn one() -> Self { 1 } }
impl Integer for u32   { fn one() -> Self { 1 } }
impl Integer for i64   { fn one() -> Self { 1 } }
impl Integer for u64   { fn one() -> Self { 1 } }
impl Integer for i128  { fn one() -> Self { 1 } }
impl Integer for u128  { fn one() -> Self { 1 } }
impl Integer for isize { fn one() -> Self { 1 } }
impl Integer for usize { fn one() -> Self { 1 } }
