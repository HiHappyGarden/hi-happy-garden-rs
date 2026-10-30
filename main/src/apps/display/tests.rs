/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
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


//! Display widget tests: each widget is driven frame by frame with the
//! encoder/button flags the display task would pass, on a [`MockLcd`], and
//! checked through what it draws and what it answers.

use alloc::string::String;

use osal_rs::os::{System, SystemFn};
use osal_rs::os::types::{EventBits, TickType};
use osal_rs::utils::{Bytes, Error, Result};

use super::check::Check;
use super::commons::{FIRST_ROW_Y, MAX_SIZE, ONLY_ONE_ROW_Y, SECOND_ROW_Y, clean_context, scroll_text};
use super::date::Date;
use super::input::Input;
use super::number::Number;
use super::select::Select;
use super::text::Text;
use super::time::Time;
use crate::apps::signals::display::DisplayFlag::{
    self, ButtonPressed, ButtonReleased, EncoderButtonPressed, EncoderButtonReleased, EncoderRotatedClockwise,
    EncoderRotatedCounterClockwise,
};
use crate::apps::test_helpers::{DRAW, DRAW_INTERNAL, MockLcd, mock_rtc};
use crate::assets::ic_check_off::IC_CHECK_OFF;
use crate::assets::ic_check_on::IC_CHECK_ON;
use crate::drivers::date_time::DateTime;
use crate::tests::{TestStats, run_tests, test_assert, test_assert_eq};
use crate::traits::integer::Integer;
use crate::traits::lcd_display::LCDWriteMode;
use crate::traits::screen::{Answer, Screen, ScreenParam, ScreenSelections};

const TAG: &str = "DisplayTests";

const TITLE: &str = "Title";

/// Longer than `LONG_PRESS_TICK` of the widgets, so a press/release pair
/// around a delay of this many ticks is a long press.
const LONG_PRESS: TickType = 350;

/// 2026-10-14 00:00:00 UTC: only read by the date/time editors when they
/// get no value to edit.
const NOW: i64 = 1_791_936_000;

#[inline]
fn bits(flag: DisplayFlag) -> EventBits {
    flag as EventBits
}

/// Draws one frame of `widget` with the given input `flags`, as the display
/// task does on each wake up, and returns its answer and the updated signal.
fn frame<T, N: Integer, const K: usize>(
    widget: &mut dyn Screen<T, N, K>,
    lcd: &mut MockLcd,
    flags: EventBits,
    param: ScreenParam<N, K>,
) -> Result<(Answer<N, K>, EventBits)> {
    let rtc = mock_rtc(NOW);
    let mut signal = flags;
    let answer = widget.draw(lcd, &mut signal, &rtc, &Bytes::<MAX_SIZE>::from(TITLE), param)?;
    Ok((answer, signal))
}

fn is_pending<N: Integer, const K: usize>(answer: &Answer<N, K>) -> bool {
    matches!(answer, Answer::Pending)
}

fn is_cancelled<N: Integer, const K: usize>(answer: &Answer<N, K>) -> bool {
    matches!(answer, Answer::Cancelled)
}

// ---------------------------------------------------------------- commons

fn test_scroll_text_fits() -> Result<()> {
    let mut signal: EventBits = 0;
    let (text, x) = scroll_text("abc", &mut signal, 2, 128, 8, 200);
    test_assert_eq!(text.as_str(), "abc");
    // Centred in the visible area: 2 + (128 - 3 * 8) / 2
    test_assert_eq!(x, 54);
    test_assert_eq!(signal, 0, "a text that fits does not need redraws");

    let (text, x) = scroll_text("", &mut signal, 2, 128, 8, 200);
    test_assert_eq!(text.as_str(), "");
    test_assert_eq!(x, 66);
    Ok(())
}

fn test_scroll_text_exact_width() -> Result<()> {
    let mut signal: EventBits = 0;
    let (text, x) = scroll_text("0123456789abcdef", &mut signal, 2, 128, 8, 200);
    test_assert_eq!(text.as_str(), "0123456789abcdef");
    test_assert_eq!(x, 2);
    Ok(())
}

