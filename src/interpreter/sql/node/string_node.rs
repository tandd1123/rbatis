use std::collections::LinkedList;

use serde_json::{json, Value};
use serde_json::map::Map;

use crate::core::convert::StmtConvert;
use crate::core::db::DriverType;
use crate::interpreter::expr;
use crate::interpreter::expr::runtime::ExprRuntime;
use crate::interpreter::sql::ast::RbatisAST;
use crate::utils::string_util;

///string抽象节点
#[derive(Clone, Debug)]
pub struct StringNode {
    pub value: String,
    //去重的，需要替换的要sql转换express map
    pub express_map: LinkedList<(String, String)>,
}

impl StringNode {
    pub fn new(v: &str) -> Self {
        Self {
            value: v.to_string(),
            express_map: string_util::find_convert_string(v),
        }
    }
}

impl RbatisAST for StringNode {
    fn name() -> &'static str {
        "string"
    }
    fn eval(&self, convert: &crate::core::db::DriverType, env: &mut Value, engine: &ExprRuntime, arg_array: &mut Vec<Value>) -> Result<String, crate::core::Error> {
        let mut result = self.value.clone();
        for (item, value) in &self.express_map {
            if item.is_empty() {
                result = result.replace(value, "");
                continue;
            }
            if value.starts_with("#") {
                result = result.replace(value, convert.stmt_convert(arg_array.len()).as_str());
                let get_v = env.get(item);
                if get_v.is_none() {
                    let v = engine.eval(item, env)?;
                    arg_array.push(v);
                } else {
                    let v = get_v.unwrap().clone();
                    arg_array.push(v);
                }
            } else {
                let v = env.get(item);
                match v {
                    Some(v) => {
                        if v.is_string() {
                            result = result.replace(value, &v.as_str().unwrap());
                        } else {
                            result = result.replace(value, &v.to_string());
                        }
                    }
                    None => {
                        let v = engine.eval(item, env)?;
                        if v.is_string() {
                            result = result.replace(value, &v.as_str().unwrap());
                        } else {
                            result = result.replace(value, &v.to_string());
                        }
                    }
                }
            }
        }
        return Result::Ok(result);
    }
}

#[cfg(test)]
mod test {
    use crate::core::db::DriverType;
    use crate::interpreter::expr::runtime::ExprRuntime;
    use crate::interpreter::sql::ast::RbatisAST;
    use crate::interpreter::sql::node::string_node::StringNode;

    #[test]
    pub fn test_string_node() {
        let mut john = json!({
        "arg": 2,
    });
        let mut engine = ExprRuntime::new();
        let s_node = StringNode::new("arg+1=#{arg+1}");
        let mut arg_array = vec![];

        let r = s_node.eval(&DriverType::Mysql, &mut john, &mut engine, &mut arg_array).unwrap();
        println!("{}", r);
        assert_eq!(r, "arg+1=?");
        assert_eq!(arg_array.len(), 1);
    }

    #[test]
    pub fn test_string_node_replace() {
        let mut john = json!({
        "arg": 2,
    });
        let mut engine = ExprRuntime::new();
        let s_node = StringNode::new("arg+1=${arg+1}");
        let mut arg_array = vec![];
        let r = s_node.eval(&DriverType::Mysql, &mut john, &mut engine, &mut arg_array).unwrap();
        println!("r:{}", r);
        assert_eq!(r, "arg+1=3");
        assert_eq!(arg_array.len(), 0);
    }
}