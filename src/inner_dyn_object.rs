use std::result::Result;
use std::ops::{Index, IndexMut};
use std::collections::HashMap;
use std::hash::Hash;
use std::any::Any;
use std::rc::Weak;
use std::cell::RefCell;

//import and reexport dyn_property
use super::dyn_property::DynProperty;

pub struct InnerDynObject<Key> {
    //initialise this allways with DynProperty::undefined();
    //FIXME move this as assoziated Konstant (with unsave) or static
    undefined_property: DynProperty,
    data: HashMap<Key, DynProperty>,
    
    //this is a SHARED! weak reference to itself, it can be used
    //to create a DynObject instance from a InnerDynObject instance
    uplink: Option<Weak<RefCell<InnerDynObject<Key>>>>
}


impl<Key> InnerDynObject<Key> where Key: Eq + Hash {
    
    pub fn new() -> InnerDynObject<Key> {
        InnerDynObject {
            undefined_property: DynProperty::undefined(),
            data: HashMap::<Key, DynProperty>::new(),
            uplink: None
        }
    }
    
    //TODO think about making set/get uplink unsafe to show it (only logicaly existing)
    //unsafeness, neverless it is not unsafe in the rust-lang unsafe sense
    /// sets the uplink of this calls
    ///
    /// this methode should mainly be used by DynObject if
    /// you are not sure why it is there don't touch it
    ///
    /// # Panics
    /// if the uplink is already set calling this methode will panic
    pub fn set_uplink(&mut self, uplink: Weak<RefCell<InnerDynObject<Key>>>) {
        match self.uplink {
            Some(_) => panic!("uplink was already set"),
            None => self.uplink = Some(uplink)
        }
    }

    /// returns the uplink of this class
    ///
    /// this methode should mainly be used by DynObject if
    /// you are not sure why it is there don't touch it
    pub fn get_uplink(&self) -> &Option<Weak<RefCell<InnerDynObject<Key>>>> {
        &self.uplink
    }


    pub fn set_property<T>(&mut self, key: &Key, value: Box<T>) -> Result<Box<T>,Box<T>> 
        where T: Any + 'static 
    {
        self.index_mut(key).set(value)
    }
    
    //FIXME if already existig init_value is lost (droped)
    pub fn create_property<T>(&mut self, key: Key, init_value: Box<T>) -> Result<(),Box<T>> 
        where T: Any + 'static  
    {
        if self.data.contains_key(&key) {
            Err(init_value)
        } else {
            self.data.insert(key, DynProperty::new(init_value));
            Ok( () )
        }
    }

    pub fn remove_property<T>(&mut self, key: &Key) -> Result<Box<T>, ()> 
        where T: Any + 'static
    {
        if !self.index(key).is_inner_type::<T>() {
            return Err( () );
        }
        Ok(self.data.remove(key).unwrap().destruct::<T>().unwrap())
    }

    pub fn exists_property(&self, key: &Key) -> bool {
        self.data.contains_key(key)
    }

    pub fn exists_property_with_type<T>(&self, key: &Key) -> bool 
        where T: Any + 'static 
    {
        self.index(key).is_inner_type::<T>()
    }
}

impl<Key: Hash+Eq> Index<Key> for InnerDynObject<Key> {
    type Output = DynProperty;
 
    //if there is no matching Key return a DynProperty of UndefinedPropertyType
    fn index<'a>(&'a self, index: &Key) -> &'a DynProperty {
        match self.data.get(index) {
            Some(data) => data,
            None => &self.undefined_property
        }
    }
}

impl<Key: Hash+Eq> IndexMut<Key> for InnerDynObject<Key> {

    fn index_mut<'a>(&'a mut self, index: &Key) -> &'a mut DynProperty {
        match self.data.get_mut(index) {
            Some(data) => data,
            None => &mut self.undefined_property
        }
    }
}

#[cfg(test)]
mod test {
    use super::InnerDynObject;
    use super::super::UndefinedProperty;
    
