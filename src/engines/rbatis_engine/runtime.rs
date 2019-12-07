use std::collections::linked_list::LinkedList;
use std::collections::HashMap;
use serde_json::Value;
use crate::engines::rbatis_engine::parser::parser;
use crate::engines::rbatis_engine::node::Node;

#[derive(Clone)]
pub struct RbatisEngine {
    pub cache:HashMap<String,Node>,
    pub opt_map:OptMap<'static>,
}

impl RbatisEngine {

    pub fn new()-> Self{
        return Self{
            cache: Default::default(),
            opt_map: OptMap::new(),
        }
    }

    pub fn eval(&mut self, lexer_arg: &str, arg: &Value) -> Result<Value, String>{
        let cached = self.cache.get(lexer_arg);
        if (&cached).is_none() {
            let nodes = parser(lexer_arg.to_string(), &self.opt_map);
            if nodes.is_err(){
                return Result::Err(nodes.err().unwrap());
            }
            let node=nodes.unwrap();
            self.cache.insert(lexer_arg.to_string(), node.clone());
            return node.eval(arg);
        } else {
            let nodes = cached.unwrap().clone();
            return nodes.eval(arg);
        }
    }

    pub fn eval_no_cache(&self, lexer_arg: &str, arg: &Value) -> Result<Value, String>{
            let nodes = parser(lexer_arg.to_string(), &self.opt_map);
            if nodes.is_err(){
                return Result::Err(nodes.err().unwrap());
            }
            let node=nodes.unwrap();
            return node.eval(arg);
    }

    pub fn clear_cache(&mut self){
        self.cache.clear();
    }

    pub fn remove_cache(&mut self, lexer_arg: &str){
        self.cache.remove(lexer_arg);
    }


}



pub fn is_number(arg: &String) -> bool {
    let chars = arg.chars();
    for item in chars {
        if item == '.' ||
            item == '0' ||
            item == '1' ||
            item == '2' ||
            item == '3' ||
            item == '4' ||
            item == '5' ||
            item == '6' ||
            item == '7' ||
            item == '8' ||
            item == '9'
        {
            // nothing do
        } else {
            return false;
        }
    }
    return true;
}


/**
 * 将原始字符串解析为 去除空格的token数组
**/
pub fn parser_tokens(s: &String, opt_map: &OptMap) -> Vec<String> {
    let chars = s.chars();
    let chars_len = s.len() as i32;
    let mut result = LinkedList::new();
    //str
    let mut find_str = false;
    let mut temp_str = String::new();
    //opt
    let mut temp_arg = String::new();
    let mut index: i32 = -1;
    for item in chars {
        index = index + 1;
        let is_opt = opt_map.is_opt(item.to_string().as_str());
        if item == '\'' || item == '`' {
            if find_str {
                //第二次找到
                find_str = false;
                temp_str.push(item);
                trim_push_back(&temp_str, &mut result);
                temp_str.clear();
                continue;
            }
            find_str = true;
            temp_str.push(item);
            continue;
        }
        if find_str {
            temp_str.push(item);
            continue;
        }
        if item != '`' && item != '\'' && is_opt == false && !find_str {
            //need reset
            temp_arg.push(item);
            if (index + 1) == chars_len {
                trim_push_back(&temp_arg, &mut result);
            }
        } else {
            trim_push_back(&temp_arg, &mut result);
            temp_arg.clear();
        }
        //opt node
        if is_opt {
            //println!("is opt:{}", item);
            if result.len() > 0 {
                let def = String::new();
                let back = result.back().unwrap_or(&def).clone();
                if back != "" && opt_map.is_opt(back.as_str()) {
                    result.pop_back();
                    let mut new_item = back.clone().to_string();
                    new_item.push(item);
                    trim_push_back(&new_item, &mut result);
                    continue;
                }
            }
            trim_push_back(&item.to_string(), &mut result);
            continue;
        }
    }
    let mut v = vec![];
    for item in result {
        v.push(item);
    }
    return v;
}

fn trim_push_back(arg: &String, list: &mut LinkedList<String>) {
    let trim_str = arg.trim().to_string();
    if trim_str.is_empty() {
        return;
    }
    list.push_back(trim_str);
}

#[derive(Clone)]
pub struct OptMap<'a> {
    //列表
    pub list: Vec<&'a str>,
    //全部操作符
    pub map: HashMap<&'a str, bool>,
    //复合操作符
    pub mul_ops_map: HashMap<&'a str, bool>,
    //单操作符
    pub single_opt_map: HashMap<&'a str, bool>,

    pub allow_priority_array: Vec<&'a str>,
}

impl<'a> OptMap<'a> {
    pub fn new() -> Self {
        let mut list = Vec::new();
        let mut def_map = HashMap::new();
        let mut mul_ops_map = HashMap::new();
        let mut single_opt_map = HashMap::new();

        //list 顺序加入操作符
        list.push("*");
        list.push("/");
        list.push("%");
        list.push("^");
        list.push("+");
        list.push("-");

        list.push("(");
        list.push(")");
        list.push("@");
        list.push("#");
        list.push("$");
        list.push("&");
        list.push("|");
        list.push("=");
        list.push("!");
        list.push(">");
        list.push("<");

        list.push("&&");
        list.push("||");
        list.push("==");
        list.push("!=");
        list.push(">=");
        list.push("<=");


        //全部加入map集合
        for item in &mut list {
            def_map.insert(*item, true);
        }
        //加入单操作符和多操作符
        for item in &mut list {
            if item.len() > 1 {
                mul_ops_map.insert(item.clone(), true);
            } else {
                single_opt_map.insert(item.clone(), true);
            }
        }


        let mut vecs = vec![];
        vecs.push("*");
        vecs.push("/");
        vecs.push("+");
        vecs.push("-");
        vecs.push("<=");
        vecs.push("<");
        vecs.push(">=");
        vecs.push(">");
        vecs.push("!=");
        vecs.push("==");
        vecs.push("&&");
        vecs.push("||");


        Self {
            list: list,
            map: def_map,
            mul_ops_map: mul_ops_map,
            single_opt_map: single_opt_map,
            allow_priority_array: vecs,
        }
    }

    //乘除优先于加减 计算优于比较,
    pub fn priority_array(&self) -> Vec<&str> {
        return self.allow_priority_array.clone();
    }

    //是否是操作符
    pub fn is_opt(&self, arg: &str) -> bool {
        let opt = self.map.get(arg);
        return opt.is_none() == false;
    }

    //是否为有效的操作符
    pub fn is_allow_opt(&self, arg: &str) -> bool {
        for item in &self.allow_priority_array {
            if arg==*item{
                return true;
            }
        }
        return false;
    }
}