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


//! DynObject is a kind of dynamic object witch allows creating and deleting properties at runtime.
//!
//! DynObject does runtime type checks over genereic functions so that the rest of your programm don't has to care mutch
//! about it. Neverless this has three backdrawings:
//!
//! 1. Accessing the variables allways returns a Result.
//! 2. it has to own the data.
//! 3. it's slower. 
//!

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
pub use inner_dyn_object::UndefinedProperty;
pub use inner_dyn_object::InnerDynObject;


mod dyn_property;
mod inner_dyn_object;


pub struct DynObject<Key> {
    inner: Rc<RefCell<InnerDynObject<Key>>>
}


impl<Key> DynObject<Key> where Key: Eq+Hash {

    /// create a new empty DynObject with Key Type `Key`
    ///
    /// Note automatic detection of the type will not
    /// allways work. It is best to use 
    /// `DynObject::<KeyType>::new()
    ///
    pub fn new() -> DynObject<Key> {
        let x =  InnerDynObject::<Key>::new();
        let cell = RefCell::new(x);
        let rc = Rc::new(cell);
        DynObject {
            inner: rc
        }
    }
    
    /// aquire the DynObject to perform operations on it,
    /// note that DynObject has interior mutablilty and
    /// therefor borrowing it mutiple times as immutable
    /// reference will panic at runtime!!
    ///
    /// # Panics
    /// panics if someone else aquired it and didn't relase it jet
    /// (by droping the returned RefMut, witch is often done 
    /// implicitly)
    ///
    /// e.g. the folowing will panic
    ///
    /// ```should_panic
    /// # use dynobject::DynObject;
    /// let obj = DynObject::<i32>::new();
    /// let v1 = obj.aquire();
    /// //compiles but PANICS!
    /// let v2 = obj.aquire();
    /// ```
    ///
    pub fn aquire(&self) -> RefMut<InnerDynObject<Key>> {
        self.inner.borrow_mut()
    }
}

impl<T> Clone for DynObject<T> where T: Eq+Hash {

    /// shalow clons `DynObject` saftily sharing the inner `InnerDynbject`
    ///
    /// for saftily sharing the inner object Reference Counting and Cells
    /// with inner mutability. There is a logical protection preventing
    /// anyone from accessing the inner data when it is currently borrowd.
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
        let x = create_dummy();
        let data = x.aquire();
    }

    #[test]
    #[should_fail]
    fn aquire_multiple_times_should_panic() {
        let x = create_dummy();
        let obj_ref_2 = x.clone();
        let data = x.aquire();
        let data2 = obj_ref_2.aquire();
    }

    #[test]
    fn aquire_multiple_times_after_relasing_each_should_not_fail() {
        let x = create_dummy();
        {
            let data = x.aquire();
        }
        let data2 = x.aquire();
    }
    
    fn set_data(target: DynObject<&'static str>, value: i32) {
        assert!(target.aquire().create_property(&"hallo", Box::new(value)).is_ok());
    }

    #[test]
    fn mutiple_cloned_dyn_object_should_share_the_same_core() {
        let value = 23i32;
        let obj1 = create_dummy(); 
        set_data(obj1.clone(), value);
        let obj = obj1.aquire();
        match obj["hallo"].as_ref::<i32>() {
            Some(data) => assert_eq!(data, &value),
            None => panic!("type mismatch, error in test or other class")
        }
    }
}