fn test_scroll_text_scrolls() -> Result<()> {
    const TEXT: &str = "abcdefghijklmnopqrst";
    let mut signal: EventBits = 0;
    let (text, x) = scroll_text(TEXT, &mut signal, 2, 128, 8, 200);
    test_assert_eq!(x, 2, "a scrolling text starts at the left margin");
    test_assert_eq!(text.as_str().chars().count(), 16);

    // Any window of the circular text, separator included.
    let mut ring = String::from(TEXT);
    ring.push_str("  ");
    ring.push_str(TEXT);
    test_assert!(ring.contains(text.as_str()), "{:?} is not a window of {TEXT:?}", text.as_str());
    Ok(())
}

fn test_scroll_text_multibyte() -> Result<()> {
    let mut signal: EventBits = 0;
    let (text, _) = scroll_text("perché è così", &mut signal, 0, 128, 8, 200);
    test_assert_eq!(text.as_str(), "perché è così");

    // 40 chars of 3 bytes do not fit the 100 bytes buffer: the copy stops
    // before the first char that would be cut in half.
    let long: String = core::iter::repeat('€').take(40).collect();
    let (text, _) = scroll_text(&long, &mut signal, 0, 255, 5, 200);
    let expected: String = core::iter::repeat('€').take(33).collect();
    test_assert_eq!(text.as_str(), expected.as_str());
    Ok(())
}

fn test_clean_context() -> Result<()> {
    let mut lcd = MockLcd::new();
    clean_context(&mut lcd)?;
    // Everything below the 11px header (17.5% of 64px)
    test_assert!(lcd.rects == [(0, 11, MockLcd::WIDTH, 53, LCDWriteMode::REMOVE)], "header or body area wrong");

    let mut lcd = MockLcd { height: Some(0), ..MockLcd::new() };
    clean_context(&mut lcd)?;
    test_assert!(lcd.rects.is_empty(), "no body to clean");
    Ok(())
}

// ------------------------------------------------------------------- text

fn test_text_one_row() -> Result<()> {
    let mut lcd = MockLcd::new();
    let rtc = mock_rtc(NOW);
    let mut signal: EventBits = 0;
    let answer = Text::new().draw(&mut lcd, &mut signal, &rtc, &Bytes::<MAX_SIZE>::from("Hello"), ScreenParam::default())?;
    test_assert!(is_pending(&answer));
    test_assert!(lcd.texts_at(ONLY_ONE_ROW_Y) == ["Hello"], "{:?}", lcd.texts_at(ONLY_ONE_ROW_Y));
    test_assert!(signal & DRAW_INTERNAL != 0, "text does not request its draw");
    test_assert!(signal & DRAW == 0, "text draw must not count as user activity");
    Ok(())
}

fn test_text_draw_requested_once() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut text = Text::new();
    let (_, signal) = frame(&mut text, &mut lcd, 0, ScreenParam::default())?;
    test_assert!(signal & DRAW_INTERNAL != 0, "first frame does not request its draw");
    let (_, signal) = frame(&mut text, &mut lcd, 0, ScreenParam::default())?;
    test_assert!(signal & DRAW_INTERNAL == 0, "draw requested again on an unchanged text");
    Ok(())
}

fn test_text_two_rows() -> Result<()> {
    let mut lcd = MockLcd::new();
    let rtc = mock_rtc(NOW);
    let mut signal: EventBits = 0;
    Text::new().draw(&mut lcd, &mut signal, &rtc, &Bytes::<MAX_SIZE>::from("Hi|There"), ScreenParam::default())?;
    test_assert_eq!(lcd.last_text_at(FIRST_ROW_Y), "Hi");
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "There");
    test_assert!(lcd.texts_at(ONLY_ONE_ROW_Y).is_empty());
    Ok(())
}

fn test_text_too_many_rows() -> Result<()> {
    let mut lcd = MockLcd::new();
    let rtc = mock_rtc(NOW);
    let mut signal: EventBits = 0;
    let ret = Text::new().draw(&mut lcd, &mut signal, &rtc, &Bytes::<MAX_SIZE>::from("a|b|c"), ScreenParam::default());
    test_assert!(matches!(ret, Err(Error::Unhandled(_))), "three rows accepted");
    Ok(())
}

