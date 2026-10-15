#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("LimeRS v0.5.1 keyboard-hook front end runs on Windows 10/11 only.");
}

#[cfg(windows)]
mod punctuation;

#[cfg(windows)]
mod win {
    use limers_core::PortableEngine;
    use crate::punctuation::PunctuationPicker;
    use std::mem::{size_of, zeroed};
    use std::path::PathBuf;
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use std::sync::{Mutex, OnceLock};

    type Hwnd = isize;
    type Handle = isize;
    type Hhook = isize;
    type Hdc = isize;
    type Wparam = usize;
    type Lparam = isize;
    type Lresult = isize;
    const WM_APP: u32 = 0x8000;
    const MSG_KEY: u32 = WM_APP + 11;
    const MSG_ACTION: u32 = WM_APP + 12;
    const MSG_TRAY: u32 = WM_APP + 13;
    const ACT_TOGGLE: usize = 1;
    const ACT_CANCEL: usize = 2;
    const ACT_BACK: usize = 3;
    const ACT_PREV: usize = 4;
    const ACT_NEXT: usize = 5;
    const ACT_ENTER: usize = 6;
    const ACT_PAGE_UP: usize = 8;
    const ACT_PAGE_DOWN: usize = 9;
    const ACT_HOME: usize = 10;
    const ACT_END: usize = 11;
    const ACT_SHIFT_UP: usize = 7;
    const ACT_SHIFT_CHOICE: usize = 100;
    const ACT_PUNCT_TOGGLE: usize = 15;
    const ACT_PUNCT_CLOSE: usize = 16;
    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_PAINT: u32 = 0x000F;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_RBUTTONUP: u32 = 0x0205;
    const WM_MOUSEACTIVATE: u32 = 0x0021;
    const MA_NOACTIVATE: Lresult = 3;
    const WM_KEYDOWN: usize = 0x0100;
    const WM_KEYUP: usize = 0x0101;
    const WM_SYSKEYDOWN: usize = 0x0104;
    const WM_SYSKEYUP: usize = 0x0105;
    const WH_KEYBOARD_LL: i32 = 13;
    const LLKHF_INJECTED: u32 = 0x10;
    const VK_BACK: u32 = 0x08;
    const VK_RETURN: u32 = 0x0D;
    const VK_ESCAPE: u32 = 0x1B;
    const VK_SPACE: u32 = 0x20;
    const VK_OEM_COMMA: u32 = 0xBC;
    const VK_LEFT: u32 = 0x25;
    const VK_UP: u32 = 0x26;
    const VK_RIGHT: u32 = 0x27;
    const VK_DOWN: u32 = 0x28;
    const VK_PRIOR: u32 = 0x21;
    const VK_NEXT: u32 = 0x22;
    const VK_HOME: u32 = 0x24;
    const VK_END: u32 = 0x23;
    const VK_F9: u32 = 0x78;
    const VK_LSHIFT: u32 = 0xA0;
    const VK_RSHIFT: u32 = 0xA1;
    const VK_SHIFT: u32 = 0x10;
    const VK_LCONTROL: u32 = 0xA2;
    const VK_RCONTROL: u32 = 0xA3;
    const VK_CONTROL: u32 = 0x11;
    const VK_LMENU: u32 = 0xA4;
    const VK_RMENU: u32 = 0xA5;
    const VK_MENU: u32 = 0x12;
    const VK_LWIN: u32 = 0x5B;
    const VK_RWIN: u32 = 0x5C;
    const WS_POPUP: u32 = 0x80000000;
    const WS_OVERLAPPED: u32 = 0x00000000;
    const WS_EX_TOPMOST: u32 = 0x00000008;
    const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    const WS_EX_NOACTIVATE: u32 = 0x08000000;
    const SW_HIDE: i32 = 0;
    const SW_SHOWNOACTIVATE: i32 = 4;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_SHOWWINDOW: u32 = 0x0040;
    const CS_HREDRAW: u32 = 0x0002;
    const CS_VREDRAW: u32 = 0x0001;
    const TRANSPARENT: i32 = 1;
    const COLOR_WINDOW: usize = 5;
    const NIF_MESSAGE: u32 = 0x1;
    const NIF_ICON: u32 = 0x2;
    const NIF_TIP: u32 = 0x4;
    const NIM_ADD: u32 = 0;
    const NIM_MODIFY: u32 = 1;
    const NIM_DELETE: u32 = 2;
    const TPM_RIGHTBUTTON: u32 = 0x2;
    const TPM_RETURNCMD: u32 = 0x0100;
    const MF_STRING: u32 = 0;
    const MF_SEPARATOR: u32 = 0x0800;
    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 2;
    const KEYEVENTF_UNICODE: u32 = 4;
    const MENU_TOGGLE: u32 = 100;
    const MENU_EXIT: u32 = 101;
    const CANDIDATE_ROWS: usize = 9;
    const ROW_H: i32 = 28;
    const HEADER_H: i32 = 68;
    const WINDOW_W: i32 = 520;

