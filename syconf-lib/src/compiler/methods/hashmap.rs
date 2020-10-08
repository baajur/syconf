use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::{Error, Value};
use crate::compiler::context::Context;
use crate::compiler::value_extraction::ValueExtractor;
use crate::parse_string;

pub fn method(name: &str) -> Option<&'static dyn Fn(&HashMap<String, Value>, &[Value]) -> Result<Value, Error>> {
    Some(match name {
        "map" => &map,
        "filter" => &filter,
        "len" => &len,
        "insert" => &insert,
        _ => return None
    })
}

fn map(hm: &HashMap<String, Value>, args: &[Value]) -> Result<Value, Error> {
    let extractor = ValueExtractor::new(args, 1)?;
    let func = extractor.extract_func(0)?;

    let mut new_hm = HashMap::new();
    for (k, v) in hm {
        let v = func.call(&[Value::String(Rc::new(k.clone())), v.clone()])?;
        match v {
            Value::List(list) => {
                let ex = ValueExtractor::new(list.as_slice(), 2)?;
                new_hm.insert(ex.extract_string(0)?.to_string(), list[1].clone());
            }
            _ => bail!("hashmap map function must return a list of 2 values"),
        }
    }
    Ok(Value::HashMap(Rc::new(new_hm)))
}

#[test]
fn func_map() {
    assert_eq!(parse_string(r#"
        {aa:3, bb:4}.map((k,v) => [k, v * 10]) == {bb: 40, aa: 30}
    "#).unwrap(), Value::Bool(true))
}


fn filter(hm: &HashMap<String, Value>, args: &[Value]) -> Result<Value, Error> {
    let func = ValueExtractor::new(args, 1)?.extract_func(0)?;
    let mut filtered = HashMap::with_capacity(hm.len());
    for (ix, val) in hm {
        let out = func.call(&[Value::String(Rc::new(ix.clone())), val.clone()])?.as_bool()?;
        if out {
            filtered.insert(ix.clone(), val.clone());
        }
    }
    Ok(Value::HashMap(Rc::new(filtered)))
}

#[test]
fn func_filter() {
    assert_eq!(parse_string(r#"
        {aa:3, bb:4}.filter((k,v) => k == "bb") == {bb: 4}
    "#).unwrap(), Value::Bool(true))
}

fn len(hm: &HashMap<String, Value>, args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 0, "expects no arguments");
    Ok(Value::Int(hm.len() as i32))
}

#[test]
fn func_len() {
    assert_eq!(parse_string(r#"
        {aa:3, bb:4}.len()
    "#).unwrap(), Value::Int(2))
}

fn insert(hm: &HashMap<String, Value>, args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 2, "expects 2 arguments");
    let mut out = hm.clone();
    out.insert(args[0].as_str()?.to_string(), args[1].clone());
    Ok(Value::HashMap(Rc::new(out)))
}

#[test]
fn func_insert() {
    assert_eq!(parse_string(r#"
        {aa: 33}.insert("bb", "abc") == {aa:33, bb:"abc"}
    "#).unwrap(), Value::Bool(true))
}