//! LimeRS 0.5.1: platform-independent mixed-candidate engine and a stable UTF-16 C ABI.
//! Slots: 1 whole sentence, 2 raw keys, 3+ local candidates.
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const MAX_INPUT: usize = 160;
// Many reading codes have hundreds of homophones. The old cap of 24 hid
// legitimate characters permanently (e.g. 隻 under first-tone ㄓ).
// Keep a generous defensive cap while allowing the entire bundled dictionary
// to be traversed with ↑/↓ and PageUp/PageDown.
const MAX_CANDIDATES: usize = 2048;

#[derive(Clone, Debug)]
struct Word { text: String, score: i64 }

struct Dictionary { entries: HashMap<String, Vec<Word>> }
impl Dictionary {
    fn new() -> Self {
        let mut result = Self { entries: HashMap::new() };
        for line in include_str!("../assets/zhuyin.tsv").lines() {
            if line.starts_with('#') { continue; }
            let mut cols = line.split('\t');
            if let (Some(code), Some(text), Some(weight))=(cols.next(), cols.next(), cols.next()) {
                if let Ok(score)=weight.parse() { result.insert(code,text,score); }
            }
        }
        result.finalize();
        result
    }
    fn insert(&mut self, code:&str, text:&str, score:i64) {
        if code.is_empty() || text.is_empty() { return; }
        self.entries.entry(code.to_ascii_lowercase()).or_default()
            .push(Word { text:text.to_owned(), score });
    }
    fn finalize(&mut self) {
        for values in self.entries.values_mut() {
            values.sort_by(|a,b|b.score.cmp(&a.score).then_with(||a.text.cmp(&b.text)));
            let mut seen=HashSet::new();
            values.retain(|w| seen.insert(w.text.clone()));
        }
    }
    fn has_code(&self,code:&str)->bool {
        self.entries.contains_key(&normal_key(code))
    }
    fn lookup(&self, code:&str, learned:&HashMap<(String,String),i64>)->Vec<Word> {
        let raw=code.to_ascii_lowercase();
        let norm=normal_key(&raw);
        let mut words=self.entries.get(&norm).cloned().unwrap_or_default();
        // A complete 6/3/4/7 or first-tone '~' is STRICT; don't pollute tone results.
        let unfinished=!raw.ends_with('~') && !has_tone(&raw) && !raw.contains('~') && !raw.is_empty();
        if unfinished {
            for tone in ['6','3','4','7'] {
                if let Some(variants)=self.entries.get(&format!("{norm}{tone}")) {
                    for w in variants {
                        words.push(Word { text:w.text.clone(), score:w.score.saturating_sub(1500) });
                    }
                }
            }
        }
        for w in &mut words {
            w.score+=learned.get(&(raw.clone(),w.text.clone())).copied().unwrap_or(0)*900;
        }
        words.sort_by(|a,b|b.score.cmp(&a.score).then_with(||a.text.cmp(&b.text)));
        let mut seen=HashSet::new();
        words.retain(|w| seen.insert(w.text.clone()));
        words
    }
}
fn has_tone(s:&str)->bool { matches!(s.chars().last(),Some('6'|'3'|'4'|'7')) }
fn normal_key(s:&str)->String {
    let mut key=s.to_ascii_lowercase().replace('~',"");
    // Support neutral tone before the syllable, in addition to suffix form.
    if key.len()>1 && key.starts_with('7') && !key.ends_with('7') {
        key.remove(0); key.push('7');
    }
    key
}
fn segment_compact(raw:&str, dict:&Dictionary)->Vec<String> {
    if raw.is_empty() {return vec![];}
    if !raw.is_ascii() || raw.len()>MAX_INPUT {return vec![raw.to_owned()];}
    let n=raw.len();
    let mut best:Vec<Option<(i64,Vec<String>)>>=vec![None;n+1];
    best[n]=Some((0,vec![]));
    for i in (0..n).rev() {
        for j in i+1..=(i+8).min(n) {
            let code=&raw[i..j];
            if !dict.has_code(code) {continue;}
            if let Some((tail_score,tail))=&best[j] {
                let norm=normal_key(code);
                let score=*tail_score+(if has_tone(&norm){300}else{15})+8*(j-i) as i64-30;
                let mut parts=vec![norm];
                parts.extend(tail.iter().cloned());
                if best[i].as_ref().is_none_or(|old|score>old.0) {best[i]=Some((score,parts));}
            }
        }
    }
    if let Some((_,parts))=best[0].take() {return parts;}
    // An unfinished suffix remains visible and editable.
    for cut in (1..n).rev() {
        if let Some(prefix)=best_prefix(&raw[..cut],dict) {
            let mut out=prefix;out.push(raw[cut..].to_owned());return out;
        }
    }
    vec![raw.to_owned()]
}
fn best_prefix(raw:&str, dict:&Dictionary)->Option<Vec<String>> {
    let n=raw.len();
    if n==0 {return None;}
    let mut best:Vec<Option<Vec<String>>>=vec![None;n+1];
    best[0]=Some(vec![]);
    for i in 0..n {
        let Some(prefix)=best[i].clone() else {continue;};
        for j in i+1..=(i+8).min(n) {
            if dict.has_code(&raw[i..j]) {
                let mut next=prefix.clone();next.push(normal_key(&raw[i..j]));
                if best[j].as_ref().is_none_or(|old|next.len()<old.len()) {best[j]=Some(next);}
            }
        }
    }
    best[n].take()
}
fn segment(raw:&str,dict:&Dictionary)->Vec<String> {
    let mut out=vec![];
    let mut start=0;
    for (i,c) in raw.char_indices() {
        if c.is_ascii_whitespace() {
            if i>start {
                let mut parts=segment_compact(&raw[start..i],dict);
                if let Some(last)=parts.last_mut() {
                    if !has_tone(last) && dict.has_code(last) && !last.ends_with('~') {
                        last.push('~'); // Space terminates a first-tone syllable
                    }
                }
                out.extend(parts);
            }
            start=i+c.len_utf8();
        }
    }
    if start<raw.len() {out.extend(segment_compact(&raw[start..],dict));}
    out
}