    #[repr(C)] #[derive(Clone, Copy, Default)]
    struct Point { x:i32,y:i32 }
    #[repr(C)] #[derive(Clone, Copy, Default)]
    struct Rect {left:i32,top:i32,right:i32,bottom:i32}
    #[repr(C)]
    struct Msg { hwnd:Hwnd,message:u32,w_param:Wparam,l_param:Lparam,time:u32,pt:Point,l_private:u32 }
    #[repr(C)]
    struct WndClassExW {
        cb_size:u32,style:u32,proc:Option<unsafe extern "system" fn(Hwnd,u32,Wparam,Lparam)->Lresult>,
        cls_extra:i32,wnd_extra:i32,instance:Handle,icon:Handle,cursor:Handle,
        background:Handle,menu_name:*const u16,class_name:*const u16,small_icon:Handle,
    }
    #[repr(C)]
    struct KbdLlHookStruct { vk_code:u32,scan_code:u32,flags:u32,time:u32,extra:usize }
    #[repr(C)]
    struct GuiThreadInfo {cb_size:u32,flags:u32,active:Hwnd,focus:Hwnd,capture:Hwnd,menu_owner:Hwnd,move_size:Hwnd,caret:Hwnd,rc_caret:Rect}
    #[repr(C)]
    struct PaintStruct {hdc:Hdc,erase:i32,paint:Rect,restore:i32,inc_update:i32,reserved:[u8;32]}
    // Full NOTIFYICONDATAW Win32 structure (x64 / x86 alignment handled by repr(C)).
    #[repr(C)]
    struct NotifyIconDataW {
        cb_size:u32,hwnd:Hwnd,id:u32,flags:u32,callback:u32,icon:Handle,tip:[u16;128],
        state:u32,state_mask:u32,info:[u16;256],version:u32,info_title:[u16;64],
        info_flags:u32,guid:[u8;16],balloon_icon:Handle,
    }
    #[repr(C)]
    struct KeyBdInput {vk:u16,scan:u16,flags:u32,time:u32,extra:usize}
    #[repr(C)]
    struct Input { kind:u32, keyboard:KeyBdInput, tail:[u8;8] } // SIZE: 40 bytes on x64

