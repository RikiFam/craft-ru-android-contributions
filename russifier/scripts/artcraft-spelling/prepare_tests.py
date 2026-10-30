"""Install regression tests beside the real native spelling entry points."""
from prepare import ROOT

TESTS={
 'wordcraft':('proof','ru/mod.rs',r'''
    #[test]
    fn bundled_russian_spell_regression() {
        let d = super::AffixDict::from_hunspell(include_str!("DICT/ru_RU.aff"), include_str!("DICT/ru_RU.dic"), &[]).unwrap();
        assert!(super::is_correct_with(&d,"документами"));
        assert!(!super::is_correct_with(&d,"превет"));
    }
 '''),
 'deckcraft':('engine','spell.rs',r'''
    #[test]
    fn bundled_russian_spell_regression() {
        let d=super::dictionary("ru-RU").unwrap();
        assert!(d.check("документами")); assert!(!d.check("превет"));
        assert!(d.suggest("превет",6).contains(&"привет".to_owned()));
        let p=deckcraft_model::Paragraph::new("превет hello");
        let words=super::paragraph_words(&p,"en-US");
        assert_eq!(words[0].lang,"ru-RU"); assert_eq!(words[1].lang,"en-US");
    }
 '''),
 'gridcraft':('engine','cmd/spelling.rs',r'''
    #[test]
    fn bundled_russian_spell_regression() {
        let mut s=crate::Session::new();
        s.execute("file.new",serde_json::json!({})).unwrap();
        s.execute("cell.set",serde_json::json!({"cell":"A1","input":"Документами привет превет"})).unwrap();
        let result=s.execute("review.spelling",serde_json::json!({})).unwrap();
        assert_eq!(result["issues"].as_array().unwrap().len(),1);
        assert_eq!(result["issues"][0]["word"],"превет");
        s.execute("review.changeSpelling",serde_json::json!({"cell":"A1","word":"превет","to":"привет"})).unwrap();
        assert_eq!(s.execute("review.spelling",serde_json::json!({})).unwrap()["done"],true);
    }
 '''),
 'designcraft':('engine','cmd/spelling.rs',r'''
    #[test]
    fn bundled_russian_spell_regression() {
        let mut s=crate::Session::new();
        s.execute("file.new",&serde_json::json!({})).unwrap();
        s.execute("frame.create",&serde_json::json!({"rect":[36,36,300,200],"content":"text","text":"Документами привет превет"})).unwrap();
        let r=s.execute("spelling.check",&serde_json::json!({})).unwrap();
        assert_eq!(r.as_array().unwrap().len(),1); assert_eq!(r[0]["word"],"превет");
        s.execute("spelling.change",&serde_json::json!({"story":r[0]["story"],"start":r[0]["start"],"end":r[0]["end"],"to":"привет"})).unwrap();
        assert!(s.execute("spelling.check",&serde_json::json!({})).unwrap().as_array().unwrap().is_empty());
    }
 '''),
 'photocraft':('text','spell.rs',r'''
    #[test]
    fn bundled_russian_spell_regression() {
        let d=super::Dictionary::english(); let user=std::collections::HashSet::new();
        assert!(d.check("документами",&user)); assert!(!d.check("превет",&user));
        assert!(d.suggest("превет",6).contains(&"привет".to_owned()));
        assert!(d.check("превет",&std::collections::HashSet::from(["превет".to_owned()])));
    }
 '''),
}

def main():
 for app,(crate,file,test) in TESTS.items():
  p=ROOT/'tmp/artcraft'/app/'repo/crates'/crate/'src'/file
  text=p.read_text('utf-8')
  marker='\n#[cfg(test)]\nmod bundled_russian_regression {'
  text=text.split(marker)[0]
  test=test.replace('DICT',(ROOT/'scripts/artcraft-russian/dictionaries').as_posix())
  p.write_text(text+marker+'\n'+test+'\n}\n',encoding='utf-8')
if __name__=='__main__':main()