fn test_text_answers() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut text = Text::new();
    let (answer, _) = frame(&mut text, &mut lcd, bits(EncoderButtonReleased), ScreenParam::default())?;
    test_assert!(matches!(answer, Answer::Confirmed(ScreenParam::None)));
    let (answer, _) = frame(&mut text, &mut lcd, bits(ButtonReleased), ScreenParam::default())?;
    test_assert!(is_cancelled(&answer));
    Ok(())
}

// ------------------------------------------------------------------ check

fn test_check_initial_value() -> Result<()> {
    let mut lcd = MockLcd::new();

    let mut check = Check::new();
    test_assert!(matches!(check.get_value(), Err(Error::NullPtr)), "value before the first draw");
    let (answer, _) = frame(&mut check, &mut lcd, 0, ScreenParam::Check(true))?;
    test_assert!(is_pending(&answer));
    test_assert_eq!(check.get_value()?, true);
    test_assert!(lcd.bitmaps.last().is_some_and(|b| b.4 == IC_CHECK_ON.2), "checked icon not drawn");

    // Any other parameter starts unchecked
    let mut lcd = MockLcd::new();
    let mut check = Check::new();
    frame(&mut check, &mut lcd, 0, ScreenParam::None)?;
    test_assert_eq!(check.get_value()?, false);
    test_assert!(lcd.bitmaps.last().is_some_and(|b| b.4 == IC_CHECK_OFF.2), "unchecked icon not drawn");
    Ok(())
}

fn test_check_toggle_and_confirm() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut check = Check::new();
    frame(&mut check, &mut lcd, 0, ScreenParam::Check(true))?;

    let (_, signal) = frame(&mut check, &mut lcd, bits(EncoderRotatedClockwise), ScreenParam::Check(true))?;
    test_assert!(signal & DRAW != 0, "no redraw after a rotation");
    test_assert!(lcd.bitmaps.last().is_some_and(|b| b.4 == IC_CHECK_OFF.2), "rotation did not uncheck");

    let (answer, _) = frame(&mut check, &mut lcd, bits(EncoderButtonReleased), ScreenParam::Check(true))?;
    test_assert!(matches!(answer, Answer::Confirmed(ScreenParam::Check(false))));
    test_assert_eq!(check.get_value()?, false);

    // Counter-clockwise toggles as well
    frame(&mut check, &mut lcd, bits(EncoderRotatedCounterClockwise), ScreenParam::Check(true))?;
    let (answer, _) = frame(&mut check, &mut lcd, bits(EncoderButtonReleased), ScreenParam::Check(true))?;
    test_assert!(matches!(answer, Answer::Confirmed(ScreenParam::Check(true))));
    Ok(())
}

fn test_check_cancel_and_drop() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut check = Check::new();
    let (answer, _) = frame(&mut check, &mut lcd, bits(ButtonReleased), ScreenParam::Check(true))?;
    test_assert!(is_cancelled(&answer));

    check.clear();
    test_assert!(matches!(check.get_value(), Err(Error::NullPtr)), "value kept after drop");
    // After a drop the parameter is read again
    frame(&mut check, &mut lcd, 0, ScreenParam::Check(false))?;
    test_assert_eq!(check.get_value()?, false);
    Ok(())
}

// ----------------------------------------------------------------- number

fn test_number_rotation_wraps() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut number = Number::<u8>::new(1, 3);

    frame(&mut number, &mut lcd, 0, ScreenParam::Number(2))?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "2");

    for (flag, expected) in [
        (EncoderRotatedClockwise, "3"),
        (EncoderRotatedClockwise, "1"),
        (EncoderRotatedCounterClockwise, "3"),
        (EncoderRotatedCounterClockwise, "2"),
    ] {
        let (answer, signal) = frame(&mut number, &mut lcd, bits(flag), ScreenParam::Number(2))?;
        test_assert!(is_pending(&answer));
        test_assert!(signal & DRAW != 0, "no redraw after {flag:?}");
        test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), expected, "after {flag:?}");
    }
    Ok(())
}

fn test_number_without_value() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut number = Number::<i16>::new(-5, 5);
    frame(&mut number, &mut lcd, 0, ScreenParam::None)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "-5", "no value shows the minimum");

    let mut number = Number::<i16>::new(-5, 5);
    frame(&mut number, &mut lcd, bits(EncoderRotatedClockwise), ScreenParam::None)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "-5", "first clockwise step starts from the minimum");

    let mut number = Number::<i16>::new(-5, 5);
    frame(&mut number, &mut lcd, bits(EncoderRotatedCounterClockwise), ScreenParam::None)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "5", "first counter-clockwise step starts from the maximum");
    Ok(())
}

