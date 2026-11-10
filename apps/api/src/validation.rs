use crate::error::{ApiError,Result};
use serde_json::Value;
pub fn text(value:&str,max:usize)->Result<()> {if value.trim().is_empty() || value.chars().count()>max {Err(ApiError::bad(format!("Text must contain 1–{max} characters")))}else{Ok(())}}
pub fn decision(value:&str,note:&str)->Result<()> {if !["approve","deny"].contains(&value){return Err(ApiError::bad("Choose approve or deny"));}if value=="deny"{text(note,2000)?;}if note.chars().count()>2000{return Err(ApiError::bad("Note is too long"));}Ok(())}
pub fn years(year:i32)->Result<()> {if (1990..=2200).contains(&year){Ok(())}else{Err(ApiError::bad("Invalid service year"))}}
pub fn validate_schema(schema:&Value)->Result<()> {
 let fields=schema.as_array().ok_or_else(||ApiError::bad("Questions must be an array"))?;if fields.is_empty()||fields.len()>40{return Err(ApiError::bad("Use 1–40 questions"));}
 let mut keys=std::collections::HashSet::new();
 for f in fields {let key=f["key"].as_str().unwrap_or("");text(key,60)?;if !key.chars().all(|c|c.is_ascii_alphanumeric()||c=='_')||!keys.insert(key){return Err(ApiError::bad("Question keys must be unique identifiers"));}text(f["label"].as_str().unwrap_or(""),240)?;
  if f.get("required").is_some_and(|v|!v.is_boolean())||f.get("sensitive").is_some_and(|v|!v.is_boolean())||f.get("max_length").is_some_and(|v|!v.as_u64().is_some_and(|n|(1..=10000).contains(&n))){return Err(ApiError::bad("Invalid question requirement or length rule"));}
  if !["text","textarea","number","date","boolean","single_select","multi_select","url"].contains(&f["type"].as_str().unwrap_or("")){return Err(ApiError::bad("Unsupported question type"));}
  if ["single_select","multi_select"].contains(&f["type"].as_str().unwrap_or("")) && !f["options"].as_array().is_some_and(|a|!a.is_empty()&&a.len()<=30&&a.iter().all(|v|v.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=120))){return Err(ApiError::bad("Select questions require 1–30 text options"));}
 }
 Ok(())
}
pub fn validate_answers(schema:&Value,answers:&Value)->Result<()> {
 let fields=schema.as_array().ok_or_else(||ApiError::bad("Invalid form"))?;
 let obj=answers.as_object().ok_or_else(||ApiError::bad("Answers must be an object"))?;
 if obj.keys().any(|k|!fields.iter().any(|f|f["key"].as_str()==Some(k))){return Err(ApiError::bad("Unknown answer field"));}
 for f in fields {let key=f["key"].as_str().unwrap_or("");let v=&answers[key];let missing=v.is_null()||v.as_str().is_some_and(|s|s.trim().is_empty())||v.as_array().is_some_and(|a|a.is_empty());if missing{if f["required"]==true{return Err(ApiError::bad(format!("{} is required",f["label"].as_str().unwrap_or(key))));}continue;}
 let kind=f["type"].as_str().unwrap_or("text");let valid=match kind{"boolean"=>v.is_boolean(),"number"=>v.is_number(),"multi_select"=>v.as_array().is_some_and(|a|a.iter().all(|x|f["options"].as_array().is_some_and(|opts|opts.contains(x)))),"single_select"=>f["options"].as_array().is_some_and(|opts|opts.contains(v)),"date"=>v.as_str().is_some_and(|s|chrono::NaiveDate::parse_from_str(s,"%Y-%m-%d").is_ok()),"url"=>v.as_str().is_some_and(|s|s.starts_with("https://")),_=>v.is_string()};
 if !valid||v.as_str().is_some_and(|s|s.chars().count()>f["max_length"].as_u64().unwrap_or(4000).min(10000) as usize){return Err(ApiError::bad(format!("Invalid answer for {key}")));}
 }Ok(())
}