struct Learning { db:Option<Connection>, counts:HashMap<(String,String),i64> }
impl Learning {
    fn new(dict:&mut Dictionary, portable_folder:Option<&Path>)->Self {
        let folder=portable_folder.map(Path::to_path_buf).unwrap_or_else(|| {
            let root=std::env::var_os("APPDATA").map(PathBuf::from)
                .or_else(||std::env::current_dir().ok()).unwrap_or_else(||PathBuf::from("."));
            root.join("LimeRS")
        });
        let _=std::fs::create_dir_all(&folder);
        let db=Connection::open(folder.join("learning.sqlite3")).ok();
        let mut counts=HashMap::new();
        if let Some(conn)=&db {
            let _=conn.execute_batch("CREATE TABLE IF NOT EXISTS choices (code TEXT NOT NULL, candidate TEXT NOT NULL, count INTEGER NOT NULL DEFAULT 0, PRIMARY KEY(code,candidate)); CREATE TABLE IF NOT EXISTS imported (code TEXT NOT NULL, candidate TEXT NOT NULL, weight INTEGER NOT NULL, PRIMARY KEY(code,candidate));");
            if let Ok(mut stmt)=conn.prepare("SELECT code,candidate,count FROM choices") {
                if let Ok(rows)=stmt.query_map([],|r|Ok(((r.get::<_,String>(0)?,r.get::<_,String>(1)?),r.get::<_,i64>(2)?))) {
                    for row in rows.flatten(){counts.insert(row.0,row.1);}
                }
            }
            if let Ok(mut stmt)=conn.prepare("SELECT code,candidate,weight FROM imported") {
                if let Ok(rows)=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?))) {
                    for (code,text,score) in rows.flatten(){dict.insert(&code,&text,score);}
                    dict.finalize();
                }
            }
        }
        Self{db,counts}
    }
    fn record(&mut self,code:&str,text:&str) {
        let key=(code.to_ascii_lowercase(),text.to_owned());
        *self.counts.entry(key.clone()).or_default()+=1;
        if let Some(db)=&self.db {
            let _=db.execute("INSERT INTO choices(code,candidate,count) VALUES (?1,?2,1) ON CONFLICT(code,candidate) DO UPDATE SET count=count+1",params![key.0,key.1]);
        }
    }
}

