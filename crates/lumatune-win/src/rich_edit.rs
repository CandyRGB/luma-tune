//! A thread-bound host for Windows Rich Edit. Positions are UTF-16 document offsets.
use std::cell::{Cell, RefCell};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::num::NonZeroIsize;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::rc::Rc;

use windows::Win32::Foundation::{
    COLORREF, E_FAIL, E_INVALIDARG, FreeLibrary, HMODULE, HWND, LPARAM, LRESULT, POINT, RECT,
    RPC_E_CHANGED_MODE, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    AC_SRC_OVER, AlphaBlend, BLENDFUNCTION, COLOR_HIGHLIGHT, ClientToScreen,
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, CreateSolidBrush, DT_NOPREFIX,
    DT_WORDBREAK, DeleteDC, DeleteObject, DrawTextW, FillRect, GetDC, GetSysColor, HDC, HGDIOBJ,
    InvalidateRect, LOGFONTW, ReleaseDC, RestoreDC, SaveDC, SelectObject, SetBkMode, SetTextColor,
    TRANSPARENT,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize,
};
use windows::Win32::System::LibraryLoader::{LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{
    CLSID_AccPropServices, IAccPropServices, PROPID_ACC_HELP, PROPID_ACC_NAME,
};
use windows::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, FR_DOWN, GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER,
    OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::Controls::RichEdit::*;
use windows::Win32::UI::Controls::{
    EM_CANUNDO, EM_CHARFROMPOS, EM_EMPTYUNDOBUFFER, EM_GETFIRSTVISIBLELINE, EM_GETRECT,
    EM_LINEINDEX, EM_POSFROMCHAR, EM_REPLACESEL, EM_SCROLLCARET, EM_SETREADONLY, EM_SETRECT,
    EM_UNDO,
};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetFocus, GetKeyState, SetFocus, VK_SHIFT, VK_TAB,
};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CHILDID_SELF, CreatePopupMenu, CreateWindowExW, DestroyMenu, DestroyWindow,
    EN_CHANGE, ES_AUTOVSCROLL, ES_MULTILINE, ES_NOHIDESEL, ES_WANTRETURN, GetClientRect,
    GetWindowThreadProcessId, IsWindow, MF_CHECKED, MF_GRAYED, MF_STRING, OBJID_CLIENT, SW_HIDE,
    SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetWindowPos, ShowWindow,
    TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_CHAR, WM_CLEAR, WM_COMMAND, WM_CONTEXTMENU, WM_COPY, WM_CUT, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_PASTE, WM_SETTEXT, WS_CHILD, WS_CLIPSIBLINGS, WS_TABSTOP, WS_VSCROLL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    HideCaret, ShowCaret, WM_GETTEXTLENGTH, WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION,
    WM_KILLFOCUS, WM_PAINT, WM_PRINTCLIENT, WM_SETFOCUS,
};
use windows::core::{Error, GUID, IUnknown, Interface, PCWSTR, PWSTR, Result, w};

