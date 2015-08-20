extern crate libvterm_sys;
extern crate libc;

use std::ops::{Deref};
use libvterm_sys::vterm as vterm_sys;
use libvterm_sys::vterm::VTermRect;
use libc::{ c_void, c_char, c_int, size_t };

/* TODO : Test
struct VTermParserCB<'self> {
    text: &'self FnMut(&str) -> bool,
    control: &'self FnMut(char) -> bool,
    escape: &'self FnMut(&str) -> bool,
    csi: &'self FnMut(&str, &[u64], &str, char) -> bool,
    osc: &'self FnMut(&str) -> bool,
    dcs: &'self FnMut(&str) -> bool,
    resize: &'self FnMut(i32, i32) -> bool,
} */

struct VTermScreenCB<'myself> {
    damage: Option<&'myself mut FnMut(/*rect: */vterm_sys::VTermRect) -> bool>,
    moverect: Option<&'myself mut FnMut(/*dest: */vterm_sys::VTermRect,
                                    /*src: */vterm_sys::VTermRect) -> bool>,
    movecursor: Option<&'myself mut FnMut(/*pos: */vterm_sys::VTermPos,
                                      /*oldpos: */vterm_sys::VTermPos,
                                      /*visible: */bool) -> bool>,
    settermprop: Option<&'myself mut FnMut(/*prop: */vterm_sys::VTermProp,
                                       /*val: */&mut vterm_sys::VTermValue) -> bool>,
    bell: Option<&'myself mut FnMut() -> bool>,
    resize: Option<&'myself mut FnMut(/*rows: */u32, /*cols: */u32) -> bool>,
    sb_pushline: Option<&'myself mut FnMut(/*cols: */u32,
                                       /*cells: */&vterm_sys::VTermScreenCell) -> bool>,
    sb_popline: Option<&'myself mut FnMut(/*cols: */u32,
                                      /*cells: */&mut vterm_sys::VTermScreenCell) -> bool>,
}

extern fn damage_cb(rect: vterm_sys::VTermRect, data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).damage.as_mut().map(|fun| fun(rect)) }.unwrap_or(false) as c_int
}

extern fn moverect_cb(dest: vterm_sys::VTermRect,
                      src: vterm_sys::VTermRect,
                      data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).moverect.as_mut().map(|fun| fun(dest, src)) }.unwrap_or(false) as c_int
}

extern fn movecursor_cb(pos: vterm_sys::VTermPos,
                        oldpos: vterm_sys::VTermPos,
                        visible: c_int,
                        data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).movecursor.as_mut().map(|fun| fun(pos, oldpos, visible == 0)) }.unwrap_or(false) as c_int
}

extern fn settermprop_cb(prop: vterm_sys::VTermProp,
                         val: *mut vterm_sys::VTermValue,
                         data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).settermprop.as_mut().map(|fun| fun(prop, &mut *val)) }.unwrap_or(false) as c_int
}

extern fn bell_cb(data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).bell.as_mut().map(|fun| fun()) }.unwrap_or(false) as c_int
}

extern fn resize_cb(rows: c_int, cols: c_int, data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).resize.as_mut().map(|fun| fun(rows as u32, cols as u32)) }.unwrap_or(false) as c_int
}

extern fn sb_pushline_cb(cols: c_int,
                         cells: *const vterm_sys::VTermScreenCell,
                         data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).sb_pushline.as_mut().map(|fun| fun(cols as u32, &*cells)) }.unwrap_or(false) as c_int
}

extern fn sb_popline_cb(cols: c_int,
                        cells: *mut vterm_sys::VTermScreenCell,
                        data: *mut c_void) -> c_int {
    let cbs = data as *mut VTermScreenCB;
    unsafe { (*cbs).sb_popline.as_mut().map(|fun| fun(cols as u32, &mut *cells)) }.unwrap_or(false) as c_int
}

const SCREEN_CBS : vterm_sys::VTermScreenCallbacks = vterm_sys::VTermScreenCallbacks {
    damage: damage_cb,
    moverect: moverect_cb,
    movecursor: movecursor_cb,
    settermprop: settermprop_cb,
    bell: bell_cb,
    resize: resize_cb,
    sb_pushline: sb_pushline_cb,
    sb_popline: sb_popline_cb,
};

pub struct VTerm<'myself> {
    vt: &'myself mut vterm_sys::VTerm,
    screen_cbs: VTermScreenCB<'myself>
}

pub struct VTermScreen<'a>(&'a mut vterm_sys::VTermScreen, &'a mut VTerm<'a>);

impl<'a> Deref for VTermScreen<'a> {
    type Target = &'a mut vterm_sys::VTermScreen;
    fn deref(&self) -> & &'a mut vterm_sys::VTermScreen {
        &self.0
    }
}