#[derive(Clone)]
struct SelectedWord {code:String,text:String,parts:usize}
#[derive(Clone)]
enum Choice {Whole,Raw,Local {parts:usize,word:String}}
#[derive(Clone)]
struct Candidate {label:String,kind:Choice}

pub struct Engine {
    raw:String,
    parts:Vec<String>,
    locked:Vec<SelectedWord>,
    locked_parts:usize,
    chosen:usize,
    uppercase:bool,
    pending:String,
    dict:Dictionary,
    learning:Learning,
}
impl Engine {
    fn new()->Self {Self::with_data_dir(None)}
    fn with_data_dir(portable_folder:Option<&Path>)->Self {
        let mut dict=Dictionary::new();
        let learning=Learning::new(&mut dict,portable_folder);
        // Optional portable user lexicon (tab-separated: keys, text, weight).
        if let Some(folder)=portable_folder {
            if let Ok(contents)=std::fs::read_to_string(folder.join("custom.tsv")) {
                for line in contents.lines() {
                    if line.trim().is_empty() || line.starts_with('#') {continue;}
                    let mut columns=line.split('\t');
                    if let (Some(code),Some(word),Some(score)) = (columns.next(),columns.next(),columns.next()) {
                        if let Ok(weight)=score.parse::<i64>() {dict.insert(code.trim(),word.trim(),weight);}
                    }
                }
                dict.finalize();
            }
        }
        Self {raw:String::new(),parts:vec![],locked:vec![],locked_parts:0,chosen:0,
            uppercase:false,pending:String::new(),dict,learning}
    }
    #[cfg(test)]
    fn for_tests()->Self {
        Self {raw:String::new(),parts:vec![],locked:vec![],locked_parts:0,chosen:0,
            uppercase:false,pending:String::new(),dict:Dictionary::new(),
            learning:Learning{db:None,counts:HashMap::new()}}
    }
    fn clear(&mut self){self.raw.clear();self.parts.clear();self.locked.clear();self.locked_parts=0;self.chosen=0;}
    fn recalc(&mut self){self.parts=segment(&self.raw.to_ascii_lowercase(),&self.dict);self.locked.clear();self.locked_parts=0;self.chosen=0;}
    fn input(&mut self,c:char) {
        if !c.is_ascii() || c.is_ascii_control() || self.raw.len()>=MAX_INPUT {return;}
        let c=if self.uppercase {c.to_ascii_uppercase()} else {c};
        self.raw.push(c);self.recalc();
    }
    fn backspace(&mut self) {
        if let Some(last)=self.locked.pop() {self.locked_parts=self.locked_parts.saturating_sub(last.parts);self.chosen=0;}
        else if !self.raw.is_empty() {self.raw.pop();self.recalc();}
    }
    fn remaining(&self)->&[String] {&self.parts[self.locked_parts.min(self.parts.len())..]}
    fn locked_text(&self)->String {self.locked.iter().map(|x|x.text.as_str()).collect()}
    fn raw_preview(&self)->String {
        if self.locked.is_empty(){return self.raw.clone();}
        let remain=self.remaining().join(" ");
        if remain.is_empty(){self.locked_text()} else {format!("{} {}",self.locked_text(),remain)}
    }
    fn sentence_segments(&self)->(String,Vec<SelectedWord>) {
        let remain=self.remaining();
        if remain.is_empty(){return (self.locked_text(),vec![]);}
        let n=remain.len();
        let mut dp:Vec<Option<(i64,Vec<SelectedWord>)>>=vec![None;n+1];
        dp[n]=Some((0,vec![]));
        for i in (0..n).rev() {
            let mut code=String::new();
            let mut selected:Option<(i64,Vec<SelectedWord>)>=None;
            for count in 1..=(n-i).min(6) {
                code.push_str(&remain[i+count-1]);
                let Some(word)=self.dict.lookup(&code,&self.learning.counts).first().cloned() else {continue;};
                if let Some((tail_score,tail))=&dp[i+count] {
                    let score=*tail_score+word.score+if count>1 {40000*((count-1) as i64)} else {0};
                    let mut result=vec![SelectedWord {code:code.clone(),text:word.text,parts:count}];
                    result.extend(tail.iter().cloned());
                    if selected.as_ref().is_none_or(|old|score>old.0){selected=Some((score,result));}
                }
            }
            if selected.is_none() {
                if let Some((tail_score,tail))=&dp[i+1] {
                    let mut result=vec![SelectedWord{code:remain[i].clone(),text:remain[i].clone(),parts:1}];
                    result.extend(tail.iter().cloned());
                    selected=Some((*tail_score-50000,result));
                }
            }
            dp[i]=selected;
        }
        let words=dp[0].take().map(|(_,v)|v).unwrap_or_default();
        let mut text=self.locked_text();
        for w in &words {text.push_str(&w.text);}
        (text,words)
    }
    fn candidates(&self)->Vec<Candidate> {
        if self.parts.is_empty(){return vec![];}
        let whole=self.sentence_segments().0;
        let mut result=vec![Candidate{label:whole,kind:Choice::Whole},Candidate{label:self.raw_preview(),kind:Choice::Raw}];
        if let Some(code)=self.remaining().first() {
            for w in self.dict.lookup(code,&self.learning.counts).into_iter().take(MAX_CANDIDATES) {
                if result.len()>=MAX_CANDIDATES {break;}
                result.push(Candidate{label:w.text.clone(),kind:Choice::Local{parts:1,word:w.text}});
            }
        }
        let remain=self.remaining();
        let mut combined=remain.first().cloned().unwrap_or_default();
        for length in 2..=remain.len().min(4) {
            combined.push_str(&remain[length-1]);
            for w in self.dict.lookup(&combined,&self.learning.counts).into_iter().take(3) {
                if result.len()>=MAX_CANDIDATES {break;}
                result.push(Candidate{label:w.text.clone(),kind:Choice::Local{parts:length,word:w.text}});
            }
        }
        result
    }
    fn choose(&mut self,index:usize){
        let Some(candidate)=self.candidates().get(index).cloned() else {return;};
        self.chosen=0;
        match candidate.kind {
            Choice::Whole=>{
                let (whole,words)=self.sentence_segments();
                for w in words {if w.code!=w.text {self.learning.record(&w.code,&w.text);}}
                self.pending=whole;self.clear();
            },
            Choice::Raw=>{self.pending=self.raw_preview();self.clear();},
            Choice::Local{parts,word}=>{
                let remain=self.remaining();
                if remain.len()<parts || parts==0 {return;}
                let code=remain[..parts].join("");
                self.learning.record(&code,&word);
                self.locked.push(SelectedWord{code,text:word,parts});
                self.locked_parts+=parts;
                if self.remaining().is_empty(){self.pending=self.locked_text();self.clear();}
            }
        }
    }
    fn navigate(&mut self,delta:i32){
        let size=self.candidates().len();
        if size>0 {
            self.chosen=((self.chosen as i64+delta as i64).rem_euclid(size as i64)) as usize;
        }
    }
    fn navigate_edge(&mut self, last:bool){
        let count=self.candidates().len();
        if count>0 {self.chosen=if last {count-1} else {0};}
    }
}