fn test_number_confirm_and_cancel() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut number = Number::<u8>::new(0, 9);
    test_assert!(matches!(number.get_value(), Err(Error::NullPtr)), "value before confirm");

    frame(&mut number, &mut lcd, 0, ScreenParam::Number(4))?;
    frame(&mut number, &mut lcd, bits(EncoderRotatedClockwise), ScreenParam::Number(4))?;
    test_assert!(matches!(number.get_value(), Err(Error::NullPtr)), "value set before confirm");

    let (answer, _) = frame(&mut number, &mut lcd, bits(EncoderButtonReleased), ScreenParam::Number(4))?;
    test_assert!(matches!(answer, Answer::Confirmed(ScreenParam::Number(5))));
    test_assert_eq!(number.get_value()?, 5);

    number.clear();
    test_assert!(matches!(number.get_value(), Err(Error::NullPtr)), "value kept after drop");

    let (answer, _) = frame(&mut number, &mut lcd, bits(ButtonReleased), ScreenParam::Number(4))?;
    test_assert!(is_cancelled(&answer));
    Ok(())
}

/// Both ends of the range passed to `new_with_range` can be selected.
fn test_number_range_inclusive() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut number = Number::<u8>::new_with_range(0..=59);
    frame(&mut number, &mut lcd, bits(EncoderRotatedCounterClockwise), ScreenParam::None)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "59", "range end not reachable");
    frame(&mut number, &mut lcd, bits(EncoderRotatedClockwise), ScreenParam::None)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "0", "range end not followed by the start");
    Ok(())
}

/// Wrapping at the bounds of the integer type must not overflow.
fn test_number_type_bounds() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut number = Number::<u8>::new(0, 9);
    frame(&mut number, &mut lcd, bits(EncoderRotatedCounterClockwise), ScreenParam::Number(0))?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "9", "below 0u8");

    let mut number = Number::<u8>::new(0, u8::MAX);
    frame(&mut number, &mut lcd, bits(EncoderRotatedClockwise), ScreenParam::Number(u8::MAX))?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "0", "above u8::MAX");

    let mut number = Number::<i8>::new(i8::MIN, i8::MAX);
    frame(&mut number, &mut lcd, bits(EncoderRotatedCounterClockwise), ScreenParam::Number(i8::MIN))?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "127", "below i8::MIN");
    Ok(())
}

// ----------------------------------------------------------------- select

fn selections(selected: Option<usize>) -> ScreenSelections<3> {
    let mut ret: ScreenSelections<3> = Default::default();
    for (i, label) in ["Alpha", "Beta", "Gamma"].into_iter().enumerate() {
        ret[i] = (Bytes::from(label), selected == Some(i));
    }
    ret
}

fn test_select_starts_on_selected() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut select = Select::<3>::new();
    frame(&mut select, &mut lcd, 0, ScreenParam::Selects(selections(Some(1))))?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "Beta");

    let mut select = Select::<3>::new();
    frame(&mut select, &mut lcd, 0, ScreenParam::Selects(selections(None)))?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "Alpha", "no selection starts on the first entry");
    Ok(())
}

fn test_select_rotation_wraps() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut select = Select::<3>::new();
    frame(&mut select, &mut lcd, 0, ScreenParam::Selects(selections(Some(1))))?;

    for (flag, expected) in [
        (EncoderRotatedClockwise, "Gamma"),
        (EncoderRotatedClockwise, "Alpha"),
        (EncoderRotatedCounterClockwise, "Gamma"),
        (EncoderRotatedCounterClockwise, "Beta"),
    ] {
        let (_, signal) = frame(&mut select, &mut lcd, bits(flag), ScreenParam::Selects(selections(Some(1))))?;
        test_assert!(signal & DRAW != 0, "no redraw after {flag:?}");
        test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), expected, "after {flag:?}");
    }
    Ok(())
}