impl<'myself> VTerm<'myself> {
    pub fn new(rows: u32, cols: u32) -> VTerm<'myself> {
        unsafe { VTerm {
            vt: &mut *vterm_sys::vterm_new(rows as c_int, cols as c_int),
            screen_cbs: VTermScreenCB {
                damage: None,
                moverect: None,
                movecursor: None,
                settermprop: None,
                bell: None,
                resize: None,
                sb_pushline: None,
                sb_popline: None,
            }
        } }
    }

    pub fn get_size(&self) -> (u32, u32) {
        let mut x : c_int = 0;
        let mut y : c_int = 0;
        unsafe { vterm_sys::vterm_get_size(self.vt, &mut x, &mut y); }
        (x as u32, y as u32)
    }

    pub fn set_size(&mut self, rows: u32, cols: u32) {
        unsafe { vterm_sys::vterm_set_size(self.vt, rows as c_int, cols as c_int) }
    }

    // TODO : UTF-8
    pub fn get_utf8(&self) -> bool {
        unsafe { vterm_sys::vterm_get_utf8(self.vt) == 0 }
    }

    pub fn set_utf8(&mut self, is_utf8: bool) {
        unsafe { vterm_sys::vterm_set_utf8(self.vt, is_utf8 as c_int) }
    }

    // input_write = Write impl

    pub fn output_buffer_size(&self) -> usize {
        unsafe { vterm_sys::vterm_output_get_buffer_size(self.vt) as usize }
    }

    pub fn output_buffer_current(&self) -> usize {
        unsafe { vterm_sys::vterm_output_get_buffer_current(self.vt) as usize }
    }

    pub fn output_buffer_remaining(&self) -> usize {
        unsafe { vterm_sys::vterm_output_get_buffer_remaining(self.vt) as usize }
    }

    // output_read = Read impl

    pub fn keyboard_unichar(&mut self, c: u32, state: vterm_sys::VTermModifier) {
        unsafe { vterm_sys::vterm_keyboard_unichar(self.vt, c, state) }
    }

    pub fn keyboard_key(&mut self, key: vterm_sys::VTermKey, state: vterm_sys::VTermModifier) {
        unsafe { vterm_sys::vterm_keyboard_key(self.vt, key, state) }
    }

    pub fn mouse_move(&mut self, row: u32, col: u32, modifier: vterm_sys::VTermModifier) {
        unsafe { vterm_sys::vterm_mouse_move(self.vt, row as c_int, col as c_int, modifier) }
    }

    pub fn mouse_button(&mut self, button: u32, pressed: u32, modifier: vterm_sys::VTermModifier) {
        unsafe { vterm_sys::vterm_mouse_button(self.vt, button as c_int, pressed as c_int, modifier) }
    }

    /*pub fn state(&mut self) -> VTermState {
        unsafe { vterm_sys::vterm_obtain_state(self.vt) }
    }*/

    pub fn screen(&'myself mut self) -> VTermScreen<'myself> {
        VTermScreen(unsafe { &mut *vterm_sys::vterm_obtain_screen(self.vt) },
                    self)
    }
}

impl<'a> VTermScreen<'a> {
    fn inner(&mut self) -> &mut vterm_sys::VTermScreen {
        self.0
    }

    pub fn enable_altscreen(&'a mut self, altscreen: bool) {
        unsafe { vterm_sys::vterm_screen_enable_altscreen(self.inner(), altscreen as c_int) }
    }
    pub fn flush_damage(&'a mut self) {
        unsafe { vterm_sys::vterm_screen_flush_damage(self.inner()) }
    }
    pub fn set_damage_merge(&'a mut self, size: vterm_sys::VTermDamageSize) {
        unsafe { vterm_sys::vterm_screen_set_damage_merge(self.inner(), size) }
    }
    pub fn reset(&'a mut self, hard: bool) {
        unsafe { vterm_sys::vterm_screen_reset(self.inner(), hard as c_int) }
    }
    pub fn get_chars(&self, chars: &mut [u32], rect: VTermRect) -> usize {
        unsafe { vterm_sys::vterm_screen_get_chars(**self, chars.as_mut_ptr(), chars.len() as size_t, rect) as usize }
    }
    pub fn get_text(&self, string: &mut String, rect: VTermRect) -> usize {
        let vec = unsafe { string.as_mut_vec() };
        unsafe { vterm_sys::vterm_screen_get_text(**self, vec.as_mut_ptr() as *mut c_char, vec.len() as size_t, rect) as usize }
    }

    pub fn on_damage<F>(&mut self, fun: &'a mut F)
        where F: FnMut(vterm_sys::VTermRect) -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.damage = Some(fun);
    }
    pub fn on_moverect<F>(&mut self, fun: &'a mut F)
        where F: FnMut(vterm_sys::VTermRect, vterm_sys::VTermRect) -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.moverect = Some(fun);

    }
    pub fn on_movecursor<F>(&mut self, fun: &'a mut F)
        where F: FnMut(vterm_sys::VTermPos, vterm_sys::VTermPos, bool) -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.movecursor = Some(fun);

    }
    pub fn on_settermprop<F>(&mut self, fun: &'a mut F)
        where F: FnMut(vterm_sys::VTermProp, &mut vterm_sys::VTermValue) -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.settermprop = Some(fun);
    }
    pub fn on_bell<F>(&mut self, fun: &'a mut F)
        where F: FnMut() -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.bell = Some(fun);

    }
    pub fn on_resize<F>(&mut self, fun: &'a mut F)
        where F: FnMut(u32, u32) -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.resize = Some(fun);

    }
    pub fn on_sb_pushline<F>(&mut self, fun: &'a mut F)
        where F: FnMut(u32, &vterm_sys::VTermScreenCell) -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.sb_pushline = Some(fun);

    }
    pub fn on_sb_popline<F>(&mut self, fun: &'a mut F)
        where F: FnMut(u32, &mut vterm_sys::VTermScreenCell) -> bool {
        unsafe { vterm_sys::vterm_screen_set_callbacks(self.inner(), &SCREEN_CBS, &mut self.1.screen_cbs as *mut _ as *mut c_void) };
        self.1.screen_cbs.sb_popline = Some(fun);
    }
}

impl<'myself> Drop for VTerm<'myself> {
    fn drop(&mut self) {
        unsafe { vterm_sys::vterm_free(self.vt) }
    }
}

#[test]
fn it_works() {
}
