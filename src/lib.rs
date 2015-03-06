// Copyright 2015 Philipp Korber
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software 
// distributed under the License is distributed on an "AS IS" BASIS, 
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. 
// See the License for the specific language governing permissions and 
// limitations under the License.

//alloc is needed for BoxAny, witch willl most like be removed in the future
//this is not a problem because the methodes of BoxAny will the most like
//become part of Any so that this lib can be updated by removing the 
//import and feature statement
#![feature(alloc)]
//this is needed 'cause Associated Types in combination with e.g. Index are not yet complety stable
#![feature(core)]
//for now it is unstable
#![unstable(feature="alloc,core")]

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;
use std::hash::Hash;

//import and reexport dyn_property
pub use dyn_property::DynProperty;
pub use dyn_property::UndefinedProperty;
pub use inner_dyn_object::InnerDynObject;

///! 
///! InnerDynObject is a kind of dynamic objects witch allows
///! creating and deleting properties at runtime.
///! This includs runtime type checks over genereic functions
///! so that the rest of your programm don't has to care mutch
///! about. Neverless this has to backdrawings:
///!   1. Accessing the variables allways returns a Result
///!   2. it has to own the data
///!   3. it's slower. If you have a group of variables putng
///!      them into a POD rust object and then puting it into
///!      InnerDynObject might be preferable

mod dyn_property;
mod inner_dyn_object;


pub struct DynObject<Key> {
    inner: Rc<RefCell<InnerDynObject<Key>>>
}


impl<Key> DynObject<Key> where Key: Eq+Hash {

    pub fn new<T>() -> DynObject<T>
        where T: Eq + Hash
    {
        let x =  InnerDynObject::<T>::new();
        let cell = RefCell::new(x);
        let rc = Rc::new(cell);
        DynObject {
            inner: rc
        }
    }
    
    /// aquire the DynObject to perform operations on it
    ///
    /// # Panics
    /// if someone else aquired it and didn't relase it jet
    /// (by droping the returned RefMut, witch is often done 
    /// implicitly)
    pub fn aquire(&mut self) -> RefMut<InnerDynObject<Key>> {
        self.inner.borrow_mut()
    }
}

impl<T> Clone for DynObject<T> where T: Eq+Hash {

    fn clone(&self) -> Self {
        DynObject {
            inner: self.inner.clone()
        }
    }
}


#[cfg(test)]
mod test_dyn_object {
    #![allow(unused_variables)]

    use super::DynObject;

    fn create_dummy() -> DynObject<&'static str> {
        DynObject::<&'static str>::new()
    }
    
    #[test]
    fn aquire_should_not_panic_if_only_on_instance_exists() {
        let mut x = create_dummy();
        let data = x.aquire();
    }

    #[test]
    #[should_fail]
    fn aquire_multiple_times_should_panic() {
        let mut x = create_dummy();
        let mut obj_ref_2 = x.clone();
        let data = x.aquire();
        let data2 = obj_ref_2.aquire();
    }

    #[test]
    fn aquire_multiple_times_after_relasing_each_should_not_fail() {
        let mut x = create_dummy();
        {
            let data = x.aquire();
        }
        let data2 = x.aquire();
    }
    
    fn set_data(mut target: DynObject<&'static str>, value: i32) {
        target.aquire().create_property(&"hallo", Box::new(value));
    }

    #[test]
    fn mutiple_cloned_dyn_object_should_share_the_same_core() {
        let value = 23i32;
        let mut obj1 = create_dummy(); 
        set_data(obj1.clone(), value);
        let obj = obj1.aquire();
        match obj["hallo"].as_ref::<i32>() {
            Some(data) => assert_eq!(data, &value),
            None => panic!("type mismatch, error in test or other class")
        }
    }
}