fn test_select_confirm() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut select = Select::<3>::new();
    frame(&mut select, &mut lcd, 0, ScreenParam::Selects(selections(Some(0))))?;
    frame(&mut select, &mut lcd, bits(EncoderRotatedCounterClockwise), ScreenParam::Selects(selections(Some(0))))?;

    let (answer, _) = frame(&mut select, &mut lcd, bits(EncoderButtonReleased), ScreenParam::Selects(selections(Some(0))))?;
    let Answer::Confirmed(ScreenParam::Selects(confirmed)) = answer else {
        return Err(Error::Unhandled("select not confirmed"));
    };
    let flags = confirmed.map(|(_, selected)| selected);
    test_assert_eq!(flags, [false, false, true], "only the entry under the cursor is selected");
    test_assert_eq!(confirmed[2].0.as_str(), "Gamma");

    let value = select.get_value()?;
    test_assert_eq!(value.map(|(_, selected)| selected), [false, false, true]);
    Ok(())
}

fn test_select_cancel_and_drop() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut select = Select::<3>::new();
    test_assert!(matches!(select.get_value(), Err(Error::NullPtr)), "value before the first draw");

    let (answer, _) = frame(&mut select, &mut lcd, bits(ButtonReleased), ScreenParam::Selects(selections(Some(2))))?;
    test_assert!(is_cancelled(&answer));

    select.clear();
    test_assert!(matches!(select.get_value(), Err(Error::NullPtr)), "value kept after drop");
    Ok(())
}

// ------------------------------------------------------------------ input

fn input_param(value: &str, secret_mode: bool) -> ScreenParam {
    ScreenParam::Input { value: Bytes::from(value), secret_mode }
}

/// Feeds `flags` to `input` one frame each, ignoring the answers.
fn input_frames(input: &mut Input, lcd: &mut MockLcd, value: &str, flags: &[DisplayFlag]) -> Result<()> {
    for &flag in flags {
        frame(input, lcd, bits(flag), input_param(value, false))?;
    }
    Ok(())
}

/// Encoder button held longer than the long press threshold: confirms.
fn input_long_encoder_press(input: &mut Input, lcd: &mut MockLcd, value: &str) -> Result<Answer> {
    frame(input, lcd, bits(EncoderButtonPressed), input_param(value, false))?;
    System::delay(LONG_PRESS);
    Ok(frame(input, lcd, bits(EncoderButtonReleased), input_param(value, false))?.0)
}

fn confirmed_input(answer: Answer) -> Result<String> {
    match answer {
        Answer::Confirmed(ScreenParam::Input { value, .. }) => Ok(String::from(value.as_str())),
        Answer::Confirmed(_) => Err(Error::Unhandled("input confirmed with the wrong parameter")),
        Answer::Pending => Err(Error::Unhandled("input still pending")),
        Answer::Cancelled => Err(Error::Unhandled("input cancelled")),
    }
}

fn test_input_type_text() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    input_frames(&mut input, &mut lcd, "", &[
        DisplayFlag::None,
        EncoderRotatedClockwise,            // ' ' -> 'a'
        EncoderButtonPressed,               // short press: next char
        EncoderButtonReleased,
        EncoderRotatedClockwise,            // ' ' -> 'a'
        EncoderRotatedClockwise,            // 'a' -> 'b'
    ])?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "ab");

    let answer = input_long_encoder_press(&mut input, &mut lcd, "")?;
    test_assert_eq!(confirmed_input(answer)?, "ab");
    test_assert_eq!(input.get_value()?.as_str(), "ab");
    Ok(())
}

fn test_input_edit_existing() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();

    // The cursor starts on a blank appended after the current value
    frame(&mut input, &mut lcd, 0, input_param("ab", false))?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "ab ");

    // Unchanged confirm drops the trailing blank
    let answer = input_long_encoder_press(&mut input, &mut lcd, "ab")?;
    test_assert_eq!(confirmed_input(answer)?, "ab");
    Ok(())
}

fn test_input_backspace() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    input_frames(&mut input, &mut lcd, "abc", &[DisplayFlag::None, ButtonPressed])?;
    let (answer, _) = frame(&mut input, &mut lcd, bits(ButtonReleased), input_param("abc", false))?;
    test_assert!(is_pending(&answer), "short press on the button left the widget");
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "ab ");

    let answer = input_long_encoder_press(&mut input, &mut lcd, "abc")?;
    test_assert_eq!(confirmed_input(answer)?, "ab");
    Ok(())
}

