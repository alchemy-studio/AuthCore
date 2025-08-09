use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

use anyhow::{anyhow, Result};

use log::debug;
use crate::common::{HtyErr, HtyErrCode};

pub trait TestScaffold {
    fn before_test(self: &Self) -> anyhow::Result<HashMap<String, String>>;
    fn after_test(self: &Self);
}

// trait TestTrait = Fn(HashMap<String, String>) -> anyhow::Result<()>;

pub type TestTask = dyn Fn(HashMap<String, String>) -> anyhow::Result<()>;

pub fn do_test(f: Box<TestTask>, scaffold: Rc<Box<dyn TestScaffold>>) {
    match scaffold.before_test() {
        Ok(params) => {
            match f.deref()(params.clone()) {
                Ok(_) => {
                    scaffold.after_test();
                }
                Err(e) => {
                    scaffold.after_test();
                    panic!("[DO TEST ERROR] -> {:?}", e);
                }
            };
        }
        Err(e) => {
            scaffold.after_test();
            panic!("[BEFORE TEST ERROR] -> {:?}", e)
        }
    }
}

// ----------------------------------------------------------------------------------------------------
pub fn my_assert_eq<T: Eq + Debug>(a: T, b: T) -> anyhow::Result<()> {
    if a == b {
        Ok(())
    } else {
        Err(anyhow!(HtyErr {
            code: HtyErrCode::NotEqualErr,
            reason: Some(format!("not equal! -> {:?} <> {:?}", a, b)),
        }))
    }
}

pub fn my_assert_not_eq<T: Eq + Debug>(a: T, b: T) -> anyhow::Result<()> {
    //debug(format!("my_assert_not_equal -> {:?}", a != b).as_str());
    debug!("my_assert_not_equal -> a: {:?} b: {:?}", a, b);
    if a != b {
        Ok(())
    } else {
        Err(anyhow!(HtyErr {
            code: HtyErrCode::NotEqualErr,
            reason: Some(format!("should not be equal! -> {:?}", a)),
        }))
    }
}

pub fn my_assert_not_none<T: Eq + Debug>(val: &Option<T>) -> Result<(), HtyErr> {
    match val {
        Some(_) => Ok(()),
        None => Err(HtyErr {
            code: HtyErrCode::NotEqualErr,
            reason: Some("is none!".to_string()),
        }),
    }
}


pub fn get_test_app_domain() -> u16 {
    env::var("TEST_APP_DOMAIN")
        .expect("TEST_APP_DOMAIN not set!!!")
        .parse()
        .unwrap()
}