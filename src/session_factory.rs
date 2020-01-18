use std::collections::HashMap;
use std::thread::ThreadId;

use rdbc::Connection;

use crate::local_session::LocalSession;
use crate::session::Session;
use crate::utils::driver_util;

///链接工厂
pub trait SessionFactory<'a> {
    fn get_thread_session(&mut self, id: &ThreadId, driver: &str) -> Result<&mut LocalSession<'a>, String>;
}


pub struct SessionFactoryImpl<'a> {
    ///是否启用异步模式，即async await
    pub async_mode: bool,
    /// data 持有session所有权，当session被删除时，session即被销毁
    pub data: HashMap<ThreadId, LocalSession<'a>>,
}


impl<'a> SessionFactory<'a> for SessionFactoryImpl<'a> {
    fn get_thread_session(&mut self, id: &ThreadId, driver: &str) -> Result<&mut LocalSession<'a>, String> {
        let item = self.data.get(id);
        if item.is_some() {
            return Ok(self.data.get_mut(&id).unwrap());
        } else {
            let session = LocalSession::new("", driver, None)?;
            self.data.insert(id.clone(), session);
            return Ok(self.data.get_mut(&id).unwrap());
        }
    }
}

impl<'a> SessionFactoryImpl<'a> {
    pub fn new(async_mode: bool) -> Self {
        return Self {
            async_mode,
            data: HashMap::new(),
        };
    }
}