fn test_input_long_press_cancels() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    input_frames(&mut input, &mut lcd, "abc", &[DisplayFlag::None, ButtonPressed])?;
    System::delay(LONG_PRESS);
    let (answer, _) = frame(&mut input, &mut lcd, bits(ButtonReleased), input_param("abc", false))?;
    test_assert!(is_cancelled(&answer));
    Ok(())
}

fn test_input_char_wrap() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    // ' ' is the first char of the table, '~' the last
    input_frames(&mut input, &mut lcd, "", &[DisplayFlag::None, EncoderRotatedCounterClockwise])?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "~");
    input_frames(&mut input, &mut lcd, "", &[EncoderRotatedClockwise])?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), " ");
    Ok(())
}

/// Counter-clockwise must step back one char wherever the cursor is,
/// the first position included.
fn test_input_rotate_back_first_char() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    input_frames(&mut input, &mut lcd, "", &[
        DisplayFlag::None,
        EncoderRotatedClockwise,            // ' ' -> 'a'
        EncoderRotatedClockwise,            // 'a' -> 'b'
        EncoderRotatedCounterClockwise,     // 'b' -> 'a'
    ])?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "a");
    Ok(())
}

/// A byte outside the char table (here the tail of an `è` that filled the
/// buffer, so the cursor sits on it) is replaced, not looked up.
fn test_input_char_outside_table() -> Result<()> {
    const VALUE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaè";
    test_assert_eq!(VALUE.len(), MAX_SIZE);
    for (flag, expected) in [(EncoderRotatedClockwise, b'a'), (EncoderRotatedCounterClockwise, b'~')] {
        let mut lcd = MockLcd::new();
        let mut input = Input::new();
        frame(&mut input, &mut lcd, 0, input_param(VALUE, false))?;
        frame(&mut input, &mut lcd, bits(flag), input_param(VALUE, false))?;
        let value = input.get_value()?;
        test_assert_eq!(value[MAX_SIZE - 1], expected, "after {flag:?}");
    }
    Ok(())
}

fn test_input_secret_mode() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    frame(&mut input, &mut lcd, 0, input_param("pw", true))?;
    // Only the char under the cursor (the appended blank) is in clear
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "** ");

    frame(&mut input, &mut lcd, bits(EncoderButtonPressed), input_param("pw", true))?;
    System::delay(LONG_PRESS);
    let (answer, _) = frame(&mut input, &mut lcd, bits(EncoderButtonReleased), input_param("pw", true))?;
    test_assert!(
        matches!(&answer, Answer::Confirmed(ScreenParam::Input { value, secret_mode: true }) if value.as_str() == "pw"),
        "secret input not confirmed as such"
    );
    Ok(())
}

fn test_input_overflow_marker() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    frame(&mut input, &mut lcd, 0, input_param("abcdefghijklmnopqrst", false))?;
    // 16 visible chars: the marker plus the last 15 bytes of "abc...t "
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "<ghijklmnopqrst ");
    Ok(())
}

fn test_input_clear() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut input = Input::new();
    test_assert!(matches!(input.get_value(), Err(Error::NullPtr)), "value before the first draw");
    frame(&mut input, &mut lcd, 0, input_param("ab", false))?;
    test_assert!(input.get_value().is_ok());
    input.clear();
    test_assert!(matches!(input.get_value(), Err(Error::NullPtr)), "value kept after clear");
    Ok(())
}

// -------------------------------------------------------- date and time

fn date(year: i32, month: u8, mday: u8) -> Result<ScreenParam> {
    Ok(ScreenParam::DateTime(DateTime::new_date(year, month, mday)?))
}

fn time(hour: u8, minute: u8, second: u8) -> Result<ScreenParam> {
    Ok(ScreenParam::DateTime(DateTime::new_time(hour, minute, second)?))
}

fn test_time_fields() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();

    // Nothing is drawn until a frame asks for it
    let (answer, _) = frame(&mut editor, &mut lcd, 0, time(12, 30, 0)?)?;
    test_assert!(is_pending(&answer));
    test_assert!(lcd.texts_at(SECOND_ROW_Y).is_empty(), "drawn without a Draw request");

    frame(&mut editor, &mut lcd, DRAW, time(12, 30, 0)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "12:30:00");

    frame(&mut editor, &mut lcd, bits(EncoderRotatedClockwise), time(12, 30, 0)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "13:30:00");
    Ok(())
}