    #[link(name="user32")]
    extern "system" {
        fn RegisterClassExW(cls:*const WndClassExW)->u16;
        fn CreateWindowExW(ex:u32,class:*const u16,name:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,
            parent:Hwnd,menu:Handle,instance:Handle,param:*mut std::ffi::c_void)->Hwnd;
        fn DefWindowProcW(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->Lresult;
        fn DestroyWindow(hwnd:Hwnd)->i32;
        fn GetMessageW(msg:*mut Msg,hwnd:Hwnd,min:u32,max:u32)->i32;
        fn TranslateMessage(msg:*const Msg)->i32;
        fn DispatchMessageW(msg:*const Msg)->Lresult;
        fn PostMessageW(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->i32;
        fn PostQuitMessage(code:i32);
        fn SetWindowsHookExW(id:i32,proc:Option<unsafe extern "system" fn(i32,Wparam,Lparam)->Lresult>,instance:Handle,thread:u32)->Hhook;
        fn UnhookWindowsHookEx(hook:Hhook)->i32;
        fn CallNextHookEx(hook:Hhook,code:i32,wp:Wparam,lp:Lparam)->Lresult;
        fn GetForegroundWindow()->Hwnd;
        fn GetWindowThreadProcessId(hwnd:Hwnd,pid:*mut u32)->u32;
        fn GetGUIThreadInfo(thread:u32,info:*mut GuiThreadInfo)->i32;
        fn ClientToScreen(hwnd:Hwnd,pt:*mut Point)->i32;
        fn GetWindowRect(hwnd:Hwnd,rect:*mut Rect)->i32;
        fn GetSystemMetrics(idx:i32)->i32;
        fn SetWindowPos(hwnd:Hwnd,after:Hwnd,x:i32,y:i32,w:i32,h:i32,flags:u32)->i32;
        fn ShowWindow(hwnd:Hwnd,cmd:i32)->i32;
        fn InvalidateRect(hwnd:Hwnd,rect:*const Rect,erase:i32)->i32;
        fn BeginPaint(hwnd:Hwnd,p:*mut PaintStruct)->Hdc;
        fn EndPaint(hwnd:Hwnd,p:*const PaintStruct)->i32;
        fn LoadIconW(inst:Handle,name:*const u16)->Handle;
        fn GetCursorPos(pt:*mut Point)->i32;
        fn CreatePopupMenu()->Handle;
        fn AppendMenuW(menu:Handle,flags:u32,id:usize,text:*const u16)->i32;
        fn TrackPopupMenu(menu:Handle,flags:u32,x:i32,y:i32,reserved:i32,hwnd:Hwnd,rect:*const Rect)->u32;
        fn DestroyMenu(menu:Handle)->i32;
        fn SetForegroundWindow(hwnd:Hwnd)->i32;
        fn MessageBoxW(hwnd:Hwnd,msg:*const u16,title:*const u16,flags:u32)->i32;
        fn FillRect(hdc:Hdc,rect:*const Rect,brush:Handle)->i32;
        fn SendInput(count:u32,inputs:*const Input,size:i32)->u32;
    }
    #[link(name="gdi32")]
    extern "system" {
        fn CreateSolidBrush(color:u32)->Handle;
        fn DeleteObject(obj:Handle)->i32;
        fn SetBkMode(hdc:Hdc,mode:i32)->i32;
        fn SetTextColor(hdc:Hdc,color:u32)->u32;
        fn CreateFontW(height:i32,width:i32,escapement:i32,orientation:i32,weight:i32,italic:u32,underline:u32,
            strike_out:u32,char_set:u32,out_precision:u32,clip_precision:u32,quality:u32,pitch_and_family:u32,face:*const u16)->Handle;
        fn SelectObject(hdc:Hdc,obj:Handle)->Handle;
        fn TextOutW(hdc:Hdc,x:i32,y:i32,text:*const u16,count:i32)->i32;
    }
    #[link(name="shell32")]
    extern "system" {fn Shell_NotifyIconW(cmd:u32,data:*const NotifyIconDataW)->i32;}
    #[link(name="kernel32")]
    extern "system" {
        fn GetModuleHandleW(name:*const u16)->Handle;
        fn CreateMutexW(attributes:*const std::ffi::c_void,initial:i32,name:*const u16)->Handle;
        fn GetLastError()->u32;
        fn CloseHandle(handle:Handle)->i32;
    }

    static MAIN: AtomicIsize=AtomicIsize::new(0);
    static ACTIVE: AtomicBool=AtomicBool::new(false);
    static COMPOSING: AtomicBool=AtomicBool::new(false);
    static PUNCT_ACTIVE: AtomicBool=AtomicBool::new(false);
    static RIGHT_SHIFT: AtomicBool=AtomicBool::new(false);
    static SHIFT_COMBO: AtomicBool=AtomicBool::new(false);
    static LEFT_SHIFT: AtomicBool=AtomicBool::new(false);
    static CTRL: AtomicBool=AtomicBool::new(false);
    static ALT: AtomicBool=AtomicBool::new(false);
    static WIN: AtomicBool=AtomicBool::new(false);
    static CONSUMED: [AtomicBool;256]=[const {AtomicBool::new(false)};256];
    static APP: OnceLock<Mutex<App>> = OnceLock::new();

    struct App {
        engine:PortableEngine,
        punct:PunctuationPicker,
        main:Hwnd,
        popup:Hwnd,
        target:Hwnd,
        enabled:bool,
        deferred:Option<String>,
        hook:Hhook,
        tray_added:bool,
    }

    fn wide(s:&str)->Vec<u16> {s.encode_utf16().chain(std::iter::once(0)).collect()}
    fn tip_copy<const N:usize>(dest:&mut [u16;N],text:&str) {
        for (i,c) in text.encode_utf16().take(N-1).enumerate(){dest[i]=c;}
    }
    fn set_composing(active:bool) {COMPOSING.store(active,Ordering::SeqCst);}
    fn update_composing(app:&App) {set_composing(app.engine.has_input() || app.punct.is_open());}
    fn update_punct_flag(app:&App) {PUNCT_ACTIVE.store(app.punct.is_open(),Ordering::SeqCst);}
    fn post_action(code:usize,extra:isize) {
        let hwnd=MAIN.load(Ordering::Relaxed);
        if hwnd!=0 { unsafe {PostMessageW(hwnd,MSG_ACTION,code,extra);} }
    }
    fn is_right_shift(vk:u32,sc:u32)->bool {vk==VK_RSHIFT || (vk==VK_SHIFT && sc==0x36)}
    fn is_left_shift(vk:u32,sc:u32)->bool {vk==VK_LSHIFT || (vk==VK_SHIFT && sc==0x2a)}
    fn modifier(set:&AtomicBool,down:bool) {set.store(down,Ordering::SeqCst);}
    fn suppress_key_up(vk:usize)->bool {vk<256 && CONSUMED[vk].swap(false,Ordering::SeqCst)}
    fn consume_down(vk:usize)->bool {if vk<256 {CONSUMED[vk].store(true,Ordering::SeqCst);} true}

    fn translate_key(vk:u32)->Option<char> {
        let shifted=LEFT_SHIFT.load(Ordering::Relaxed)||RIGHT_SHIFT.load(Ordering::Relaxed);
        match vk {
            0x41..=0x5a=>Some(if shifted{char::from_u32(vk).unwrap()}else{char::from_u32(vk+32).unwrap()}),
            0x30..=0x39=>char::from_u32(vk),
            0x20=>Some(' '),
            0xBA=>Some(';'),0xBB=>Some('='),0xBC=>Some(','),0xBD=>Some('-'),
            0xBE=>Some('.'),0xBF=>Some('/'),0xC0=>Some('`'),0xDB=>Some('['),
            0xDC=>Some('\\'),0xDD=>Some(']'),0xDE=>Some('\''),
            _=>None,
        }
    }

    unsafe extern "system" fn keyboard_hook(code:i32,wp:Wparam,lp:Lparam)->Lresult {
        if code<0 {return CallNextHookEx(0,code,wp,lp);}
        let kb=&*(lp as *const KbdLlHookStruct);
        if kb.flags & LLKHF_INJECTED !=0 {return CallNextHookEx(0,code,wp,lp);}
        let vk=kb.vk_code;
        let down=wp==WM_KEYDOWN||wp==WM_SYSKEYDOWN;
        let up=wp==WM_KEYUP||wp==WM_SYSKEYUP;
        if !down && !up {return CallNextHookEx(0,code,wp,lp);}
        let rk=is_right_shift(vk,kb.scan_code);
        let lk=is_left_shift(vk,kb.scan_code);
        if rk {
            if down && !RIGHT_SHIFT.swap(true,Ordering::SeqCst) {SHIFT_COMBO.store(false,Ordering::SeqCst);}
            if up {
                RIGHT_SHIFT.store(false,Ordering::SeqCst);
                if ACTIVE.load(Ordering::Relaxed) {post_action(ACT_SHIFT_UP,(!SHIFT_COMBO.swap(false,Ordering::SeqCst)) as isize);}
            }
        }
        if lk {modifier(&LEFT_SHIFT,down);}
        match vk {
            VK_LCONTROL|VK_RCONTROL|VK_CONTROL=>modifier(&CTRL,down),
            VK_LMENU|VK_RMENU|VK_MENU=>modifier(&ALT,down),
            VK_LWIN|VK_RWIN=>modifier(&WIN,down),
            _=>{}
        }
        if down && RIGHT_SHIFT.load(Ordering::Relaxed) && !rk {SHIFT_COMBO.store(true,Ordering::SeqCst);}
        if vk==VK_F9 && CTRL.load(Ordering::Relaxed)&&ALT.load(Ordering::Relaxed) && !WIN.load(Ordering::Relaxed) {
            if down && !CONSUMED[vk as usize].swap(true,Ordering::SeqCst) {
                post_action(ACT_TOGGLE,0); // once per physical key press
            }
            if up {suppress_key_up(vk as usize);}
            return 1;
        }
        if up && suppress_key_up(vk as usize){return 1;}
        if !down || !ACTIVE.load(Ordering::Relaxed) || CTRL.load(Ordering::Relaxed) || ALT.load(Ordering::Relaxed) || WIN.load(Ordering::Relaxed) {
            return CallNextHookEx(0,code,wp,lp);
        }
        // Shift+, opens or closes a punctuation candidate panel instead of
        // forwarding '>' to the target application. Both Shift keys work.
        if vk==VK_OEM_COMMA && (LEFT_SHIFT.load(Ordering::Relaxed)||RIGHT_SHIFT.load(Ordering::Relaxed)) {
            if !CONSUMED[vk as usize].swap(true,Ordering::SeqCst) {post_action(ACT_PUNCT_TOGGLE,0);}
            return 1;
        }
        let punct_open=PUNCT_ACTIVE.load(Ordering::Relaxed);
        let composing=COMPOSING.load(Ordering::Relaxed);
        if composing && RIGHT_SHIFT.load(Ordering::Relaxed) && (0x30..=0x39).contains(&vk) {
            let slot=if vk==0x30 {9} else {(vk-0x31) as usize};
            if !CONSUMED[vk as usize].swap(true,Ordering::SeqCst) {
                post_action(ACT_SHIFT_CHOICE+slot,0);
            }
            return 1;
        }
        if composing {
            // Candidate list is vertical: ↑/↓ select rows; ←/→ are not used
            // for candidate selection. PageUp/PageDown move a whole visible page.
            let action=match vk {
                VK_ESCAPE=>Some(ACT_CANCEL),VK_BACK=>Some(ACT_BACK),
                VK_UP=>Some(ACT_PREV),VK_DOWN=>Some(ACT_NEXT),
                VK_PRIOR=>Some(ACT_PAGE_UP),VK_NEXT=>Some(ACT_PAGE_DOWN),
                VK_HOME=>Some(ACT_HOME),VK_END=>Some(ACT_END),
                VK_RETURN=>Some(ACT_ENTER),_=>None
            };
            if let Some(action)=action {post_action(action,0);consume_down(vk as usize);return 1;}
        }
        // Without an active composition, Shift + number belongs to Windows
        // (e.g. ! @ #), not to the raw digit input buffer.
        if !composing && (RIGHT_SHIFT.load(Ordering::Relaxed)||LEFT_SHIFT.load(Ordering::Relaxed))
            && (0x30..=0x39).contains(&vk) {
            return CallNextHookEx(0,code,wp,lp);
        }
        if let Some(c)=translate_key(vk) {
            // Typing ordinary text closes the punctuation palette first,
            // then processes the key normally as a new/continuing composition.
            if punct_open {post_action(ACT_PUNCT_CLOSE,0);}
            if c==' ' && !composing {return CallNextHookEx(0,code,wp,lp);}
            let hwnd=MAIN.load(Ordering::Relaxed);
            if hwnd!=0 && PostMessageW(hwnd,MSG_KEY,c as Wparam,0)!=0 {consume_down(vk as usize);return 1;}
        }
        CallNextHookEx(0,code,wp,lp)
    }

    fn with_app<F:FnOnce(&mut App)>(f:F) {
        if let Some(lock)=APP.get() {if let Ok(mut app)=lock.lock(){f(&mut app);}}
    }
    unsafe fn inject_unicode(s:&str)->bool {
        if s.is_empty(){return true;}
        let mut inputs=Vec::new();
        for unit in s.encode_utf16() {
            inputs.push(Input{kind:INPUT_KEYBOARD,keyboard:KeyBdInput{vk:0,scan:unit,flags:KEYEVENTF_UNICODE,time:0,extra:0},tail:[0;8]});
            inputs.push(Input{kind:INPUT_KEYBOARD,keyboard:KeyBdInput{vk:0,scan:unit,flags:KEYEVENTF_UNICODE|KEYEVENTF_KEYUP,time:0,extra:0},tail:[0;8]});
        }
        if inputs.is_empty(){return true;}
        // SendInput may fail across integrity boundaries (e.g. non-elevated to elevated).
        SendInput(inputs.len() as u32,inputs.as_ptr(),size_of::<Input>() as i32)==inputs.len() as u32
    }
    unsafe fn send_to_original_target(text:&str)->bool {
        let mut target=0;
        if let Some(lock)=APP.get(){if let Ok(app)=lock.lock(){target=app.target;}}
        target!=0 && GetForegroundWindow()==target && inject_unicode(text)
    }
    fn commit_result(app:&mut App,text:Option<String>,from_shift:bool)->Option<String> {
        let text=text?;
        set_composing(app.engine.has_input());
        if from_shift && RIGHT_SHIFT.load(Ordering::Relaxed) {
            app.deferred=Some(text);
            None
        } else {Some(text)}
    }
    unsafe fn handle_message_key(c:char) {
        with_app(|app| {
            if !app.enabled{return;}
            let fg=GetForegroundWindow();
            if (app.engine.has_input() || app.punct.is_open()) && fg!=app.target {
                app.engine.cancel();app.punct.close();update_punct_flag(app);
            }
            if !app.engine.has_input(){app.target=fg;}
            app.engine.push(c);
            update_composing(app);
        });
        refresh_popup();
    }
    fn choose_punctuation(app:&mut App,index:usize,from_shift:bool)->Option<String> {
        let Some(symbol)=app.punct.pick(index) else{return None;};
        update_punct_flag(app);
        // When composing, committing punctuation first commits the currently
        // composed sentence, so punctuation appears AFTER it in the editor.
        let mut text=app.engine.choose(0).unwrap_or_default();
        text.push_str(symbol);
        update_composing(app);
        if from_shift && RIGHT_SHIFT.load(Ordering::Relaxed) {
            app.deferred=Some(text);
            None
        } else {Some(text)}
    }
    unsafe fn handle_action(action:usize,param:isize) {
        let mut to_send:Option<String>=None;
        let mut exit=false;
        with_app(|app|{
            match action {
                ACT_TOGGLE=>{
                    app.enabled=!app.enabled;
                    app.engine.cancel();app.punct.close();update_punct_flag(app);app.deferred=None;
                    ACTIVE.store(app.enabled,Ordering::SeqCst);
                    set_composing(false);
                    if app.tray_added {let tray=tray_data(app.main);unsafe{Shell_NotifyIconW(NIM_MODIFY,&tray);}}
                }
                ACT_SHIFT_UP=>{
                    if param!=0 && app.enabled {app.engine.toggle_case();}
                    to_send=app.deferred.take();
                }
                ACT_PUNCT_TOGGLE=>{
                    if app.enabled {
                        if app.punct.is_open() {app.punct.close();}
                        else {
                            // A palette opened without an existing composition
                            // must capture the focused editor for SendInput.
                            if !app.engine.has_input() {app.target=GetForegroundWindow();}
                            app.punct.open();
                        }
                        update_punct_flag(app);
                    }
                }
                ACT_PUNCT_CLOSE=>{app.punct.close();update_punct_flag(app);}
                ACT_CANCEL=>{
                    if app.punct.is_open() {app.punct.close();update_punct_flag(app);}
                    else {app.engine.cancel();}
                }
                ACT_BACK=>{
                    if app.punct.is_open() {app.punct.close();update_punct_flag(app);}
                    else {app.engine.backspace();}
                }
                ACT_PREV=>{
                    if app.punct.is_open() {app.punct.navigate(-1);} else {app.engine.navigate(-1);}
                }
                ACT_NEXT=>{
                    if app.punct.is_open() {app.punct.navigate(1);} else {app.engine.navigate(1);}
                }
                ACT_PAGE_UP=>{
                    if app.punct.is_open() {app.punct.navigate(-(CANDIDATE_ROWS as i32));}
                    else {app.engine.navigate(-(CANDIDATE_ROWS as i32));}
                }
                ACT_PAGE_DOWN=>{
                    if app.punct.is_open() {app.punct.navigate(CANDIDATE_ROWS as i32);}
                    else {app.engine.navigate(CANDIDATE_ROWS as i32);}
                }
                ACT_HOME=>{
                    if app.punct.is_open() {app.punct.navigate_edge(false);}
                    else {app.engine.navigate_edge(false);}
                }
                ACT_END=>{
                    if app.punct.is_open() {app.punct.navigate_edge(true);}
                    else {app.engine.navigate_edge(true);}
                }
                ACT_ENTER=>{
                    if app.punct.is_open() {
                        to_send=choose_punctuation(app,app.punct.selected(),false);
                    } else if app.engine.has_input() {
                        let result=app.engine.choose_selected();
                        to_send=commit_result(app,result,false);
                    }
                }
                v if v>=ACT_SHIFT_CHOICE && v<ACT_SHIFT_CHOICE+10=>{
                    let slot=v-ACT_SHIFT_CHOICE;
                    if app.punct.is_open() {
                        let first=visible_start(app.punct.selected(),app.punct.len());
                        let index=if slot==9 {9} else {first+slot};
                        to_send=choose_punctuation(app,index,true);
                    } else if app.engine.has_input() {
                        let count=app.engine.candidates().len();
                        let first=visible_start(app.engine.selected(),count);
                        let index=if slot==9 {9} else {first+slot};
                        if index<count {
                            let result=app.engine.choose(index);
                            to_send=commit_result(app,result,true);
                        }
                    }
                }
                998=>exit=true,
                _=>{}
            }
            update_composing(app);
        });
        if let Some(text)=to_send {let _=send_to_original_target(&text);}
        refresh_popup();
        if exit {shutdown();}
    }
    // Keep keyboard shortcuts, painted numbers and mouse hit targets aligned.
    fn visible_start(selected:usize,count:usize)->usize {
        if count==0 {return 0;}
        // Stable nine-row pages make PageUp/PageDown and Shift+1..9 predictable.
        (selected.min(count-1)/CANDIDATE_ROWS)*CANDIDATE_ROWS
    }
    fn relative_number(index:usize,first:usize)->String {
        (index-first+1).to_string()
    }
    unsafe fn refresh_popup() {
        let mut snapshot:Option<(Hwnd,Hwnd,bool,usize,usize)>=None;
        if let Some(lock)=APP.get() {
            if let Ok(app)=lock.lock() {
                let (count,selected)=if app.punct.is_open() {
                    (app.punct.len(),app.punct.selected())
                } else {(app.engine.candidates().len(),app.engine.selected())};
                snapshot=Some((app.popup,app.target,app.enabled&&(app.engine.has_input()||app.punct.is_open()),
                    count,selected));
            }
        }
        let Some((popup,target,visible,count,selected))=snapshot else{return;};
        if popup==0{return;}
        if !visible {ShowWindow(popup,SW_HIDE);return;}
        let row_count=count.saturating_sub(visible_start(selected,count)).min(CANDIDATE_ROWS) as i32;
        let height=HEADER_H+row_count*ROW_H+12;
        let mut x=150;let mut y=150;
        if target!=0 {
            let thread=GetWindowThreadProcessId(target,null_mut());
            let mut info:GuiThreadInfo=zeroed();info.cb_size=size_of::<GuiThreadInfo>() as u32;
            if thread!=0 && GetGUIThreadInfo(thread,&mut info)!=0 && info.caret!=0 {
                let mut pt=Point{x:info.rc_caret.left,y:info.rc_caret.bottom+6};
                if ClientToScreen(info.caret,&mut pt)!=0 {x=pt.x;y=pt.y;}
            } else {
                let mut r:Rect=zeroed();
                if GetWindowRect(target,&mut r)!=0 {x=r.left+40;y=r.top+70;}
            }
        }
        let sx=GetSystemMetrics(0).max(WINDOW_W);let sy=GetSystemMetrics(1).max(height);
        x=x.clamp(0,sx-WINDOW_W);
        if y+height>sy {y=(y-height-28).max(0);}
        y=y.clamp(0,sy-height);
        SetWindowPos(popup,-1,x,y,WINDOW_W,height,SWP_NOACTIVATE|SWP_SHOWWINDOW);
        InvalidateRect(popup,null(),1);
    }
    unsafe fn paint_popup(hwnd:Hwnd) {
        let mut ps:PaintStruct=zeroed();
        let hdc=BeginPaint(hwnd,&mut ps);
        if hdc==0{return;}
        let mut display:Option<(String,Vec<String>,usize,bool,bool)>=None;
        if let Some(lock)=APP.get() {if let Ok(app)=lock.lock(){
            display=Some(if app.punct.is_open() {
                ("常用標點符號".to_owned(),app.punct.labels(),app.punct.selected(),app.engine.uppercase(),true)
            } else {
                (app.engine.preedit(),app.engine.candidates(),app.engine.selected(),app.engine.uppercase(),false)
            });
        }}
        if let Some((raw,candidates,selected,upper,is_punct))=display {
            let start=visible_start(selected,candidates.len());
            let rows=candidates.len().saturating_sub(start).min(CANDIDATE_ROWS) as i32;
            let h=(HEADER_H+ROW_H*rows+12).max(HEADER_H);
            let back=CreateSolidBrush(0x00FFFDFC);
            FillRect(hdc,&Rect{left:0,top:0,right:WINDOW_W,bottom:h},back);DeleteObject(back);
            SetBkMode(hdc,TRANSPARENT);
            let face=wide("Microsoft JhengHei");
            let font=CreateFontW(-19,0,0,0,400,0,0,0,1,0,0,5,0,face.as_ptr());
            let old=SelectObject(hdc,font);
            let header=if is_punct {"LimeRS 標點符號   ·   ↑↓ 選擇 / Enter 送出".to_owned()}
                else {format!("LimeRS   {}    ·    Enter 送出",if upper {"ABC"}else{"abc"})};
            draw_line(hdc,12,8,&header,0x003F3F3F);
            let composition=if is_punct {"標點：Shift + ,  開關 / Esc 取消".to_owned()}
                else {format!("組字：{}",raw)};
            draw_line(hdc,12,36,&composition,0x00665522);
            for (i,c) in candidates.iter().enumerate().skip(start).take(CANDIDATE_ROWS) {
                let y=HEADER_H+((i-start) as i32)*ROW_H;
                if i==selected {
                    let highlight=CreateSolidBrush(0x00F7E7C4);
                    FillRect(hdc,&Rect{left:6,top:y,right:WINDOW_W-6,bottom:y+ROW_H-2},highlight);
                    DeleteObject(highlight);
                }
                let text=format!("{}  {}",relative_number(i,start),c);
                draw_line(hdc,14,y+3,&text,if i==selected{0x005B3A0B}else{0x00333333});
            }
            SelectObject(hdc,old);DeleteObject(font);
        }
        EndPaint(hwnd,&ps);
    }
    unsafe fn draw_line(hdc:Hdc,x:i32,y:i32,s:&str,color:u32) {
        SetTextColor(hdc,color);
        let ws:Vec<u16>=s.encode_utf16().collect();
        TextOutW(hdc,x,y,ws.as_ptr(),ws.len() as i32);
    }
    fn tray_data(hwnd:Hwnd)->NotifyIconDataW {
        let mut d:NotifyIconDataW=unsafe{zeroed()};
        d.cb_size=size_of::<NotifyIconDataW>() as u32;
        d.hwnd=hwnd;d.id=1;d.flags=NIF_MESSAGE|NIF_ICON|NIF_TIP;
        d.callback=MSG_TRAY;
        d.icon=unsafe{LoadIconW(0,32512usize as *const u16)};
        tip_copy(&mut d.tip,if ACTIVE.load(Ordering::Relaxed) {"LimeRS v0.5.2｜啟用中｜Shift+, 標點符號"} else {"LimeRS v0.5.2｜已暫停｜Ctrl+Alt+F9 啟用"});
        d
    }
    unsafe fn popup_tray_menu(hwnd:Hwnd) {
        let menu=CreatePopupMenu();if menu==0{return;}
        let enabled=ACTIVE.load(Ordering::Relaxed);
        let toggle=wide(if enabled{"暫停 LimeRS"}else{"啟用 LimeRS"});
        let exit=wide("結束 LimeRS");
        AppendMenuW(menu,MF_STRING,MENU_TOGGLE as usize,toggle.as_ptr());
        AppendMenuW(menu,MF_SEPARATOR,0,null());
        AppendMenuW(menu,MF_STRING,MENU_EXIT as usize,exit.as_ptr());
        let mut pt=Point::default();GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        let choice=TrackPopupMenu(menu,TPM_RETURNCMD|TPM_RIGHTBUTTON,pt.x,pt.y,0,hwnd,null());
        DestroyMenu(menu);
        match choice {MENU_TOGGLE=>handle_action(ACT_TOGGLE,0),MENU_EXIT=>shutdown(),_=>{}}
        PostMessageW(hwnd,0,0,0);
    }
    unsafe fn shutdown() {
        with_app(|app| {
            ACTIVE.store(false,Ordering::SeqCst);
            COMPOSING.store(false,Ordering::SeqCst);
            PUNCT_ACTIVE.store(false,Ordering::SeqCst);
            if app.hook!=0 {UnhookWindowsHookEx(app.hook);app.hook=0;}
            if app.tray_added {let d=tray_data(app.main);Shell_NotifyIconW(NIM_DELETE,&d);app.tray_added=false;}
            ShowWindow(app.popup,SW_HIDE);
        });
        PostQuitMessage(0);
    }
    unsafe extern "system" fn main_proc(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->Lresult {
        match msg {
            MSG_KEY=>{if let Some(c)=char::from_u32(wp as u32){handle_message_key(c);}0},
            MSG_ACTION=>{handle_action(wp,lp);0},
            MSG_TRAY=>{match lp as u32 {WM_LBUTTONUP=>handle_action(ACT_TOGGLE,0),WM_RBUTTONUP=>popup_tray_menu(hwnd),_=>{}}0},
            WM_DESTROY=>{shutdown();0},
            _=>DefWindowProcW(hwnd,msg,wp,lp)
        }
    }
    unsafe extern "system" fn popup_proc(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->Lresult {
        match msg {
            WM_MOUSEACTIVATE=>MA_NOACTIVATE,
            WM_PAINT=>{paint_popup(hwnd);0},
            WM_LBUTTONDOWN=>{
                let y=((lp as u64 >>16)&0xFFFF) as i16 as i32;
                if y>=HEADER_H {
                    let mut selected=None;
                    with_app(|app|{
                        if app.punct.is_open() {
                            let start=visible_start(app.punct.selected(),app.punct.len());
                            let index=start+((y-HEADER_H)/ROW_H) as usize;
                            selected=choose_punctuation(app,index,false);
                        } else {
                            let candidates=app.engine.candidates();
                            let start=visible_start(app.engine.selected(),candidates.len());
                            let index=start+((y-HEADER_H)/ROW_H) as usize;
                            if index<candidates.len(){
                                let result=app.engine.choose(index);
                                selected=commit_result(app,result,false);
                            }
                        }
                    });
                    if let Some(text)=selected {let _=send_to_original_target(&text);}
                    refresh_popup();
                }
                0
            }
            _=>DefWindowProcW(hwnd,msg,wp,lp)
        }
    }
    unsafe fn create_class(name:&[u16],instance:Handle,proc:unsafe extern "system" fn(Hwnd,u32,Wparam,Lparam)->Lresult)->bool {
        let wc=WndClassExW{cb_size:size_of::<WndClassExW>() as u32,style:CS_HREDRAW|CS_VREDRAW,proc:Some(proc),
            cls_extra:0,wnd_extra:0,instance,icon:0,cursor:0,background:(COLOR_WINDOW+1) as Handle,
            menu_name:null(),class_name:name.as_ptr(),small_icon:0};
        RegisterClassExW(&wc)!=0
    }

    pub fn run() {
        unsafe {
            let mutex_name=wide("Local\\LimeRS_Portable_v05_OneInstance");
            let mutex=CreateMutexW(null(),0,mutex_name.as_ptr());
            if mutex==0 || GetLastError()==183 {
                let msg=wide("LimeRS 已經在執行。請檢查系統匣。");let title=wide("LimeRS");
                MessageBoxW(0,msg.as_ptr(),title.as_ptr(),0);
                if mutex!=0 {CloseHandle(mutex);}return;
            }
            let dir=std::env::current_exe().ok().and_then(|p|p.parent().map(PathBuf::from)).unwrap_or_else(||PathBuf::from("."));
            let data=dir.join("data");let _=std::fs::create_dir_all(&data);
            let engine=PortableEngine::new(&data);
            let inst=GetModuleHandleW(null());
            let main_class=wide("LimeRS_Portable_MessageWindow");
            let popup_class=wide("LimeRS_Portable_Candidates");
            if !create_class(&main_class,inst,main_proc)||!create_class(&popup_class,inst,popup_proc) {
                let msg=wide("無法註冊 LimeRS 視窗類別。");let title=wide("LimeRS");MessageBoxW(0,msg.as_ptr(),title.as_ptr(),0);CloseHandle(mutex);return;
            }
            let name=wide("LimeRS Portable");
            let main=CreateWindowExW(0,main_class.as_ptr(),name.as_ptr(),WS_OVERLAPPED,0,0,0,0,0,0,inst,null_mut());
            let popup=CreateWindowExW(WS_EX_TOPMOST|WS_EX_TOOLWINDOW|WS_EX_NOACTIVATE,popup_class.as_ptr(),name.as_ptr(),WS_POPUP,
                0,0,WINDOW_W,340,0,0,inst,null_mut());
            if main==0||popup==0 {
                let msg=wide("無法建立 LimeRS 視窗。");let title=wide("LimeRS");MessageBoxW(0,msg.as_ptr(),title.as_ptr(),0);CloseHandle(mutex);return;
            }
            MAIN.store(main,Ordering::SeqCst);
            APP.set(Mutex::new(App{engine,punct:PunctuationPicker::default(),main,popup,target:0,enabled:false,deferred:None,hook:0,tray_added:false})).ok();
            let d=tray_data(main);
            if Shell_NotifyIconW(NIM_ADD,&d)!=0 {with_app(|app|app.tray_added=true);}
            let hook=SetWindowsHookExW(WH_KEYBOARD_LL,Some(keyboard_hook),inst,0);
            if hook==0 {
                let msg=wide("無法建立鍵盤攔截。程式將結束。請以一般使用者權限執行。");let title=wide("LimeRS");
                MessageBoxW(0,msg.as_ptr(),title.as_ptr(),0);shutdown();
                DestroyWindow(popup);DestroyWindow(main);CloseHandle(mutex);return;
            }
            with_app(|app|{app.hook=hook;app.enabled=true;});
            ACTIVE.store(true,Ordering::SeqCst);
            // A successful launch is immediately active. The user may still
            // suspend with Ctrl+Alt+F9 or the tray menu at any time.
            let status=tray_data(main);
            Shell_NotifyIconW(NIM_MODIFY,&status);
            let mut msg:Msg=zeroed();
            while GetMessageW(&mut msg,0,0,0)>0 {TranslateMessage(&msg);DispatchMessageW(&msg);}
            shutdown();DestroyWindow(popup);DestroyWindow(main);CloseHandle(mutex);
        }
    }
}

#[cfg(windows)]
fn main(){win::run();}