// All external strings are UTF-16. A returned length excludes the terminator.
fn write_wide(text:&str,dest:*mut u16,cap:usize)->usize {
    if dest.is_null() || cap==0 {return 0;}
    let units:Vec<u16>=text.encode_utf16().collect();
    let len=units.len().min(cap-1);
    unsafe {std::ptr::copy_nonoverlapping(units.as_ptr(),dest,len);*dest.add(len)=0;}
    len
}
#[no_mangle] pub extern "C" fn lrs_new()->*mut Engine {Box::into_raw(Box::new(Engine::new()))}
/// Creates an engine with its SQLite files in a UTF-16 directory supplied by the TSF DLL.
/// When running from a portable folder this is <DLL folder>\data, not APPDATA.
/// SAFETY: `dir` must be a valid NUL-terminated UTF-16 pointer from the caller.
#[no_mangle] pub unsafe extern "C" fn lrs_new_in_dir(dir:*const u16)->*mut Engine {
    if dir.is_null() {return lrs_new();}
    let mut len=0usize;
    while len<4096 && *dir.add(len)!=0 {len+=1;}
    if len>=4096 {return std::ptr::null_mut();}
    let folder=String::from_utf16_lossy(std::slice::from_raw_parts(dir,len));
    if folder.is_empty() {return std::ptr::null_mut();}
    Box::into_raw(Box::new(Engine::with_data_dir(Some(Path::new(&folder)))))
}