fn test_time_from_rtc() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();

    // Without a value the fields come from the RTC, and the editor asks for an internal draw
    let (answer, signal) = frame(&mut editor, &mut lcd, 0, ScreenParam::None)?;
    test_assert!(is_pending(&answer));
    test_assert!(signal & DRAW_INTERNAL != 0, "loading the fields does not request a draw");
    test_assert!(signal & DRAW == 0, "loading the fields must not count as user activity");

    // NOW is midnight UTC: only the hour depends on the local timezone
    frame(&mut editor, &mut lcd, DRAW, ScreenParam::None)?;
    let shown = lcd.last_text_at(SECOND_ROW_Y);
    test_assert!(shown.len() == 8 && shown.ends_with(":00:00"), "{shown:?} is not the RTC time");
    Ok(())
}

fn test_time_internal_redraw() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();
    frame(&mut editor, &mut lcd, DRAW, time(12, 30, 0)?)?;
    lcd.reset();

    // The header raises DrawInternal on its own: the editor must not leave its area blank
    frame(&mut editor, &mut lcd, DRAW_INTERNAL, time(12, 30, 0)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "12:30:00", "fields wiped by an internal redraw");
    Ok(())
}

fn test_time_wrap() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();
    frame(&mut editor, &mut lcd, 0, time(23, 59, 59)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderRotatedClockwise), time(23, 59, 59)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "00:59:59", "hour past 23");

    let mut editor = Time::new();
    frame(&mut editor, &mut lcd, 0, time(0, 0, 0)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), time(0, 0, 0)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderRotatedCounterClockwise), time(0, 0, 0)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "00:59:00", "minute before 0");
    Ok(())
}

fn test_time_confirm() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();
    test_assert!(matches!(editor.get_value(), Err(Error::NullPtr)), "value before confirm");

    frame(&mut editor, &mut lcd, 0, time(12, 30, 15)?)?;
    // hour -> minute -> second -> confirm
    for _ in 0..2 {
        let (answer, _) = frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), time(12, 30, 15)?)?;
        test_assert!(is_pending(&answer), "confirmed before the last field");
    }
    let (answer, _) = frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), time(12, 30, 15)?)?;
    test_assert!(
        matches!(answer, Answer::Confirmed(ScreenParam::DateTime(dt)) if (dt.hour, dt.minute, dt.second) == (12, 30, 15)),
        "time not confirmed"
    );
    let value = editor.get_value()?;
    test_assert_eq!((value.hour, value.minute, value.second), (12, 30, 15));

    editor.clear();
    test_assert!(matches!(editor.get_value(), Err(Error::NullPtr)), "value kept after drop");
    Ok(())
}

/// A value changed with the encoder must survive the following frames:
/// the screens pass the current RTC time as parameter on every frame.
fn test_time_edit_kept() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();
    frame(&mut editor, &mut lcd, 0, time(12, 30, 0)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderRotatedClockwise), time(12, 30, 0)?)?;
    for _ in 0..3 {
        frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), time(12, 30, 0)?)?;
    }
    let value = editor.get_value()?;
    test_assert_eq!((value.hour, value.minute, value.second), (13, 30, 0), "edited hour lost");
    Ok(())
}

fn test_time_drop_reloads() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();
    frame(&mut editor, &mut lcd, 0, time(12, 30, 0)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderRotatedClockwise), time(12, 30, 0)?)?;

    editor.clear();
    frame(&mut editor, &mut lcd, 0, time(8, 15, 0)?)?;
    frame(&mut editor, &mut lcd, DRAW, time(8, 15, 0)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "08:15:00", "old edit kept after drop");
    Ok(())
}

fn test_time_cancel() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Time::new();
    frame(&mut editor, &mut lcd, 0, time(12, 30, 0)?)?;
    let (answer, _) = frame(&mut editor, &mut lcd, bits(ButtonReleased), time(12, 30, 0)?)?;
    test_assert!(is_cancelled(&answer), "back on the first field does not cancel");

    // From the second field, back returns to the first one
    let mut editor = Time::new();
    frame(&mut editor, &mut lcd, 0, time(12, 30, 0)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), time(12, 30, 0)?)?;
    let (answer, _) = frame(&mut editor, &mut lcd, bits(ButtonReleased), time(12, 30, 0)?)?;
    test_assert!(is_pending(&answer), "back on the second field cancelled");
    let (answer, _) = frame(&mut editor, &mut lcd, bits(ButtonReleased), time(12, 30, 0)?)?;
    test_assert!(is_cancelled(&answer));
    Ok(())
}

