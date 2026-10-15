//! Common Traditional Chinese punctuation panel. No Windows API dependency.
//! Keep the panel separate from Chinese composition so Esc can return to the
//! original unfinished phrase without destroying what the user typed.

const SYMBOLS: &[&str] = &[
    "，", "。", "、", "？", "！", "：", "；", "「", "」",
    "『", "』", "（", "）", "《", "》", "〈", "〉", "…",
    "—", "～", "．", "·", "／", "＋", "－", "＝", "％",
    "【", "】", "〔", "〕", "［", "］", "｛", "｝", "｜",
    "＠", "＃", "＆", "＊", "＄", "＾", "＼", "＿", "﹏",
];

#[derive(Default)]
pub struct PunctuationPicker { opened: bool, selected: usize }
impl PunctuationPicker {
    pub fn open(&mut self) { self.opened=true; self.selected=0; }
    pub fn close(&mut self) { self.opened=false; self.selected=0; }
    pub fn is_open(&self)->bool { self.opened }
    pub fn selected(&self)->usize { self.selected }
    pub fn len(&self)->usize { SYMBOLS.len() }
    pub fn labels(&self)->Vec<String> { SYMBOLS.iter().map(|s|(*s).to_owned()).collect() }
    pub fn navigate(&mut self,delta:i32) {
        if self.opened && !SYMBOLS.is_empty() {
            self.selected=(self.selected as i64 + delta as i64).rem_euclid(SYMBOLS.len() as i64) as usize;
        }
    }
    pub fn navigate_edge(&mut self,last:bool) {
        if self.opened {self.selected=if last {SYMBOLS.len()-1} else {0};}
    }
    /// Returns the selected text and closes the panel. Invalid indexes keep it open.
    pub fn pick(&mut self,index:usize)->Option<&'static str> {
        if !self.opened {return None;}
        let symbol=*SYMBOLS.get(index)?;
        self.close();
        Some(symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn starts_on_comma_and_has_standard_punctuation() {
        let mut p=PunctuationPicker::default();p.open();
        assert_eq!(p.selected(),0);
        assert_eq!(p.labels()[0],"，");
        for mark in ["。","、","？","！","「","」","《","》","…","—"] {
            assert!(p.labels().iter().any(|s|s==mark),"missing {mark}");
        }
    }
    #[test] fn navigates_by_row_and_page() {
        let mut p=PunctuationPicker::default();p.open();
        p.navigate(1);assert_eq!(p.selected(),1);
        p.navigate(9);assert_eq!(p.selected(),10);
        p.navigate(-9);assert_eq!(p.selected(),1);
        p.navigate_edge(true);assert_eq!(p.selected(),p.len()-1);
        p.navigate_edge(false);assert_eq!(p.selected(),0);
    }
    #[test] fn submit_closes_without_touching_external_composition() {
        let mut p=PunctuationPicker::default();p.open();
        assert_eq!(p.pick(2),Some("、"));
        assert!(!p.is_open());
        assert_eq!(p.pick(0),None);
    }
    #[test] fn invalid_index_does_not_close() {
        let mut p=PunctuationPicker::default();p.open();
        assert_eq!(p.pick(999),None);
        assert!(p.is_open());
        p.close();assert!(!p.is_open());
    }
}