const UTF16: u32 = 1200;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Selection {
    pub start: i32,
    pub end: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Effect {
    Bold,
    Italic,
    Underline,
}

impl Effect {
    fn mask(self) -> CFM_MASK {
        match self {
            Self::Bold => CFM_BOLD,
            Self::Italic => CFM_ITALIC,
            Self::Underline => CFM_UNDERLINE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Edit {
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    Undo,
    Redo,
    Toggle(Effect),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Placement {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub padding_x: i32,
    pub padding_y: i32,
}

#[derive(Clone, Copy, Debug)]
pub enum EditorRequest {
    ContextMenu { x: i32, y: i32 },
    Traverse { backward: bool },
}

struct EditorEvents {
    window: HWND,
    parent: HWND,
    changed: Cell<bool>,
    request: Cell<Option<EditorRequest>>,
    placeholder: RefCell<Placeholder>,
    composing: Cell<bool>,
    search_matches: RefCell<Vec<Selection>>,
}

struct Placeholder {
    text: Vec<u16>,
    ink: COLORREF,
    family: [u16; 32],
    logical_size: f32,
    scale: f32,
}

impl Placeholder {
    fn new(scale: f32) -> Self {
        Self {
            text: Vec::new(),
            ink: color_ref([96, 96, 96]),
            family: [0; 32],
            logical_size: 14.0,
            scale,
        }
    }

    fn draw(&mut self, dc: HDC, mut bounds: RECT) {
        if dc.is_invalid() || bounds.left >= bounds.right || bounds.top >= bounds.bottom {
            return;
        }
        let font = LOGFONTW {
            lfHeight: -(self.logical_size * self.scale).round().max(1.0) as i32,
            lfWeight: 400,
            lfFaceName: self.family,
            ..Default::default()
        };
        // Preserve the DC state and release the font after deselecting it.
        unsafe {
            let saved = SaveDC(dc);
            if saved == 0 {
                return;
            }
            let font = CreateFontIndirectW(&font);
            if !font.is_invalid() {
                SelectObject(dc, HGDIOBJ(font.0));
            }
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, self.ink);
            DrawTextW(dc, &mut self.text, &mut bounds, DT_NOPREFIX | DT_WORDBREAK);
            let _ = RestoreDC(dc, saved);
            if !font.is_invalid() {
                let _ = DeleteObject(HGDIOBJ(font.0));
            }
        }
    }
}

impl EditorEvents {
    fn attach(window: HWND, parent: HWND) -> Result<Box<Self>> {
        let events = Box::new(Self {
            window,
            parent,
            changed: Cell::new(true),
            request: Cell::new(None),
            placeholder: RefCell::new(Placeholder::new(
                unsafe { GetDpiForWindow(window) } as f32 / 96.0,
            )),
            composing: Cell::new(false),
            search_matches: RefCell::default(),
        });
        let key = (&*events as *const Self) as usize;
        // The boxed state stays put and both subclass hooks are removed before it is freed.
        unsafe {
            SetWindowSubclass(window, Some(editor_event), key, key).ok()?;
            if let Err(error) = SetWindowSubclass(parent, Some(parent_event), key, key).ok() {
                let _ = RemoveWindowSubclass(window, Some(editor_event), key);
                return Err(error);
            }
        }
        Ok(events)
    }

    fn invalidate_content(&self) {
        let _ = unsafe { InvalidateRect(Some(self.window), None, false) };
    }

    fn clear_matches(&self) {
        if let Ok(mut matches) = self.search_matches.try_borrow_mut()
            && !matches.is_empty()
        {
            matches.clear();
            self.invalidate_content();
        }
    }

    fn draws_placeholder(&self) -> bool {
        if self.composing.get() || self.placeholder.borrow().text.is_empty() {
            return false;
        }
        (unsafe { SendMessageW(self.window, WM_GETTEXTLENGTH, None, None) }).0 == 0
    }

    fn paint_decorations(&self, dc: HDC, placeholder: bool, highlight: bool) {
        if dc.is_invalid() {
            return;
        }
        let mut bounds = RECT::default();
        // Fetch the current native formatting rect, including changes to margins and scrolling.
        unsafe {
            SendMessageW(
                self.window,
                EM_GETRECT,
                None,
                Some(LPARAM(&mut bounds as *mut RECT as isize)),
            );
        }
        if placeholder && let Ok(mut cue) = self.placeholder.try_borrow_mut() {
            cue.draw(dc, bounds);
        }
        if highlight && let Ok(matches) = self.search_matches.try_borrow() {
            paint_matches(self.window, dc, bounds, &matches);
        }
    }
}

impl Drop for EditorEvents {
    fn drop(&mut self) {
        let key = self as *mut Self as usize;
        unsafe {
            let _ = RemoveWindowSubclass(self.parent, Some(parent_event), key);
            let _ = RemoveWindowSubclass(self.window, Some(editor_event), key);
        }
    }
}

unsafe extern "system" fn parent_event(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    data: usize,
) -> LRESULT {
    let events = unsafe { &*(data as *const EditorEvents) };
    if msg == WM_COMMAND
        && lparam.0 == events.window.0 as isize
        && (wparam.0 >> 16) as u16 == EN_CHANGE as u16
    {
        events.changed.set(true);
        events.clear_matches();
    }
    if msg == WM_LBUTTONDOWN && unsafe { GetFocus() } == events.window {
        let _ = unsafe { SetFocus(Some(hwnd)) };
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn editor_event(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    data: usize,
) -> LRESULT {
    let events = unsafe { &*(data as *const EditorEvents) };
    if msg == WM_SETFOCUS {
        events.clear_matches();
    }
    if msg == WM_IME_STARTCOMPOSITION || msg == WM_IME_ENDCOMPOSITION || msg == WM_KILLFOCUS {
        let composing = msg == WM_IME_STARTCOMPOSITION;
        if events.composing.replace(composing) != composing {
            events.invalidate_content();
        }
    }
    if msg == WM_PAINT || msg == WM_PRINTCLIENT {
        let placeholder = events.draws_placeholder();
        let highlight = !events.search_matches.borrow().is_empty();
        if !placeholder && !highlight {
            return unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        }
        if msg == WM_PRINTCLIENT {
            let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
            events.paint_decorations(HDC(wparam.0 as *mut _), placeholder, highlight);
            return result;
        }
        // Draw over fresh native pixels to avoid tint accumulating after partial repaints.
        events.invalidate_content();
        // The native caret is drawn above the hint. Balance HideCaret only when it succeeds.
        let hidden = unsafe { HideCaret(Some(hwnd)) }.is_ok();
        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
        let dc = unsafe { GetDC(Some(hwnd)) };
        if !dc.is_invalid() {
            events.paint_decorations(dc, placeholder, highlight);
            unsafe { ReleaseDC(Some(hwnd), dc) };
        }
        if hidden {
            let _ = unsafe { ShowCaret(Some(hwnd)) };
        }
        return result;
    }
    if msg == WM_CONTEXTMENU {
        let x = lparam.0 as i16 as i32;
        let y = (lparam.0 >> 16) as i16 as i32;
        events
            .request
            .set(Some(EditorRequest::ContextMenu { x, y }));
        return LRESULT(0);
    }
    if wparam.0 == VK_TAB.0 as usize {
        if msg == WM_KEYDOWN {
            let backward = unsafe { GetKeyState(i32::from(VK_SHIFT.0)) } < 0;
            events
                .request
                .set(Some(EditorRequest::Traverse { backward }));
            return LRESULT(0);
        }
        if msg == WM_CHAR {
            return LRESULT(0);
        }
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

struct RichEditLibrary(HMODULE);

impl Drop for RichEditLibrary {
    fn drop(&mut self) {
        // Each successful load belongs to this instance.
        let _ = unsafe { FreeLibrary(self.0) };
    }
}

pub struct RichEdit {
    window: HWND,
    read_only: bool,
    events: Box<EditorEvents>,
    _library: RichEditLibrary,
    _thread: PhantomData<Rc<()>>,
}

impl RichEdit {
    pub fn new(parent: NonZeroIsize) -> Result<Self> {
        let parent = HWND(parent.get() as *mut _);
        // Every message and HWND lifetime operation must stay on the parent's UI thread.
        if unsafe { GetWindowThreadProcessId(parent, None) } != unsafe { GetCurrentThreadId() } {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        let library = RichEditLibrary(unsafe {
            LoadLibraryExW(w!("msftedit.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32)?
        });
        let styles = WS_CHILD
            | WS_CLIPSIBLINGS
            | WS_TABSTOP
            | WS_VSCROLL
            | WINDOW_STYLE((ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN | ES_NOHIDESEL) as u32);
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                MSFTEDIT_CLASS,
                PCWSTR::null(),
                styles,
                0,
                0,
                0,
                0,
                Some(parent),
                None,
                None,
                None,
            )?
        };
        let events = match EditorEvents::attach(window, parent) {
            Ok(events) => events,
            Err(error) => {
                let _ = unsafe { DestroyWindow(window) };
                return Err(error);
            }
        };
        let editor = Self {
            window,
            read_only: false,
            events,
            _library: library,
            _thread: PhantomData,
        };
        editor.message(
            EM_SETEDITSTYLE,
            (SES_EXTENDBACKCOLOR | SES_USECTF) as usize,
            (SES_EXTENDBACKCOLOR | SES_USECTF) as isize,
        );
        editor.message(EM_EXLIMITTEXT, 0, i32::MAX as isize);
        editor.message(EM_SETEVENTMASK, 0, ENM_CHANGE as isize);
        Ok(editor)
    }

    fn message(&self, code: u32, wparam: usize, lparam: isize) -> isize {
        // This call is synchronous, including when the parameters borrow stack data.
        unsafe {
            SendMessageW(
                self.window,
                code,
                Some(WPARAM(wparam)),
                Some(LPARAM(lparam)),
            )
            .0
        }
    }

    pub fn place(&self, placement: Placement, visible: bool) -> Result<()> {
        if !visible || placement.width <= 0 || placement.height <= 0 {
            let _ = unsafe { ShowWindow(self.window, SW_HIDE) };
            return Ok(());
        }
        let mut content = RECT::default();
        unsafe {
            SetWindowPos(
                self.window,
                None,
                placement.left,
                placement.top,
                placement.width,
                placement.height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            GetClientRect(self.window, &mut content)?;
        }
        content.left += placement.padding_x.min((content.right - 1).max(0) / 2);
        content.right -= content.left;
        content.top += placement.padding_y.min((content.bottom - 1).max(0) / 2);
        content.bottom -= content.top;
        self.message(EM_SETRECT, 0, &content as *const RECT as isize);
        let _ = unsafe { ShowWindow(self.window, SW_SHOWNOACTIVATE) };
        Ok(())
    }

    pub fn focus(&self) {
        // GetFocus returns NULL when another thread owns the active window.
        if !self.has_focus() {
            let _ = unsafe { SetFocus(Some(self.window)) };
        }
    }

    pub fn has_focus(&self) -> bool {
        unsafe { GetFocus() == self.window }
    }

    pub fn yield_focus(&self) {
        let _ = unsafe { SetFocus(Some(self.events.parent)) };
    }

    pub fn take_request(&self) -> Option<EditorRequest> {
        self.events.request.take()
    }

    pub fn take_text_changed(&self) -> bool {
        self.events.changed.replace(false)
    }

    pub fn set_enabled(&self, enabled: bool) {
        let _ = unsafe { EnableWindow(self.window, enabled) };
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.message(EM_SETREADONLY, usize::from(read_only), 0);
        self.read_only = read_only;
    }

    pub fn set_background(&self, rgb: [u8; 3]) {
        self.message(EM_SETBKGNDCOLOR, 0, color_ref(rgb).0 as isize);
    }

    pub fn set_scale(&self, scale: f32) -> Result<()> {
        let dpi = unsafe { GetDpiForWindow(self.window) };
        if !scale.is_finite() || scale <= 0.0 || dpi == 0 {
            return Err(Error::from_hresult(E_INVALIDARG));
        }
        // Rich Edit already scales by monitor DPI, while a host may override its logical scale.
        let zoom = (scale * 96.0 * 1024.0 / dpi as f32)
            .round()
            .clamp(17.0, 65535.0) as usize;
        self.message(EM_SETZOOM, zoom, 1024);
        self.events.placeholder.borrow_mut().scale = scale;
        self.events.invalidate_content();
        Ok(())
    }

    pub fn set_accessible_name(&self, name: &str) -> Result<()> {
        self.set_accessible_string(PROPID_ACC_NAME, &terminated(name)?)
    }

    pub fn set_placeholder(&self, text: &str) -> Result<()> {
        let mut wide = terminated(text)?;
        let result = self.set_accessible_string(PROPID_ACC_HELP, &wide);
        wide.pop();
        self.events.placeholder.borrow_mut().text = wide;
        self.events.invalidate_content();
        result
    }

    pub fn set_placeholder_color(&self, rgb: [u8; 3]) {
        self.events.placeholder.borrow_mut().ink = color_ref(rgb);
        self.events.invalidate_content();
    }

    fn set_accessible_string(&self, property: GUID, value: &[u16]) -> Result<()> {
        let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if init != RPC_E_CHANGED_MODE {
            init.ok()?;
        }
        let result = (|| unsafe {
            let service: IAccPropServices =
                CoCreateInstance(&CLSID_AccPropServices, None, CLSCTX_INPROC_SERVER)?;
            service.SetHwndPropStr(
                self.window,
                OBJID_CLIENT.0 as u32,
                CHILDID_SELF,
                property,
                PCWSTR(value.as_ptr()),
            )
        })();
        if init.is_ok() {
            unsafe { CoUninitialize() };
        }
        result
    }

    pub fn set_default_font(&self, family: &str, logical_size: f32) {
        let mut format = char_format();
        format.Base.dwMask = CFM_FACE | CFM_SIZE;
        format.Base.yHeight = (logical_size * 15.0).round().max(15.0) as i32;
        let limit = format.Base.szFaceName.len() - 1;
        let mut offset = 0;
        for ch in family.chars() {
            if offset + ch.len_utf16() > limit || ch == '\0' {
                break;
            }
            ch.encode_utf16(&mut format.Base.szFaceName[offset..]);
            offset += ch.len_utf16();
        }
        self.apply_format(SCF_DEFAULT, &format);
        let mut placeholder = self.events.placeholder.borrow_mut();
        placeholder.family = format.Base.szFaceName;
        placeholder.logical_size = format.Base.yHeight as f32 / 15.0;
        drop(placeholder);
        self.events.invalidate_content();
    }

    pub fn text(&self) -> String {
        let length = self.text_length();
        let mut text = vec![0u16; length + 1];
        let request = GETTEXTEX {
            cb: (text.len() * 2) as u32,
            codepage: UTF16,
            ..Default::default()
        };
        let count = self.message(
            EM_GETTEXTEX,
            &request as *const GETTEXTEX as usize,
            text.as_mut_ptr() as isize,
        );
        decode_text(&text, count)
    }

    fn text_length(&self) -> usize {
        let request = GETTEXTLENGTHEX {
            flags: GTL_PRECISE | GTL_NUMCHARS,
            codepage: UTF16,
        };
        self.message(
            EM_GETTEXTLENGTHEX,
            &request as *const GETTEXTLENGTHEX as usize,
            0,
        )
        .max(0) as usize
    }

    pub fn has_text(&self) -> bool {
        self.text_length() != 0
    }

    pub fn set_text(&self, text: &str) -> Result<()> {
        let wide = terminated(text)?;
        if self.message(WM_SETTEXT, 0, wide.as_ptr() as isize) == 0 {
            return Err(Error::from_hresult(E_FAIL));
        }
        self.message(EM_EMPTYUNDOBUFFER, 0, 0);
        self.events.changed.set(true);
        self.events.clear_matches();
        Ok(())
    }

    pub fn replace_selection(&self, text: &str) -> Result<()> {
        if self.read_only {
            return Ok(());
        }
        let wide = terminated(text)?;
        self.message(EM_STOPGROUPTYPING, 0, 0);
        self.message(EM_REPLACESEL, 1, wide.as_ptr() as isize);
        Ok(())
    }

    pub fn selection(&self) -> Selection {
        let mut range = CHARRANGE::default();
        self.message(EM_EXGETSEL, 0, &mut range as *mut CHARRANGE as isize);
        Selection {
            start: range.cpMin,
            end: range.cpMax,
        }
    }

    pub fn set_selection(&self, range: Selection) {
        let range = CHARRANGE {
            cpMin: range.start,
            cpMax: range.end,
        };
        self.message(EM_EXSETSEL, 0, &range as *const CHARRANGE as isize);
    }

    pub fn selected_text(&self) -> String {
        let range = self.selection();
        let mut text = vec![0u16; (range.end - range.start).max(0) as usize + 1];
        let count = self.message(EM_GETSELTEXT, 0, text.as_mut_ptr() as isize);
        decode_text(&text, count)
    }

    pub fn can_undo(&self) -> bool {
        !self.read_only && self.message(EM_CANUNDO, 0, 0) != 0
    }

    pub fn can_redo(&self) -> bool {
        !self.read_only && self.message(EM_CANREDO, 0, 0) != 0
    }

    pub fn can_paste(&self) -> bool {
        !self.read_only && self.message(EM_CANPASTE, 0, 0) != 0
    }

    pub fn show_context_menu(&self, x: i32, y: i32) -> Result<()> {
        let mut point = POINT { x, y };
        if x == -1 && y == -1 {
            point = POINT::default();
            self.message(
                EM_POSFROMCHAR,
                &mut point as *mut POINT as usize,
                self.selection().end as isize,
            );
            point.y += 20;
            unsafe { ClientToScreen(self.window, &mut point).ok()? };
        }
        let selection = self.selection();
        let has_selection = selection.start != selection.end;
        let actions = [
            (
                Edit::Cut,
                w!("Cut\tCtrl+X"),
                has_selection && !self.read_only,
                false,
            ),
            (Edit::Copy, w!("Copy\tCtrl+C"), has_selection, false),
            (Edit::Paste, w!("Paste\tCtrl+V"), self.can_paste(), false),
            (Edit::Undo, w!("Undo\tCtrl+Z"), self.can_undo(), false),
            (Edit::Redo, w!("Redo\tCtrl+Y"), self.can_redo(), false),
            (
                Edit::SelectAll,
                w!("Select All\tCtrl+A"),
                self.text_length() > 0,
                false,
            ),
            (
                Edit::Toggle(Effect::Bold),
                w!("Bold\tCtrl+B"),
                !self.read_only,
                self.effect_active(Effect::Bold),
            ),
            (
                Edit::Toggle(Effect::Italic),
                w!("Italic\tCtrl+I"),
                !self.read_only,
                self.effect_active(Effect::Italic),
            ),
            (
                Edit::Toggle(Effect::Underline),
                w!("Underline\tCtrl+U"),
                !self.read_only,
                self.effect_active(Effect::Underline),
            ),
        ];
        let menu = unsafe { CreatePopupMenu()? };
        let outcome = (|| unsafe {
            for (index, (_, label, enabled, checked)) in actions.iter().enumerate() {
                let mut flags = MF_STRING;
                if !enabled {
                    flags |= MF_GRAYED;
                }
                if *checked {
                    flags |= MF_CHECKED;
                }
                AppendMenuW(menu, flags, index + 1, *label)?;
            }
            let index = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
                point.x,
                point.y,
                None,
                self.window,
                None,
            )
            .0 as usize;
            if let Some((command, _, true, _)) = index.checked_sub(1).and_then(|i| actions.get(i)) {
                self.execute(*command);
            }
            Ok(())
        })();
        let _ = unsafe { DestroyMenu(menu) };
        outcome
    }

    pub fn pick_rtf_file(&self, save: bool) -> io::Result<Option<PathBuf>> {
        let mut path = vec![0u16; 32768];
        if save {
            for (slot, unit) in path.iter_mut().zip("New Document.rtf".encode_utf16()) {
                *slot = unit;
            }
        }
        let mut dialog = OPENFILENAMEW {
            lStructSize: size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: self.events.parent,
            lpstrFilter: w!("Rich Text Format (*.rtf)\0*.rtf\0"),
            lpstrFile: PWSTR(path.as_mut_ptr()),
            nMaxFile: path.len() as u32,
            lpstrDefExt: w!("rtf"),
            Flags: OFN_EXPLORER
                | OFN_NOCHANGEDIR
                | OFN_PATHMUSTEXIST
                | if save {
                    OFN_OVERWRITEPROMPT
                } else {
                    OFN_FILEMUSTEXIST
                },
            ..Default::default()
        };
        let accepted = unsafe {
            if save {
                GetSaveFileNameW(&mut dialog)
            } else {
                GetOpenFileNameW(&mut dialog)
            }
        };
        if accepted.as_bool() {
            let length = path
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(path.len());
            return Ok(Some(PathBuf::from(OsString::from_wide(&path[..length]))));
        }
        let error = unsafe { CommDlgExtendedError() }.0;
        if error == 0 {
            Ok(None)
        } else {
            Err(io::Error::other(format!(
                "Windows file dialog failed ({error:#x})"
            )))
        }
    }

    pub fn execute(&self, edit: Edit) {
        let message = match edit {
            Edit::Copy => WM_COPY,
            Edit::SelectAll => {
                self.set_selection(Selection { start: 0, end: -1 });
                return;
            }
            _ if self.read_only => return,
            Edit::Toggle(effect) => {
                let mut format = char_format();
                format.Base.dwMask = effect.mask();
                if !self.effect_active(effect) {
                    format.Base.dwEffects = CFE_EFFECTS(effect.mask().0);
                }
                self.apply_format(SCF_SELECTION, &format);
                return;
            }
            Edit::Cut => WM_CUT,
            Edit::Paste => WM_PASTE,
            Edit::Delete => WM_CLEAR,
            Edit::Undo => EM_UNDO,
            Edit::Redo => EM_REDO,
        };
        self.message(message, 0, 0);
    }

    pub fn effect_active(&self, effect: Effect) -> bool {
        let format = self.selection_format();
        format.Base.dwMask.contains(effect.mask()) && format.Base.dwEffects.0 & effect.mask().0 != 0
    }

    pub fn selection_color(&self) -> Option<[u8; 3]> {
        let format = self.selection_format();
        if !format.Base.dwMask.contains(CFM_COLOR) || format.Base.dwEffects.contains(CFE_AUTOCOLOR)
        {
            return None;
        }
        let [red, green, blue, _] = format.Base.crTextColor.0.to_le_bytes();
        Some([red, green, blue])
    }

    pub fn set_selection_color(&self, rgb: Option<[u8; 3]>) {
        if !self.read_only {
            let mut format = char_format();
            format.Base.dwMask = CFM_COLOR;
            if let Some(rgb) = rgb {
                format.Base.crTextColor = color_ref(rgb);
            } else {
                format.Base.dwEffects = CFE_AUTOCOLOR;
            }
            self.apply_format(SCF_SELECTION, &format);
        }
    }

    pub fn find_from(&self, needle: &str, start: i32) -> Option<Selection> {
        if needle.is_empty() {
            return None;
        }
        let needle = terminated(needle).ok()?;
        self.find_wide(&needle, start)
    }

    fn find_wide(&self, needle: &[u16], start: i32) -> Option<Selection> {
        let mut query = FINDTEXTEXW {
            chrg: CHARRANGE {
                cpMin: start.max(0),
                cpMax: -1,
            },
            lpstrText: PCWSTR(needle.as_ptr()),
            ..Default::default()
        };
        let result = self.message(
            EM_FINDTEXTEXW,
            FR_DOWN.0 as usize,
            &mut query as *mut FINDTEXTEXW as isize,
        );
        (result >= 0).then_some(Selection {
            start: query.chrgText.cpMin,
            end: query.chrgText.cpMax,
        })
    }

    pub fn reveal_selection(&self) {
        self.message(EM_SCROLLCARET, 0, 0);
    }

    /// Highlight non-overlapping search matches without editing the document or its undo stack.
    pub fn highlight_matches(&self, needle: &str) -> Result<usize> {
        let needle = terminated(needle)?;
        let mut matches = Vec::new();
        if needle.len() > 1 {
            let mut next = 0;
            while let Some(selection) = self.find_wide(&needle, next) {
                if selection.end <= next {
                    break;
                }
                matches.push(selection);
                next = selection.end;
            }
        }
        let count = matches.len();
        if *self.events.search_matches.borrow() != matches {
            *self.events.search_matches.borrow_mut() = matches;
            self.events.invalidate_content();
        }
        Ok(count)
    }

    fn selection_format(&self) -> CHARFORMAT2W {
        let mut format = char_format();
        self.message(
            EM_GETCHARFORMAT,
            SCF_SELECTION as usize,
            &mut format as *mut CHARFORMAT2W as isize,
        );
        format
    }

    fn apply_format(&self, scope: u32, format: &CHARFORMAT2W) {
        self.message(
            EM_SETCHARFORMAT,
            scope as usize,
            format as *const CHARFORMAT2W as isize,
        );
    }

    /// Load a local RTF document using the operating system's reader.
    pub fn import_rtf(&self, bytes: &[u8]) -> io::Result<()> {
        let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
        if !bytes.starts_with(b"{\\rtf") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected an RTF document",
            ));
        }
        // Native stream-in can report an error after partially replacing a document.
        // Parse with an invisible sibling before changing the active document and its history.
        let parent =
            NonZeroIsize::new(self.events.parent.0 as isize).ok_or(io::ErrorKind::InvalidInput)?;
        Self::new(parent)
            .map_err(io::Error::other)?
            .stream_in(bytes)?;
        let backup = self.export_rtf()?;
        let selection = self.selection();
        if let Err(error) = self.stream_in(bytes) {
            self.stream_in(&backup)?;
            self.set_selection(selection);
            return Err(error);
        }
        self.message(EM_EMPTYUNDOBUFFER, 0, 0);
        self.events.changed.set(true);
        self.events.clear_matches();
        Ok(())
    }

    fn stream_in(&self, mut input: &[u8]) -> io::Result<()> {
        let mut stream = EDITSTREAM {
            dwCookie: &mut input as *mut &[u8] as usize,
            pfnCallback: Some(read_rtf),
            ..Default::default()
        };
        self.message(
            EM_STREAMIN,
            SF_RTF as usize,
            &mut stream as *mut EDITSTREAM as isize,
        );
        let error = stream.dwError;
        if error != 0 {
            return Err(io::Error::other(format!(
                "Rich Edit could not read RTF ({error:#x})"
            )));
        }
        Ok(())
    }

    pub fn export_rtf(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut stream = EDITSTREAM {
            dwCookie: &mut bytes as *mut Vec<u8> as usize,
            pfnCallback: Some(write_rtf),
            ..Default::default()
        };
        self.message(
            EM_STREAMOUT,
            SF_RTF as usize,
            &mut stream as *mut EDITSTREAM as isize,
        );
        let error = stream.dwError;
        if error != 0 {
            return Err(io::Error::other(format!(
                "Rich Edit could not write RTF ({error:#x})"
            )));
        }
        Ok(bytes)
    }
}

impl Drop for RichEdit {
    fn drop(&mut self) {
        // Closing the parent can destroy this child before its Rust handle is dropped.
        unsafe {
            if IsWindow(Some(self.window)).as_bool() {
                let _ = DestroyWindow(self.window);
            }
        }
    }
}

fn char_format() -> CHARFORMAT2W {
    let mut format = CHARFORMAT2W::default();
    format.Base.cbSize = size_of::<CHARFORMAT2W>() as u32;
    format
}

fn text_document(window: HWND) -> Option<ITextDocument2> {
    let mut pointer = std::ptr::null_mut::<std::ffi::c_void>();
    if unsafe {
        SendMessageW(
            window,
            EM_GETOLEINTERFACE,
            None,
            Some(LPARAM(&mut pointer as *mut _ as isize)),
        )
    }
    .0 == 0
        || pointer.is_null()
    {
        return None;
    }
    // EM_GETOLEINTERFACE returns an owned COM reference.
    unsafe { IUnknown::from_raw(pointer) }.cast().ok()
}

fn paint_matches(window: HWND, dc: HDC, viewport: RECT, matches: &[Selection]) {
    let Some(document) = text_document(window) else {
        return;
    };
    let line_of = |offset| unsafe {
        SendMessageW(
            window,
            EM_EXLINEFROMCHAR,
            None,
            Some(LPARAM(offset as isize)),
        )
        .0
    };
    let line_start = |line| unsafe {
        SendMessageW(window, EM_LINEINDEX, Some(WPARAM(line as usize)), None).0 as i32
    };
    let probe = POINT {
        x: viewport.left,
        y: viewport.bottom - 1,
    };
    let last_visible = unsafe {
        SendMessageW(
            window,
            EM_CHARFROMPOS,
            None,
            Some(LPARAM(&probe as *const _ as isize)),
        )
        .0
    };
    let first = unsafe { SendMessageW(window, EM_GETFIRSTVISIBLELINE, None, None) }.0;
    let last = line_of(last_visible as i32).max(first);
    let mut rectangles = Vec::new();
    let first_offset = line_start(first);
    let first_match = matches.partition_point(|range| range.end <= first_offset);
    for range in &matches[first_match..] {
        let past_start = line_of(range.start);
        if past_start > last {
            break;
        }
        let through_end = line_of(range.end - 1);
        for line in past_start.max(first)..=through_end.min(last) {
            let start = if line == past_start {
                range.start
            } else {
                line_start(line)
            };
            let end = if line == through_end {
                range.end
            } else {
                line_start(line + 1)
            };
            if start < 0 || end <= start {
                continue;
            }
            let Ok(slice) = (unsafe { document.Range2(end, start) }) else {
                continue;
            };
            let mut bounds = RECT::default();
            let mut hit = 0;
            if unsafe {
                slice.GetRect(
                    tomClientCoord.0 | tomAllowOffClient.0,
                    &mut bounds.left,
                    &mut bounds.top,
                    &mut bounds.right,
                    &mut bounds.bottom,
                    &mut hit,
                )
            }
            .is_err()
            {
                continue;
            }
            bounds.left = bounds.left.max(viewport.left);
            bounds.right = bounds.right.min(viewport.right);
            bounds.top = bounds.top.max(viewport.top);
            bounds.bottom = bounds.bottom.min(viewport.bottom);
            if bounds.right > bounds.left && bounds.bottom > bounds.top {
                rectangles.push(bounds);
            }
        }
    }
    tint_matches(dc, &rectangles);
}

fn tint_matches(dc: HDC, rectangles: &[RECT]) {
    if rectangles.is_empty() {
        return;
    }
    unsafe {
        let source = CreateCompatibleDC(Some(dc));
        if source.is_invalid() {
            return;
        }
        let bitmap = CreateCompatibleBitmap(dc, 1, 1);
        let brush = CreateSolidBrush(COLORREF(GetSysColor(COLOR_HIGHLIGHT)));
        if !bitmap.is_invalid() && !brush.is_invalid() {
            let old = SelectObject(source, HGDIOBJ(bitmap.0));
            FillRect(
                source,
                &RECT {
                    left: 0,
                    top: 0,
                    right: 1,
                    bottom: 1,
                },
                brush,
            );
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                SourceConstantAlpha: 96,
                ..Default::default()
            };
            for rect in rectangles {
                let _ = AlphaBlend(
                    dc,
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    source,
                    0,
                    0,
                    1,
                    1,
                    blend,
                );
            }
            SelectObject(source, old);
        }
        let _ = DeleteObject(HGDIOBJ(brush.0));
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(source);
    }
}

fn color_ref([r, g, b]: [u8; 3]) -> COLORREF {
    COLORREF(u32::from_le_bytes([r, g, b, 0]))
}

fn terminated(text: &str) -> Result<Vec<u16>> {
    if text.contains('\0') {
        return Err(Error::from_hresult(E_INVALIDARG));
    }
    Ok(text.encode_utf16().chain([0]).collect())
}

fn decode_text(text: &[u16], count: isize) -> String {
    let count = (count.max(0) as usize).min(text.len());
    String::from_utf16_lossy(&text[..count]).replace('\r', "\n")
}

unsafe extern "system" fn read_rtf(
    cookie: usize,
    buffer: *mut u8,
    length: i32,
    count: *mut i32,
) -> u32 {
    if length <= 0 {
        unsafe { *count = 0 };
        return u32::from(length < 0);
    }
    // EDITSTREAM callbacks run synchronously. The cookie is the caller's input slice
    // and Rich Edit supplies a writable buffer of length bytes.
    let input = unsafe { &mut *(cookie as *mut &[u8]) };
    let buffer = unsafe { std::slice::from_raw_parts_mut(buffer, length as usize) };
    match input.read(buffer) {
        Ok(bytes) => {
            unsafe { *count = bytes as i32 };
            0
        }
        Err(_) => 1,
    }
}

unsafe extern "system" fn write_rtf(
    cookie: usize,
    buffer: *mut u8,
    length: i32,
    count: *mut i32,
) -> u32 {
    if length <= 0 {
        unsafe { *count = 0 };
        return u32::from(length < 0);
    }
    // The caller owns the output vector until the synchronous stream finishes.
    let output = unsafe { &mut *(cookie as *mut Vec<u8>) };
    let buffer = unsafe { std::slice::from_raw_parts(buffer, length as usize) };
    match output.write(buffer) {
        Ok(bytes) => {
            unsafe { *count = bytes as i32 };
            0
        }
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPED;

    struct TestWindow(HWND);

    impl TestWindow {
        fn new() -> Self {
            Self(unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("STATIC"),
                    PCWSTR::null(),
                    WS_OVERLAPPED,
                    0,
                    0,
                    300,
                    120,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap()
            })
        }

        fn editor(&self) -> RichEdit {
            RichEdit::new(NonZeroIsize::new(self.0.0 as isize).unwrap()).unwrap()
        }
    }

    impl Drop for TestWindow {
        fn drop(&mut self) {
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }

    #[test]
    fn placeholder_follows_document_emptiness_without_becoming_document_content() {
        let parent = TestWindow::new();
        let editor = parent.editor();
        let before = editor.export_rtf().unwrap();
        editor.set_placeholder("Enter text & 输入").unwrap();
        assert!(editor.events.draws_placeholder());
        assert_eq!(editor.text(), "");
        assert_eq!(editor.export_rtf().unwrap(), before);
        assert!(!editor.can_undo());

        editor.replace_selection("a").unwrap();
        assert!(!editor.events.draws_placeholder());
        editor.execute(Edit::Undo);
        assert!(editor.events.draws_placeholder());
        editor.set_placeholder("").unwrap();
        assert!(!editor.events.draws_placeholder());
    }

    #[test]
    fn search_highlights_leave_rtf_selection_style_and_history_intact() {
        let parent = TestWindow::new();
        let editor = parent.editor();
        editor.set_text("one two 😀 one").unwrap();
        let selection = Selection { start: 4, end: 7 };
        editor.set_selection(selection);
        editor.set_selection_color(Some([0x48, 0x2b, 0x78]));
        let before = editor.export_rtf().unwrap();
        let undo = editor.can_undo();
        assert_eq!(editor.highlight_matches("ONE").unwrap(), 2);
        assert_eq!(before, editor.export_rtf().unwrap());
        assert_eq!(editor.highlight_matches("two").unwrap(), 1);
        assert_eq!(editor.highlight_matches("").unwrap(), 0);
        assert_eq!(editor.selection(), selection);
        assert_eq!(editor.selection_color(), Some([0x48, 0x2b, 0x78]));
        assert_eq!(editor.can_undo(), undo);
        assert_eq!(before, editor.export_rtf().unwrap());
    }

    #[test]
    fn formats_and_round_trips_unicode_and_line_breaks() {
        let parent = TestWindow::new();
        let editor = parent.editor();
        let text = "Hi 😀\n第二行";
        editor.set_text(text).unwrap();
        assert_eq!(editor.text(), text);
        editor.set_selection(Selection { start: 3, end: 5 });
        assert_eq!(editor.selected_text(), "😀");
        editor.execute(Edit::Toggle(Effect::Bold));
        editor.execute(Edit::Toggle(Effect::Italic));
        editor.set_selection_color(Some([0x31, 0x91, 0x74]));
        assert!(editor.effect_active(Effect::Bold));
        let rtf = editor.export_rtf().unwrap();

        let loaded = parent.editor();
        loaded.import_rtf(&rtf).unwrap();
        assert_eq!(loaded.text(), text);
        loaded.set_selection(Selection { start: 3, end: 5 });
        assert!(loaded.effect_active(Effect::Bold));
        assert!(loaded.effect_active(Effect::Italic));
        assert_eq!(loaded.selection_color(), Some([0x31, 0x91, 0x74]));
        assert!(!loaded.can_undo());
        loaded.set_selection_color(None);
        assert_eq!(loaded.selection_color(), None);
        assert!(loaded.import_rtf(b"not RTF").is_err());
        assert_eq!(loaded.text(), text);
    }

    #[test]
    fn undo_redo_search_and_read_only_share_the_document() {
        let parent = TestWindow::new();
        let mut editor = parent.editor();
        editor.set_text("one TWO one two").unwrap();
        let range = editor.find_from("two", 0).unwrap();
        assert_eq!(range, Selection { start: 4, end: 7 });
        editor.set_selection(range);
        editor.replace_selection("three").unwrap();
        assert_eq!(editor.text(), "one three one two");
        assert!(editor.can_undo());
        editor.execute(Edit::Undo);
        assert_eq!(editor.text(), "one TWO one two");
        assert!(editor.can_redo());
        editor.execute(Edit::Redo);
        assert_eq!(editor.text(), "one three one two");
        editor.set_read_only(true);
        editor.execute(Edit::SelectAll);
        editor.replace_selection("x").unwrap();
        editor.execute(Edit::Delete);
        editor.execute(Edit::Undo);
        assert_eq!(editor.selected_text(), "one three one two");
    }
}