fn test_date_fields() -> Result<()> {
    let mut lcd = MockLcd::new();

    // Year does not wrap
    let mut editor = Date::new();
    frame(&mut editor, &mut lcd, 0, date(2026, 10, 14)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderRotatedCounterClockwise), date(2026, 10, 14)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "2025-10-14");

    // Month wraps
    let mut editor = Date::new();
    frame(&mut editor, &mut lcd, 0, date(2026, 12, 14)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), date(2026, 12, 14)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderRotatedClockwise), date(2026, 12, 14)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "2026-01-14");
    Ok(())
}

fn test_date_days_in_month() -> Result<()> {
    let mut lcd = MockLcd::new();
    for (year, expected) in [(2026, "2026-02-01"), (2028, "2028-02-29")] {
        let mut editor = Date::new();
        frame(&mut editor, &mut lcd, 0, date(year, 2, 28)?)?;
        frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), date(year, 2, 28)?)?;
        frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), date(year, 2, 28)?)?;
        frame(&mut editor, &mut lcd, bits(EncoderRotatedClockwise), date(year, 2, 28)?)?;
        test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), expected, "day after Feb 28 {year}");
    }
    Ok(())
}

/// Moving from January 31 to February caps the day, so the confirmed date
/// is valid.
fn test_date_day_capped() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Date::new();
    frame(&mut editor, &mut lcd, 0, date(2026, 1, 31)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), date(2026, 1, 31)?)?;
    frame(&mut editor, &mut lcd, bits(EncoderRotatedClockwise), date(2026, 1, 31)?)?;
    test_assert_eq!(lcd.last_text_at(SECOND_ROW_Y), "2026-02-28");

    for _ in 0..2 {
        frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), date(2026, 1, 31)?)?;
    }
    let value = editor.get_value()?;
    test_assert_eq!((value.year, value.month, value.mday), (2026, 2, 28));
    Ok(())
}

fn test_date_confirm() -> Result<()> {
    let mut lcd = MockLcd::new();
    let mut editor = Date::new();
    frame(&mut editor, &mut lcd, 0, date(2026, 10, 14)?)?;
    for _ in 0..3 {
        frame(&mut editor, &mut lcd, bits(EncoderButtonReleased), date(2026, 10, 14)?)?;
    }
    let value = editor.get_value()?;
    test_assert_eq!((value.year, value.month, value.mday), (2026, 10, 14));
    Ok(())
}

pub(in crate::apps) fn run_all_tests(stats: &mut TestStats) {
    run_tests!(TAG, stats;
        test_scroll_text_fits,
        test_scroll_text_exact_width,
        test_scroll_text_scrolls,
        test_scroll_text_multibyte,
        test_clean_context,
        test_text_one_row,
        test_text_draw_requested_once,
        test_text_two_rows,
        test_text_too_many_rows,
        test_text_answers,
        test_check_initial_value,
        test_check_toggle_and_confirm,
        test_check_cancel_and_drop,
        test_number_rotation_wraps,
        test_number_without_value,
        test_number_confirm_and_cancel,
        test_number_range_inclusive,
        test_number_type_bounds,
        test_select_starts_on_selected,
        test_select_rotation_wraps,
        test_select_confirm,
        test_select_cancel_and_drop,
        test_input_type_text,
        test_input_edit_existing,
        test_input_backspace,
        test_input_long_press_cancels,
        test_input_char_wrap,
        test_input_rotate_back_first_char,
        test_input_char_outside_table,
        test_input_secret_mode,
        test_input_overflow_marker,
        test_input_clear,
        test_time_fields,
        test_time_from_rtc,
        test_time_internal_redraw,
        test_time_wrap,
        test_time_confirm,
        test_time_edit_kept,
        test_time_drop_reloads,
        test_time_cancel,
        test_date_fields,
        test_date_days_in_month,
        test_date_day_capped,
        test_date_confirm,
    );
}
