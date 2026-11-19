// Generated model-specific JSON codecs. Schema declarations are compiled out.
#![allow(unused_mut)]
use super::models::*;
use super::contracts::{ContractError, ContractResult, ErrorReason};
use serde_json::{Value, Map};
fn failure(reason: ErrorReason, id: &str, member: &str, location: &str, detail: impl ToString)->Box<ContractError>{
 Box::new(ContractError{reason,model_id:Some(id.into()),attribute:(!member.is_empty()).then(||member.into()),format:Some("json".into()),rule:None,location:location.into(),klass:None,detail:Some(detail.to_string())})
}
fn object(value:Value,id:&str)->ContractResult<Map<String,Value>> { value.as_object().cloned().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/input","expected object")) }
#[allow(dead_code)]
fn empty(v:&Value)->bool { match v { Value::String(v)=>v.is_empty(),Value::Array(v)=>v.is_empty(),Value::Object(v)=>v.is_empty(),_=>false } }
fn numeric_json_err()->&'static str{"numeric JSON is not a portable exact boundary: the portable-exact-json-v1 profile requires exact values as strings; this looks like ruby-native-v1 numeric JSON (Ruby to_json), which is not accepted"}
// Only a plain 64-bit integer (ordered-content indexes) may be a JSON number; any
// fractional, exponent, negative-zero or oversized spelling is not a portable exact boundary.
fn plain_integer(n:&str)->Result<Value,&'static str>{
 let digits=n.strip_prefix('-').unwrap_or(n);
 let ok=!digits.is_empty()&&digits.bytes().all(|c|c.is_ascii_digit())&&n!="-0"&&(n.parse::<i64>().is_ok()||n.parse::<u64>().is_ok());
 if ok{serde_json::from_str::<Value>(n).map_err(|_|numeric_json_err())}else{Err(numeric_json_err())}
}
// RFC 8259 number grammar, nothing else (no sign prefix, spaces, quotes or leading zeros).
#[allow(dead_code)]
fn json_number(s:&str)->bool{
 let b=s.as_bytes();let mut i=usize::from(b.first()==Some(&b'-'));
 let digits=|i:&mut usize|{let at=*i;while *i<b.len()&&b[*i].is_ascii_digit(){*i+=1;}*i-at};
 let whole=i;let n=digits(&mut i);
 if n==0||(n>1&&b[whole]==b'0'){return false;}
 if i<b.len()&&b[i]==b'.'{i+=1;if digits(&mut i)==0{return false;}}
 if i<b.len()&&(b[i]==b'e'||b[i]==b'E'){i+=1;
  if i<b.len()&&(b[i]==b'+'||b[i]==b'-'){i+=1;}
  if digits(&mut i)==0{return false;}}
 i==b.len()
}
// Numbers in a keyed-collection body that no member mapping extracts would be dropped
// unseen; refuse them like every other loose number (`keep` lists the extracted paths).
#[allow(dead_code)]
fn no_loose_except(v:&Value,keep:&[&[&str]],id:&str)->ContractResult<()>{
 if keep.iter().any(|p|p.is_empty()){return Ok(());}
 match v {
  Value::Object(o)=>o.iter().try_for_each(|(k,x)|{let sub:Vec<&[&str]>=keep.iter().filter(|p|p[0]==k.as_str()).map(|p|&p[1..]).collect();no_loose_except(x,&sub,id)}),
  other=>no_loose(other,id),
 }
}
// Bundles with a `hash` attribute parse numbers loosely; every other position must
// still refuse them, so the decoded model is walked once (`strict_*`).
#[allow(dead_code)]
fn no_loose(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>plain_integer(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e)),
  Value::Array(a)=>a.iter().try_for_each(|x|no_loose(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|no_loose(x,id)),
  _=>Ok(()),
 }
}
// Inside a hash a number must be exact (the same rule the TS strict reader applies).
#[allow(dead_code)]
fn neutral_numbers(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>NeutralNumber::parse(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",format!("{e}: {n}"))),
  Value::Array(a)=>a.iter().try_for_each(|x|neutral_numbers(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|neutral_numbers(x,id)),
  _=>Ok(()),
 }
}
#[allow(dead_code)]
fn strict_each(v:&Value,f:&dyn std::ops::Fn(&Value)->ContractResult<()>)->ContractResult<()>{ match v { Value::Array(a)=>a.iter().try_for_each(f), other=>f(other) } }
// A hash keeps its key order only when serde_json keeps insertion order.
#[allow(dead_code)]
fn ordered_maps(id:&str)->ContractResult<()>{
 let mut m=Map::new(); m.insert("b".into(),Value::Null); m.insert("a".into(),Value::Null);
 if m.keys().next().map(String::as_str)==Some("b"){Ok(())}else{Err(failure(ErrorReason::InvalidDocument,id,"","/input","hash attributes need serde_json with the preserve_order feature"))}
}
fn parse_json(input:&str,id:&str)->ContractResult<Value>{
 // Reject duplicate keys before mapping aliases. serde_json::Value alone would
 // silently keep only the last key, erasing an observable input ambiguity.
 struct Unique(Value);
 impl<'de> serde::Deserialize<'de> for Unique {
  fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
   struct Visitor;
   impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value=Unique;
    fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("JSON without duplicate keys")}
    fn visit_bool<E:serde::de::Error>(self,v:bool)->Result<Unique,E>{Ok(Unique(Value::Bool(v)))}
    // serde_json reads "-0" as the integer 0, so a signed zero is -0 spelled as an integer; keep it as a float
    // that no integer, string or hash position accepts (negative zero is not exact).
    fn visit_i64<E:serde::de::Error>(self,v:i64)->Result<Unique,E>{if NEUTRAL_NUMBERS&&v==0{return Ok(Unique(serde_json::Number::from_f64(-0.0).map_or(Value::Null,Value::Number)));}Ok(Unique(v.into()))}
    fn visit_u64<E:serde::de::Error>(self,v:u64)->Result<Unique,E>{Ok(Unique(v.into()))}
    fn visit_f64<E:serde::de::Error>(self,_:f64)->Result<Unique,E>{Err(E::custom(numeric_json_err()))}
    fn visit_str<E:serde::de::Error>(self,v:&str)->Result<Unique,E>{Ok(Unique(Value::String(v.into())))}
    fn visit_string<E:serde::de::Error>(self,v:String)->Result<Unique,E>{Ok(Unique(Value::String(v)))}
    fn visit_unit<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_none<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{let mut v=Vec::new();while let Some(Unique(x))=a.next_element()?{v.push(x);}Ok(Unique(Value::Array(v)))}
    fn visit_map<A:serde::de::MapAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{
     let mut v=Map::new();let mut first=true;
     while let Some(k)=a.next_key::<String>()?{
      // With serde_json arbitrary_precision (always on for consumers of this crate)
      // every JSON number arrives as a one-entry map, never through visit_f64.
      // The marker's value must be a lone string in the strict JSON number grammar: nothing
      // else may travel through it (no quoted text, objects, spaces or sibling keys).
      if first && k=="$serde_json::private::Number"{let n:String=a.next_value()?;if a.next_key::<String>()?.is_some(){return Err(serde::de::Error::custom("not a JSON number"));}
      if NEUTRAL_NUMBERS{if !json_number(&n){return Err(serde::de::Error::custom("not a JSON number"));}return serde_json::from_str::<Value>(&n).map(Unique).map_err(serde::de::Error::custom);}
      return plain_integer(&n).map(Unique).map_err(serde::de::Error::custom);}
      first=false;
      let Unique(x)=a.next_value()?;
      if v.insert(k,x).is_some(){return Err(serde::de::Error::custom("duplicate JSON key"));}
     }
     Ok(Unique(Value::Object(v)))
    }
   }
   d.deserialize_any(Visitor)
  }
 }
 serde_json::from_str::<Unique>(input).map(|v|v.0).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e))
}
#[allow(dead_code)]
fn get_path(v:&Value,path:&[&str])->Option<Value>{let mut at=v;for part in path{at=at.as_object()?.get(*part)?;}Some(at.clone())}
#[allow(dead_code)]
fn put_path(v:&mut Value,path:&[&str],value:Value,id:&str)->ContractResult<()>{
 if path.is_empty(){if !v.is_null(){return Err(failure(ErrorReason::UnsupportedMapping,id,"","/output","overlapping value mappings"));}*v=value;return Ok(());}
 if v.is_null(){*v=Value::Object(Map::new());}
 let map=v.as_object_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/output","overlapping mapping paths"))?;
 let slot=map.entry(path[0]).or_insert(Value::Null);put_path(slot,&path[1..],value,id)
}
const NEUTRAL_NUMBERS: bool = false;
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_Document_json(input:&str)->ContractResult<Asciiunits_Model_Document>{let data=decode_Asciiunits_Model_Document(parse_json(input,"Asciiunits::Model::Document@default")?)?; let model:Asciiunits_Model_Document=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Document@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_Document_json(model:&Asciiunits_Model_Document)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Document@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_Document(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Document@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_Document(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::Document@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Document@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Document@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_Document(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_Document(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_Document(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::Document@default")?;let mut wire=Map::new();
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_Expression_json(input:&str)->ContractResult<Asciiunits_Model_Expression>{let data=decode_Asciiunits_Model_Expression(parse_json(input,"Asciiunits::Model::Expression@default")?)?; let model:Asciiunits_Model_Expression=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_Expression_json(model:&Asciiunits_Model_Expression)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_Expression(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_Expression(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::Expression@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("close_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "close_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("close_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dimension") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "dimension", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dimension".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("extender") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "extender", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("extender".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_int_exp_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "first_int_exp_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_int_exp_set".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "first_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_set".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hyphen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "hyphen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hyphen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("implicit_extended") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "implicit_extended", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("implicit_extended".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("int_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "int_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("int_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("integer") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "integer", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("integer".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("open_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "open_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("open_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_numerator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "power_numerator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_numerator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("prefix".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_int_exp_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "second_int_exp_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_int_exp_set".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "second_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_set".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sqrt") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "sqrt", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sqrt".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unit") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "unit", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unit".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Expression@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_Expression(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_Expression(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("close_paren"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("close_paren".into(),v);}
if let Some(mut v)=data.remove("dimension"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("dimension".into(),v);}
if let Some(mut v)=data.remove("extender"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("extender".into(),v);}
if let Some(mut v)=data.remove("first_int_exp_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_FirstIntExpSet(v)?;
}
data.insert("first_int_exp_set".into(),v);}
if let Some(mut v)=data.remove("first_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_FirstSet(v)?;
}
data.insert("first_set".into(),v);}
if let Some(mut v)=data.remove("hyphen"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("hyphen".into(),v);}
if let Some(mut v)=data.remove("implicit_extended"){
if !v.is_null() {
v=decode_Asciiunits_Model_ImplicitExtended(v)?;
}
data.insert("implicit_extended".into(),v);}
if let Some(mut v)=data.remove("int_exp"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("int_exp".into(),v);}
if let Some(mut v)=data.remove("integer"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("integer".into(),v);}
if let Some(mut v)=data.remove("open_paren"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("open_paren".into(),v);}
if let Some(mut v)=data.remove("power_numerator"){
if !v.is_null() {
v=decode_Asciiunits_Model_PowerNumerator(v)?;
}
data.insert("power_numerator".into(),v);}
if let Some(mut v)=data.remove("prefix"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("prefix".into(),v);}
if let Some(mut v)=data.remove("second_int_exp_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_SecondIntExpSet(v)?;
}
data.insert("second_int_exp_set".into(),v);}
if let Some(mut v)=data.remove("second_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_SecondSet(v)?;
}
data.insert("second_set".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("sqrt"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sqrt".into(),v);}
if let Some(mut v)=data.remove("unit"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("unit".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_Expression(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::Expression@default")?;let mut wire=Map::new();
for mut v in data.remove("close_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("close_paren".into(),v);}
for mut v in data.remove("dimension").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("dimension".into(),v);}
for mut v in data.remove("extender").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("extender".into(),v);}
for mut v in data.remove("first_int_exp_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_FirstIntExpSet(v)?;
}
wire.insert("first_int_exp_set".into(),v);}
for mut v in data.remove("first_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_FirstSet(v)?;
}
wire.insert("first_set".into(),v);}
for mut v in data.remove("hyphen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("hyphen".into(),v);}
for mut v in data.remove("implicit_extended").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_ImplicitExtended(v)?;
}
wire.insert("implicit_extended".into(),v);}
for mut v in data.remove("int_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("int_exp".into(),v);}
for mut v in data.remove("integer").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("integer".into(),v);}
for mut v in data.remove("open_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("open_paren".into(),v);}
for mut v in data.remove("power_numerator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_PowerNumerator(v)?;
}
wire.insert("power_numerator".into(),v);}
for mut v in data.remove("prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("prefix".into(),v);}
for mut v in data.remove("second_int_exp_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_SecondIntExpSet(v)?;
}
wire.insert("second_int_exp_set".into(),v);}
for mut v in data.remove("second_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_SecondSet(v)?;
}
wire.insert("second_set".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("sqrt").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sqrt".into(),v);}
for mut v in data.remove("unit").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("unit".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_FirstIntExpSet_json(input:&str)->ContractResult<Asciiunits_Model_FirstIntExpSet>{let data=decode_Asciiunits_Model_FirstIntExpSet(parse_json(input,"Asciiunits::Model::FirstIntExpSet@default")?)?; let model:Asciiunits_Model_FirstIntExpSet=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstIntExpSet@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_FirstIntExpSet_json(model:&Asciiunits_Model_FirstIntExpSet)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstIntExpSet@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_FirstIntExpSet(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstIntExpSet@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_FirstIntExpSet(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::FirstIntExpSet@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("extender") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstIntExpSet@default", "extender", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("extender".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("int_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstIntExpSet@default", "int_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("int_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstIntExpSet@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstIntExpSet@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_FirstIntExpSet(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_FirstIntExpSet(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("extender"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("extender".into(),v);}
if let Some(mut v)=data.remove("int_exp"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("int_exp".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sequence".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_FirstIntExpSet(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::FirstIntExpSet@default")?;let mut wire=Map::new();
for mut v in data.remove("extender").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("extender".into(),v);}
for mut v in data.remove("int_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("int_exp".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sequence".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_FirstSet_json(input:&str)->ContractResult<Asciiunits_Model_FirstSet>{let data=decode_Asciiunits_Model_FirstSet(parse_json(input,"Asciiunits::Model::FirstSet@default")?)?; let model:Asciiunits_Model_FirstSet=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_FirstSet_json(model:&Asciiunits_Model_FirstSet)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_FirstSet(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_FirstSet(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::FirstSet@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("dimension") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "dimension", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dimension".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("extender") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "extender", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("extender".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_numerator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "power_numerator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_numerator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("prefix".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sqrt") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "sqrt", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sqrt".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unit") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "unit", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unit".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::FirstSet@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_FirstSet(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_FirstSet(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("dimension"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("dimension".into(),v);}
if let Some(mut v)=data.remove("extender"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("extender".into(),v);}
if let Some(mut v)=data.remove("power_numerator"){
if !v.is_null() {
v=decode_Asciiunits_Model_PowerNumerator(v)?;
}
data.insert("power_numerator".into(),v);}
if let Some(mut v)=data.remove("prefix"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("prefix".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("sqrt"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sqrt".into(),v);}
if let Some(mut v)=data.remove("unit"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("unit".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_FirstSet(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::FirstSet@default")?;let mut wire=Map::new();
for mut v in data.remove("dimension").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("dimension".into(),v);}
for mut v in data.remove("extender").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("extender".into(),v);}
for mut v in data.remove("power_numerator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_PowerNumerator(v)?;
}
wire.insert("power_numerator".into(),v);}
for mut v in data.remove("prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("prefix".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("sqrt").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sqrt".into(),v);}
for mut v in data.remove("unit").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("unit".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_ImplicitExtended_json(input:&str)->ContractResult<Asciiunits_Model_ImplicitExtended>{let data=decode_Asciiunits_Model_ImplicitExtended(parse_json(input,"Asciiunits::Model::ImplicitExtended@default")?)?; let model:Asciiunits_Model_ImplicitExtended=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_ImplicitExtended_json(model:&Asciiunits_Model_ImplicitExtended)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_ImplicitExtended(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_ImplicitExtended(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::ImplicitExtended@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("extender") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "extender", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("extender".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_int_exp_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "first_int_exp_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_int_exp_set".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "first_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_set".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_int_exp_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "second_int_exp_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_int_exp_set".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_set") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "second_set", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_set".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::ImplicitExtended@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_ImplicitExtended(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_ImplicitExtended(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("extender"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("extender".into(),v);}
if let Some(mut v)=data.remove("first_int_exp_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_FirstIntExpSet(v)?;
}
data.insert("first_int_exp_set".into(),v);}
if let Some(mut v)=data.remove("first_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_FirstSet(v)?;
}
data.insert("first_set".into(),v);}
if let Some(mut v)=data.remove("second_int_exp_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_SecondIntExpSet(v)?;
}
data.insert("second_int_exp_set".into(),v);}
if let Some(mut v)=data.remove("second_set"){
if !v.is_null() {
v=decode_Asciiunits_Model_SecondSet(v)?;
}
data.insert("second_set".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_ImplicitExtended(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::ImplicitExtended@default")?;let mut wire=Map::new();
for mut v in data.remove("extender").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("extender".into(),v);}
for mut v in data.remove("first_int_exp_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_FirstIntExpSet(v)?;
}
wire.insert("first_int_exp_set".into(),v);}
for mut v in data.remove("first_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_FirstSet(v)?;
}
wire.insert("first_set".into(),v);}
for mut v in data.remove("second_int_exp_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_SecondIntExpSet(v)?;
}
wire.insert("second_int_exp_set".into(),v);}
for mut v in data.remove("second_set").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_SecondSet(v)?;
}
wire.insert("second_set".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_PowerNumerator_json(input:&str)->ContractResult<Asciiunits_Model_PowerNumerator>{let data=decode_Asciiunits_Model_PowerNumerator(parse_json(input,"Asciiunits::Model::PowerNumerator@default")?)?; let model:Asciiunits_Model_PowerNumerator=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::PowerNumerator@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_PowerNumerator_json(model:&Asciiunits_Model_PowerNumerator)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::PowerNumerator@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_PowerNumerator(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::PowerNumerator@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_PowerNumerator(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::PowerNumerator@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("int_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::PowerNumerator@default", "int_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("int_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("integer") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::PowerNumerator@default", "integer", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("integer".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::PowerNumerator@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_PowerNumerator(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_PowerNumerator(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("int_exp"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("int_exp".into(),v);}
if let Some(mut v)=data.remove("integer"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("integer".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_PowerNumerator(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::PowerNumerator@default")?;let mut wire=Map::new();
for mut v in data.remove("int_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("int_exp".into(),v);}
for mut v in data.remove("integer").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("integer".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_SecondIntExpSet_json(input:&str)->ContractResult<Asciiunits_Model_SecondIntExpSet>{let data=decode_Asciiunits_Model_SecondIntExpSet(parse_json(input,"Asciiunits::Model::SecondIntExpSet@default")?)?; let model:Asciiunits_Model_SecondIntExpSet=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondIntExpSet@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_SecondIntExpSet_json(model:&Asciiunits_Model_SecondIntExpSet)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondIntExpSet@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_SecondIntExpSet(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondIntExpSet@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_SecondIntExpSet(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::SecondIntExpSet@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("extender") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondIntExpSet@default", "extender", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("extender".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("int_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondIntExpSet@default", "int_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("int_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondIntExpSet@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondIntExpSet@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_SecondIntExpSet(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_SecondIntExpSet(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("extender"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("extender".into(),v);}
if let Some(mut v)=data.remove("int_exp"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("int_exp".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sequence".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_SecondIntExpSet(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::SecondIntExpSet@default")?;let mut wire=Map::new();
for mut v in data.remove("extender").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("extender".into(),v);}
for mut v in data.remove("int_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("int_exp".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sequence".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_SecondSet_json(input:&str)->ContractResult<Asciiunits_Model_SecondSet>{let data=decode_Asciiunits_Model_SecondSet(parse_json(input,"Asciiunits::Model::SecondSet@default")?)?; let model:Asciiunits_Model_SecondSet=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_SecondSet_json(model:&Asciiunits_Model_SecondSet)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_SecondSet(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_SecondSet(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::SecondSet@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("dimension") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "dimension", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dimension".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("extender") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "extender", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("extender".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_numerator") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "power_numerator", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_numerator".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("prefix") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "prefix", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("prefix".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sqrt") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "sqrt", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sqrt".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unit") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "unit", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unit".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::SecondSet@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_SecondSet(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_SecondSet(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("dimension"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("dimension".into(),v);}
if let Some(mut v)=data.remove("extender"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("extender".into(),v);}
if let Some(mut v)=data.remove("power_numerator"){
if !v.is_null() {
v=decode_Asciiunits_Model_PowerNumerator(v)?;
}
data.insert("power_numerator".into(),v);}
if let Some(mut v)=data.remove("prefix"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("prefix".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("sqrt"){
if !v.is_null() {
v=decode_Asciiunits_Model_Expression(v)?;
}
data.insert("sqrt".into(),v);}
if let Some(mut v)=data.remove("unit"){
if !v.is_null() {
v=decode_Asciiunits_Model_Slice(v)?;
}
data.insert("unit".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_SecondSet(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::SecondSet@default")?;let mut wire=Map::new();
for mut v in data.remove("dimension").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("dimension".into(),v);}
for mut v in data.remove("extender").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("extender".into(),v);}
for mut v in data.remove("power_numerator").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_PowerNumerator(v)?;
}
wire.insert("power_numerator".into(),v);}
for mut v in data.remove("prefix").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("prefix".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("sqrt").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Expression(v)?;
}
wire.insert("sqrt".into(),v);}
for mut v in data.remove("unit").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Asciiunits_Model_Slice(v)?;
}
wire.insert("unit".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Asciiunits_Model_Slice_json(input:&str)->ContractResult<Asciiunits_Model_Slice>{let data=decode_Asciiunits_Model_Slice(parse_json(input,"Asciiunits::Model::Slice@default")?)?; let model:Asciiunits_Model_Slice=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Asciiunits_Model_Slice_json(model:&Asciiunits_Model_Slice)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "", "/output",e))?;serde_json::to_string(&encode_Asciiunits_Model_Slice(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Asciiunits_Model_Slice(input:Value)->ContractResult<Value>{let mut wire=object(input,"Asciiunits::Model::Slice@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("offset") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "offset", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("offset".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("length") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "length", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("length".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("line") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "line", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("line".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("column") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "column", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("column".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "", "/input", "unknown wire key"));} hydrate_Asciiunits_Model_Slice(data) }
#[allow(non_snake_case)]
fn hydrate_Asciiunits_Model_Slice(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("offset"){
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "offset", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "offset", "/input",e.to_string()))?;
}
data.insert("offset".into(),v);}
if let Some(mut v)=data.remove("length"){
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "length", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "length", "/input",e.to_string()))?;
}
data.insert("length".into(),v);}
if let Some(mut v)=data.remove("line"){
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "line", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "line", "/input",e.to_string()))?;
}
data.insert("line".into(),v);}
if let Some(mut v)=data.remove("column"){
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "column", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "column", "/input",e.to_string()))?;
}
data.insert("column".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Asciiunits_Model_Slice(input:Value)->ContractResult<Value>{let mut data=object(input,"Asciiunits::Model::Slice@default")?;let mut wire=Map::new();
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("offset").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "offset", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "offset", "/input",e.to_string()))?;
}
wire.insert("offset".into(),v);}
for mut v in data.remove("length").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "length", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "length", "/input",e.to_string()))?;
}
wire.insert("length".into(),v);}
for mut v in data.remove("line").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "line", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "line", "/input",e.to_string()))?;
}
wire.insert("line".into(),v);}
for mut v in data.remove("column").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let lexical=v.as_str().ok_or_else(||failure(ErrorReason::InvalidDocument,"Asciiunits::Model::Slice@default", "column", "/input", "exact value requires string (portable-exact-json-v1 requires exact values as strings; numeric JSON is ruby-native-v1 and is not accepted)"))?; super::values::ExactInteger::parse(lexical).map_err(|e|failure(e.reason.clone(),"Asciiunits::Model::Slice@default", "column", "/input",e.to_string()))?;
}
wire.insert("column".into(),v);}
Ok(Value::Object(wire))}