    fn create_dummy() -> InnerDynObject<&'static str> {
        InnerDynObject::<&'static str>::new()
    }

    #[test]
    fn exist_property_should_return_false_if_inexisting() {
        let obj = create_dummy();
        assert!( !obj.exists_property(&"hallo") );
    }


    #[test]
    fn after_creating_a_property_should_exist() {
        let mut obj = create_dummy();
        assert!(obj.create_property("hallo", Box::new(23i32)).is_ok());
        assert!(obj.exists_property(&"hallo"));
    }


    #[test]
    fn exist_property_with_type_should_return_true_if_property_exists_and_has_given_type() {
        let mut obj = create_dummy();
        assert!(obj.create_property("hallo", Box::new(23i32)).is_ok());
        assert!(obj.exists_property_with_type::<i32>(&"hallo"));
    }

    #[test]
    fn exists_property_with_type_should_return_false_if_property_is_undefined() {
        let mut obj = create_dummy();
        assert!(obj.create_property("hallo", Box::new(23i32)).is_ok());
        assert!(!obj.exists_property_with_type::<i32>(&"NOThallo"));
    }

    #[test]
    fn exists_property_with_type_should_return_fals_if_the_type_mismatches() {
        let mut obj = create_dummy();
        assert!(obj.create_property("hallo", Box::new(23i32)).is_ok());
        assert!(!obj.exists_property_with_type::<u16>(&"hallo"));
    }

    #[test]
    fn create_property_should_return_true_if_key_is_new() {
        let mut obj = create_dummy();
        assert!(obj.create_property("hallo", Box::new(23i32)).is_ok());
    }

    

    #[test]
    fn create_property_should_return_false_if_key_already_exists() {
        let mut obj = create_dummy();
        assert!(  obj.create_property("hallo", Box::new(23i32)).is_ok());
        assert!( !obj.create_property("hallo", Box::new(20i32)).is_ok());

    }

    #[test]
    fn set_property_should_err_if_property_does_not_exist() {
        let mut obj = create_dummy();
        assert!( !obj.exists_property(&"hallo") );
        let res = obj.set_property(&"hallo", Box::new(44i32));
        assert_eq!(res, Err(Box::new(44i32)));
    }

    #[test]
    fn set_property_should_err_if_property_type_is_wrong() {
        let mut obj = create_dummy();
        assert!( obj.create_property("hallo", Box::new(23i32)).is_ok());
        let res = obj.set_property(&"hallo", Box::new("oh falsch"));
        assert_eq!(res, Err(Box::new("oh falsch")));
    }
    
    #[test]
    fn set_property_should_return_the_old_value_if_property_exists_and_type_matches() {
        let mut obj = create_dummy();
        assert!( obj.create_property("hallo", Box::new(23i32)).is_ok());
        let res = obj.set_property(&"hallo", Box::new(44i32));
        assert_eq!(res, Ok(Box::new(23i32)));
        //obj["hallo"] expands to obj.index(&"hallo") 
        assert_eq!(obj["hallo"].as_ref::<i32>().unwrap(), &44i32);
    }

    #[test]
    fn remove_property_should_fail_if_property_does_not_exists() {
        let mut obj = create_dummy();
        let res = obj.remove_property::<i32>(&"hallo");
        assert_eq!(res, Err( () ));
    }

    #[test]
    fn remove_property_should_fail_if_the_type_mismatches() {
        let mut obj = create_dummy();
        assert!( obj.create_property("hallo", Box::new(23i32)).is_ok());
        let res = obj.remove_property::<u16>(&"hallo");
        assert_eq!(res, Err( () ));
        assert!(obj.exists_property(&"hallo"));
    }

    #[test]
    fn remove_property_should_return_the_property_value_if_succesful() {
        let mut obj = create_dummy();
        assert!( obj.create_property("hallo", Box::new(23i32)).is_ok());
        let res = obj.remove_property::<i32>(&"hallo");
        assert_eq!(res, Ok(Box::new(23i32)));
    }

    //TODO test index
    #[test]
    fn index_should_return_the_property_if_existing() {
        let mut obj = create_dummy();
        assert!( obj.create_property("hallo", Box::new(23i32)).is_ok());
        let ref res = obj["hallo"];
        assert!(res.is_inner_type::<i32>());
        match res.as_ref::<i32>() {
            Some(ref_val) => assert_eq!(ref_val, &23i32),
            None => panic!("expected to be borrowable in this situration")
        }   
    }

    #[test]
    fn index_should_return_a_property_of_type_undefined_if_inexisting() {
        let obj = create_dummy();
        assert!(obj["hallo"].is_inner_type::<UndefinedProperty>());
    }

    #[test]
    fn index_should_also_allow_mutable_access() {
        let mut obj = create_dummy();
        assert!( obj.create_property("hallo", Box::new(23i32)).is_ok());
        match obj["hallo"].as_mut::<i32>() {
            Some(ref_val) => {
                let mut dummy = 23i32;
                assert_eq!(ref_val, &mut dummy)
            }
            None => panic!("expected to be borrowable in this situration")
        }   
    }
}