#[no_mangle] pub unsafe extern "C" fn lrs_free(ptr:*mut Engine) {if !ptr.is_null(){drop(Box::from_raw(ptr));}}
#[no_mangle] pub unsafe extern "C" fn lrs_input(ptr:*mut Engine,ch:u32){if let Some(e)=ptr.as_mut(){if let Some(c)=char::from_u32(ch){e.input(c);}}}
#[no_mangle] pub unsafe extern "C" fn lrs_backspace(ptr:*mut Engine){if let Some(e)=ptr.as_mut(){e.backspace();}}
#[no_mangle] pub unsafe extern "C" fn lrs_cancel(ptr:*mut Engine){if let Some(e)=ptr.as_mut(){e.clear();}}
#[no_mangle] pub unsafe extern "C" fn lrs_toggle_case(ptr:*mut Engine){if let Some(e)=ptr.as_mut(){e.uppercase=!e.uppercase;}}
#[no_mangle] pub unsafe extern "C" fn lrs_choose(ptr:*mut Engine,index:usize){if let Some(e)=ptr.as_mut(){e.choose(index);}}
#[no_mangle] pub unsafe extern "C" fn lrs_move(ptr:*mut Engine,delta:i32){if let Some(e)=ptr.as_mut(){e.navigate(delta);}}
#[no_mangle] pub unsafe extern "C" fn lrs_has_input(ptr:*const Engine)->bool{ptr.as_ref().is_some_and(|e|!e.raw.is_empty())}
#[no_mangle] pub unsafe extern "C" fn lrs_count(ptr:*const Engine)->usize{ptr.as_ref().map_or(0,|e|e.candidates().len())}
#[no_mangle] pub unsafe extern "C" fn lrs_selected(ptr:*const Engine)->usize{ptr.as_ref().map_or(0,|e|e.chosen)}
#[no_mangle] pub unsafe extern "C" fn lrs_preedit(ptr:*const Engine,out:*mut u16,cap:usize)->usize{
    ptr.as_ref().map_or(0,|e|write_wide(&e.raw_preview(),out,cap))
}
#[no_mangle] pub unsafe extern "C" fn lrs_candidate(ptr:*const Engine,index:usize,out:*mut u16,cap:usize)->usize{
    ptr.as_ref().and_then(|e|e.candidates().get(index).map(|x|x.label.clone()))
        .map_or(0,|word|write_wide(&word,out,cap))
}
#[no_mangle] pub unsafe extern "C" fn lrs_take_commit(ptr:*mut Engine,out:*mut u16,cap:usize)->usize{
    let Some(e)=ptr.as_mut() else {return 0;};
    if e.pending.is_empty(){return 0;}
    // Don't consume the result when the caller's buffer is too small.
    if e.pending.encode_utf16().count()+1>cap {return 0;}
    let written=write_wide(&e.pending,out,cap);
    e.pending.clear();
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn tones_are_distinct(){
        let d=Dictionary::new();
        let counts=HashMap::new();
        for (code,expect) in [("a86","麻"),("a83","馬"),("a84","罵"),("a87","嗎")] {
            assert!(d.lookup(code,&counts).iter().any(|w|w.text==expect),"{code}: {expect}");
        }
    }
    #[test] fn choices_preserve_whole_raw_order(){
        let mut e=Engine::for_tests();
        for c in "dj94xk4".chars(){e.input(c);}
        let c=e.candidates();
        assert_eq!(c[0].label,"快樂");
        assert_eq!(c[1].label,"dj94xk4");
        e.navigate(1);assert_eq!(e.chosen,1);
        e.choose(e.chosen);assert_eq!(e.pending,"dj94xk4");
    }
    #[test] fn extra_unknown_remains_raw(){
        let mut e=Engine::for_tests();
        for c in "NXVET".chars(){e.input(c);}
        assert_eq!(e.candidates()[1].label,"NXVET");
    }
}

