/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023-2026  Antonio Salsi <passy.linux@zresa.it>
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

/// This module defines types used for assets, such as icons and fonts.
/// It provides a type alias for icons, which consist of a width, height, and pixel data.
/// The pixel data is represented as a fixed-size array of bytes, where each byte corresponds to a pixel (0 for off, 1 for on).
/// The `Icon` type is generic over the size of the pixel data array, allowing for flexibility in defining icons of different sizes.
/// For example, an icon with a width of 12 pixels and a height of 10 pixels would have a pixel data array of size 120 (12 * 10).
/// This module can be extended in the future to include additional types for other kinds of assets, such as fonts or sprites.
/// The use of a type alias for icons helps to improve code readability and maintainability, as it provides a clear and consistent way to represent icon data throughout the codebase.
/// Overall, this module serves as a central place for defining and managing the types used for assets in the Hi Happy Garden project, making it easier to work with icons and other visual elements in the application.
/// The `Icon` type is defined as a tuple consisting of a width (u8), height (u8), and a fixed-size array of bytes representing the pixel data. The size of the pixel data array is determined by the generic parameter `COUNT`, which allows for flexibility in defining icons of different sizes. This design allows for a clear and consistent way to represent icon data throughout the codebase, improving readability and maintainability.
pub type Icon<const COUNT: usize> = (u8, u8, [u8; COUNT]);  //width, height, data