/// Native portable front-end API; built into LimeRS.exe (no COM, TSF or DLL needed).
pub struct PortableEngine { inner: Engine }
impl PortableEngine {
    pub fn new(folder:&Path)->Self { Self {inner:Engine::with_data_dir(Some(folder))} }
    pub fn push(&mut self, c:char) { self.inner.input(c); }
    pub fn backspace(&mut self) { self.inner.backspace(); }
    pub fn cancel(&mut self) { self.inner.clear(); }
    pub fn toggle_case(&mut self) { self.inner.uppercase = !self.inner.uppercase; }
    pub fn uppercase(&self)->bool { self.inner.uppercase }
    pub fn has_input(&self)->bool { !self.inner.raw.is_empty() }
    pub fn preedit(&self)->String { self.inner.raw_preview() }
    pub fn candidates(&self)->Vec<String> {self.inner.candidates().into_iter().map(|c|c.label).collect()}
    pub fn selected(&self)->usize {self.inner.chosen}
    pub fn navigate(&mut self,delta:i32) {self.inner.navigate(delta);}
    pub fn navigate_edge(&mut self,last:bool) {self.inner.navigate_edge(last);}
    /// Returns None when the choice locks one segment and composition should continue.
    pub fn choose(&mut self, index:usize)->Option<String> {
        self.inner.choose(index);
        if self.inner.pending.is_empty() {None} else {Some(std::mem::take(&mut self.inner.pending))}
    }
    pub fn choose_selected(&mut self)->Option<String> {self.choose(self.inner.chosen)}
}
#[cfg(test)]
mod expanded_dictionary_tests {
    use super::*;
    #[test]
    fn zhi_first_tone_contains_traditional_zhi_near_top() {
        let dict = Dictionary::new();
        let words = dict.lookup("5~", &HashMap::new());
        let pos = words.iter().position(|w| w.text == "隻").expect("隻 must be in ㄓ first tone");
        assert!(pos < 8, "隻 should be readily available, got position {pos}");
    }
    #[test]
    fn low_frequency_homophones_are_not_truncated_to_24() {
        let mut engine = Engine::for_tests();
        engine.input('u');
        engine.input('4');
        // This pronunciation has hundreds of readings in the bundled source.
        assert!(engine.candidates().len()>24, "candidate list is still artificially truncated");
        engine.navigate(35);
        assert_eq!(engine.chosen,35);
        engine.navigate_edge(false);
        assert_eq!(engine.chosen,0);
        engine.navigate_edge(true);
        assert_eq!(engine.chosen,engine.candidates().len()-1);
    }
    #[test]
    fn default_whole_sentence_still_works() {
        let mut engine=Engine::for_tests();
        for ch in "dj94xk4".chars() {engine.input(ch);}
        assert_eq!(engine.candidates()[0].label,"快樂");
        assert_eq!(engine.candidates()[1].label,"dj94xk4");
    }
}

#[cfg(test)]
mod portable_tests {
    use super::*;
    #[test]
    fn portable_sequence_keeps_candidate_order() {
        let mut p=PortableEngine {inner:Engine::for_tests()};
        for c in "dj94xk4".chars() {p.push(c);}
        assert_eq!(p.candidates()[0],"快樂");
        assert_eq!(p.candidates()[1],"dj94xk4");
        assert_eq!(p.choose_selected().as_deref(),Some("快樂"));
        assert!(!p.has_input());
    }
    #[test]
    fn portable_navigation_home_end_matches_visible_choices() {
        let mut p=PortableEngine {inner:Engine::for_tests()};
        for ch in "dj94xk4".chars() {p.push(ch);}
        assert!(p.candidates().len() >= 3);
        p.navigate_edge(true);
        assert_eq!(p.selected(),p.candidates().len()-1);
        p.navigate_edge(false);
        assert_eq!(p.selected(),0);
        p.navigate(1);
        assert_eq!(p.selected(),1);
    }
    #[test]
    fn shift_case_changes_raw_not_zhuyin_decoding() {
        let mut p=PortableEngine {inner:Engine::for_tests()};
        p.toggle_case();
        for c in "su3".chars(){p.push(c);}
        assert_eq!(p.candidates()[1],"SU3");
        assert!(p.candidates().iter().any(|v|v=="你"));
    }
}
